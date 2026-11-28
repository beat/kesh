//! Story 14-3a — tests E2E HTTP pour `/api/v1/accounts`.
//!
//! Il n'existait **aucune** suite E2E dédiée aux comptes avant cette story
//! (seul `idor_multi_tenant_e2e.rs` touchait la ressource). Pattern hérité de
//! `reconciliation_rules_e2e.rs` : forge JWT directement + `spawn_app` éphémère.
//!
//! Couvre : rôles et postabilité sur les 3 verbes, conflit de rôle singleton
//! **avec le compte en conflit nommé dans `details`**, contrat full-replace du
//! PUT, cycle archive → réactivation (#269) et ses gardes, RBAC, IDOR.
#![allow(clippy::too_many_arguments)]

mod common;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{Datelike, NaiveDate, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::account::{AccountRole, AccountType};
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{
    Language, NewAccount, NewCompany, NewFiscalYear, NewJournalEntry, NewJournalEntryLine, NewUser,
    OrgType, Role,
};
use kesh_db::repositories::{accounts, companies, fiscal_years, journal_entries, users};
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";

// ============================================================
// Spawn helpers
// ============================================================

struct TestApp {
    base_url: String,
    client: reqwest::Client,
}

impl TestApp {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

fn test_config() -> Config {
    Config::from_fields_for_test(
        "mysql://test".into(),
        "admin".into(),
        "e2e-test-password".into(),
        std::str::from_utf8(TEST_JWT_SECRET).unwrap().to_string(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        100,
        TimeDelta::minutes(30),
        12,
    )
}

async fn spawn_app(pool: MySqlPool) -> TestApp {
    let config = test_config();
    let rate_limiter = kesh_api::middleware::rate_limit::RateLimiter::new(&config);
    let i18n = Arc::new(
        kesh_i18n::I18nBundle::load(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("load test i18n"),
    );
    let state = AppState::new_for_tests(pool, Arc::new(config), Arc::new(rate_limiter), i18n);
    let app = build_router(state.clone(), "nonexistent-static-dir".to_string());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_) => break,
            Err(_) if std::time::Instant::now() < deadline => {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
            Err(e) => panic!("test server not ready in 2s: {e}"),
        }
    }
    TestApp {
        base_url: format!("http://{}", addr),
        client: reqwest::Client::new(),
    }
}

fn forge_jwt(user_id: i64, role: &str, company_id: i64) -> String {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        company_id,
        iat: now,
        exp: now + 3600,
    };
    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET),
    )
    .unwrap()
}

fn role_str(role: Role) -> &'static str {
    match role {
        Role::Admin => "Admin",
        Role::Comptable => "Comptable",
        Role::Consultation => "Consultation",
    }
}

// ============================================================
// Domain seed helpers
// ============================================================

async fn create_company(pool: &MySqlPool, name: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: name.into(),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "Rue Test".into(),
                building: "1".into(),
                postal_code: "1000".into(),
                city: "Lausanne".into(),
                country: "CH".into(),
            },
            ide_number: None,
            org_type: OrgType::Independant,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap()
    .id
}

async fn create_user(pool: &MySqlPool, username: &str, role: Role, company_id: i64) -> i64 {
    users::create(
        pool,
        NewUser {
            username: username.into(),
            password_hash: hash_password("password123").unwrap(),
            role,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap()
    .id
}

/// Crée un compte via le repository (chemin non-HTTP, pour préparer un état).
async fn mk_account(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    number: &str,
    role: Option<AccountRole>,
) -> i64 {
    accounts::create(
        pool,
        user_id,
        NewAccount::new(
            company_id,
            number,
            format!("Compte {number}"),
            AccountType::Asset,
            None,
        )
        .with_role(role, true),
    )
    .await
    .unwrap()
    .id
}

/// Contexte minimal : 1 société + 1 admin.
async fn setup(pool: &MySqlPool) -> (i64, i64, String) {
    let company_id = create_company(pool, "Acme SA").await;
    let admin_id = create_user(pool, "admin", Role::Admin, company_id).await;
    let token = forge_jwt(admin_id, role_str(Role::Admin), company_id);
    (company_id, admin_id, token)
}

// ============================================================
// Story 25-2-a ([#382], [#274]) — retyper un compte mouvementé
// ============================================================

/// ⚠️ `mk_account` ci-dessus force `AccountType::Asset`. Retyper exige de partir
/// d'un autre type — d'où ce helper plutôt qu'une modification de l'existant,
/// qui ferait porter à quinze tests étrangers le risque de cette story.
async fn mk_typed_account(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    number: &str,
    account_type: AccountType,
) -> i64 {
    accounts::create(
        pool,
        user_id,
        NewAccount::new(
            company_id,
            number,
            format!("Compte {number}"),
            account_type,
            None,
        )
        .with_role(None, true),
    )
    .await
    .unwrap()
    .id
}

/// Exercice **ouvert** de l'année de `jour` — l'écriture porte ce jour, elle doit
/// donc y tomber. `jour` est le jour de l'horloge de test, lu **une fois** par le
/// test (Story 35-5c, revue P1 E2 : deux lectures pouvaient chevaucher le
/// 31 décembre et créer l'exercice d'une année et l'écriture de la suivante).
async fn mk_fiscal_year(pool: &MySqlPool, company_id: i64, user_id: i64, jour: NaiveDate) -> i64 {
    let year = jour.year();
    fiscal_years::create(
        pool,
        user_id,
        NewFiscalYear {
            company_id,
            name: format!("Exercice {year}"),
            start_date: NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(year, 12, 31).unwrap(),
        },
    )
    .await
    .expect("exercice")
    .id
}

async fn mk_entry(
    pool: &MySqlPool,
    company_id: i64,
    fiscal_year_id: i64,
    user_id: i64,
    debit_account: i64,
    credit_account: i64,
    jour: NaiveDate,
) {
    journal_entries::create(
        pool,
        fiscal_year_id,
        kesh_db::entities::audit_log::Acteur::utilisateur(user_id),
        NewJournalEntry {
            company_id,
            entry_date: jour,
            journal: Journal::OD,
            description: "Écriture de test 25-2-a".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: debit_account,
                    debit: dec!(100),
                    credit: dec!(0),
                    project_id: None,
                    foreign: None,
                },
                NewJournalEntryLine {
                    account_id: credit_account,
                    debit: dec!(0),
                    credit: dec!(100),
                    project_id: None,
                    foreign: None,
                },
            ],
        },
    )
    .await
    .expect("écriture");
}

/// AC 1 et 2 — le refus traverse réellement la frontière HTTP, et il **nomme
/// l'ampleur**. C'est le seul niveau qui vérifie que les quatre champs de
/// `details` arrivent jusqu'au client : les tests du dépôt voient la variante
/// d'erreur, pas le JSON.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn retyper_un_compte_mouvemente_rend_409_et_nomme_l_ampleur(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;
    let jour = common::jour::aujourd_hui_test();
    let fy = mk_fiscal_year(&pool, company_id, admin_id, jour).await;

    let charge = mk_typed_account(&pool, company_id, admin_id, "6500", AccountType::Expense).await;
    let caisse = mk_typed_account(&pool, company_id, admin_id, "1000", AccountType::Asset).await;
    mk_entry(&pool, company_id, fy, admin_id, charge, caisse, jour).await;

    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{charge}")))
        .bearer_auth(&token)
        .json(&json!({
            "name": "Compte 6500", "accountType": "Asset",
            "role": null, "postable": true, "version": 1
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 409);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ACCOUNT_HAS_ENTRIES");
    assert_eq!(body["error"]["details"]["entryCount"], 1);
    assert_eq!(body["error"]["details"]["fromType"], "Expense");
    assert_eq!(body["error"]["details"]["toType"], "Asset");
    assert_eq!(
        body["error"]["details"]["closedFiscalYears"]
            .as_array()
            .expect("closedFiscalYears doit être un tableau")
            .len(),
        0
    );
}

/// AC 2 — l'exercice **clos** est nommé : c'est lui dont le résultat changerait
/// sans qu'aucune écriture ne l'explique.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_409_nomme_les_exercices_clos_touches(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;
    let jour = common::jour::aujourd_hui_test();
    let fy = mk_fiscal_year(&pool, company_id, admin_id, jour).await;

    let charge = mk_typed_account(&pool, company_id, admin_id, "6500", AccountType::Expense).await;
    let caisse = mk_typed_account(&pool, company_id, admin_id, "1000", AccountType::Asset).await;
    mk_entry(&pool, company_id, fy, admin_id, charge, caisse, jour).await;

    // Fermer APRÈS avoir écrit — l'ordre de la vraie vie.
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(fy)
        .execute(&pool)
        .await
        .unwrap();

    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{charge}")))
        .bearer_auth(&token)
        .json(&json!({
            "name": "Compte 6500", "accountType": "Asset",
            "role": null, "postable": true, "version": 1
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 409);
    let body: Value = res.json().await.unwrap();
    assert_eq!(
        body["error"]["details"]["closedFiscalYears"][0],
        format!("Exercice {}", jour.year())
    );
}

/// AC 4 — **avertissement bloquant, pas refus sec** : `confirmAccountRetype`
/// laisse passer. Sans ce test, une garde qui refuserait toujours satisferait
/// les deux précédents.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn retyper_passe_avec_confirm_account_retype(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;
    let jour = common::jour::aujourd_hui_test();
    let fy = mk_fiscal_year(&pool, company_id, admin_id, jour).await;

    let charge = mk_typed_account(&pool, company_id, admin_id, "6500", AccountType::Expense).await;
    let caisse = mk_typed_account(&pool, company_id, admin_id, "1000", AccountType::Asset).await;
    mk_entry(&pool, company_id, fy, admin_id, charge, caisse, jour).await;

    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{charge}")))
        .bearer_auth(&token)
        .json(&json!({
            "name": "Compte 6500", "accountType": "Asset",
            "role": null, "postable": true, "version": 1,
            "confirmAccountRetype": true
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["accountType"], "Asset");
}

/// AC 3 — la garde ne mord que sur le type : renommer un compte mouvementé
/// reste libre, sans drapeau.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn renommer_un_compte_mouvemente_ne_demande_aucune_confirmation(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;
    let jour = common::jour::aujourd_hui_test();
    let fy = mk_fiscal_year(&pool, company_id, admin_id, jour).await;

    let charge = mk_typed_account(&pool, company_id, admin_id, "6500", AccountType::Expense).await;
    let caisse = mk_typed_account(&pool, company_id, admin_id, "1000", AccountType::Asset).await;
    mk_entry(&pool, company_id, fy, admin_id, charge, caisse, jour).await;

    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{charge}")))
        .bearer_auth(&token)
        .json(&json!({
            "name": "Frais de bureau", "accountType": "Expense",
            "role": null, "postable": true, "version": 1
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["name"], "Frais de bureau");
    assert_eq!(body["accountType"], "Expense");
}

/// AC 8 — un compte sans écriture se retype comme avant : le drapeau, absent,
/// ne change rien.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn retyper_un_compte_vierge_ne_demande_aucune_confirmation(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;

    let compte = mk_typed_account(&pool, company_id, admin_id, "6500", AccountType::Expense).await;

    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{compte}")))
        .bearer_auth(&token)
        .json(&json!({
            "name": "Compte 6500", "accountType": "Asset",
            "role": null, "postable": true, "version": 1
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["accountType"], "Asset");
}

// ============================================================
// Rôle & postabilité — contrat des 3 verbes
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_and_get_expose_role_and_postable(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (_company_id, _admin_id, token) = setup(&pool).await;

    let res = app
        .client
        .post(app.url("/api/v1/accounts"))
        .bearer_auth(&token)
        .json(&json!({
            "number": "1100",
            "name": "Débiteurs",
            "accountType": "Asset",
            "role": "Receivable",
            "postable": false
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["role"], "Receivable");
    assert_eq!(body["postable"], false);

    // GET renvoie les mêmes champs (contrat rétro-compatible : ajout de champs).
    let list: Value = app
        .client
        .get(app.url("/api/v1/accounts"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list[0]["role"], "Receivable");
    assert_eq!(list[0]["postable"], false);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_defaults_role_null_and_postable_true(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (_c, _a, token) = setup(&pool).await;

    let body: Value = app
        .client
        .post(app.url("/api/v1/accounts"))
        .bearer_auth(&token)
        .json(&json!({ "number": "1000", "name": "Caisse", "accountType": "Asset" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body["role"], Value::Null);
    assert_eq!(body["postable"], true);
}

/// Contrat **full-replace** : omettre `role` ou `postable` est un 400, pas un
/// effacement silencieux. C'est le point de conception le plus discutable de la
/// story — il est donc testé explicitement dans les deux sens.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_requires_role_and_postable_explicitly(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;
    let id = mk_account(
        &pool,
        company_id,
        admin_id,
        "1100",
        Some(AccountRole::Receivable),
    )
    .await;

    // `role` omis → 400 (et surtout : le rôle N'EST PAS effacé).
    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{id}")))
        .bearer_auth(&token)
        .json(
            &json!({ "name": "Débiteurs", "accountType": "Asset", "postable": true, "version": 1 }),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        400,
        "omettre `role` doit être un 400 explicite"
    );

    // `postable` omis → 400.
    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{id}")))
        .bearer_auth(&token)
        .json(&json!({ "name": "Débiteurs", "accountType": "Asset", "role": "Receivable", "version": 1 }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        400,
        "omettre `postable` doit être un 400 explicite"
    );

    // Le rôle est intact après les deux refus.
    let after = accounts::find_by_id(&pool, id).await.unwrap().unwrap();
    assert_eq!(after.role, Some(AccountRole::Receivable));

    // `role: null` explicite → le rôle est bien retiré (intention du client).
    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{id}")))
        .bearer_auth(&token)
        .json(&json!({
            "name": "Débiteurs", "accountType": "Asset",
            "role": null, "postable": true, "version": 1
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let after = accounts::find_by_id(&pool, id).await.unwrap().unwrap();
    assert_eq!(after.role, None);
}

// ============================================================
// Conflit de rôle singleton
// ============================================================

/// L'exigence n'est PAS seulement « 409 » : le corps doit **nommer** le compte
/// qui porte déjà le rôle, sans quoi l'utilisateur ne sait pas quoi corriger.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn singleton_role_conflict_names_the_holding_account(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;
    let holder = mk_account(
        &pool,
        company_id,
        admin_id,
        "1100",
        Some(AccountRole::Receivable),
    )
    .await;
    let other = mk_account(&pool, company_id, admin_id, "1101", None).await;

    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{other}")))
        .bearer_auth(&token)
        .json(&json!({
            "name": "Débiteurs bis", "accountType": "Asset",
            "role": "Receivable", "postable": true, "version": 1
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 409);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ACCOUNT_ROLE_ALREADY_ASSIGNED");
    assert_eq!(body["error"]["details"]["accountId"], holder);
    assert_eq!(body["error"]["details"]["accountNumber"], "1100");
    assert_eq!(body["error"]["details"]["role"], "Receivable");
    assert!(
        body["error"]["message"].as_str().unwrap().contains("1100"),
        "le message doit nommer le compte en conflit : {}",
        body["error"]["message"]
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn multi_valued_role_accepted_twice(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (_c, _a, token) = setup(&pool).await;

    for number in ["2850", "2860"] {
        let res = app
            .client
            .post(app.url("/api/v1/accounts"))
            .bearer_auth(&token)
            .json(&json!({
                "number": number, "name": format!("Fonds {number}"),
                "accountType": "Liability", "role": "EquityOther"
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 201, "EquityOther n'est pas singleton");
    }
}

// ============================================================
// Réactivation (#269)
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn archive_then_reactivate_round_trip(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;
    let id = mk_account(&pool, company_id, admin_id, "1000", None).await;

    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{id}/archive")))
        .bearer_auth(&token)
        .json(&json!({ "version": 1 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{id}/reactivate")))
        .bearer_auth(&token)
        .json(&json!({ "version": 2 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["active"], true);
    assert_eq!(body["version"], 3);

    // Idempotent : réactiver un compte actif ne bump pas la version.
    let body: Value = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{id}/reactivate")))
        .bearer_auth(&token)
        .json(&json!({ "version": 3 }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body["version"], 3, "réactiver un compte actif = no-op");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reactivate_refused_when_parent_archived(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;

    let parent = mk_account(&pool, company_id, admin_id, "10", None).await;
    let child = accounts::create(
        &pool,
        admin_id,
        NewAccount::new(
            company_id,
            "1000",
            "Caisse",
            AccountType::Asset,
            Some(parent),
        ),
    )
    .await
    .unwrap();

    accounts::archive(&pool, child.id, child.version, admin_id)
        .await
        .unwrap();
    accounts::archive(&pool, parent, 1, admin_id).await.unwrap();

    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{}/reactivate", child.id)))
        .bearer_auth(&token)
        .json(&json!({ "version": 2 }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        409,
        "un compte actif sous un parent archivé serait incohérent"
    );
    let body: Value = res.json().await.unwrap();
    assert_eq!(
        body["error"]["code"], "ACCOUNT_PARENT_ARCHIVED",
        "code dédié, pour un message qui dit à l'utilisateur quoi corriger"
    );
    assert!(
        body["error"]["message"].as_str().unwrap().contains("10"),
        "le message doit nommer le parent archivé : {}",
        body["error"]["message"]
    );
}

/// Code review 14-3a (HIGH) : le conflit de rôle singleton doit être détecté au
/// `POST`, avec le compte détenteur nommé — pas remonté en `RESOURCE_CONFLICT`
/// générique que le formulaire traduirait par « ce numéro existe déjà ».
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn create_names_the_holder_on_singleton_role_conflict(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;
    let holder = mk_account(
        &pool,
        company_id,
        admin_id,
        "1100",
        Some(AccountRole::Receivable),
    )
    .await;

    let res = app
        .client
        .post(app.url("/api/v1/accounts"))
        .bearer_auth(&token)
        .json(&json!({
            "number": "1101", "name": "Débiteurs bis", "accountType": "Asset",
            "role": "Receivable"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 409);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ACCOUNT_ROLE_ALREADY_ASSIGNED");
    assert_eq!(body["error"]["details"]["accountId"], holder);
    assert_eq!(body["error"]["details"]["accountNumber"], "1100");
    assert!(
        body["error"]["message"].as_str().unwrap().contains("1100"),
        "le message doit nommer le compte détenteur : {}",
        body["error"]["message"]
    );
}

/// Code review 14-3a (D1) : la frontière bilan / résultat est validée au POST
/// comme au PUT — un rôle de bilan sur une charge est un 400 typé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn create_rejects_role_on_incompatible_type(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (_company_id, _admin_id, token) = setup(&pool).await;

    let res = app
        .client
        .post(app.url("/api/v1/accounts"))
        .bearer_auth(&token)
        .json(&json!({
            "number": "6500", "name": "Frais admin", "accountType": "Expense",
            "role": "Payable"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 400);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ACCOUNT_ROLE_INVALID_FOR_TYPE");
}

/// Code review 14-3a (HIGH) : `PUT /accounts/{id}` doit refuser un compte d'une
/// autre société (IDOR) — la garde existait sur archive/reactivate mais pas ici,
/// et la story y fait désormais transiter `role`/`postable`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn update_is_scoped_to_company(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (_company_a, _admin_a, token_a) = setup(&pool).await;

    // Compte appartenant à une AUTRE société.
    let company_b = create_company(&pool, "Autre SA").await;
    let admin_b = create_user(&pool, "admin_b", Role::Admin, company_b).await;
    let victim = mk_account(&pool, company_b, admin_b, "1100", None).await;

    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{victim}")))
        .bearer_auth(&token_a)
        .json(&json!({
            "name": "Détourné", "accountType": "Asset",
            "role": "Receivable", "postable": true, "version": 1
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(
        res.status(),
        404,
        "IDOR : modifier le compte d'une autre société doit renvoyer 404"
    );

    // Et le compte victime est intact.
    let still = accounts::find_by_id(&pool, victim).await.unwrap().unwrap();
    assert_eq!(still.name, "Compte 1100");
    assert_eq!(still.role, None);
}

/// Code review 14-3a (D4) : `clearRole: true` réactive un compte dont le rôle a
/// été repris, en le laissant sans rôle plutôt qu'en échouant.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reactivate_with_clear_role_succeeds_after_conflict(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;

    let a = accounts::create(
        &pool,
        admin_id,
        NewAccount::new(company_id, "1100", "Débiteurs", AccountType::Asset, None)
            .with_role(Some(AccountRole::Receivable), true),
    )
    .await
    .unwrap();
    let a = accounts::archive(&pool, a.id, a.version, admin_id)
        .await
        .unwrap();
    // B reprend le rôle.
    mk_account(
        &pool,
        company_id,
        admin_id,
        "1101",
        Some(AccountRole::Receivable),
    )
    .await;

    // Sans le drapeau : 409.
    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{}/reactivate", a.id)))
        .bearer_auth(&token)
        .json(&json!({ "version": a.version }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 409);
    assert_eq!(
        res.json::<Value>().await.unwrap()["error"]["code"],
        "ACCOUNT_ROLE_ALREADY_ASSIGNED"
    );

    // Avec clearRole : 200, sans rôle.
    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{}/reactivate", a.id)))
        .bearer_auth(&token)
        .json(&json!({ "version": a.version, "clearRole": true }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["active"], true);
    assert_eq!(body["role"], Value::Null, "le rôle doit avoir été retiré");
}

/// Conséquence directe du `active AND` de la colonne générée : le rôle est
/// libéré à l'archivage, donc réactiver après reprise doit échouer **proprement**
/// (409 nommant le repreneur) et jamais en 500 sur le 1062 brut.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reactivate_refused_when_singleton_role_was_taken(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;

    let a = mk_account(
        &pool,
        company_id,
        admin_id,
        "1100",
        Some(AccountRole::Receivable),
    )
    .await;
    accounts::archive(&pool, a, 1, admin_id).await.unwrap();
    let b = mk_account(
        &pool,
        company_id,
        admin_id,
        "1101",
        Some(AccountRole::Receivable),
    )
    .await;

    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{a}/reactivate")))
        .bearer_auth(&token)
        .json(&json!({ "version": 2 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 409);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ACCOUNT_ROLE_ALREADY_ASSIGNED");
    assert_eq!(body["error"]["details"]["accountId"], b);
}

// ============================================================
// RBAC & IDOR
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn consultation_role_cannot_write(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "Acme SA").await;
    let admin_id = create_user(&pool, "admin", Role::Admin, company_id).await;
    let viewer_id = create_user(&pool, "viewer", Role::Consultation, company_id).await;
    let viewer_token = forge_jwt(viewer_id, role_str(Role::Consultation), company_id);
    let id = mk_account(&pool, company_id, admin_id, "1000", None).await;

    // Lecture autorisée…
    let res = app
        .client
        .get(app.url("/api/v1/accounts"))
        .bearer_auth(&viewer_token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    // …écriture refusée, réactivation comprise.
    for (method, path) in [
        ("POST", "/api/v1/accounts".to_string()),
        ("PUT", format!("/api/v1/accounts/{id}/reactivate")),
        // Story 47-2 (AC-1) : le `PUT` d'un compte, corps portant `mirrorAccountId`.
        ("PUT", format!("/api/v1/accounts/{id}")),
    ] {
        let req = if method == "POST" {
            app.client.post(app.url(&path))
        } else {
            app.client.put(app.url(&path))
        };
        let res = req
            .bearer_auth(&viewer_token)
            .json(&json!({
                "number": "9", "name": "X", "accountType": "Asset", "version": 1,
                "role": null, "postable": true, "mirrorAccountId": id,
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            403,
            "{method} {path} doit être refusé au rôle Consultation"
        );
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reactivate_is_scoped_to_company(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (_c1, _a1, token) = setup(&pool).await;

    // Compte appartenant à une AUTRE société.
    let other_company = create_company(&pool, "Autre SA").await;
    let other_admin = create_user(&pool, "other-admin", Role::Admin, other_company).await;
    let foreign = mk_account(&pool, other_company, other_admin, "1000", None).await;
    accounts::archive(&pool, foreign, 1, other_admin)
        .await
        .unwrap();

    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{foreign}/reactivate")))
        .bearer_auth(&token)
        .json(&json!({ "version": 2 }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        404,
        "IDOR : un compte d'une autre société doit être invisible"
    );
}

// ============================================================
// Story 37-1 (AC-4, AC-4 bis) — `AccountResponse` : devise, verrou de
// devise, comptes de différence propres. Les DTO de requête et les gardes
// viennent en phase suivante : la devise est posée ici en SQL direct.
// ============================================================

/// C-37-9 — le JSON d'un compte CHF sans ligne n'a **aucune** des quatre clés
/// (`currency`, `currencyLocked`, `fxGainAccountId`, `fxLossAccountId`), en
/// réponse à un compte comme en liste ; un compte en devise à comptes propres
/// les porte ; un compte qui porte une ligne a `currencyLocked: true` (calculé
/// pour la liste en une requête) ; un `PUT` sans le champ garde la devise.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn account_response_porte_devise_verrou_et_comptes_propres(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;

    let res = app
        .client
        .post(app.url("/api/v1/accounts"))
        .bearer_auth(&token)
        .json(&json!({ "number": "1000", "name": "Caisse", "accountType": "Asset" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let chf: Value = res.json().await.unwrap();
    for cle in [
        "currency",
        "currencyLocked",
        "fxGainAccountId",
        "fxLossAccountId",
    ] {
        assert!(
            chf.get(cle).is_none(),
            "compte CHF sans ligne : {cle} absente — {chf}"
        );
    }
    let caisse = chf["id"].as_i64().unwrap();

    let gain = mk_typed_account(&pool, company_id, admin_id, "6950", AccountType::Revenue).await;
    let perte = mk_typed_account(&pool, company_id, admin_id, "6940", AccountType::Expense).await;
    let usd = mk_typed_account(&pool, company_id, admin_id, "1030", AccountType::Asset).await;
    sqlx::query(
        "UPDATE accounts SET currency = 'USD', fx_gain_account_id = ?, fx_loss_account_id = ? \
         WHERE id = ?",
    )
    .bind(gain)
    .bind(perte)
    .bind(usd)
    .execute(&pool)
    .await
    .unwrap();
    let jour = common::jour::aujourd_hui_test();
    let fy = mk_fiscal_year(&pool, company_id, admin_id, jour).await;
    mk_entry(&pool, company_id, fy, admin_id, caisse, perte, jour).await;

    let liste: Value = app
        .client
        .get(app.url("/api/v1/accounts"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let par_id = |id: i64| {
        liste
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["id"] == id)
            .cloned()
            .unwrap_or_else(|| panic!("compte {id} absent de la liste"))
    };
    let c = par_id(caisse);
    assert_eq!(
        c["currencyLocked"], true,
        "une ligne : devise verrouillée — {c}"
    );
    assert!(c.get("currency").is_none(), "{c}");
    let u = par_id(usd);
    assert_eq!(u["currency"], "USD");
    assert_eq!(u["fxGainAccountId"], gain);
    assert_eq!(u["fxLossAccountId"], perte);
    assert!(u.get("currencyLocked").is_none(), "aucune ligne : {u}");
    let g = par_id(gain);
    for cle in [
        "currency",
        "currencyLocked",
        "fxGainAccountId",
        "fxLossAccountId",
    ] {
        assert!(g.get(cle).is_none(), "compte CHF sans ligne : {cle} — {g}");
    }

    // Un PUT qui ne porte pas le champ garde la devise et les comptes propres.
    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{usd}")))
        .bearer_auth(&token)
        .json(&json!({
            "name": "Banque USD", "accountType": "Asset",
            "role": null, "postable": true, "version": 1
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let apres: Value = res.json().await.unwrap();
    assert_eq!(apres["name"], "Banque USD");
    assert_eq!(apres["currency"], "USD");
    assert_eq!(apres["fxGainAccountId"], gain);
    assert_eq!(apres["fxLossAccountId"], perte);

    // La réponse à un compte porte aussi le verrou (une requête par compte).
    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{caisse}")))
        .bearer_auth(&token)
        .json(&json!({
            "name": "Caisse principale", "accountType": "Asset",
            "role": null, "postable": true, "version": 1
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let apres: Value = res.json().await.unwrap();
    assert_eq!(apres["currencyLocked"], true, "{apres}");
}

// ============================================================
// Correctif `fix-api-parent-compte` — `PUT /accounts/{id}` accepte `parentId`
// ============================================================

/// Le `PUT` d'un compte, nom, type, rôle et postabilité tels qu'en base, plus
/// les champs de `extra` (dont `parentId`) ; rend `(statut, corps)`.
async fn put_parent(
    app: &TestApp,
    token: &str,
    pool: &MySqlPool,
    id: i64,
    extra: Value,
) -> (u16, Value) {
    let a = accounts::find_by_id(pool, id).await.unwrap().unwrap();
    let mut body = json!({
        "name": a.name, "accountType": a.account_type.as_str(),
        "role": a.role.map(|r| r.as_str()), "postable": a.postable, "version": a.version,
    });
    for (k, v) in extra.as_object().unwrap() {
        body[k] = v.clone();
    }
    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{id}")))
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = res.status().as_u16();
    (status, res.json().await.unwrap())
}

/// Le défaut relevé à la recette 2024 : un compte livré, qui porte des
/// écritures, se range après coup sous un compte de groupe ; `null` le rend
/// racine ; l'absence du champ ne change rien.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_range_un_compte_mouvemente_sous_un_groupe(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;
    let jour = common::jour::aujourd_hui_test();
    let fy = mk_fiscal_year(&pool, company_id, admin_id, jour).await;
    let banque = mk_account(&pool, company_id, admin_id, "1020", None).await;
    let caisse = mk_account(&pool, company_id, admin_id, "1000", None).await;
    mk_entry(&pool, company_id, fy, admin_id, banque, caisse, jour).await;
    let groupe = mk_account(&pool, company_id, admin_id, "10", None).await;

    // Revue P1-2 : un parent qui porte des écritures est refusé, avec un
    // message traduit et le parent en `details`.
    let (status, body) =
        put_parent(&app, &token, &pool, banque, json!({ "parentId": caisse })).await;
    assert_eq!(status, 409, "{body}");
    assert_eq!(body["error"]["code"], "ACCOUNT_PARENT_HAS_ENTRIES");
    assert_eq!(body["error"]["details"]["parentAccountId"], caisse);
    assert_eq!(body["error"]["details"]["parentNumber"], "1000");
    assert!(
        body["error"]["message"].as_str().unwrap().contains("1000"),
        "{body}"
    );

    let (status, body) =
        put_parent(&app, &token, &pool, banque, json!({ "parentId": groupe })).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["parentId"], groupe);
    assert_eq!(body["postable"], true, "le compte déplacé reste imputable");
    let g = accounts::find_by_id(&pool, groupe).await.unwrap().unwrap();
    assert!(!g.postable, "le groupe devient un compte de regroupement");

    // Absent : inchangé (et no-op, sans bump de version).
    let version = body["version"].clone();
    let (status, body) = put_parent(&app, &token, &pool, banque, json!({})).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["parentId"], groupe);
    assert_eq!(body["version"], version);

    // `null` : racine.
    let (status, body) = put_parent(&app, &token, &pool, banque, json!({ "parentId": null })).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["parentId"], Value::Null);
}

/// Les contrôles de la création, rendus par le `PUT` avec les mêmes codes, et
/// le cycle : le compte lui-même, puis l'un de ses sous-comptes.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_parent_refuse_ce_que_la_creation_refuse_et_le_cycle(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;
    let groupe = mk_account(&pool, company_id, admin_id, "10", None).await;
    let enfant = accounts::create(
        &pool,
        admin_id,
        NewAccount::new(
            company_id,
            "100",
            "Liquidités",
            AccountType::Asset,
            Some(groupe),
        ),
    )
    .await
    .unwrap()
    .id;
    let petit_enfant = accounts::create(
        &pool,
        admin_id,
        NewAccount::new(
            company_id,
            "1000",
            "Caisse",
            AccountType::Asset,
            Some(enfant),
        ),
    )
    .await
    .unwrap()
    .id;

    // Le compte lui-même, puis un sous-compte de deuxième niveau.
    for cible in [groupe, petit_enfant] {
        let (status, body) =
            put_parent(&app, &token, &pool, groupe, json!({ "parentId": cible })).await;
        assert_eq!(status, 409, "{body}");
        assert_eq!(body["error"]["code"], "ACCOUNT_PARENT_CYCLE");
    }
    // Message traduit qui nomme le parent demandé.
    let (_, body) = put_parent(
        &app,
        &token,
        &pool,
        groupe,
        json!({ "parentId": petit_enfant }),
    )
    .await;
    assert!(
        body["error"]["message"].as_str().unwrap().contains("1000"),
        "{body}"
    );

    // Parent d'une autre société : 400, comme à la création (anti-énumération).
    let autre = create_company(&pool, "Autre SA").await;
    let admin_b = create_user(&pool, "admin_b", Role::Admin, autre).await;
    let etranger = mk_account(&pool, autre, admin_b, "20", None).await;
    let (status, body) = put_parent(
        &app,
        &token,
        &pool,
        petit_enfant,
        json!({ "parentId": etranger }),
    )
    .await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    assert_eq!(body["error"]["message"], "parent account not found");

    // Parent archivé : 400, comme à la création.
    let archive = mk_account(&pool, company_id, admin_id, "19", None).await;
    accounts::archive(&pool, archive, 1, admin_id)
        .await
        .unwrap();
    let (status, body) = put_parent(
        &app,
        &token,
        &pool,
        petit_enfant,
        json!({ "parentId": archive }),
    )
    .await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["message"], "parent account is archived");

    // Rien n'a bougé.
    let pe = accounts::find_by_id(&pool, petit_enfant)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(pe.parent_id, Some(enfant));
    let g = accounts::find_by_id(&pool, groupe).await.unwrap().unwrap();
    assert_eq!(g.parent_id, None);
}

// ============================================================
// Story 47-2 (AC-1) — `mirrorAccountId` à l'API des comptes
// ============================================================

/// Un compte de type et de postabilité donnés, sans rôle ni parent.
async fn mk_mirror_account(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    number: &str,
    account_type: AccountType,
    postable: bool,
) -> i64 {
    accounts::create(
        pool,
        user_id,
        NewAccount::new(
            company_id,
            number,
            format!("Compte {number}"),
            account_type,
            None,
        )
        .with_role(None, postable),
    )
    .await
    .unwrap()
    .id
}

/// Les entrées d'audit d'un compte, de la plus récente à la plus ancienne.
async fn audit_of(pool: &MySqlPool, id: i64) -> Vec<kesh_db::entities::AuditLogEntry> {
    kesh_db::repositories::audit_log::find_by_entity(pool, "account", id, 50)
        .await
        .unwrap()
}

/// Les trois états du `PUT` (absent = inchangé, entier = posé, `null` =
/// retiré), le miroir reposé sans changement de version, l'audit de la source
/// qui porte `mirrorAccountId`, la cible ni versionnée ni auditée, et la liste
/// qui rend le lien (le sens inverse s'en déduit).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_mirror_trois_etats_audit_et_liste(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;
    let source = mk_mirror_account(
        &pool,
        company_id,
        admin_id,
        "2270",
        AccountType::Liability,
        true,
    )
    .await;
    let cible = mk_mirror_account(
        &pool,
        company_id,
        admin_id,
        "1179",
        AccountType::Asset,
        false,
    )
    .await;
    let cible_avant = accounts::find_by_id(&pool, cible).await.unwrap().unwrap();
    let audit_cible_avant = audit_of(&pool, cible).await.len();

    // Sans miroir : la clé est absente de la réponse.
    let (status, body) = put_parent(&app, &token, &pool, source, json!({})).await;
    assert_eq!(status, 200, "{body}");
    assert!(body.get("mirrorAccountId").is_none(), "{body}");

    // Posé.
    let (status, body) = put_parent(
        &app,
        &token,
        &pool,
        source,
        json!({ "mirrorAccountId": cible }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["mirrorAccountId"], cible);
    let version = body["version"].clone();
    let audit = audit_of(&pool, source).await;
    assert_eq!(audit[0].action, "account.updated");
    let details = audit[0].details_json.as_ref().unwrap();
    assert_eq!(details["after"]["mirrorAccountId"], cible, "{details}");
    assert_eq!(
        details["before"]["mirrorAccountId"],
        Value::Null,
        "{details}"
    );

    // Absent : inchangé, sans changement de version.
    let (status, body) = put_parent(&app, &token, &pool, source, json!({})).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["mirrorAccountId"], cible);
    assert_eq!(body["version"], version);

    // Reposé : ni refus ni changement de version.
    let (status, body) = put_parent(
        &app,
        &token,
        &pool,
        source,
        json!({ "mirrorAccountId": cible }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["version"], version);

    // La liste rend le lien sur la source, rien sur la cible.
    let res = app
        .client
        .get(app.url("/api/v1/accounts"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    let list: Vec<Value> = res.json().await.unwrap();
    let ligne = |id: i64| list.iter().find(|a| a["id"] == id).unwrap().clone();
    assert_eq!(ligne(source)["mirrorAccountId"], cible);
    assert!(ligne(cible).get("mirrorAccountId").is_none());

    // Retiré.
    let (status, body) = put_parent(
        &app,
        &token,
        &pool,
        source,
        json!({ "mirrorAccountId": null }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert!(body.get("mirrorAccountId").is_none(), "{body}");
    let details = audit_of(&pool, source).await[0]
        .details_json
        .clone()
        .unwrap();
    assert_eq!(details["before"]["mirrorAccountId"], cible, "{details}");
    assert_eq!(
        details["after"]["mirrorAccountId"],
        Value::Null,
        "{details}"
    );

    // La cible : ni version touchée, ni entrée d'audit.
    let cible_apres = accounts::find_by_id(&pool, cible).await.unwrap().unwrap();
    assert_eq!(cible_apres.version, cible_avant.version);
    assert_eq!(audit_of(&pool, cible).await.len(), audit_cible_avant);
}

/// `POST` : un entier pose le miroir (et l'audit `account.created` le porte) ;
/// `null` ou absent, aucun miroir.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_avec_miroir(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;
    let cible = mk_mirror_account(
        &pool,
        company_id,
        admin_id,
        "1179",
        AccountType::Asset,
        false,
    )
    .await;

    let post = |body: Value| {
        let req = app
            .client
            .post(app.url("/api/v1/accounts"))
            .bearer_auth(&token)
            .json(&body);
        async move {
            let res = req.send().await.unwrap();
            let status = res.status().as_u16();
            (status, res.json::<Value>().await.unwrap())
        }
    };
    let (status, body) = post(json!({
        "number": "2270", "name": "Dettes sociales", "accountType": "Liability",
        "mirrorAccountId": cible,
    }))
    .await;
    assert_eq!(status, 201, "{body}");
    assert_eq!(body["mirrorAccountId"], cible);
    let id = body["id"].as_i64().unwrap();
    let audit = audit_of(&pool, id).await;
    assert_eq!(audit[0].action, "account.created");
    assert_eq!(
        audit[0].details_json.as_ref().unwrap()["mirrorAccountId"],
        cible,
        "{:?}",
        audit[0].details_json
    );

    for (numero, extra) in [
        ("2271", json!({ "mirrorAccountId": null })),
        ("2273", json!({})),
    ] {
        let mut body = json!({ "number": numero, "name": "Autre", "accountType": "Liability" });
        for (k, v) in extra.as_object().unwrap() {
            body[k] = v.clone();
        }
        let (status, rendu) = post(body).await;
        assert_eq!(status, 201, "{rendu}");
        assert!(rendu.get("mirrorAccountId").is_none(), "{rendu}");
    }

    // Création refusée : la raison et son message.
    let (status, body) = post(json!({
        "number": "2272", "name": "X", "accountType": "Asset", "mirrorAccountId": cible,
    }))
    .await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "MIRROR_ACCOUNT_INVALID");
    assert_eq!(body["error"]["details"]["reason"], "accountType");
}

/// Chaque refus du `PUT`, avec son code, sa raison et un message traduit ; une
/// cible d'une autre société rend la même `unknown` qu'une cible inexistante ;
/// la version périmée rend `OPTIMISTIC_LOCK_CONFLICT`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_mirror_refus_et_details(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;
    let mk = |n: &'static str, t: AccountType, p: bool| {
        let pool = pool.clone();
        async move { mk_mirror_account(&pool, company_id, admin_id, n, t, p).await }
    };
    let source = mk("2270", AccountType::Liability, true).await;
    let cible = mk("1179", AccountType::Asset, false).await;
    let imputable = mk("1170", AccountType::Asset, true).await;
    let passif = mk("2279", AccountType::Liability, false).await;
    let archivee = mk("1178", AccountType::Asset, false).await;
    accounts::archive(&pool, archivee, 1, admin_id)
        .await
        .unwrap();
    // Une cible qui a elle-même un miroir (2100 → 2101 n'est pas possible :
    // même côté) : 1177 (actif, non imputable) désigne 2278.
    let chaine = mk("1177", AccountType::Asset, false).await;
    let au_bout = mk("2278", AccountType::Liability, false).await;
    let (status, body) = put_parent(
        &app,
        &token,
        &pool,
        chaine,
        json!({ "mirrorAccountId": au_bout }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    let fonds_propres = accounts::create(
        &pool,
        admin_id,
        NewAccount::new(company_id, "2800", "Capital", AccountType::Liability, None)
            .with_role(Some(AccountRole::EquityCapital), true),
    )
    .await
    .unwrap()
    .id;
    let autre = create_company(&pool, "Autre SA").await;
    let admin_b = create_user(&pool, "admin_b", Role::Admin, autre).await;
    let etrangere =
        mk_mirror_account(&pool, autre, admin_b, "1179", AccountType::Asset, false).await;

    let refus = |status: u16, body: &Value, reason: &str| {
        assert_eq!(status, 400, "{body}");
        assert_eq!(body["error"]["code"], "MIRROR_ACCOUNT_INVALID", "{body}");
        assert_eq!(body["error"]["details"]["reason"], reason, "{body}");
        assert!(
            !body["error"]["message"].as_str().unwrap().is_empty(),
            "{body}"
        );
    };
    for (compte, cible_demandee, reason) in [
        (source, source, "self"),
        (source, 999_999_999, "unknown"),
        (source, etrangere, "unknown"),
        (source, archivee, "archived"),
        (source, passif, "accountType"),
        (fonds_propres, cible, "equity"),
        (source, imputable, "notPresentationAccount"),
        (source, chaine, "chain"),
    ] {
        let (status, body) = put_parent(
            &app,
            &token,
            &pool,
            compte,
            json!({ "mirrorAccountId": cible_demandee }),
        )
        .await;
        refus(status, &body, reason);
        if reason == "chain" {
            assert_eq!(
                body["error"]["details"]["linkedAccountNumber"], "1177",
                "{body}"
            );
            assert!(
                body["error"]["message"].as_str().unwrap().contains("1177"),
                "{body}"
            );
        }
    }
    // Inexistante et étrangère : le même corps (anti-énumération).
    let (_, inexistante) = put_parent(
        &app,
        &token,
        &pool,
        source,
        json!({ "mirrorAccountId": 999_999_999 }),
    )
    .await;
    let (_, etranger) = put_parent(
        &app,
        &token,
        &pool,
        source,
        json!({ "mirrorAccountId": etrangere }),
    )
    .await;
    assert_eq!(inexistante, etranger);

    // Version périmée.
    let a = accounts::find_by_id(&pool, source).await.unwrap().unwrap();
    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{source}")))
        .bearer_auth(&token)
        .json(&json!({
            "name": a.name, "accountType": "Liability", "role": null, "postable": true,
            "version": a.version + 7, "mirrorAccountId": cible,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 409);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["error"]["code"], "OPTIMISTIC_LOCK_CONFLICT", "{body}");

    // Rien n'a été posé.
    let a = accounts::find_by_id(&pool, source).await.unwrap().unwrap();
    assert_eq!(a.mirror_account_id, None);
}

/// `ACCOUNT_IS_MIRROR` : rendre le miroir imputable, le retyper, lui poser un
/// rôle, l'archiver — chaque refus nomme la source ; une source archivée
/// ajoute le remède au message de l'archivage.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn account_is_mirror_par_l_api(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;
    let source = mk_mirror_account(
        &pool,
        company_id,
        admin_id,
        "2270",
        AccountType::Liability,
        true,
    )
    .await;
    let cible = mk_mirror_account(
        &pool,
        company_id,
        admin_id,
        "1179",
        AccountType::Asset,
        false,
    )
    .await;
    let (status, body) = put_parent(
        &app,
        &token,
        &pool,
        source,
        json!({ "mirrorAccountId": cible }),
    )
    .await;
    assert_eq!(status, 200, "{body}");

    for (extra, reason) in [
        (json!({ "postable": true }), "postable"),
        (json!({ "accountType": "Expense" }), "accountType"),
        (json!({ "role": "Receivable" }), "role"),
    ] {
        let (status, body) = put_parent(&app, &token, &pool, cible, extra).await;
        assert_eq!(status, 409, "{body}");
        assert_eq!(body["error"]["code"], "ACCOUNT_IS_MIRROR", "{body}");
        assert_eq!(body["error"]["details"]["reason"], reason, "{body}");
        assert!(
            body["error"]["message"].as_str().unwrap().contains("2270"),
            "{body}"
        );
    }

    let archive = |id: i64| {
        let req = app
            .client
            .put(app.url(&format!("/api/v1/accounts/{id}/archive")));
        let token = token.clone();
        let pool = pool.clone();
        async move {
            let v = accounts::find_by_id(&pool, id)
                .await
                .unwrap()
                .unwrap()
                .version;
            let res = req
                .bearer_auth(&token)
                .json(&json!({ "version": v, "confirmDesignated": true }))
                .send()
                .await
                .unwrap();
            let status = res.status().as_u16();
            (status, res.json::<Value>().await.unwrap())
        }
    };
    let (status, body) = archive(cible).await;
    assert_eq!(status, 409, "{body}");
    assert_eq!(body["error"]["code"], "ACCOUNT_IS_MIRROR");
    assert_eq!(body["error"]["details"]["reason"], "archive");
    assert_eq!(body["error"]["details"]["sources"][0]["number"], "2270");
    let message_actif = body["error"]["message"].as_str().unwrap().to_string();

    // Source archivée : le refus dit de la réactiver d'abord.
    let (status, body) = archive(source).await;
    assert_eq!(status, 200, "{body}");
    let (status, body) = archive(cible).await;
    assert_eq!(status, 409, "{body}");
    assert_eq!(body["error"]["details"]["sources"][0]["active"], false);
    let message = body["error"]["message"].as_str().unwrap();
    assert!(message.len() > message_actif.len(), "{message}");
    assert!(message.starts_with(&message_actif), "{message}");
}

/// Revue d'intégration P1 de l'Epic 47 (M-1) — par l'API : un compte de groupe
/// demandé comme miroir rend `400 MIRROR_ACCOUNT_INVALID` `hasChildren` ; un
/// sous-compte créé sous un compte miroir, ou rangé sous lui, rend
/// `409 ACCOUNT_IS_MIRROR` `child`, qui nomme la source. Messages traduits.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_groupe_n_est_pas_miroir_ni_un_miroir_parent_par_l_api(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;
    let mk = |n: &'static str, t: AccountType, p: bool| {
        let pool = pool.clone();
        async move { mk_mirror_account(&pool, company_id, admin_id, n, t, p).await }
    };
    let source = mk("2270", AccountType::Liability, true).await;
    let cible = mk("1179", AccountType::Asset, false).await;
    let groupe = mk("1170", AccountType::Asset, false).await;
    let enfant = mk("1171", AccountType::Asset, false).await;
    let (status, body) =
        put_parent(&app, &token, &pool, enfant, json!({ "parentId": groupe })).await;
    assert_eq!(status, 200, "{body}");

    let (status, body) = put_parent(
        &app,
        &token,
        &pool,
        source,
        json!({ "mirrorAccountId": groupe }),
    )
    .await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "MIRROR_ACCOUNT_INVALID", "{body}");
    assert_eq!(
        body["error"]["details"],
        json!({ "reason": "hasChildren" }),
        "{body}"
    );
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("compte de groupe"),
        "{body}"
    );

    let (status, body) = put_parent(
        &app,
        &token,
        &pool,
        source,
        json!({ "mirrorAccountId": cible }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    let refus_enfant = |status: u16, body: &Value| {
        assert_eq!(status, 409, "{body}");
        assert_eq!(body["error"]["code"], "ACCOUNT_IS_MIRROR", "{body}");
        assert_eq!(body["error"]["details"]["reason"], "child", "{body}");
        assert_eq!(body["error"]["details"]["sources"][0]["number"], "2270");
        let message = body["error"]["message"].as_str().unwrap();
        assert!(
            message.contains("2270") && message.contains("sous-compte"),
            "{body}"
        );
    };
    let res = app
        .client
        .post(app.url("/api/v1/accounts"))
        .bearer_auth(&token)
        .json(&json!({
            "number": "1179.1", "name": "Sous le miroir", "accountType": "Asset",
            "parentId": cible,
        }))
        .send()
        .await
        .unwrap();
    let status = res.status().as_u16();
    refus_enfant(status, &res.json::<Value>().await.unwrap());
    let (status, body) =
        put_parent(&app, &token, &pool, enfant, json!({ "parentId": cible })).await;
    refus_enfant(status, &body);
    let a = accounts::find_by_id(&pool, enfant).await.unwrap().unwrap();
    assert_eq!(a.parent_id, Some(groupe));
}
