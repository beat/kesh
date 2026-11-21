//! Route `POST /api/v1/wise-imports` — Story 39-11b (Epic 39, phase 1).
//!
//! Importe d'un geste un ou plusieurs relevés CSV d'un **compte Wise**
//! (multipart : un ou plusieurs `file`, `account` = le libellé). Wise exporte
//! un relevé **par solde** : chaque fichier porte une devise (lecteur de la
//! Story 39-11a) et va au compte bancaire Kesh, solde de ce compte Wise dans
//! cette devise (`bank_accounts.provider = 'wise'`, `provider_account` = le
//! libellé, devise = celle du compte du plan lié).
//!
//! Comptable+, clés d'API `read-write` admises, comme `POST
//! /api/v1/bank-imports`. Taille maximale du corps entier : celle de l'import
//! bancaire.
//!
//! **Ordre des refus**, rien d'écrit à aucun : `400 VALIDATION_ERROR` (champ
//! manquant, aucun fichier) → `404 PROVIDER_ACCOUNT_NOT_FOUND` → `422
//! WISE_CSV_INVALID` (premier fichier refusé par le lecteur, nommé) → `422
//! WISE_DUPLICATE_CURRENCY_FILE` → `422 PROVIDER_CURRENCY_ACCOUNT_MISSING` →
//! `409 PROVIDER_CURRENCY_DUPLICATE` → `409 BANK_ACCOUNT_CLOSED` → `422
//! BANK_IMPORT_DUPLICATE_FILE` (jugé sous les verrous, dans la transaction
//! d'écriture). Un verrou de compte non obtenu rend `409
//! RECONCILIATION_ACCOUNT_LOCKED`.
//!
//! **Rangement** : par le cœur commun de la Story 38-11b
//! ([`run_provider_import`], [`insert_currency_import`]), paramétré par le
//! prestataire — verrous de chaque compte visé par identifiant croissant, une
//! seule transaction pour tous les fichiers (tout ou rien), relecture
//! verrouillante des soldes, empreinte, rejeu sur interblocage. Par fichier,
//! une entrée `bank_imports` (`CSV`, période `first_date..last_date`, soldes
//! d'ouverture et de clôture du relevé) ; une ligne = une transaction
//! ([`transaction_of_row`]) ; `Total fees` n'en est jamais une (les frais sont
//! leurs propres lignes).
//!
//! **Doublons** : une ligne dont l'identifiant Wise est déjà sur le compte
//! bancaire **dans le même sens** est ignorée et comptée (la fenêtre
//! date-montant-référence des imports bancaires ne s'applique pas) ; un
//! fichier tout en doublons ne crée pas d'entrée. La clé est (identifiant,
//! sens), comme au lecteur : Wise écrit sous un même identifiant le débit puis
//! le recrédit d'une réserve de carte expirée ou d'une vérification de carte
//! (Story 39-11a2, [`existing_keys`]). **Continuité** : avertissement `gap` quand le relevé ne
//! prolonge pas le précédent du compte
//! (`kesh_db::repositories::bank_imports::continuity_gap`, la règle unique
//! que relit aussi la revue de la Story 39-11c) ; l'import passe.
//!
//! ⚠️ Les règles de doublon propres à Wise R1 à R3 (frais de conversion inclus,
//! conversion automatique) n'ont d'objet qu'avec le téléchargement par l'API
//! (Story 39-12a3, `wise_duplicates.rs`, point d'accroche `rules` du cœur
//! commun) : sans ligne téléchargée en base, aucune ne se déclenche.

use std::collections::HashSet;

use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use rust_decimal::Decimal;
use serde::Serialize;

use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::bank_account::PROVIDER_WISE;
use kesh_db::entities::bank_transaction::NewBankTransaction;
use kesh_db::repositories::bank_accounts::{self, ProviderBalance};
use kesh_db::repositories::{audit_log, bank_imports, bank_transactions};
use kesh_import::wise::{WiseCsv, WiseCsvError, WiseDirection, WiseRow, parse_wise_csv};

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;
use crate::routes::paypal_imports::{
    CurrencyImport, DETAILS_MAX_CHARS, LockedTarget, NAME_MAX_CHARS, PROVIDER_TYPE_MAX_CHARS,
    ProviderWrite, TXID_COLUMN_MAX_CHARS, WriteFuture, file_currency_hash, insert_currency_import,
    resolve_currency_balances, run_provider_import, truncate_bytes, truncate_chars,
};

/// Nom d'un fichier sans nom dans le formulaire.
const DEFAULT_FILENAME: &str = "wise.csv";
/// Borne de `bank_imports.filename`, en octets.
const FILENAME_MAX_BYTES: usize = 255;
/// Longueur maximale d'un IBAN (`counterparty_iban VARCHAR(34)`).
const IBAN_MAX_CHARS: usize = 34;
/// Séparateur des éléments du libellé.
const DETAILS_SEPARATOR: &str = " · ";
/// Code de l'avertissement de continuité.
pub const WARNING_GAP: &str = "gap";

// ---------------------------------------------------------------------------
// DTO
// ---------------------------------------------------------------------------

/// Avertissement d'un fichier — Story 39-11b (AC-5) : `gap`, le relevé ne
/// prolonge pas le précédent du compte (`expected` = sa clôture, `found` =
/// l'ouverture du fichier).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileWarning {
    pub code: &'static str,
    pub expected: Decimal,
    pub found: Decimal,
    pub previous_import_id: i64,
}

/// Compte rendu d'un fichier : son compte bancaire, l'entrée `bank_imports`
/// créée (`null` si toutes ses lignes étaient des doublons), les nombres de
/// transactions insérées et ignorées, ses soldes et ses avertissements.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileReport {
    pub filename: String,
    pub currency: String,
    pub bank_account_id: i64,
    pub import_id: Option<i64>,
    pub inserted: usize,
    pub duplicates: usize,
    pub opening_balance: Decimal,
    pub closing_balance: Decimal,
    pub warnings: Vec<FileWarning>,
}

/// Réponse `201` de l'import (AC-5).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WiseImportResponse {
    pub account: String,
    pub files: Vec<FileReport>,
}

// ---------------------------------------------------------------------------
// Une ligne → une transaction (pure)
// ---------------------------------------------------------------------------

/// Le texte rogné d'une cellule facultative, `None` s'il est vide.
fn non_empty(value: &Option<String>) -> Option<&str> {
    value.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

/// Un numéro de compte en **forme d'IBAN** — Story 39-11b (AC-4) : sans
/// espaces, en majuscules, deux lettres, deux chiffres puis 11 à 30
/// caractères alphanumériques ASCII, 34 au plus. Sinon `None` (les relevés
/// portent aussi des numéros nationaux et des numéros masqués d'étoiles).
/// Contrôle de forme seul, sans clé de contrôle : le numéro est celui que Wise
/// a écrit.
pub fn iban_shaped(account: &str) -> Option<String> {
    let compact: String = account
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_ascii_uppercase();
    let bytes = compact.as_bytes();
    let shaped = (15..=IBAN_MAX_CHARS).contains(&bytes.len())
        && bytes[..2].iter().all(u8::is_ascii_uppercase)
        && bytes[2..4].iter().all(u8::is_ascii_digit)
        && bytes[4..].iter().all(u8::is_ascii_alphanumeric);
    shaped.then_some(compact)
}

/// La transaction d'une ligne du relevé Wise — Story 39-11b (AC-4) : montant
/// signé, date de la ligne, identifiant Wise, référence de paiement,
/// contrepartie (`Merchant`, sinon `Payee Name` d'un débit ou `Payer Name`
/// d'un crédit), IBAN du bénéficiaire s'il en a la forme, type = préfixe de
/// l'identifiant (filtre des règles, G5), libellé = description, type de
/// détail, numéro de compte non IBAN, conversion et note, joints par « · » et
/// tronqués à la frontière d'un caractère.
pub fn transaction_of_row(
    row: &WiseRow,
    company_id: i64,
    bank_account_id: i64,
) -> NewBankTransaction {
    let account = non_empty(&row.payee_account);
    let iban = account.and_then(iban_shaped);
    let other_account = if iban.is_none() { account } else { None };
    let exchange = match (non_empty(&row.exchange_from), non_empty(&row.exchange_to)) {
        (Some(from), Some(to)) => {
            // Segment fixe et relisible (Story 39-11c) :
            // « DE→VERS [MONTANT] [au cours COURS] », le montant étant celui
            // reçu dans la devise VERS (`Exchange To Amount`). Aucun champ
            // structuré de `bank_transactions` ne le porte ; il n'existe qu'au
            // CSV, qui n'est pas conservé.
            let mut segment = format!("{from}→{to}");
            if let Some(to_amount) = row.exchange_to_amount {
                segment.push_str(&format!(" {to_amount}"));
            }
            if let Some(rate) = row.exchange_rate {
                segment.push_str(&format!(" au cours {rate}"));
            }
            Some(segment)
        }
        _ => None,
    };
    // La tête (description, type de détail, numéro) est rognée avant le segment
    // de conversion : une longue description ne doit pas faire perdre le
    // montant de la conversion (revue P2, L1). La note, en queue, reste rognée
    // la première par la troncature finale.
    let head = [
        non_empty(&row.description).map(str::to_string),
        Some(row.details_type.trim().to_string()).filter(|s| !s.is_empty()),
        other_account.map(str::to_string),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(DETAILS_SEPARATOR);
    let head = match &exchange {
        Some(segment) => {
            let reserved = DETAILS_SEPARATOR.chars().count() + segment.chars().count();
            truncate_chars(&head, DETAILS_MAX_CHARS.saturating_sub(reserved))
        }
        None => head,
    };
    let details = [
        Some(head).filter(|s| !s.is_empty()),
        exchange,
        non_empty(&row.note).map(str::to_string),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(DETAILS_SEPARATOR);
    let counterparty = non_empty(&row.merchant).or(match row.direction {
        WiseDirection::Debit => non_empty(&row.payee_name),
        WiseDirection::Credit => non_empty(&row.payer_name),
    });
    NewBankTransaction {
        company_id,
        bank_account_id,
        booking_date: row.date,
        value_date: None,
        amount: row.amount,
        currency: row.currency.clone(),
        reference: non_empty(&row.payment_reference)
            .map(|s| truncate_chars(s, TXID_COLUMN_MAX_CHARS)),
        details: truncate_chars(&details, DETAILS_MAX_CHARS),
        end_to_end_id: None,
        transaction_id: Some(row.id.clone()),
        counterparty_iban: iban,
        counterparty_name: counterparty.map(|s| truncate_chars(s, NAME_MAX_CHARS)),
        provider_type: Some(truncate_chars(&row.id_prefix, PROVIDER_TYPE_MAX_CHARS)),
        counterparty_email: None,
    }
}

// ---------------------------------------------------------------------------
// Multipart
// ---------------------------------------------------------------------------

/// Un fichier du formulaire.
struct UploadedFile {
    filename: String,
    bytes: axum::body::Bytes,
}

struct WiseImportForm {
    files: Vec<UploadedFile>,
    account: String,
}

async fn parse_form(mut multipart: Multipart) -> Result<WiseImportForm, AppError> {
    let mut files: Vec<UploadedFile> = Vec::new();
    let mut account: Option<String> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("multipart : {e}")))?
    {
        match field.name() {
            Some("file") => {
                let name = field.file_name().unwrap_or(DEFAULT_FILENAME).to_string();
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Validation(format!("multipart : {e}")))?;
                let filename = truncate_bytes(name.trim(), FILENAME_MAX_BYTES);
                files.push(UploadedFile {
                    filename: if filename.is_empty() {
                        DEFAULT_FILENAME.into()
                    } else {
                        filename
                    },
                    bytes,
                });
            }
            Some("account") => {
                if account.is_some() {
                    return Err(AppError::Validation("champ `account` en double".into()));
                }
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::Validation(format!("multipart : {e}")))?;
                account = Some(text);
            }
            _ => {}
        }
    }
    if files.is_empty() {
        return Err(AppError::Validation("champ `file` manquant".into()));
    }
    let account = account
        .map(|a| a.trim().to_string())
        .filter(|a| !a.is_empty())
        .ok_or_else(|| AppError::Validation("champ `account` manquant".into()))?;
    Ok(WiseImportForm { files, account })
}

/// Traduit le refus du lecteur (39-11a) en `422 WISE_CSV_INVALID`, le fichier
/// nommé.
fn csv_error(filename: &str, e: WiseCsvError) -> AppError {
    let reason = e.code();
    let details = match &e {
        WiseCsvError::MissingColumns(columns) => serde_json::json!({ "columns": columns }),
        WiseCsvError::DuplicateColumn(column) => serde_json::json!({ "columns": [column] }),
        WiseCsvError::Lines(lines) => serde_json::json!({ "lines": lines }),
        WiseCsvError::Encoding(inner) => serde_json::json!({ "cause": inner.to_string() }),
        WiseCsvError::MixedCurrencies(currencies) => {
            serde_json::json!({ "currencies": currencies })
        }
        WiseCsvError::DuplicateId { id, lines } => serde_json::json!({ "id": id, "lines": lines }),
        WiseCsvError::RunningBalanceBreak {
            line,
            expected,
            found,
        } => serde_json::json!({ "line": line, "expected": expected, "found": found }),
        WiseCsvError::Empty => serde_json::json!({}),
    };
    AppError::WiseCsvInvalid {
        filename: filename.to_string(),
        reason,
        details,
    }
}

/// Le premier couple de fichiers de même devise, dans l'ordre du formulaire
/// (`422 WISE_DUPLICATE_CURRENCY_FILE`, tous les fichiers de cette devise).
fn duplicate_currency(read: &[(String, WiseCsv)]) -> Option<AppError> {
    read.iter().enumerate().find_map(|(i, (_, csv))| {
        let same: Vec<String> = read
            .iter()
            .filter(|(_, other)| other.currency == csv.currency)
            .map(|(name, _)| name.clone())
            .collect();
        let first = read
            .iter()
            .position(|(_, other)| other.currency == csv.currency);
        (same.len() > 1 && first == Some(i)).then(|| AppError::WiseDuplicateCurrencyFile {
            currency: csv.currency.clone(),
            filenames: same,
        })
    })
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// Un fichier lu et son compte bancaire.
struct Target<'a> {
    filename: String,
    csv: WiseCsv,
    balance: &'a ProviderBalance,
    file_hash: String,
}

/// `POST /api/v1/wise-imports` — voir l'en-tête du module.
pub async fn create(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<WiseImportResponse>), AppError> {
    let form = parse_form(multipart).await?;

    let balances = bank_accounts::list_provider_balances(
        &state.pool,
        current_user.company_id,
        PROVIDER_WISE,
        &form.account,
    )
    .await?;
    if balances.is_empty() {
        return Err(AppError::ProviderAccountNotFound {
            account: form.account,
        });
    }

    let mut read: Vec<(String, WiseCsv)> = Vec::with_capacity(form.files.len());
    for file in &form.files {
        let csv = parse_wise_csv(&file.bytes).map_err(|e| csv_error(&file.filename, e))?;
        read.push((file.filename.clone(), csv));
    }
    if let Some(refusal) = duplicate_currency(&read) {
        return Err(refusal);
    }

    let currencies: Vec<String> = read.iter().map(|(_, csv)| csv.currency.clone()).collect();
    let resolved = resolve_currency_balances(&balances, &form.account, &currencies)?;
    let targets: Vec<Target<'_>> = read
        .into_iter()
        .zip(&form.files)
        .zip(resolved)
        .map(|(((filename, csv), file), balance)| Target {
            file_hash: file_currency_hash(&file.bytes, &csv.currency),
            filename,
            csv,
            balance,
        })
        .collect();

    let locked: Vec<LockedTarget> = targets
        .iter()
        .map(|t| LockedTarget {
            currency: t.csv.currency.clone(),
            bank_account_id: t.balance.account.id,
            file_hash: t.file_hash.clone(),
        })
        .collect();
    let audit_entity_id = balances
        .iter()
        .map(|b| b.account.id)
        .min()
        .expect("au moins un compte, contrôlé ci-dessus");
    let writer = WiseWriter {
        current_user: &current_user,
        account: &form.account,
        targets: &targets,
        audit_entity_id,
    };
    let files = run_provider_import(
        &state.pool,
        &current_user,
        PROVIDER_WISE,
        &form.account,
        &locked,
        "wise_imports::create",
        &writer,
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(WiseImportResponse {
            account: form.account,
            files,
        }),
    ))
}

/// Les clés (identifiant, sens) des transactions déjà sur le compte (Story
/// 39-11a2) : le sens d'une transaction en base se lit au signe de son
/// montant (débit ≤ 0, crédit ≥ 0, garanti par le lecteur). Une transaction de
/// montant **nul** vaut pour les deux sens : elle ne fait jamais insérer une
/// seconde ligne, et une ligne nulle ne change pas le solde. Les identifiants
/// sont comparés exactement : la base les rend tels que stockés, et le
/// lecteur les écrit en majuscules.
fn existing_keys(
    existing: &[bank_transactions::ExistingTransaction],
) -> HashSet<(&str, WiseDirection)> {
    let mut keys = HashSet::with_capacity(existing.len());
    for t in existing {
        if t.amount <= Decimal::ZERO {
            keys.insert((t.transaction_id.as_str(), WiseDirection::Debit));
        }
        if t.amount >= Decimal::ZERO {
            keys.insert((t.transaction_id.as_str(), WiseDirection::Credit));
        }
    }
    keys
}

/// L'écriture propre à Wise, dans la transaction du cœur commun : par fichier,
/// doublons lus dans la transaction, continuité jugée avant l'insertion, une
/// entrée `bank_imports` s'il reste au moins une ligne ; puis la trace
/// `wise_import.created`.
struct WiseWriter<'a> {
    current_user: &'a CurrentUser,
    account: &'a str,
    targets: &'a [Target<'a>],
    audit_entity_id: i64,
}

impl ProviderWrite for WiseWriter<'_> {
    type Output = Vec<FileReport>;

    fn write<'a>(
        &'a self,
        tx: &'a mut sqlx::Transaction<'static, sqlx::MySql>,
    ) -> WriteFuture<'a, Self::Output> {
        Box::pin(async move {
            let current_user = self.current_user;
            let company_id = current_user.company_id;
            let mut reports = Vec::with_capacity(self.targets.len());
            for target in self.targets {
                let bank_account_id = target.balance.account.id;
                let csv = &target.csv;
                let ids: Vec<String> = csv.rows.iter().map(|r| r.id.clone()).collect();
                let existing = bank_transactions::existing_transactions_by_ids(
                    tx,
                    company_id,
                    bank_account_id,
                    &ids,
                )
                .await?;
                let existing = existing_keys(&existing);
                let new_txs: Vec<NewBankTransaction> = csv
                    .rows
                    .iter()
                    .filter(|r| !existing.contains(&(r.id.as_str(), r.direction)))
                    .map(|r| transaction_of_row(r, company_id, bank_account_id))
                    .collect();
                let duplicates = csv.rows.len() - new_txs.len();
                let warnings: Vec<FileWarning> = bank_imports::continuity_gap(
                    tx,
                    company_id,
                    bank_account_id,
                    csv.first_date,
                    csv.opening_balance,
                    None,
                )
                .await?
                .into_iter()
                .map(|gap| FileWarning {
                    code: WARNING_GAP,
                    expected: gap.expected,
                    found: gap.found,
                    previous_import_id: gap.previous_import_id,
                })
                .collect();
                let inserted = new_txs.len();
                let import_id = if new_txs.is_empty() {
                    None
                } else {
                    Some(
                        insert_currency_import(
                            tx,
                            current_user,
                            CurrencyImport {
                                provider: PROVIDER_WISE,
                                account: self.account,
                                currency: &csv.currency,
                                bank_account_id,
                                filename: &target.filename,
                                file_hash: &target.file_hash,
                                period_from: csv.first_date,
                                period_to: csv.last_date,
                                opening_balance: Some(csv.opening_balance),
                                closing_balance: Some(csv.closing_balance),
                                transactions: new_txs,
                                duplicates,
                            },
                        )
                        .await?,
                    )
                };
                reports.push(FileReport {
                    filename: target.filename.clone(),
                    currency: csv.currency.clone(),
                    bank_account_id,
                    import_id,
                    inserted,
                    duplicates,
                    opening_balance: csv.opening_balance,
                    closing_balance: csv.closing_balance,
                    warnings,
                });
            }

            audit_log::insert_in_tx(
                tx,
                NewAuditLogEntry::for_actor(
                    current_user.user_id,
                    current_user.api_key_id,
                    "wise_import.created",
                    "bank_account",
                    self.audit_entity_id,
                    Some(serde_json::json!({
                        "provider": PROVIDER_WISE,
                        "provider_account": self.account,
                        "files": reports,
                    })),
                ),
            )
            .await?;
            Ok(reports)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    /// Story 39-11a2 — clés (identifiant, sens) des transactions en base : le
    /// signe du montant donne le sens, un montant nul vaut pour les deux.
    #[test]
    fn existing_keys_read_the_direction_from_the_sign() {
        let t = |id: &str, amount| bank_transactions::ExistingTransaction {
            transaction_id: id.into(),
            amount,
            booking_date: chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        };
        let existing = [
            t("CARD-1", dec!(-32.10)),
            t("CARD-2", dec!(0.50)),
            t("CARD-3", dec!(0.00)),
        ];
        let keys = existing_keys(&existing);
        let has = |id, d| keys.contains(&(id, d));
        assert!(has("CARD-1", WiseDirection::Debit));
        assert!(!has("CARD-1", WiseDirection::Credit));
        assert!(has("CARD-2", WiseDirection::Credit));
        assert!(!has("CARD-2", WiseDirection::Debit));
        assert!(has("CARD-3", WiseDirection::Debit));
        assert!(has("CARD-3", WiseDirection::Credit));
        assert_eq!(keys.len(), 4);
    }

    const CHF: &[u8] =
        include_bytes!("../../../kesh-import/tests/fixtures/wise-releve-chf-fictif.csv");
    const EUR: &[u8] =
        include_bytes!("../../../kesh-import/tests/fixtures/wise-releve-eur-fictif.csv");
    const USD: &[u8] =
        include_bytes!("../../../kesh-import/tests/fixtures/wise-releve-usd-fictif.csv");

    fn rows(bytes: &[u8]) -> Vec<WiseRow> {
        parse_wise_csv(bytes).expect("fixture lisible").rows
    }

    fn row(bytes: &[u8], id: &str) -> WiseRow {
        rows(bytes).into_iter().find(|r| r.id == id).unwrap()
    }

    #[test]
    fn la_forme_d_un_iban() {
        assert_eq!(
            iban_shaped("CH56 0483 5012 3456 7800 9").as_deref(),
            Some("CH5604835012345678009")
        );
        assert_eq!(
            iban_shaped("ch93 0076 2011 6238 5295 7").as_deref(),
            Some("CH9300762011623852957")
        );
        // Numéro national, numéro masqué, trop court, trop long.
        assert_eq!(iban_shaped("0123456789"), None);
        assert_eq!(iban_shaped("12345678"), None);
        assert_eq!(iban_shaped("CH56 **** **** **** 7800 9"), None);
        assert_eq!(
            iban_shaped("CH5604835012345"),
            Some("CH5604835012345".into())
        );
        assert_eq!(iban_shaped("CH560483501234"), None, "14 caractères");
        assert_eq!(
            iban_shaped(&format!("CH56{}", "1".repeat(30))).map(|s| s.len()),
            Some(34)
        );
        assert_eq!(
            iban_shaped(&format!("CH56{}", "1".repeat(31))),
            None,
            "35 caractères"
        );
        assert_eq!(
            iban_shaped("1H5604835012345678009"),
            None,
            "pays non alphabétique"
        );
        assert_eq!(
            iban_shaped("CHX604835012345678009"),
            None,
            "clé non numérique"
        );
    }

    #[test]
    fn un_virement_vers_un_iban() {
        let r = row(CHF, "TRANSFER-90000005");
        let t = transaction_of_row(&r, 7, 9);
        assert_eq!((t.company_id, t.bank_account_id), (7, 9));
        assert_eq!(t.amount, dec!(-120.00));
        assert_eq!(t.currency, "CHF");
        assert_eq!(
            t.booking_date,
            chrono::NaiveDate::from_ymd_opt(2020, 2, 15).unwrap()
        );
        assert_eq!(t.value_date, None);
        assert_eq!(t.transaction_id.as_deref(), Some("TRANSFER-90000005"));
        assert_eq!(t.reference.as_deref(), Some("Facture 2020-001"));
        assert_eq!(t.end_to_end_id, None);
        assert_eq!(t.counterparty_name.as_deref(), Some("Atelier Exemple Sàrl"));
        assert_eq!(
            t.counterparty_iban.as_deref(),
            Some("CH5604835012345678009")
        );
        assert_eq!(t.counterparty_email, None);
        assert_eq!(t.provider_type.as_deref(), Some("TRANSFER"));
        assert_eq!(
            t.details,
            "Envoi de 120\u{202f}CHF à Atelier Exemple Sàrl · TRANSFER"
        );
    }

    #[test]
    fn un_numero_de_compte_non_iban_va_au_libelle() {
        let t = transaction_of_row(&row(CHF, "TRANSFER-90000006"), 1, 1);
        assert_eq!(t.counterparty_iban, None);
        assert_eq!(
            t.details,
            "Envoi vers Association Fictive · TRANSFER · 0123456789"
        );
        assert_eq!(t.reference, None);
    }

    #[test]
    fn contrepartie_marchand_payeur_et_frais() {
        let carte = transaction_of_row(&row(CHF, "CARD-90000007"), 1, 1);
        assert_eq!(carte.counterparty_name.as_deref(), Some("Boutique Exemple"));
        assert_eq!(carte.provider_type.as_deref(), Some("CARD"));
        let recu = transaction_of_row(&row(CHF, "BALANCE-90000008"), 1, 1);
        assert_eq!(
            recu.counterparty_name.as_deref(),
            Some("Stripe Payments Europe")
        );
        assert_eq!(recu.amount, dec!(1500.00));
        let frais = transaction_of_row(&row(CHF, "FEE-TRANSFER-90000004"), 1, 1);
        assert_eq!(frais.amount, dec!(-1.20));
        assert_eq!(frais.provider_type.as_deref(), Some("FEE-TRANSFER"));
        assert_eq!(frais.counterparty_name, None);
        // `Total fees` (1.20) du virement n'est jamais une transaction : le
        // virement garde son montant.
        let virement = transaction_of_row(&row(CHF, "TRANSFER-90000004"), 1, 1);
        assert_eq!(virement.amount, dec!(-300.00));
    }

    #[test]
    fn une_conversion_porte_son_montant_et_son_cours() {
        let t = transaction_of_row(&row(EUR, "CONVERSION_ORDER-90000011"), 1, 1);
        assert_eq!(
            t.details,
            "Converti 100.00 EUR en 109.00 USD · CONVERSION · EUR→USD 109.00 au cours 1.09"
        );
        assert_eq!(t.provider_type.as_deref(), Some("CONVERSION_ORDER"));
    }

    /// Revue P1, M1 — le montant de l'autre devise (`Exchange To Amount`)
    /// est écrit au segment de conversion ; sans lui ni cours, le segment
    /// reste « DE→VERS ».
    #[test]
    fn le_segment_de_conversion_a_une_forme_fixe() {
        let mut r = row(EUR, "CONVERSION_ORDER-90000011");
        r.exchange_to_amount = Some(dec!(41.85));
        r.exchange_rate = Some(dec!(0.9329));
        let t = transaction_of_row(&r, 1, 1);
        assert!(t.details.contains(" · EUR→USD 41.85 au cours 0.9329"));
        r.exchange_rate = None;
        let t = transaction_of_row(&r, 1, 1);
        assert!(t.details.ends_with(" · EUR→USD 41.85"), "{}", t.details);
        r.exchange_to_amount = None;
        let t = transaction_of_row(&r, 1, 1);
        assert!(t.details.ends_with(" · EUR→USD"), "{}", t.details);
    }

    /// AC-4 — la somme des transactions d'un fichier est sa clôture moins son
    /// ouverture.
    #[test]
    fn la_somme_d_un_fichier_est_cloture_moins_ouverture() {
        for bytes in [CHF, EUR, USD] {
            let csv = parse_wise_csv(bytes).unwrap();
            let somme: Decimal = csv
                .rows
                .iter()
                .map(|r| transaction_of_row(r, 1, 1).amount)
                .sum();
            assert_eq!(
                somme,
                csv.closing_balance - csv.opening_balance,
                "{}",
                csv.currency
            );
        }
    }

    /// AC-4 — le libellé se tronque à la frontière d'un caractère (U+202F tient
    /// sur trois octets).
    #[test]
    fn libelle_et_contrepartie_bornes() {
        let mut r = row(CHF, "TRANSFER-90000005");
        r.note = Some("\u{202f}é".repeat(400));
        r.payee_name = Some("n".repeat(300));
        let t = transaction_of_row(&r, 1, 1);
        assert_eq!(t.details.chars().count(), DETAILS_MAX_CHARS);
        assert_eq!(t.counterparty_name.unwrap().chars().count(), NAME_MAX_CHARS);
    }

    /// Revue P2, L1 — une très longue description ne rogne pas le segment de
    /// conversion (montant et cours) ; la note, en queue, est rognée la première.
    #[test]
    fn une_longue_description_ne_rogne_pas_la_conversion() {
        let mut r = row(EUR, "CONVERSION_ORDER-90000011");
        r.description = Some("d".repeat(800));
        r.note = Some("n".repeat(300));
        let t = transaction_of_row(&r, 1, 1);
        assert!(t.details.chars().count() <= DETAILS_MAX_CHARS);
        assert!(
            t.details.contains(" · EUR→USD 109.00 au cours 1.09"),
            "{}",
            t.details
        );
    }

    #[test]
    fn deux_fichiers_de_meme_devise() {
        let read = vec![
            ("a.csv".to_string(), parse_wise_csv(CHF).unwrap()),
            ("b.csv".to_string(), parse_wise_csv(EUR).unwrap()),
            ("c.csv".to_string(), parse_wise_csv(CHF).unwrap()),
        ];
        match duplicate_currency(&read) {
            Some(AppError::WiseDuplicateCurrencyFile {
                currency,
                filenames,
            }) => {
                assert_eq!(currency, "CHF");
                assert_eq!(filenames, ["a.csv", "c.csv"]);
            }
            other => panic!("refus attendu : {other:?}"),
        }
        assert!(duplicate_currency(&read[..2]).is_none());
    }
}
