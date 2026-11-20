//! Lecteur du CSV « relevé » de Wise (Story 39-11a).
//!
//! Wise exporte, pour **un solde** (une devise) et une période, un CSV UTF-8 à
//! virgule et guillemets doubles, de 23 colonnes **en anglais**, la ligne la
//! plus récente en tête. Ce module le lit **par noms de colonnes**
//! ([`crate::csv::ColumnsByName`]), contrôle chaque ligne et rend un
//! [`WiseCsv`] aux lignes en ordre chronologique. Il ne classe rien : conversions,
//! virements, versements Stripe sont l'affaire du rapprochement (Story 39-11c).
//!
//! Garanties, chacune refusant le fichier **entier** (pas d'import partiel) :
//!
//! - les colonnes requises ([`REQUIRED_COLUMNS`]) sont toutes là, sinon
//!   [`WiseCsvError::MissingColumns`] les nomme toutes ;
//! - les montants sont à forme fermée ([`crate::csv::parse_dot_decimal_amount`]) ;
//! - le fichier ne porte qu'une devise, aucun identifiant deux fois ;
//! - le **solde courant se suit** : chaque `Running Balance` est le précédent
//!   plus le montant de la ligne ([`WiseCsvError::RunningBalanceBreak`]).
//!
//! L'horodatage est rendu tel quel, **sans conversion de fuseau**. Les colonnes
//! de carte (`Card Last Four Digits`, `Card Holder Full Name`) et
//! `Attachment` ne sont jamais lues.
//!
//! Fonction pure, sans I/O, sans dépendance interne au workspace.

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;

use crate::csv::encoding::{decode_bytes, detect_encoding};
use crate::csv::{ColumnsByName, HeaderByNameError, parse_dot_decimal_amount, read_records};
use crate::error::{CsvError, MAX_CSV_LINE_ERRORS, truncate_line_error_value};

/// Colonne `TransferWise ID`.
pub const COL_ID: &str = "TransferWise ID";
/// Colonne `Date`.
pub const COL_DATE: &str = "Date";
/// Colonne `Date Time`.
pub const COL_DATE_TIME: &str = "Date Time";
/// Colonne `Amount`.
pub const COL_AMOUNT: &str = "Amount";
/// Colonne `Currency`.
pub const COL_CURRENCY: &str = "Currency";
/// Colonne `Description`.
pub const COL_DESCRIPTION: &str = "Description";
/// Colonne `Payment Reference`.
pub const COL_PAYMENT_REFERENCE: &str = "Payment Reference";
/// Colonne `Running Balance`.
pub const COL_RUNNING_BALANCE: &str = "Running Balance";
/// Colonne `Exchange From`.
pub const COL_EXCHANGE_FROM: &str = "Exchange From";
/// Colonne `Exchange To`.
pub const COL_EXCHANGE_TO: &str = "Exchange To";
/// Colonne `Exchange Rate`.
pub const COL_EXCHANGE_RATE: &str = "Exchange Rate";
/// Colonne `Payer Name`.
pub const COL_PAYER_NAME: &str = "Payer Name";
/// Colonne `Payee Name`.
pub const COL_PAYEE_NAME: &str = "Payee Name";
/// Colonne `Payee Account Number`.
pub const COL_PAYEE_ACCOUNT: &str = "Payee Account Number";
/// Colonne `Merchant`.
pub const COL_MERCHANT: &str = "Merchant";
/// Colonne `Note`.
pub const COL_NOTE: &str = "Note";
/// Colonne `Total fees`.
pub const COL_TOTAL_FEES: &str = "Total fees";
/// Colonne `Exchange To Amount`.
pub const COL_EXCHANGE_TO_AMOUNT: &str = "Exchange To Amount";
/// Colonne `Transaction Type`.
pub const COL_TRANSACTION_TYPE: &str = "Transaction Type";
/// Colonne `Transaction Details Type`.
pub const COL_DETAILS_TYPE: &str = "Transaction Details Type";

/// Colonnes requises, dans l'ordre où un refus les nomme.
pub const REQUIRED_COLUMNS: [&str; 8] = [
    COL_ID,
    COL_DATE,
    COL_AMOUNT,
    COL_CURRENCY,
    COL_DESCRIPTION,
    COL_RUNNING_BALANCE,
    COL_TRANSACTION_TYPE,
    COL_DETAILS_TYPE,
];

/// Colonnes lues si présentes ; absentes, elles se lisent vides.
pub const OPTIONAL_COLUMNS: [&str; 12] = [
    COL_DATE_TIME,
    COL_PAYMENT_REFERENCE,
    COL_EXCHANGE_FROM,
    COL_EXCHANGE_TO,
    COL_EXCHANGE_RATE,
    COL_EXCHANGE_TO_AMOUNT,
    COL_PAYER_NAME,
    COL_PAYEE_NAME,
    COL_PAYEE_ACCOUNT,
    COL_MERCHANT,
    COL_TOTAL_FEES,
    COL_NOTE,
];

/// Longueur maximale d'un identifiant Wise accepté.
const MAX_ID_LEN: usize = 64;

/// Sens d'une ligne sur le solde (colonne `Transaction Type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WiseDirection {
    /// Entrée (`CREDIT`) : montant positif ou nul.
    Credit,
    /// Sortie (`DEBIT`) : montant négatif ou nul.
    Debit,
}

/// Une ligne du CSV Wise, contrôlée.
///
/// Les champs texte facultatifs valent `None` quand la cellule est vide après
/// `trim` ; sinon le texte est gardé tel quel (espaces fines insécables
/// U+202F comprises).
#[derive(Debug, Clone, PartialEq)]
pub struct WiseRow {
    /// Numéro de ligne dans le fichier (1 = l'en-tête).
    pub line: usize,
    /// Identifiant Wise, en majuscules (`TRANSFER-…`, `FEE-BALANCE-…`).
    pub id: String,
    /// Partie de l'identifiant avant le premier chiffre, sans tiret final
    /// (`TRANSFER`, `CARD`, `FEE-TRANSFER`, `CONVERSION_ORDER`…).
    pub id_prefix: String,
    /// L'identifiant commence par `FEE-` : ligne de frais.
    pub is_fee: bool,
    /// Pour une ligne de frais, l'identifiant de la ligne d'origine
    /// (l'identifiant sans `FEE-`).
    pub fee_of: Option<String>,
    /// Date de la ligne.
    pub date: NaiveDate,
    /// Horodatage, sans conversion de fuseau.
    pub date_time: Option<NaiveDateTime>,
    /// Montant signé.
    pub amount: Decimal,
    /// Code de devise, trois lettres ASCII majuscules.
    pub currency: String,
    /// Description.
    pub description: Option<String>,
    /// Référence de paiement.
    pub payment_reference: Option<String>,
    /// Solde courant après la ligne.
    pub running_balance: Decimal,
    /// Nom du payeur.
    pub payer_name: Option<String>,
    /// Nom du bénéficiaire.
    pub payee_name: Option<String>,
    /// Numéro de compte du bénéficiaire (IBAN ou autre).
    pub payee_account: Option<String>,
    /// Marchand d'un paiement par carte.
    pub merchant: Option<String>,
    /// Devise source d'une conversion.
    pub exchange_from: Option<String>,
    /// Devise cible d'une conversion.
    pub exchange_to: Option<String>,
    /// Cours d'une conversion.
    pub exchange_rate: Option<Decimal>,
    /// Montant reçu dans la devise cible d'une conversion.
    pub exchange_to_amount: Option<Decimal>,
    /// Frais indiqués sur la ligne (jamais comptés : les frais sont des
    /// lignes à part).
    pub total_fees: Option<Decimal>,
    /// Sens sur le solde.
    pub direction: WiseDirection,
    /// Type de détail (`TRANSFER`, `CARD`, `CONVERSION`, `DEPOSIT`…), tel
    /// qu'écrit.
    pub details_type: String,
    /// Note.
    pub note: Option<String>,
}

/// Le relevé lu : ses lignes en ordre chronologique et ses soldes.
#[derive(Debug, Clone, PartialEq)]
pub struct WiseCsv {
    /// Devise unique du fichier.
    pub currency: String,
    /// Les lignes, de la plus ancienne à la plus récente.
    pub rows: Vec<WiseRow>,
    /// Solde avant la première ligne : premier solde courant moins premier
    /// montant.
    pub opening_balance: Decimal,
    /// Solde après la dernière ligne.
    pub closing_balance: Decimal,
    /// Date la plus ancienne.
    pub first_date: NaiveDate,
    /// Date la plus récente.
    pub last_date: NaiveDate,
}

/// Code stable d'une erreur de ligne (sérialisé en `SCREAMING_SNAKE_CASE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WiseLineErrorCode {
    /// La ligne n'a pas autant de cellules que l'en-tête.
    FieldCount,
    /// Date illisible, hors format ou hors plage d'années.
    Date,
    /// Horodatage illisible.
    DateTime,
    /// Montant hors de la forme fermée (ou vide pour `Amount`, `Running
    /// Balance`).
    Amount,
    /// Cours hors de la forme `^\d+(\.\d{1,10})?$`.
    Rate,
    /// Devise autre que trois lettres ASCII.
    Currency,
    /// Identifiant vide, hors `[A-Z0-9_-]` ou trop long.
    Id,
    /// `Transaction Type` autre que `CREDIT`, `DEBIT`.
    Direction,
    /// Montant négatif sur un crédit, ou positif sur un débit.
    SignMismatch,
    /// Ligne de frais créditée.
    FeeSign,
    /// Un contrôle de solde dépasse la capacité d'un `Decimal`.
    Overflow,
    /// Cellule requise vide (`Transaction Details Type`).
    Required,
}

impl WiseLineErrorCode {
    /// Graphie stable du code, identique à sa sérialisation serde.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FieldCount => "FIELD_COUNT",
            Self::Date => "DATE",
            Self::DateTime => "DATE_TIME",
            Self::Amount => "AMOUNT",
            Self::Rate => "RATE",
            Self::Currency => "CURRENCY",
            Self::Id => "ID",
            Self::Direction => "DIRECTION",
            Self::SignMismatch => "SIGN_MISMATCH",
            Self::FeeSign => "FEE_SIGN",
            Self::Overflow => "OVERFLOW",
            Self::Required => "REQUIRED",
        }
    }
}

/// Une erreur de ligne, nommant la ligne, le champ et la valeur fautive.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct WiseLineError {
    /// Numéro de ligne dans le fichier (1 = l'en-tête).
    pub line: usize,
    /// Nom de la colonne en cause ; `None` pour [`WiseLineErrorCode::FieldCount`].
    pub field: Option<&'static str>,
    /// Code stable.
    pub code: WiseLineErrorCode,
    /// Valeur fautive, tronquée à [`crate::error::MAX_CSV_LINE_ERROR_VALUE_CHARS`]
    /// caractères (suffixe `…`).
    pub value: Option<String>,
}

impl WiseLineError {
    fn new(
        line: usize,
        field: Option<&'static str>,
        code: WiseLineErrorCode,
        value: Option<&str>,
    ) -> Self {
        Self {
            line,
            field,
            code,
            value: value.map(truncate_line_error_value),
        }
    }
}

/// Refus du fichier Wise. Type fermé : chaque variante a un code stable
/// ([`WiseCsvError::code`]).
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum WiseCsvError {
    /// Fichier vide, ou en-tête sans ligne de données.
    #[error("fichier Wise vide")]
    Empty,
    /// Encodage non reconnu ou décodage impossible.
    #[error("encodage du fichier Wise : {0}")]
    Encoding(CsvError),
    /// Colonnes requises absentes, toutes, dans l'ordre de [`REQUIRED_COLUMNS`].
    #[error("colonnes Wise requises absentes : {}", .0.join(", "))]
    MissingColumns(Vec<String>),
    /// Une colonne lue apparaît deux fois dans l'en-tête.
    #[error("colonne Wise en double : {0}")]
    DuplicateColumn(String),
    /// Le fichier porte plus d'une devise (les devises, dans l'ordre de leur
    /// première apparition dans le fichier).
    #[error("plusieurs devises dans le fichier Wise : {}", .0.join(", "))]
    MixedCurrencies(Vec<String>),
    /// Un identifiant apparaît sur plusieurs lignes du fichier.
    #[error("identifiant Wise en double : {id}")]
    DuplicateId {
        /// L'identifiant.
        id: String,
        /// Ses lignes, dans l'ordre du fichier.
        lines: Vec<usize>,
    },
    /// Le solde courant d'une ligne n'est pas celui d'avant plus son montant.
    #[error("solde courant discontinu à la ligne {line}")]
    RunningBalanceBreak {
        /// Ligne du fichier où le solde ne se suit pas.
        line: usize,
        /// Solde attendu (précédent + montant).
        expected: Decimal,
        /// Solde écrit.
        found: Decimal,
    },
    /// Erreurs de ligne, au plus [`MAX_CSV_LINE_ERRORS`] (la collecte s'arrête
    /// au plafond).
    #[error("{} ligne(s) Wise en erreur", .0.len())]
    Lines(Vec<WiseLineError>),
}

impl WiseCsvError {
    /// Code stable de la variante (placé par l'import dans `details`).
    pub fn code(&self) -> &'static str {
        match self {
            Self::Empty => "WISE_CSV_EMPTY",
            Self::Encoding(_) => "WISE_CSV_ENCODING",
            Self::MissingColumns(_) => "WISE_CSV_MISSING_COLUMNS",
            Self::DuplicateColumn(_) => "WISE_CSV_DUPLICATE_COLUMN",
            Self::MixedCurrencies(_) => "WISE_CSV_MIXED_CURRENCIES",
            Self::DuplicateId { .. } => "WISE_CSV_DUPLICATE_ID",
            Self::RunningBalanceBreak { .. } => "WISE_CSV_RUNNING_BALANCE_BREAK",
            Self::Lines(_) => "WISE_CSV_LINES",
        }
    }
}

/// Collecte plafonnée des erreurs de ligne.
#[derive(Default)]
struct Errors(Vec<WiseLineError>);

impl Errors {
    fn push(&mut self, e: WiseLineError) {
        if self.0.len() < MAX_CSV_LINE_ERRORS {
            self.0.push(e);
        }
    }
}

/// `jj-mm-aaaa` ou `aaaa-mm-jj`, chiffres ASCII, date existante, année dans
/// la plage de la base.
fn parse_date(raw: &str) -> Option<NaiveDate> {
    let parts: Vec<&str> = raw.split('-').collect();
    let [a, b, c] = parts.as_slice() else {
        return None;
    };
    let digits = |s: &str, len: usize| s.len() == len && s.bytes().all(|x| x.is_ascii_digit());
    let (y, m, d) = if digits(a, 4) && digits(b, 2) && digits(c, 2) {
        (a, b, c)
    } else if digits(a, 2) && digits(b, 2) && digits(c, 4) {
        (c, b, a)
    } else {
        return None;
    };
    NaiveDate::from_ymd_opt(y.parse().ok()?, m.parse().ok()?, d.parse().ok()?)
        .filter(|date| crate::annee_recevable(*date))
}

/// `jj-mm-aaaa hh:mm:ss`, avec ou sans `.fff`, ou la même heure après une
/// date `aaaa-mm-jj` ; sans conversion de fuseau.
fn parse_date_time(raw: &str) -> Option<NaiveDateTime> {
    let (date, time) = raw.split_once(' ')?;
    let date = parse_date(date)?;
    let (hms, frac) = match time.split_once('.') {
        Some((h, f)) => (h, Some(f)),
        None => (time, None),
    };
    if let Some(f) = frac
        && !(1..=9).contains(&f.len()) | !f.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let parsed = chrono::NaiveTime::parse_from_str(hms, "%H:%M:%S").ok()?;
    if hms.len() != 8 {
        return None;
    }
    let nanos = match frac {
        Some(f) => format!("{f:0<9}").parse::<u32>().ok()?,
        None => 0,
    };
    let time = chrono::NaiveTime::from_hms_nano_opt(
        chrono::Timelike::hour(&parsed),
        chrono::Timelike::minute(&parsed),
        chrono::Timelike::second(&parsed),
        nanos,
    )?;
    Some(date.and_time(time))
}

/// Cours : chiffres, puis au plus dix décimales au point.
fn parse_rate(raw: &str) -> Option<Decimal> {
    let (int, frac) = match raw.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (raw, None),
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if !digits(int) || frac.is_some_and(|f| !digits(f) || f.len() > 10) {
        return None;
    }
    Decimal::from_str_exact(raw).ok()
}

/// Identifiant : `trim` fait par la cellule, majuscules, `[A-Z0-9_-]`, au plus
/// [`MAX_ID_LEN`] caractères.
fn parse_id(raw: &str) -> Option<String> {
    let id = raw.to_ascii_uppercase();
    (id.len() <= MAX_ID_LEN
        && id
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_' || b == b'-'))
    .then_some(id)
}

/// Partie de l'identifiant avant le premier chiffre, sans tiret final.
fn id_prefix(id: &str) -> String {
    let end = id.find(|c: char| c.is_ascii_digit()).unwrap_or(id.len());
    id[..end].trim_end_matches('-').to_string()
}

/// Lit le CSV « relevé » de Wise.
///
/// Décodage UTF-8 (BOM retranché) ou ISO-8859-1, en-tête par noms, contrôles
/// de chaque ligne puis du fichier ; une seule erreur refuse le fichier entier.
/// Voir le module pour les règles.
///
/// # Erreurs
///
/// [`WiseCsvError`], dont [`WiseCsvError::Lines`] agrège les erreurs de ligne
/// jusqu'à [`MAX_CSV_LINE_ERRORS`].
pub fn parse_wise_csv(bytes: &[u8]) -> Result<WiseCsv, WiseCsvError> {
    let (encoding, bom_len) = detect_encoding(bytes).map_err(|e| match e {
        CsvError::EmptyFile { .. } => WiseCsvError::Empty,
        other => WiseCsvError::Encoding(other),
    })?;
    let text = decode_bytes(&bytes[bom_len..], encoding).map_err(WiseCsvError::Encoding)?;

    let raw = read_records(&text);
    let mut errors = Errors::default();
    for (line, value) in &raw.malformed {
        errors.push(WiseLineError::new(
            *line,
            None,
            WiseLineErrorCode::FieldCount,
            Some(value),
        ));
    }
    let Some(header) = raw.header else {
        return Err(WiseCsvError::Empty);
    };
    let columns = ColumnsByName::locate(header.iter(), &REQUIRED_COLUMNS, &OPTIONAL_COLUMNS)
        .map_err(|e| match e {
            HeaderByNameError::Missing(names) => WiseCsvError::MissingColumns(names),
            HeaderByNameError::Duplicate(name) => WiseCsvError::DuplicateColumn(name),
        })?;
    if raw.records.is_empty() && errors.0.is_empty() {
        return Err(WiseCsvError::Empty);
    }

    let mut rows = Vec::with_capacity(raw.records.len());
    for (line, record) in &raw.records {
        if let Some(row) = read_row(*line, record, &columns, &mut errors) {
            rows.push(row);
        }
    }
    if !errors.0.is_empty() {
        errors.0.sort_by_key(|e| e.line);
        return Err(WiseCsvError::Lines(errors.0));
    }

    check_single_currency(&rows)?;
    check_unique_ids(&rows)?;
    let rows = chronological(rows);
    check_running_balance(&rows)?;

    let (first, last) = (&rows[0], &rows[rows.len() - 1]);
    let opening_balance = first
        .running_balance
        .checked_sub(first.amount)
        .ok_or_else(|| overflow(first))?;
    Ok(WiseCsv {
        currency: first.currency.clone(),
        opening_balance,
        closing_balance: last.running_balance,
        first_date: rows.iter().map(|r| r.date).min().unwrap_or(first.date),
        last_date: rows.iter().map(|r| r.date).max().unwrap_or(last.date),
        rows,
    })
}

/// Toutes les lignes ont la même devise.
fn check_single_currency(rows: &[WiseRow]) -> Result<(), WiseCsvError> {
    let mut currencies: Vec<String> = Vec::new();
    for row in rows {
        if !currencies.contains(&row.currency) {
            currencies.push(row.currency.clone());
        }
    }
    if currencies.len() > 1 {
        return Err(WiseCsvError::MixedCurrencies(currencies));
    }
    Ok(())
}

/// Aucun identifiant sur deux lignes ; le premier doublon dans l'ordre du
/// fichier est cité avec toutes ses lignes.
fn check_unique_ids(rows: &[WiseRow]) -> Result<(), WiseCsvError> {
    for (i, row) in rows.iter().enumerate() {
        if rows[..i].iter().any(|r| r.id == row.id) {
            let lines = rows
                .iter()
                .filter(|r| r.id == row.id)
                .map(|r| r.line)
                .collect();
            return Err(WiseCsvError::DuplicateId {
                id: row.id.clone(),
                lines,
            });
        }
    }
    Ok(())
}

/// Ordre chronologique : sur l'horodatage lu quand **toutes** les lignes en
/// ont un, à égalité (et dès qu'une seule en manque) sur l'ordre **inverse**
/// du fichier (Wise écrit la plus récente en tête ; une ligne de frais
/// précède d'une milliseconde sa ligne d'origine).
fn chronological(mut rows: Vec<WiseRow>) -> Vec<WiseRow> {
    rows.reverse();
    if rows.iter().all(|r| r.date_time.is_some()) {
        // Tri stable : les égalités gardent l'ordre inverse du fichier.
        rows.sort_by_key(|r| r.date_time);
    }
    rows
}

/// Refus d'une ligne dont un contrôle de solde déborde d'un `Decimal`.
fn overflow(row: &WiseRow) -> WiseCsvError {
    WiseCsvError::Lines(vec![WiseLineError::new(
        row.line,
        Some(COL_RUNNING_BALANCE),
        WiseLineErrorCode::Overflow,
        Some(&row.running_balance.to_string()),
    )])
}

/// Chaque solde courant est le précédent plus le montant de la ligne.
fn check_running_balance(rows: &[WiseRow]) -> Result<(), WiseCsvError> {
    for pair in rows.windows(2) {
        let expected = pair[0]
            .running_balance
            .checked_add(pair[1].amount)
            .ok_or_else(|| overflow(&pair[1]))?;
        if pair[1].running_balance != expected {
            return Err(WiseCsvError::RunningBalanceBreak {
                line: pair[1].line,
                expected,
                found: pair[1].running_balance,
            });
        }
    }
    Ok(())
}

/// Lit et contrôle une ligne ; rend `None` (erreurs poussées) si elle est
/// fautive.
fn read_row(
    line: usize,
    record: &csv::StringRecord,
    columns: &ColumnsByName,
    errors: &mut Errors,
) -> Option<WiseRow> {
    let mut push = |field, code, value: Option<&str>| {
        errors.push(WiseLineError::new(line, Some(field), code, value));
    };
    let cell = |name| columns.cell(record, name);
    let text = |name| cell(name).map(str::to_string);

    let raw_id = cell(COL_ID);
    let id = raw_id.and_then(parse_id);
    if id.is_none() {
        push(COL_ID, WiseLineErrorCode::Id, raw_id);
    }

    let raw_date = cell(COL_DATE);
    let date = raw_date.and_then(parse_date);
    if date.is_none() {
        push(COL_DATE, WiseLineErrorCode::Date, raw_date);
    }

    // `Date Time` est facultatif : vide, il vaut `None` sans erreur.
    let raw_date_time = cell(COL_DATE_TIME);
    let date_time = raw_date_time.map(parse_date_time);
    if matches!(date_time, Some(None)) {
        push(COL_DATE_TIME, WiseLineErrorCode::DateTime, raw_date_time);
    }

    let raw_currency = cell(COL_CURRENCY);
    let currency = raw_currency
        .filter(|c| c.len() == 3 && c.bytes().all(|b| b.is_ascii_alphabetic()))
        .map(str::to_ascii_uppercase);
    if currency.is_none() {
        push(COL_CURRENCY, WiseLineErrorCode::Currency, raw_currency);
    }

    let raw_direction = cell(COL_TRANSACTION_TYPE);
    let direction = match raw_direction.map(str::to_ascii_uppercase).as_deref() {
        Some("CREDIT") => Some(WiseDirection::Credit),
        Some("DEBIT") => Some(WiseDirection::Debit),
        _ => None,
    };
    if direction.is_none() {
        push(
            COL_TRANSACTION_TYPE,
            WiseLineErrorCode::Direction,
            raw_direction,
        );
    }

    let details_type = text(COL_DETAILS_TYPE);
    if details_type.is_none() {
        push(COL_DETAILS_TYPE, WiseLineErrorCode::Required, None);
    }

    // Montants : obligatoires (`Amount`, `Running Balance`) ou facultatifs
    // (vides = `None`, illisibles = erreur).
    let mut amount = |name: &'static str| -> Option<Decimal> {
        let raw = cell(name);
        let value = raw.and_then(parse_dot_decimal_amount);
        if value.is_none() {
            push(name, WiseLineErrorCode::Amount, raw);
        }
        value
    };
    let amount_value = amount(COL_AMOUNT);
    let running_balance = amount(COL_RUNNING_BALANCE);
    let to_amount = cell(COL_EXCHANGE_TO_AMOUNT).map(|_| amount(COL_EXCHANGE_TO_AMOUNT));
    let total_fees = cell(COL_TOTAL_FEES).map(|_| amount(COL_TOTAL_FEES));

    let raw_rate = cell(COL_EXCHANGE_RATE);
    let exchange_rate = raw_rate.map(parse_rate);
    if matches!(exchange_rate, Some(None)) {
        push(COL_EXCHANGE_RATE, WiseLineErrorCode::Rate, raw_rate);
    }
    let optional_ok = !matches!(to_amount, Some(None))
        && !matches!(total_fees, Some(None))
        && !matches!(exchange_rate, Some(None))
        && !matches!(date_time, Some(None));

    let is_fee = id.as_deref().is_some_and(|i| i.starts_with("FEE-"));
    let mut incoherent = false;
    if let Some(value) = amount_value {
        let mismatch = match direction {
            Some(WiseDirection::Credit) => value < Decimal::ZERO,
            Some(WiseDirection::Debit) => value > Decimal::ZERO,
            None => false,
        };
        if mismatch {
            incoherent = true;
            push(
                COL_AMOUNT,
                WiseLineErrorCode::SignMismatch,
                cell(COL_AMOUNT),
            );
        } else if is_fee && value > Decimal::ZERO {
            incoherent = true;
            push(COL_AMOUNT, WiseLineErrorCode::FeeSign, cell(COL_AMOUNT));
        }
    }
    if incoherent || !optional_ok {
        return None;
    }

    let id = id?;
    Some(WiseRow {
        line,
        id_prefix: id_prefix(&id),
        is_fee,
        fee_of: id.strip_prefix("FEE-").map(str::to_string),
        id,
        date: date?,
        date_time: date_time.flatten(),
        amount: amount_value?,
        currency: currency?,
        description: text(COL_DESCRIPTION),
        payment_reference: text(COL_PAYMENT_REFERENCE),
        running_balance: running_balance?,
        payer_name: text(COL_PAYER_NAME),
        payee_name: text(COL_PAYEE_NAME),
        payee_account: text(COL_PAYEE_ACCOUNT),
        merchant: text(COL_MERCHANT),
        exchange_from: text(COL_EXCHANGE_FROM),
        exchange_to: text(COL_EXCHANGE_TO),
        exchange_rate: exchange_rate.flatten(),
        exchange_to_amount: to_amount.flatten(),
        total_fees: total_fees.flatten(),
        direction: direction?,
        details_type: details_type?,
        note: text(COL_NOTE),
    })
}
