//! #1431 — les refus d'un parent qui deviendrait non imputable, à travers la
//! frontière HTTP, sur les trois chemins qui le rendent tel : `POST /accounts`
//! avec `parentId`, `PUT /accounts/{id}/reactivate` d'un sous-compte archivé, et
//! `PUT /accounts/{id}` avec `parentId` (avec `POST /accounts/groups`, qui en
//! partage la garde).
//!
//! - un parent qui porte des écritures en CHF → `409 ACCOUNT_PARENT_HAS_ENTRIES`
//!   (le code du changement de parent, désormais aussi à la création et à la
//!   réactivation) ;
//! - un parent désigné dans les réglages de facturation, ou compte de produit
//!   par défaut d'un article → `409 ACCOUNT_PARENT_DESIGNATED` (code neuf) ;
//! - un parent CHF vide et non désigné reste accepté, et passe non imputable.
//!
//! Sur chaque refus : rien n'est écrit — ni compte créé, ni sous-compte
//! réactivé, ni parent rendu non imputable, ni `version` du parent consommée.
//!
//! Harnais repris de `account_groups_e2e.rs` (JWT forgé, `spawn_app` éphémère).

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
use kesh_db::entities::account::AccountType;
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

struct Ctx {
    company_id: i64,
    admin_id: i64,
    jwt: String,
}

async fn setup(pool: &MySqlPool) -> Ctx {
    let company_id = create_company(pool, "Acme SA").await;
    let admin_id = create_user(pool, "admin", Role::Admin, company_id).await;
    let jwt = forge_jwt(admin_id, "Admin", company_id);
    Ctx {
        company_id,
        admin_id,
        jwt,
    }
}

/// Un compte créé par le dépôt, du type donné, racine ou sous `parent`.
async fn compte(
    pool: &MySqlPool,
    c: &Ctx,
    numero: &str,
    account_type: AccountType,
    parent: Option<i64>,
) -> i64 {
    accounts::create(
        pool,
        c.admin_id,
        NewAccount::new(
            c.company_id,
            numero,
            format!("Compte {numero}"),
            account_type,
            parent,
        ),
    )
    .await
    .unwrap()
    .id
}

/// `(postable, version)` d'un compte, relus.
async fn etat(pool: &MySqlPool, id: i64) -> (bool, i32) {
    let a = accounts::find_by_id(pool, id).await.unwrap().unwrap();
    (a.postable, a.version)
}

/// Une écriture du jour de l'horloge de test entre `debit` et `credit`, dans un
/// exercice de l'année (créé au premier appel).
async fn ecriture(pool: &MySqlPool, c: &Ctx, debit: i64, credit: i64) {
    let jour: NaiveDate = common::jour::aujourd_hui_test();
    let year = jour.year();
    let fy = match fiscal_years::list_by_company(pool, c.company_id)
        .await
        .unwrap()
        .into_iter()
        .find(|fy| fy.start_date <= jour && jour <= fy.end_date)
    {
        Some(fy) => fy.id,
        None => {
            fiscal_years::create(
                pool,
                c.admin_id,
                NewFiscalYear {
                    company_id: c.company_id,
                    name: format!("Exercice {year}"),
                    start_date: NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
                    end_date: NaiveDate::from_ymd_opt(year, 12, 31).unwrap(),
                },
            )
            .await
            .expect("exercice")
            .id
        }
    };
    journal_entries::create(
        pool,
        fy,
        kesh_db::entities::audit_log::Acteur::utilisateur(c.admin_id),
        NewJournalEntry {
            company_id: c.company_id,
            entry_date: jour,
            journal: Journal::OD,
            description: "Écriture de test #1431".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: debit,
                    debit: dec!(100),
                    credit: dec!(0),
                    project_id: None,
                    foreign: None,
                },
                NewJournalEntryLine {
                    account_id: credit,
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

/// Désigne `account_id` comme compte de créance client des réglages de
/// facturation de la société (ligne créée au besoin).
async fn designer_creance(pool: &MySqlPool, c: &Ctx, account_id: i64) {
    sqlx::query(
        "INSERT INTO company_invoice_settings (company_id, default_receivable_account_id) \
         VALUES (?, ?) ON DUPLICATE KEY UPDATE default_receivable_account_id = VALUES(default_receivable_account_id)",
    )
    .bind(c.company_id)
    .bind(account_id)
    .execute(pool)
    .await
    .unwrap();
}

/// Un article (actif ou archivé) dont `account_id` est le compte de produit par
/// défaut ; rend son identifiant.
async fn article(pool: &MySqlPool, c: &Ctx, nom: &str, account_id: i64, actif: bool) -> i64 {
    sqlx::query(
        "INSERT INTO products (company_id, name, unit_price, vat_rate, active, default_revenue_account_id) \
         VALUES (?, ?, 10, 8.1, ?, ?)",
    )
    .bind(c.company_id)
    .bind(nom)
    .bind(actif)
    .bind(account_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

async fn nombre_de_comptes(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM accounts WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// `POST /accounts` d'un sous-compte `numero` sous `parent`.
async fn creer_sous_compte(
    app: &TestApp,
    c: &Ctx,
    numero: &str,
    account_type: &str,
    parent: i64,
) -> (u16, Value) {
    let resp = app
        .client
        .post(app.url("/api/v1/accounts"))
        .bearer_auth(&c.jwt)
        .json(&json!({
            "number": numero,
            "name": format!("Sous-compte {numero}"),
            "accountType": account_type,
            "parentId": parent,
        }))
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(Value::Null))
}

/// Un groupe `parent` et son sous-compte `numero_enfant`, archivé ; le groupe
/// repassé imputable (ce que permet l'archivage de son dernier sous-compte).
/// Rend `(parent, enfant)`.
async fn groupe_au_sous_compte_archive(
    pool: &MySqlPool,
    c: &Ctx,
    numero_parent: &str,
    numero_enfant: &str,
    account_type: AccountType,
) -> (i64, i64) {
    let parent = compte(pool, c, numero_parent, account_type, None).await;
    let enfant = compte(pool, c, numero_enfant, account_type, Some(parent)).await;
    let v = etat(pool, enfant).await.1;
    accounts::archive(pool, enfant, v, c.admin_id)
        .await
        .unwrap();
    sqlx::query("UPDATE accounts SET postable = TRUE WHERE id = ?")
        .bind(parent)
        .execute(pool)
        .await
        .unwrap();
    (parent, enfant)
}

/// `PUT /accounts/{enfant}/reactivate` à la version courante.
async fn reactiver(app: &TestApp, pool: &MySqlPool, c: &Ctx, enfant: i64) -> (u16, Value) {
    let version = etat(pool, enfant).await.1;
    let resp = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{enfant}/reactivate")))
        .bearer_auth(&c.jwt)
        .json(&json!({ "version": version }))
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(Value::Null))
}

/// `PUT /accounts/{id}` qui ne change que le parent (contrat full-replace :
/// nom, type, rôle et postabilité relus).
async fn ranger_sous(
    app: &TestApp,
    pool: &MySqlPool,
    c: &Ctx,
    id: i64,
    parent: i64,
) -> (u16, Value) {
    let a = accounts::find_by_id(pool, id).await.unwrap().unwrap();
    let resp = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{id}")))
        .bearer_auth(&c.jwt)
        .json(&json!({
            "name": a.name,
            "accountType": a.account_type,
            "role": a.role,
            "postable": a.postable,
            "version": a.version,
            "parentId": parent,
        }))
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(Value::Null))
}

/// Le sous-compte réactivé reste archivé, à la même version.
async fn reste_archive(pool: &MySqlPool, enfant: i64, version: i32) {
    let a = accounts::find_by_id(pool, enfant).await.unwrap().unwrap();
    assert!(!a.active, "le sous-compte reste archivé");
    assert_eq!(a.version, version, "rien n'est écrit sur le sous-compte");
}

// ---------------------------------------------------------------- POST /accounts

/// Cas 1 : un parent CHF qui porte des écritures → `409
/// ACCOUNT_PARENT_HAS_ENTRIES`, le code du changement de parent ; rien n'est
/// créé, le parent garde `postable` et `version`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn creation_sous_un_parent_mouvemente_refusee(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let parent = compte(&pool, &c, "1000", AccountType::Asset, None).await;
    let contre = compte(&pool, &c, "2000", AccountType::Liability, None).await;
    ecriture(&pool, &c, parent, contre).await;
    let avant = etat(&pool, parent).await;
    let n = nombre_de_comptes(&pool, c.company_id).await;

    let (status, json) = creer_sous_compte(&app, &c, "1001", "Asset", parent).await;
    assert_eq!(status, 409, "{json}");
    let e = &json["error"];
    assert_eq!(e["code"], "ACCOUNT_PARENT_HAS_ENTRIES");
    assert_eq!(
        e["details"],
        json!({ "parentAccountId": parent, "parentNumber": "1000" })
    );
    assert!(e["message"].as_str().unwrap().contains("1000"), "{e}");
    assert_eq!(
        nombre_de_comptes(&pool, c.company_id).await,
        n,
        "rien n'est créé"
    );
    assert_eq!(etat(&pool, parent).await, avant, "parent inchangé");
    assert!(avant.0, "le parent était imputable");
}

/// Cas 2 : un parent désigné dans les réglages de facturation → `409
/// ACCOUNT_PARENT_DESIGNATED`, `details.designation = invoiceSettings`,
/// `setting = receivable`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn creation_sous_un_compte_des_reglages_refusee(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let parent = compte(&pool, &c, "1100", AccountType::Asset, None).await;
    designer_creance(&pool, &c, parent).await;
    let avant = etat(&pool, parent).await;
    let n = nombre_de_comptes(&pool, c.company_id).await;

    let (status, json) = creer_sous_compte(&app, &c, "1101", "Asset", parent).await;
    assert_eq!(status, 409, "{json}");
    let e = &json["error"];
    assert_eq!(e["code"], "ACCOUNT_PARENT_DESIGNATED");
    assert_eq!(
        e["details"],
        json!({
            "parentAccountId": parent,
            "parentNumber": "1100",
            "designation": "invoiceSettings",
            "setting": "receivable",
        })
    );
    let message = e["message"].as_str().unwrap();
    assert!(
        message.contains("1100") && message.contains("paramètres de facturation"),
        "{message}"
    );
    assert_eq!(
        nombre_de_comptes(&pool, c.company_id).await,
        n,
        "rien n'est créé"
    );
    assert_eq!(etat(&pool, parent).await, avant, "parent inchangé");
}

/// Cas 3 : un parent compte de produit par défaut d'un article → `409
/// ACCOUNT_PARENT_DESIGNATED`, `details.designation = product`, l'article nommé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn creation_sous_le_compte_d_un_article_refusee(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let parent = compte(&pool, &c, "3200", AccountType::Revenue, None).await;
    let produit = article(&pool, &c, "Conseil", parent, true).await;
    let avant = etat(&pool, parent).await;
    let n = nombre_de_comptes(&pool, c.company_id).await;

    let (status, json) = creer_sous_compte(&app, &c, "3201", "Revenue", parent).await;
    assert_eq!(status, 409, "{json}");
    let e = &json["error"];
    assert_eq!(e["code"], "ACCOUNT_PARENT_DESIGNATED");
    assert_eq!(
        e["details"],
        json!({
            "parentAccountId": parent,
            "parentNumber": "3200",
            "designation": "product",
            "productId": produit,
            "productName": "Conseil",
        })
    );
    let message = e["message"].as_str().unwrap();
    assert!(
        message.contains("3200") && message.contains("Conseil"),
        "{message}"
    );
    assert_eq!(
        nombre_de_comptes(&pool, c.company_id).await,
        n,
        "rien n'est créé"
    );
    assert_eq!(etat(&pool, parent).await, avant, "parent inchangé");
}

/// Le cas qui passe toujours : un parent CHF vide, non désigné — `201`, et le
/// parent devient non imputable (compte de regroupement), sans hausse de sa
/// `version`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn creation_sous_un_parent_vide_acceptee(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let parent = compte(&pool, &c, "1500", AccountType::Asset, None).await;
    let avant = etat(&pool, parent).await;

    let (status, json) = creer_sous_compte(&app, &c, "1501", "Asset", parent).await;
    assert_eq!(status, 201, "{json}");
    assert_eq!(json["parentId"], parent);
    let apres = etat(&pool, parent).await;
    assert!(!apres.0, "le parent devient non imputable");
    assert_eq!(apres.1, avant.1, "sans hausse de version");
}

// ------------------------------------------------- PUT /accounts/{id}/reactivate

/// Cas 1 : le parent, repassé imputable, a reçu des écritures en CHF → `409
/// ACCOUNT_PARENT_HAS_ENTRIES`, mêmes `details` ; le message nomme le
/// sous-compte, qui reste archivé ; le parent garde `postable` et `version`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reactivation_sous_un_parent_mouvemente_refusee(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let (parent, enfant) =
        groupe_au_sous_compte_archive(&pool, &c, "1020", "1021", AccountType::Asset).await;
    let contre = compte(&pool, &c, "2100", AccountType::Liability, None).await;
    ecriture(&pool, &c, parent, contre).await;
    let avant = etat(&pool, parent).await;
    let v_enfant = etat(&pool, enfant).await.1;

    let (status, json) = reactiver(&app, &pool, &c, enfant).await;
    assert_eq!(status, 409, "{json}");
    let e = &json["error"];
    assert_eq!(e["code"], "ACCOUNT_PARENT_HAS_ENTRIES");
    assert_eq!(
        e["details"],
        json!({ "parentAccountId": parent, "parentNumber": "1020" })
    );
    let message = e["message"].as_str().unwrap();
    assert!(
        message.contains("1021") && message.contains("1020") && message.contains("reste archivé"),
        "{message}"
    );
    reste_archive(&pool, enfant, v_enfant).await;
    assert_eq!(etat(&pool, parent).await, avant, "parent inchangé");
    assert!(avant.0, "le parent était imputable");
}

/// Cas 2 : le parent, repassé imputable, est désigné dans les réglages de
/// facturation → `409 ACCOUNT_PARENT_DESIGNATED`, message de réactivation.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reactivation_sous_un_compte_des_reglages_refusee(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let (parent, enfant) =
        groupe_au_sous_compte_archive(&pool, &c, "1100", "1101", AccountType::Asset).await;
    designer_creance(&pool, &c, parent).await;
    let avant = etat(&pool, parent).await;
    let v_enfant = etat(&pool, enfant).await.1;

    let (status, json) = reactiver(&app, &pool, &c, enfant).await;
    assert_eq!(status, 409, "{json}");
    let e = &json["error"];
    assert_eq!(e["code"], "ACCOUNT_PARENT_DESIGNATED");
    assert_eq!(
        e["details"],
        json!({
            "parentAccountId": parent,
            "parentNumber": "1100",
            "designation": "invoiceSettings",
            "setting": "receivable",
        })
    );
    let message = e["message"].as_str().unwrap();
    assert!(
        message.contains("1101") && message.contains("1100") && message.contains("reste archivé"),
        "{message}"
    );
    reste_archive(&pool, enfant, v_enfant).await;
    assert_eq!(etat(&pool, parent).await, avant, "parent inchangé");
}

/// Cas 3 : le parent est le compte de produit d'un article **archivé** (un
/// article archivé se réactive) → `409 ACCOUNT_PARENT_DESIGNATED`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reactivation_sous_le_compte_d_un_article_refusee(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let (parent, enfant) =
        groupe_au_sous_compte_archive(&pool, &c, "3200", "3201", AccountType::Revenue).await;
    let produit = article(&pool, &c, "Ancien service", parent, false).await;
    let avant = etat(&pool, parent).await;
    let v_enfant = etat(&pool, enfant).await.1;

    let (status, json) = reactiver(&app, &pool, &c, enfant).await;
    assert_eq!(status, 409, "{json}");
    let e = &json["error"];
    assert_eq!(e["code"], "ACCOUNT_PARENT_DESIGNATED");
    assert_eq!(e["details"]["designation"], "product");
    assert_eq!(e["details"]["productId"], produit);
    assert_eq!(e["details"]["productName"], "Ancien service");
    let message = e["message"].as_str().unwrap();
    assert!(
        message.contains("3201")
            && message.contains("Ancien service")
            && message.contains("reste archivé"),
        "{message}"
    );
    reste_archive(&pool, enfant, v_enfant).await;
    assert_eq!(etat(&pool, parent).await, avant, "parent inchangé");
}

/// Le cas qui passe toujours : parent CHF vide, non désigné — `200`, le
/// sous-compte est actif et le parent repasse non imputable, sans hausse de sa
/// `version`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reactivation_sous_un_parent_vide_acceptee(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let (parent, enfant) =
        groupe_au_sous_compte_archive(&pool, &c, "1500", "1501", AccountType::Asset).await;
    let avant = etat(&pool, parent).await;

    let (status, json) = reactiver(&app, &pool, &c, enfant).await;
    assert_eq!(status, 200, "{json}");
    assert_eq!(json["active"], true);
    let apres = etat(&pool, parent).await;
    assert!(!apres.0, "le parent repasse non imputable");
    assert_eq!(apres.1, avant.1, "sans hausse de version");
}

// ------------------------------------------- PUT /accounts/{id} et route groupée

/// `PUT /accounts/{id}` avec `parentId` : un parent désigné dans les réglages
/// de facturation, puis le compte de produit d'un article → `409
/// ACCOUNT_PARENT_DESIGNATED` (aligné sur la création) ; le compte n'est pas
/// déplacé, les parents gardent `postable` et `version`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn changement_de_parent_vers_un_compte_designe_refuse(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let creance = compte(&pool, &c, "1100", AccountType::Asset, None).await;
    designer_creance(&pool, &c, creance).await;
    let produit_compte = compte(&pool, &c, "3200", AccountType::Asset, None).await;
    let produit = article(&pool, &c, "Conseil", produit_compte, true).await;
    let deplace = compte(&pool, &c, "1110", AccountType::Asset, None).await;
    let v_deplace = etat(&pool, deplace).await.1;

    for (parent, numero, attendu) in [
        (
            creance,
            "1100",
            json!({ "designation": "invoiceSettings", "setting": "receivable" }),
        ),
        (
            produit_compte,
            "3200",
            json!({ "designation": "product", "productId": produit, "productName": "Conseil" }),
        ),
    ] {
        let avant = etat(&pool, parent).await;
        let (status, json) = ranger_sous(&app, &pool, &c, deplace, parent).await;
        assert_eq!(status, 409, "{json}");
        let e = &json["error"];
        assert_eq!(e["code"], "ACCOUNT_PARENT_DESIGNATED");
        let mut details = attendu.clone();
        details["parentAccountId"] = json!(parent);
        details["parentNumber"] = json!(numero);
        assert_eq!(e["details"], details);
        assert_eq!(etat(&pool, parent).await, avant, "parent {numero} inchangé");
    }
    let relu = accounts::find_by_id(&pool, deplace).await.unwrap().unwrap();
    assert_eq!(relu.parent_id, None, "le compte n'est pas déplacé");
    assert_eq!(relu.version, v_deplace);
}

/// `POST /accounts/groups`, qui partage la garde du changement de parent : un
/// compte désigné dans les réglages, réutilisé comme groupe → `409
/// ACCOUNT_PARENT_DESIGNATED` avec `details.groupStep` ; le même compte rangé
/// comme **membre** reste accepté et imputable.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn route_groupee_compte_designe_refuse_en_groupe_accepte_en_membre(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let creance = compte(&pool, &c, "1100", AccountType::Asset, None).await;
    designer_creance(&pool, &c, creance).await;
    let autre = compte(&pool, &c, "1110", AccountType::Asset, None).await;
    let avant = etat(&pool, creance).await;
    let v = |id| {
        let pool = pool.clone();
        async move { etat(&pool, id).await.1 }
    };

    let refus = json!({ "groups": [ {
        "kind": "reuse", "accountId": creance,
        "accounts": [ { "id": autre, "version": v(autre).await } ],
    } ] });
    let resp = app
        .client
        .post(app.url("/api/v1/accounts/groups"))
        .bearer_auth(&c.jwt)
        .json(&refus)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 409);
    let json: Value = resp.json().await.unwrap();
    let e = &json["error"];
    assert_eq!(e["code"], "ACCOUNT_PARENT_DESIGNATED", "{json}");
    assert_eq!(e["details"]["setting"], "receivable");
    assert_eq!(
        e["details"]["groupStep"],
        json!({ "groupIndex": 0, "accountId": autre })
    );
    assert_eq!(
        etat(&pool, creance).await,
        avant,
        "postable et version inchangés"
    );

    let membre = json!({ "groups": [ {
        "kind": "create", "number": "11", "name": "Créances",
        "accounts": [ { "id": creance, "version": v(creance).await } ],
    } ] });
    let resp = app
        .client
        .post(app.url("/api/v1/accounts/groups"))
        .bearer_auth(&c.jwt)
        .json(&membre)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let json: Value = resp.json().await.unwrap();
    assert_eq!(status, 200, "{json}");
    assert!(
        etat(&pool, creance).await.0,
        "rangé comme membre, il reste imputable"
    );
}

/// Ordre des refus (contrat) : un parent à la fois désigné dans les réglages et
/// mouvementé rend `ACCOUNT_PARENT_DESIGNATED` (désignation jugée avant les
/// écritures) ; lié de plus à un compte bancaire, `ACCOUNT_PARENT_BANK_LINKED`
/// (lien jugé avant la désignation).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ordre_des_refus_lien_bancaire_puis_designation_puis_ecritures(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let parent = compte(&pool, &c, "1100", AccountType::Asset, None).await;
    let contre = compte(&pool, &c, "2000", AccountType::Liability, None).await;
    ecriture(&pool, &c, parent, contre).await;
    designer_creance(&pool, &c, parent).await;

    let (status, json) = creer_sous_compte(&app, &c, "1101", "Asset", parent).await;
    assert_eq!(status, 409, "{json}");
    assert_eq!(json["error"]["code"], "ACCOUNT_PARENT_DESIGNATED");

    sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, description, iban, journal_account_id) \
         VALUES (?, 'Banque Test', 'Courant', 'CH9300762011623852957', ?)",
    )
    .bind(c.company_id)
    .bind(parent)
    .execute(&pool)
    .await
    .unwrap();
    let (status, json) = creer_sous_compte(&app, &c, "1101", "Asset", parent).await;
    assert_eq!(status, 409, "{json}");
    assert_eq!(json["error"]["code"], "ACCOUNT_PARENT_BANK_LINKED");
}

/// Choix C-1431-5 : la garde vaut aussi pour un parent **déjà compte de
/// groupe** (non imputable, un sous-compte actif) qui porte des écritures ou une
/// désignation — la règle que le changement de parent et le rattachement d'un
/// groupe appliquaient déjà. Rien n'est créé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn groupe_existant_mouvemente_ou_designe_ne_recoit_pas_de_sous_compte(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    // Un groupe mouvementé (l'état que laissait l'écart #1431), posé en SQL :
    // écriture passée quand il était imputable, puis un sous-compte.
    let groupe = compte(&pool, &c, "1000", AccountType::Asset, None).await;
    let contre = compte(&pool, &c, "2000", AccountType::Liability, None).await;
    ecriture(&pool, &c, groupe, contre).await;
    let premier = compte(&pool, &c, "1001", AccountType::Asset, None).await;
    sqlx::query("UPDATE accounts SET parent_id = ? WHERE id = ?")
        .bind(groupe)
        .bind(premier)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
        .bind(groupe)
        .execute(&pool)
        .await
        .unwrap();
    let n = nombre_de_comptes(&pool, c.company_id).await;
    let (status, json) = creer_sous_compte(&app, &c, "1002", "Asset", groupe).await;
    assert_eq!(status, 409, "{json}");
    assert_eq!(json["error"]["code"], "ACCOUNT_PARENT_HAS_ENTRIES");
    assert_eq!(nombre_de_comptes(&pool, c.company_id).await, n);
}

/// `POST /accounts/groups` : un groupe **rattaché** désigné par un article, et un
/// groupe **créé** sous un parent désigné dans les réglages → `409
/// ACCOUNT_PARENT_DESIGNATED` avec `groupStep` ; ni `postable` ni `version` des
/// comptes désignés ne bougent, aucun compte n'est créé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn route_groupee_rattachement_et_parent_de_creation_designes_refuses(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    // Groupe 32 (un sous-compte actif), désigné par un article, puis un compte à y rattacher.
    let groupe = compte(&pool, &c, "32", AccountType::Revenue, None).await;
    let _sous = compte(&pool, &c, "3201", AccountType::Revenue, Some(groupe)).await;
    let produit = article(&pool, &c, "Conseil", groupe, true).await;
    let libre = compte(&pool, &c, "3202", AccountType::Revenue, None).await;
    // Parent désigné dans les réglages, pour un groupe à créer.
    let creance = compte(&pool, &c, "11", AccountType::Asset, None).await;
    designer_creance(&pool, &c, creance).await;
    let a1 = compte(&pool, &c, "1101", AccountType::Asset, None).await;

    let post = |body: Value| {
        let app = &app;
        let jwt = c.jwt.clone();
        async move {
            let resp = app
                .client
                .post(app.url("/api/v1/accounts/groups"))
                .bearer_auth(jwt)
                .json(&body)
                .send()
                .await
                .unwrap();
            let status = resp.status().as_u16();
            (status, resp.json::<Value>().await.unwrap_or(Value::Null))
        }
    };

    let avant_groupe = etat(&pool, groupe).await;
    let (status, json) = post(json!({ "groups": [ {
        "kind": "attach", "accountId": groupe,
        "accounts": [ { "id": libre, "version": etat(&pool, libre).await.1 } ],
    } ] }))
    .await;
    assert_eq!(status, 409, "{json}");
    let e = &json["error"];
    assert_eq!(e["code"], "ACCOUNT_PARENT_DESIGNATED");
    assert_eq!(e["details"]["designation"], "product");
    assert_eq!(e["details"]["productId"], produit);
    assert_eq!(
        e["details"]["groupStep"],
        json!({ "groupIndex": 0, "accountId": libre })
    );
    assert_eq!(etat(&pool, groupe).await, avant_groupe);

    let avant_creance = etat(&pool, creance).await;
    let n = nombre_de_comptes(&pool, c.company_id).await;
    let (status, json) = post(json!({ "groups": [ {
        "kind": "create", "number": "110", "name": "Créances", "parentId": creance,
        "accounts": [ { "id": a1, "version": etat(&pool, a1).await.1 } ],
    } ] }))
    .await;
    assert_eq!(status, 409, "{json}");
    let e = &json["error"];
    assert_eq!(e["code"], "ACCOUNT_PARENT_DESIGNATED");
    assert_eq!(e["details"]["setting"], "receivable");
    assert_eq!(e["details"]["groupStep"], json!({ "groupIndex": 0 }));
    assert_eq!(
        etat(&pool, creance).await,
        avant_creance,
        "postable et version inchangés"
    );
    assert_eq!(
        nombre_de_comptes(&pool, c.company_id).await,
        n,
        "aucun groupe créé"
    );
}
