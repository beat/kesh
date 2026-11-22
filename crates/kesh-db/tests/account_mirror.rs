//! Story 47-1 — le compte miroir au dépôt : pose, changement et retrait
//! (`accounts::create`, `update`), règles de validité (AC-4), garde du compte
//! miroir (AC-5), aucune écriture sur un miroir au point de passage des lignes
//! (AC-6), concurrence (AC-7), lecteur des liens (AC-8) et aucune écriture
//! comptable produite par le lien (AC-12).

use std::time::Duration;

use kesh_db::entities::account::{
    Account, AccountRole, AccountType, AccountUpdate, CurrencyChange, MirrorChange, NewAccount,
    OwnFxAccountsChange,
};
use kesh_db::entities::{Acteur, Journal, NewJournalEntry};
use kesh_db::errors::{AccountIsMirrorReason, DbError, MirrorReason, MirrorSourceAccount};
use kesh_db::repositories::{accounts, journal_entries};
use kesh_db::test_fixtures::{
    SeededCompany, attendre_une_requete_en_cours, chf_line, seed_accounting_company,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

const ATTENTE: Duration = Duration::from_secs(20);

/// Le texte de la requête par laquelle une pose verrouille la cible du miroir
/// (`lock_mirror_target_in_tx`) ; la lecture de la source n'a pas `company_id`.
const CIBLE_DE_LA_POSE: &str = "FROM accounts WHERE id = ? AND company_id = ? FOR UPDATE";
/// Le texte de la requête par laquelle l'archivage verrouille le compte
/// (`archive_guarded`).
const CIBLE_DE_L_ARCHIVAGE: &str = "SELECT company_id FROM accounts WHERE id = ? FOR UPDATE";

// ---------------------------------------------------------------------------
// Montage
// ---------------------------------------------------------------------------

/// Un compte écrit en SQL direct : type, imputabilité, rôle.
async fn compte(
    pool: &MySqlPool,
    company_id: i64,
    numero: &str,
    type_: &str,
    postable: bool,
    role: Option<&str>,
) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, postable, role) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(numero)
    .bind(format!("Compte {numero}"))
    .bind(type_)
    .bind(postable)
    .bind(role)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Une source (passif imputable) et une cible de présentation (actif non
/// imputable, sans rôle, sans écriture) : le cas des dettes sociales.
async fn paire(pool: &MySqlPool, s: &SeededCompany) -> (i64, i64) {
    let source = compte(pool, s.company_id, "2270", "Liability", true, None).await;
    let cible = compte(pool, s.company_id, "1179", "Asset", false, None).await;
    (source, cible)
}

async fn lire(pool: &MySqlPool, id: i64) -> Account {
    accounts::find_by_id(pool, id).await.unwrap().unwrap()
}

fn tel_quel(a: &Account) -> AccountUpdate {
    AccountUpdate {
        name: a.name.clone(),
        account_type: a.account_type,
        role: a.role,
        postable: a.postable,
        currency: CurrencyChange::Unchanged,
        own_fx_accounts: OwnFxAccountsChange::Unchanged,
        mirror: MirrorChange::Unchanged,
        parent: Default::default(),
    }
}

async fn modifier(
    pool: &MySqlPool,
    s: &SeededCompany,
    id: i64,
    f: impl FnOnce(&mut AccountUpdate),
) -> Result<Account, DbError> {
    let a = lire(pool, id).await;
    let mut changes = tel_quel(&a);
    f(&mut changes);
    accounts::update(pool, id, a.version, s.admin_user_id, changes, true).await
}

async fn poser(
    pool: &MySqlPool,
    s: &SeededCompany,
    source: i64,
    cible: i64,
) -> Result<Account, DbError> {
    modifier(pool, s, source, |u| u.mirror = MirrorChange::Set(cible)).await
}

async fn archiver(pool: &MySqlPool, s: &SeededCompany, id: i64) -> Result<Account, DbError> {
    let a = lire(pool, id).await;
    accounts::archive_guarded(pool, id, a.version, s.admin_user_id, true).await
}

fn refus(e: Result<Account, DbError>) -> (MirrorReason, Option<String>) {
    match e {
        Err(DbError::MirrorAccountInvalid {
            reason,
            linked_account_number,
        }) => (reason, linked_account_number),
        other => panic!("MirrorAccountInvalid attendu, reçu {other:?}"),
    }
}

fn garde(e: Result<Account, DbError>) -> (AccountIsMirrorReason, Vec<MirrorSourceAccount>) {
    match e {
        Err(DbError::AccountIsMirror { reason, sources }) => (reason, sources),
        other => panic!("AccountIsMirror attendu, reçu {other:?}"),
    }
}

/// Une ligne d'écriture posée en SQL direct sur `compte` (contre la caisse).
async fn ligne_sql(pool: &MySqlPool, s: &SeededCompany, compte: i64) {
    let je: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, journal, description) \
         VALUES (?, ?, (SELECT n FROM (SELECT COALESCE(MAX(entry_number), 0) + 1 AS n FROM journal_entries WHERE company_id = ?) x), \
         '2026-06-01', 'Banque', 'semis') RETURNING id",
    )
    .bind(s.company_id)
    .bind(s.fiscal_year_id)
    .bind(s.company_id)
    .fetch_one(pool)
    .await
    .unwrap();
    for (ordre, (id, debit, credit)) in [(compte, 10, 0), (s.accounts["1000"], 0, 10)]
        .into_iter()
        .enumerate()
    {
        sqlx::query(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(je)
        .bind(id)
        .bind(ordre as i32 + 1)
        .bind(debit)
        .bind(credit)
        .execute(pool)
        .await
        .unwrap();
    }
}

fn ecriture(s: &SeededCompany, debit: i64, credit: i64) -> NewJournalEntry {
    NewJournalEntry {
        company_id: s.company_id,
        entry_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
        journal: Journal::OD,
        description: "Miroir".into(),
        project_id: None,
        lines: vec![
            chf_line(debit, dec!(25.00), Decimal::ZERO),
            chf_line(credit, Decimal::ZERO, dec!(25.00)),
        ],
    }
}

/// Une écriture par le flux automatique (`enforce_postable = false`).
async fn ecrire_auto(pool: &MySqlPool, s: &SeededCompany, debit: i64) -> Result<(), DbError> {
    let mut tx = pool.begin().await.unwrap();
    journal_entries::create_in_tx(
        &mut tx,
        s.fiscal_year_id,
        Acteur::utilisateur(s.admin_user_id),
        ecriture(s, debit, s.accounts["1000"]),
        false,
    )
    .await?;
    tx.commit().await.unwrap();
    Ok(())
}

async fn compter_ecritures(pool: &MySqlPool) -> (i64, i64) {
    let e: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries")
        .fetch_one(pool)
        .await
        .unwrap();
    let l: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entry_lines")
        .fetch_one(pool)
        .await
        .unwrap();
    (e, l)
}

// ---------------------------------------------------------------------------
// AC-3, AC-12 — pose, changement, retrait ; audit ; aucune écriture comptable
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn poser_changer_retirer_sans_ecriture_comptable(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let (source, cible) = paire(&pool, &s).await;
    let cible2 = compte(&pool, s.company_id, "1178", "Asset", false, None).await;
    let avant = compter_ecritures(&pool).await;
    let cible_avant = lire(&pool, cible).await;

    let posee = poser(&pool, &s, source, cible).await.expect("pose");
    assert_eq!(posee.mirror_account_id, Some(cible));
    assert_eq!(posee.version, 2);
    let audit: Vec<u8> = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE entity_type = 'account' AND entity_id = ? \
         AND action = 'account.updated' ORDER BY id DESC LIMIT 1",
    )
    .bind(source)
    .fetch_one(&pool)
    .await
    .unwrap();
    let audit: serde_json::Value = serde_json::from_slice(&audit).unwrap();
    assert_eq!(audit["before"]["mirrorAccountId"], serde_json::Value::Null);
    assert_eq!(audit["after"]["mirrorAccountId"], serde_json::json!(cible));

    // Reposer le même miroir : no-op, sans changement de version.
    let meme = poser(&pool, &s, source, cible).await.expect("no-op");
    assert_eq!(meme.version, 2, "reposer le miroir est un no-op");

    let changee = poser(&pool, &s, source, cible2).await.expect("changement");
    assert_eq!(changee.mirror_account_id, Some(cible2));
    let retiree = modifier(&pool, &s, source, |u| u.mirror = MirrorChange::Clear)
        .await
        .expect("retrait");
    assert_eq!(retiree.mirror_account_id, None);
    assert_eq!(retiree.version, 4);

    // AC-12 : aucune écriture créée ni modifiée, la cible jamais écrite.
    assert_eq!(compter_ecritures(&pool).await, avant);
    let cible_apres = lire(&pool, cible).await;
    assert_eq!(cible_apres.version, cible_avant.version);
    assert_eq!(cible_apres.updated_at, cible_avant.updated_at);
}

#[sqlx::test(migrations = "./test-schema")]
async fn un_miroir_se_pose_a_la_creation_sous_le_meme_controle(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let cible = compte(&pool, s.company_id, "1179", "Asset", false, None).await;
    let mut new = NewAccount::new(
        s.company_id,
        "2271",
        "Dettes sociales",
        AccountType::Liability,
        None,
    );
    new.mirror_account_id = Some(cible);
    let cree = accounts::create(&pool, s.admin_user_id, new.clone())
        .await
        .expect("création avec miroir");
    assert_eq!(cree.mirror_account_id, Some(cible));

    // Même contrôle qu'à la modification : une cible imputable est refusée.
    let imputable = compte(&pool, s.company_id, "1177", "Asset", true, None).await;
    new.number = "2272".into();
    new.mirror_account_id = Some(imputable);
    let (raison, _) = refus(accounts::create(&pool, s.admin_user_id, new).await);
    assert_eq!(raison, MirrorReason::NotPresentationAccount);
}

/// Story 47-5 (revue P2 de la 47-2, P2-L3) — à la création, le refus du miroir
/// passe **avant** ceux du parent : un parent compte de différence de change et
/// une cible imputable rendent `MIRROR_ACCOUNT_INVALID` ; sans le miroir, le même
/// geste rend `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT` (le parent est bien refusable).
#[sqlx::test(migrations = "./test-schema")]
async fn a_la_creation_le_refus_du_miroir_passe_avant_celui_du_parent(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let gain = compte(&pool, s.company_id, "6950", "Revenue", true, None).await;
    sqlx::query(
        "UPDATE companies SET multi_currency_enabled = TRUE, fx_gain_account_id = ? WHERE id = ?",
    )
    .bind(gain)
    .bind(s.company_id)
    .execute(&pool)
    .await
    .unwrap();
    let imputable = compte(&pool, s.company_id, "1177", "Asset", true, None).await;
    let mut new = NewAccount::new(
        s.company_id,
        "2271",
        "Dettes sociales",
        AccountType::Liability,
        None,
    );
    new.parent_id = Some(gain);
    new.mirror_account_id = Some(imputable);
    let (raison, _) = refus(accounts::create(&pool, s.admin_user_id, new.clone()).await);
    assert_eq!(raison, MirrorReason::NotPresentationAccount);

    new.mirror_account_id = None;
    assert!(matches!(
        accounts::create(&pool, s.admin_user_id, new).await,
        Err(DbError::AccountIsFxDifferenceAccount { .. })
    ));
}

// ---------------------------------------------------------------------------
// AC-4 — règles de validité, une raison chacune, dans leur ordre
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn chaque_regle_de_validite_refuse_avec_sa_raison(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let (source, cible) = paire(&pool, &s).await;

    // self
    assert_eq!(
        refus(poser(&pool, &s, source, source).await).0,
        MirrorReason::SelfLink
    );
    // unknown : introuvable, puis d'une autre société (non distingués).
    assert_eq!(
        refus(poser(&pool, &s, source, 9_999_999).await).0,
        MirrorReason::Unknown
    );
    let autre: i64 = sqlx::query_scalar(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre SA', 'Rue 2, 1000 Lausanne', 'Pme', 'FR', 'FR') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let etranger = compte(&pool, autre, "1179", "Asset", false, None).await;
    assert_eq!(
        refus(poser(&pool, &s, source, etranger).await).0,
        MirrorReason::Unknown
    );
    // archived
    let archivee = compte(&pool, s.company_id, "1176", "Asset", false, None).await;
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(archivee)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        refus(poser(&pool, &s, source, archivee).await).0,
        MirrorReason::Archived
    );
    // accountType : passif → passif, et passif → charge.
    let passif = compte(&pool, s.company_id, "2279", "Liability", false, None).await;
    assert_eq!(
        refus(poser(&pool, &s, source, passif).await).0,
        MirrorReason::AccountType
    );
    let charge = compte(&pool, s.company_id, "6999", "Expense", false, None).await;
    assert_eq!(
        refus(poser(&pool, &s, source, charge).await).0,
        MirrorReason::AccountType
    );
    // equity : la source porte un rôle de fonds propres.
    let fonds = compte(
        &pool,
        s.company_id,
        "2950",
        "Liability",
        true,
        Some("EquityOther"),
    )
    .await;
    assert_eq!(
        refus(poser(&pool, &s, fonds, cible).await).0,
        MirrorReason::Equity
    );
    // notPresentationAccount : imputable, à rôle, porteuse d'une écriture.
    let imputable = compte(&pool, s.company_id, "1175", "Asset", true, None).await;
    assert_eq!(
        refus(poser(&pool, &s, source, imputable).await).0,
        MirrorReason::NotPresentationAccount
    );
    let a_role = compte(
        &pool,
        s.company_id,
        "1174",
        "Asset",
        false,
        Some("VatRecoverable"),
    )
    .await;
    assert_eq!(
        refus(poser(&pool, &s, source, a_role).await).0,
        MirrorReason::NotPresentationAccount
    );
    let mouvementee = compte(&pool, s.company_id, "1173", "Asset", false, None).await;
    ligne_sql(&pool, &s, mouvementee).await;
    assert_eq!(
        refus(poser(&pool, &s, source, mouvementee).await).0,
        MirrorReason::NotPresentationAccount
    );
    // chain (1) : la cible a elle-même un miroir — elle est nommée.
    let enchainee = compte(&pool, s.company_id, "1172", "Asset", false, None).await;
    let au_bout = compte(&pool, s.company_id, "2278", "Liability", false, None).await;
    sqlx::query("UPDATE accounts SET mirror_account_id = ? WHERE id = ?")
        .bind(au_bout)
        .bind(enchainee)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        refus(poser(&pool, &s, source, enchainee).await),
        (MirrorReason::Chain, Some("1172".to_string()))
    );
    // chain (2) : la source est déjà le miroir d'un autre compte — il est nommé.
    poser(&pool, &s, source, cible).await.expect("pose valide");
    let autre_cible = compte(&pool, s.company_id, "2277", "Liability", false, None).await;
    assert_eq!(
        refus(poser(&pool, &s, cible, autre_cible).await),
        (MirrorReason::Chain, Some("2270".to_string()))
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn plusieurs_sources_et_une_cible_en_devise_sont_admises(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let (source, cible) = paire(&pool, &s).await;
    let source2 = compte(&pool, s.company_id, "2271", "Liability", true, None).await;
    poser(&pool, &s, source, cible)
        .await
        .expect("première source");
    poser(&pool, &s, source2, cible)
        .await
        .expect("seconde source (R3)");

    let en_devise = compte(&pool, s.company_id, "1171", "Asset", false, None).await;
    sqlx::query("UPDATE accounts SET currency = 'EUR' WHERE id = ?")
        .bind(en_devise)
        .execute(&pool)
        .await
        .unwrap();
    let source3 = compte(&pool, s.company_id, "2272", "Liability", true, None).await;
    poser(&pool, &s, source3, en_devise)
        .await
        .expect("cible en devise admise (R8)");
}

/// AC-4 — un compte qui garde son miroir : retypage et rôle de fonds propres
/// rejoués sur l'état résultant ; retirer le miroir dans le même `PUT` lève le
/// refus ; un état forgé de la cible n'empêche pas de renommer la source.
#[sqlx::test(migrations = "./test-schema")]
async fn les_regles_de_la_source_se_rejouent_quand_le_miroir_reste(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let (source, cible) = paire(&pool, &s).await;
    poser(&pool, &s, source, cible).await.unwrap();

    let retype = modifier(&pool, &s, source, |u| u.account_type = AccountType::Asset).await;
    assert_eq!(refus(retype).0, MirrorReason::AccountType);
    let fonds = modifier(&pool, &s, source, |u| {
        u.role = Some(AccountRole::EquityOther)
    })
    .await;
    assert_eq!(refus(fonds).0, MirrorReason::Equity);

    let leve = modifier(&pool, &s, source, |u| {
        u.role = Some(AccountRole::EquityOther);
        u.mirror = MirrorChange::Clear;
    })
    .await
    .expect("retirer le miroir dans le même PUT lève le refus");
    assert_eq!(leve.mirror_account_id, None);

    // État forgé de la cible (une ligne, un rôle) : renommer la source passe.
    let source2 = compte(&pool, s.company_id, "2271", "Liability", true, None).await;
    poser(&pool, &s, source2, cible).await.unwrap();
    ligne_sql(&pool, &s, cible).await;
    let renommee = modifier(&pool, &s, source2, |u| u.name = "Renommée".into())
        .await
        .expect("renommer la source malgré l'état forgé de la cible");
    assert_eq!(renommee.name, "Renommée");

    // Story 47-5 (revue P1 de la 47-1, L3) — `Set` du miroir actuel vaut
    // `Unchanged` : le lien forgé n'est pas revalidé (la cible porte une ligne),
    // mais les règles de la source se rejouent sur un retypage.
    let reposee = modifier(&pool, &s, source2, |u| {
        u.name = "Reposée".into();
        u.mirror = MirrorChange::Set(cible);
    })
    .await
    .expect("reposer le miroir actuel ne revalide pas la cible forgée");
    assert_eq!(reposee.mirror_account_id, Some(cible));
    let retype = modifier(&pool, &s, source2, |u| {
        u.mirror = MirrorChange::Set(cible);
        u.account_type = AccountType::Asset;
    })
    .await;
    assert_eq!(refus(retype).0, MirrorReason::AccountType);

    // Une cible introuvable (lien forgé) n'empêche rien : le retypage passe.
    let source3 = compte(&pool, s.company_id, "2273", "Liability", true, None).await;
    sqlx::query("UPDATE accounts SET mirror_account_id = ? WHERE id = ?")
        .bind(i64::MAX)
        .bind(source3)
        .execute(&pool)
        .await
        .unwrap();
    let retypee = modifier(&pool, &s, source3, |u| u.account_type = AccountType::Asset)
        .await
        .expect("cible introuvable : rien n'empêche le retypage");
    assert_eq!(retypee.account_type, AccountType::Asset);
}

// ---------------------------------------------------------------------------
// AC-5 — garde du compte miroir
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn le_compte_miroir_ne_s_archive_ni_ne_change_tant_qu_il_est_designe(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let (source, cible) = paire(&pool, &s).await;
    let source2 = compte(&pool, s.company_id, "2260", "Liability", true, None).await;
    poser(&pool, &s, source, cible).await.unwrap();
    poser(&pool, &s, source2, cible).await.unwrap();

    let (raison, sources) = garde(archiver(&pool, &s, cible).await);
    assert_eq!(raison, AccountIsMirrorReason::Archive);
    let numeros: Vec<&str> = sources.iter().map(|x| x.number.as_str()).collect();
    assert_eq!(numeros, ["2260", "2270"], "sources triées par numéro");
    // `confirm_designated = true` (archiver) ne lève pas le refus.

    assert_eq!(
        garde(modifier(&pool, &s, cible, |u| u.postable = true).await).0,
        AccountIsMirrorReason::Postable
    );
    assert_eq!(
        garde(
            modifier(&pool, &s, cible, |u| u.account_type =
                AccountType::Liability)
            .await
        )
        .0,
        AccountIsMirrorReason::AccountType
    );
    assert_eq!(
        garde(
            modifier(&pool, &s, cible, |u| u.role =
                Some(AccountRole::VatRecoverable))
            .await
        )
        .0,
        AccountIsMirrorReason::Role
    );
    // Renommer le miroir reste admis.
    modifier(&pool, &s, cible, |u| u.name = "Créances sociales".into())
        .await
        .expect("renommer le miroir");

    // Archiver une SOURCE reste admis ; elle compte encore pour la garde.
    archiver(&pool, &s, source2)
        .await
        .expect("archiver une source");
    let (_, sources) = garde(archiver(&pool, &s, cible).await);
    assert_eq!(sources.len(), 2);
    assert!(sources.iter().any(|x| x.number == "2260" && !x.active));

    // Une fois les liens retirés (la source archivée réactivée d'abord), le
    // miroir s'archive.
    let a = lire(&pool, source2).await;
    accounts::reactivate(&pool, source2, a.version, s.admin_user_id, false)
        .await
        .unwrap();
    for id in [source, source2] {
        modifier(&pool, &s, id, |u| u.mirror = MirrorChange::Clear)
            .await
            .unwrap();
    }
    archiver(&pool, &s, cible)
        .await
        .expect("archivage sans lien");
}

// ---------------------------------------------------------------------------
// AC-6 — aucune écriture sur un compte miroir
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn aucune_ecriture_ne_se_passe_sur_un_compte_miroir(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let (source, cible) = paire(&pool, &s).await;
    poser(&pool, &s, source, cible).await.unwrap();

    let nomme = |e: DbError| match e {
        DbError::AccountsNotPostable(list) => list.numbers().join(","),
        other => panic!("AccountsNotPostable attendu, reçu {other:?}"),
    };
    // Saisie manuelle.
    let manuelle = journal_entries::create(
        &pool,
        s.fiscal_year_id,
        Acteur::utilisateur(s.admin_user_id),
        ecriture(&s, cible, s.accounts["1000"]),
    )
    .await
    .expect_err("saisie manuelle refusée");
    assert_eq!(nomme(manuelle), "1179");
    // Flux automatique : `enforce_postable = false` ne lève pas la garde.
    let auto = ecrire_auto(&pool, &s, cible)
        .await
        .expect_err("flux automatique refusé");
    assert_eq!(nomme(auto), "1179");
    // Les deux familles (non imputable, miroir) réunies dans une seule erreur.
    let non_postable = compte(&pool, s.company_id, "1170", "Asset", false, None).await;
    let deux = journal_entries::create(
        &pool,
        s.fiscal_year_id,
        Acteur::utilisateur(s.admin_user_id),
        ecriture(&s, cible, non_postable),
    )
    .await
    .expect_err("deux comptes refusés");
    assert_eq!(nomme(deux), "1170,1179");
    // Modification d'une écriture (`update_in_tx`, revue P1 de la 47-1, L6) : une
    // ligne déplacée vers le miroir est refusée au même point de passage.
    let passee = journal_entries::create(
        &pool,
        s.fiscal_year_id,
        Acteur::utilisateur(s.admin_user_id),
        ecriture(&s, source, s.accounts["1000"]),
    )
    .await
    .expect("écriture sur la source");
    let deplacee = journal_entries::update(
        &pool,
        s.company_id,
        passee.entry.id,
        passee.entry.version,
        Acteur::utilisateur(s.admin_user_id),
        ecriture(&s, cible, s.accounts["1000"]),
    )
    .await
    .expect_err("modification vers le miroir refusée");
    assert_eq!(nomme(deplacee), "1179");

    // Le lien retiré : le flux automatique passe ; la saisie manuelle, une fois
    // le compte rendu imputable.
    modifier(&pool, &s, source, |u| u.mirror = MirrorChange::Clear)
        .await
        .unwrap();
    ecrire_auto(&pool, &s, cible)
        .await
        .expect("flux automatique après retrait");
    modifier(&pool, &s, cible, |u| u.postable = true)
        .await
        .unwrap();
    journal_entries::create(
        &pool,
        s.fiscal_year_id,
        Acteur::utilisateur(s.admin_user_id),
        ecriture(&s, cible, s.accounts["1000"]),
    )
    .await
    .expect("saisie manuelle après retrait");
}

/// Revue P1 de la 47-5, L3 : la clause « miroir, toujours » de la garde des
/// lignes (`journal_entries.rs`) tient seule, côté saisie manuelle et
/// modification, quand la cible est **imputable** (état forgé en SQL, que
/// l'AC-4 interdit par l'API). Sans elle, seul `!postable` refuserait, et ce
/// test ne verrait rien : la cible de `paire()` est non imputable.
#[sqlx::test(migrations = "./test-schema")]
async fn la_clause_miroir_tient_seule_sur_une_cible_imputable(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let (source, cible) = paire(&pool, &s).await;
    poser(&pool, &s, source, cible).await.unwrap();
    sqlx::query("UPDATE accounts SET postable = TRUE WHERE id = ?")
        .bind(cible)
        .execute(&pool)
        .await
        .unwrap();
    let nomme = |e: DbError| match e {
        DbError::AccountsNotPostable(list) => list.numbers().join(","),
        other => panic!("AccountsNotPostable attendu, reçu {other:?}"),
    };
    let manuelle = journal_entries::create(
        &pool,
        s.fiscal_year_id,
        Acteur::utilisateur(s.admin_user_id),
        ecriture(&s, cible, s.accounts["1000"]),
    )
    .await
    .expect_err("saisie manuelle refusée sur un miroir imputable");
    assert_eq!(nomme(manuelle), "1179");
    let passee = journal_entries::create(
        &pool,
        s.fiscal_year_id,
        Acteur::utilisateur(s.admin_user_id),
        ecriture(&s, source, s.accounts["1000"]),
    )
    .await
    .expect("écriture sur la source");
    let deplacee = journal_entries::update(
        &pool,
        s.company_id,
        passee.entry.id,
        passee.entry.version,
        Acteur::utilisateur(s.admin_user_id),
        ecriture(&s, cible, s.accounts["1000"]),
    )
    .await
    .expect_err("modification vers un miroir imputable refusée");
    assert_eq!(nomme(deplacee), "1179");
}

// ---------------------------------------------------------------------------
// AC-7 — concurrence
// ---------------------------------------------------------------------------

/// Pose contre écriture : la pose (simulée par une transaction qui tient la
/// cible `FOR UPDATE` et écrit le lien) commite pendant que l'écriture, qui a
/// déjà fait une lecture NON verrouillante (instantané ouvert), attend le
/// verrou de la cible. La lecture verrouillante de la garde voit la pose :
/// l'écriture est refusée. Un `EXISTS` d'instantané ne la verrait pas.
#[sqlx::test(migrations = "./test-schema")]
async fn course_pose_contre_ecriture(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let (source, cible) = paire(&pool, &s).await;

    let mut pose = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM accounts WHERE id = ? FOR UPDATE")
        .bind(cible)
        .fetch_one(&mut *pose)
        .await
        .unwrap();
    sqlx::query("UPDATE accounts SET mirror_account_id = ? WHERE id = ?")
        .bind(cible)
        .bind(source)
        .execute(&mut *pose)
        .await
        .unwrap();

    let (pool_b, s_b) = (pool.clone(), s.clone());
    let ecriture_tache = tokio::spawn(async move {
        let mut tx = pool_b.begin().await.unwrap();
        // Lecture non verrouillante : l'instantané REPEATABLE READ s'ouvre ici,
        // avant le commit de la pose.
        let _: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM accounts WHERE company_id = ?")
            .bind(s_b.company_id)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        let r = journal_entries::create_in_tx(
            &mut tx,
            s_b.fiscal_year_id,
            Acteur::utilisateur(s_b.admin_user_id),
            ecriture(&s_b, cible, s_b.accounts["1000"]),
            false,
        )
        .await;
        if r.is_ok() {
            tx.commit().await.unwrap();
        }
        r
    });
    assert!(
        attendre_une_requete_en_cours(&pool, &["FROM accounts", "LOCK IN SHARE MODE"], || {
            ecriture_tache.is_finished()
        })
        .await,
        "l'écriture doit attendre la cible"
    );
    pose.commit().await.unwrap();
    let r = tokio::time::timeout(ATTENTE, ecriture_tache)
        .await
        .unwrap()
        .unwrap();
    match r {
        Err(DbError::AccountsNotPostable(list)) => assert_eq!(list.numbers(), ["1179"]),
        other => panic!("l'écriture devait être refusée, reçu {other:?}"),
    }
    let lignes: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entry_lines WHERE account_id = ?")
            .bind(cible)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(lignes, 0, "aucune ligne sur le miroir");
}

/// Écriture contre pose : l'écriture (flux automatique) tient la cible en
/// partage et y insère une ligne ; la pose attend la cible, puis voit la ligne
/// par sa lecture verrouillante et refuse (`notPresentationAccount`).
#[sqlx::test(migrations = "./test-schema")]
async fn course_ecriture_contre_pose(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let (source, cible) = paire(&pool, &s).await;

    let mut tx = pool.begin().await.unwrap();
    journal_entries::create_in_tx(
        &mut tx,
        s.fiscal_year_id,
        Acteur::utilisateur(s.admin_user_id),
        ecriture(&s, cible, s.accounts["1000"]),
        false,
    )
    .await
    .expect("écriture automatique sur un compte non imputable, non miroir");

    let (pool_b, s_b) = (pool.clone(), s.clone());
    let pose = tokio::spawn(async move { poser(&pool_b, &s_b, source, cible).await });
    // Le verrou de la cible par la pose (`lock_mirror_target_in_tx`), non la
    // lecture de la source, qui porte aussi « FOR UPDATE » (revue P1 de la 47-1, L1).
    assert!(
        attendre_une_requete_en_cours(&pool, &[CIBLE_DE_LA_POSE], || pose.is_finished()).await,
        "la pose doit attendre la cible"
    );
    tx.commit().await.unwrap();
    let r = tokio::time::timeout(ATTENTE, pose).await.unwrap().unwrap();
    assert_eq!(refus(r).0, MirrorReason::NotPresentationAccount);
    assert_eq!(lire(&pool, source).await.mirror_account_id, None);
}

/// Pose contre archivage du miroir : les deux attendent la cible, tenue par
/// une troisième transaction ; l'un passe, l'autre est refusé (refus nommé ou
/// interblocage `1213`), jamais une cible archivée et désignée.
#[sqlx::test(migrations = "./test-schema")]
async fn course_pose_contre_archivage(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let (source, cible) = paire(&pool, &s).await;

    let mut tient = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM accounts WHERE id = ? FOR UPDATE")
        .bind(cible)
        .fetch_one(&mut *tient)
        .await
        .unwrap();
    let (pool_b, s_b) = (pool.clone(), s.clone());
    let pose = tokio::spawn(async move { poser(&pool_b, &s_b, source, cible).await });
    let (pool_c, s_c) = (pool.clone(), s.clone());
    let archivage = tokio::spawn(async move { archiver(&pool_c, &s_c, cible).await });
    // Les DEUX attendent la cible, chacune par sa propre requête (revue P1 de la
    // 47-1, L1) : un seul témoin rendrait vrai avant que la seconde attende.
    assert!(
        attendre_une_requete_en_cours(&pool, &[CIBLE_DE_LA_POSE], || pose.is_finished()).await,
        "la pose doit attendre la cible"
    );
    assert!(
        attendre_une_requete_en_cours(&pool, &[CIBLE_DE_L_ARCHIVAGE], || {
            archivage.is_finished()
        })
        .await,
        "l'archivage doit attendre la cible"
    );
    tient.commit().await.unwrap();
    let rp = tokio::time::timeout(ATTENTE, pose).await.unwrap().unwrap();
    let ra = tokio::time::timeout(ATTENTE, archivage)
        .await
        .unwrap()
        .unwrap();
    let interblocage =
        |e: &DbError| format!("{e:?}").contains("1213") || format!("{e:?}").contains("Deadlock");
    match (&rp, &ra) {
        (Ok(_), Err(DbError::AccountIsMirror { reason, .. })) => {
            assert_eq!(*reason, AccountIsMirrorReason::Archive)
        }
        (Err(DbError::MirrorAccountInvalid { reason, .. }), Ok(_)) => {
            assert_eq!(*reason, MirrorReason::Archived)
        }
        (Err(e), _) | (_, Err(e)) if interblocage(e) => {}
        other => panic!("issue inattendue : {other:?}"),
    }
    let c = lire(&pool, cible).await;
    let src = lire(&pool, source).await;
    assert!(
        !(!c.active && src.mirror_account_id == Some(cible)),
        "jamais une cible archivée et désignée"
    );
}

// ---------------------------------------------------------------------------
// AC-8 — lecteur des liens
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn le_lecteur_rend_les_liens_valides_et_ecarte_les_liens_hors_regle(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    assert!(
        accounts::mirror_links(&pool, s.company_id)
            .await
            .unwrap()
            .is_empty(),
        "société sans lien → vecteur vide"
    );
    let (source, cible) = paire(&pool, &s).await;
    poser(&pool, &s, source, cible).await.unwrap();

    // Liens forgés par SQL (import, SQL direct) : cible d'une autre société,
    // même type (et lien vers soi-même), chaîne.
    let autre: i64 = sqlx::query_scalar(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre SA', 'Rue 2, 1000 Lausanne', 'Pme', 'FR', 'FR') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let etranger = compte(&pool, autre, "1179", "Asset", false, None).await;
    let src_autre = compte(&pool, autre, "2270", "Liability", true, None).await;
    let forge = |src: i64, cib: i64| {
        let pool = pool.clone();
        async move {
            sqlx::query("UPDATE accounts SET mirror_account_id = ? WHERE id = ?")
                .bind(cib)
                .bind(src)
                .execute(&pool)
                .await
                .unwrap();
        }
    };
    let a = compte(&pool, s.company_id, "2201", "Liability", true, None).await;
    forge(a, etranger).await; // (a) cible d'une autre société
    let b = compte(&pool, s.company_id, "2202", "Liability", true, None).await;
    let meme_type = compte(&pool, s.company_id, "2203", "Liability", false, None).await;
    forge(b, meme_type).await; // (b) même type
    let soi = compte(&pool, s.company_id, "2204", "Liability", true, None).await;
    forge(soi, soi).await; // (b) soi-même
    let c = compte(&pool, s.company_id, "2205", "Liability", true, None).await;
    let chainee = compte(&pool, s.company_id, "1105", "Asset", false, None).await;
    let bout = compte(&pool, s.company_id, "2206", "Liability", false, None).await;
    forge(c, chainee).await;
    forge(chainee, bout).await; // (c) la cible de c a un miroir
    forge(src_autre, etranger).await; // lien valide d'une autre société

    let liens = accounts::mirror_links(&pool, s.company_id).await.unwrap();
    // Restent : source → cible ; et chainee → bout (Asset → Liability, bout
    // sans miroir) — un lien valide tel que lu.
    let paires: Vec<(i64, i64)> = liens.iter().map(|l| (l.source_id, l.mirror_id)).collect();
    assert_eq!(paires, vec![(chainee, bout), (source, cible)]);
    let l = liens.iter().find(|l| l.source_id == source).unwrap();
    assert_eq!(l.mirror_number, "1179");
    assert_eq!(l.mirror_name, "Compte 1179");
    assert_eq!(l.mirror_account_type, AccountType::Asset);
    assert!(l.mirror_active);
    // L'autre société n'y paraît jamais ; la sienne, seule.
    let autres = accounts::mirror_links(&pool, autre).await.unwrap();
    assert_eq!(
        autres.iter().map(|l| l.source_id).collect::<Vec<_>>(),
        vec![src_autre]
    );
}
