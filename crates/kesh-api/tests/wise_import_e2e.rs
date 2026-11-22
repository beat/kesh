//! Story 39-11b (Epic 39, phase 1) — le compte Wise et l'import de ses
//! relevés CSV, par les routes : réglages Wise d'un compte bancaire (AC-2),
//! route d'import (AC-3), rangement par fichier (AC-4), doublons, continuité
//! et compte rendu (AC-5), refus aux autres chemins et non-régression (AC-6).
//!
//! Fixtures fictives seulement : les relevés de 39-11a
//! (`kesh-import/tests/fixtures/wise-releve-*-fictif.csv`), des relevés
//! construits ici, un compte « Wise Fictif » sans existence réelle.

mod common;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::test_fixtures::{
    SeededCompany, attendre_une_requete_en_cours, disable_rounding_to_5_centimes,
    mark_onboarding_complete, seed_accounting_company,
};
use reqwest::multipart;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";
const CHF: &[u8] = include_bytes!("../../kesh-import/tests/fixtures/wise-releve-chf-fictif.csv");
const EUR: &[u8] = include_bytes!("../../kesh-import/tests/fixtures/wise-releve-eur-fictif.csv");
const USD: &[u8] = include_bytes!("../../kesh-import/tests/fixtures/wise-releve-usd-fictif.csv");
const WISE: &str = "Wise Fictif";
/// L'en-tête du relevé Wise (23 colonnes, 39-11a).
const HEADER: &str = "TransferWise ID,Date,Date Time,Amount,Currency,Description,Payment Reference,Running Balance,Exchange From,Exchange To,Exchange Rate,Payer Name,Payee Name,Payee Account Number,Merchant,Card Last Four Digits,Card Holder Full Name,Attachment,Note,Total fees,Exchange To Amount,Transaction Type,Transaction Details Type";

struct TestApp {
    base_url: String,
    client: reqwest::Client,
}

fn test_config() -> Config {
    Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        TEST_ADMIN_PASSWORD.to_string(),
        String::from_utf8(TEST_JWT_SECRET.to_vec()).unwrap(),
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
        base_url: format!("http://{addr}"),
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

/// Une société multi-devises : comptes du plan CHF, EUR, USD (et un second
/// EUR) pour les soldes Wise, comptes de frais ; le serveur et un JWT `Admin`.
struct Ctx {
    pool: MySqlPool,
    app: TestApp,
    jwt: String,
    seeded: SeededCompany,
    chf: i64,
    eur: i64,
    usd: i64,
    eur2: i64,
    frais_chf: i64,
    frais_usd: i64,
    produit: i64,
}

async fn compte(
    pool: &MySqlPool,
    company_id: i64,
    numero: &str,
    type_: &str,
    devise: Option<&str>,
) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, active, postable, currency) \
         VALUES (?, ?, ?, ?, 1, 1, ?)",
    )
    .bind(company_id)
    .bind(numero)
    .bind(format!("Compte {numero}"))
    .bind(type_)
    .bind(devise)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

async fn monter(pool: &MySqlPool) -> Ctx {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    mark_onboarding_complete(pool).await.expect("onboarding");
    disable_rounding_to_5_centimes(pool, seeded.company_id)
        .await
        .expect("arrondi");
    let c = seeded.company_id;
    let gain = compte(pool, c, "6950", "Revenue", None).await;
    let perte = compte(pool, c, "6960", "Expense", None).await;
    let transit = compte(pool, c, "1090", "Asset", None).await;
    sqlx::query(
        "UPDATE companies SET multi_currency_enabled = TRUE, fx_gain_account_id = ?, \
         fx_loss_account_id = ?, fx_transit_account_id = ? WHERE id = ?",
    )
    .bind(gain)
    .bind(perte)
    .bind(transit)
    .bind(c)
    .execute(pool)
    .await
    .unwrap();
    Ctx {
        pool: pool.clone(),
        app: spawn_app(pool.clone()).await,
        jwt: forge_jwt(seeded.admin_user_id, "Admin", c),
        chf: compte(pool, c, "1040", "Asset", None).await,
        eur: compte(pool, c, "1041", "Asset", Some("EUR")).await,
        usd: compte(pool, c, "1042", "Asset", Some("USD")).await,
        eur2: compte(pool, c, "1043", "Asset", Some("EUR")).await,
        frais_chf: compte(pool, c, "6940", "Expense", None).await,
        frais_usd: compte(pool, c, "6942", "Expense", Some("USD")).await,
        produit: compte(pool, c, "3400", "Revenue", None).await,
        seeded,
    }
}

fn code(b: &Value) -> &str {
    b["error"]["code"].as_str().unwrap_or("")
}

/// Une ligne de relevé Wise construite : les 23 colonnes de [`HEADER`],
/// vides hors celles données.
fn ligne(id: &str, date: &str, montant: &str, devise: &str, solde: &str) -> String {
    let sens = if montant.starts_with('-') {
        "DEBIT"
    } else {
        "CREDIT"
    };
    let mut cellules = vec![String::new(); 23];
    cellules[0] = id.into();
    cellules[1] = date.into();
    cellules[2] = format!("{date} 10:00:00.000");
    cellules[3] = montant.into();
    cellules[4] = devise.into();
    cellules[5] = format!("Mouvement {id}");
    cellules[7] = solde.into();
    cellules[21] = sens.into();
    cellules[22] = "CARD".into();
    assert_eq!(HEADER.split(',').count(), cellules.len());
    cellules.join(",")
}

/// Un relevé : l'en-tête puis les lignes, **la plus récente en tête**.
fn releve(lignes: &[String]) -> Vec<u8> {
    let mut s = String::from(HEADER);
    for l in lignes {
        s.push('\n');
        s.push_str(l);
    }
    s.push('\n');
    s.into_bytes()
}

impl Ctx {
    async fn req(
        &self,
        methode: reqwest::Method,
        chemin: &str,
        corps: Option<Value>,
    ) -> (u16, Value) {
        let mut r = self
            .app
            .client
            .request(methode, format!("{}{chemin}", self.app.base_url))
            .bearer_auth(&self.jwt);
        if let Some(c) = corps {
            r = r.json(&c);
        }
        let resp = r.send().await.unwrap();
        let status = resp.status().as_u16();
        (status, resp.json().await.unwrap_or(Value::Null))
    }

    async fn post(&self, chemin: &str, corps: Value) -> (u16, Value) {
        self.req(reqwest::Method::POST, chemin, Some(corps)).await
    }

    async fn put(&self, chemin: &str, corps: Value) -> (u16, Value) {
        self.req(reqwest::Method::PUT, chemin, Some(corps)).await
    }

    async fn count(&self, sql: &str) -> i64 {
        sqlx::query_scalar(sql).fetch_one(&self.pool).await.unwrap()
    }

    /// Le solde Wise lié à `ledger`, créé par la route ; rend le corps.
    async fn solde(&self, ledger: i64, fee: Option<i64>) -> Value {
        let (s, b) = self
            .post(
                "/api/v1/bank-accounts",
                json!({
                    "bankName": "Wise",
                    "journalAccountId": ledger,
                    "wise": { "account": WISE, "feeAccountId": fee },
                }),
            )
            .await;
        assert_eq!(s, 201, "solde Wise : {b}");
        b
    }

    /// Les trois soldes CHF, EUR, USD ; rend leurs identifiants.
    async fn trois_soldes(&self) -> (i64, i64, i64) {
        let c = self.solde(self.chf, Some(self.frais_chf)).await;
        let e = self.solde(self.eur, None).await;
        let u = self.solde(self.usd, Some(self.frais_usd)).await;
        (
            c["id"].as_i64().unwrap(),
            e["id"].as_i64().unwrap(),
            u["id"].as_i64().unwrap(),
        )
    }

    /// `POST /api/v1/wise-imports` avec le jeton donné : `fichiers` (nom,
    /// octets), `compte` facultatif.
    async fn importer_avec(
        &self,
        jeton: &str,
        fichiers: &[(&str, &[u8])],
        compte: Option<&str>,
    ) -> (u16, Value) {
        let mut form = multipart::Form::new();
        for (nom, octets) in fichiers {
            form = form.part(
                "file",
                multipart::Part::bytes(octets.to_vec())
                    .file_name(nom.to_string())
                    .mime_str("text/csv")
                    .unwrap(),
            );
        }
        if let Some(c) = compte {
            form = form.text("account", c.to_string());
        }
        let resp = self
            .app
            .client
            .post(format!("{}/api/v1/wise-imports", self.app.base_url))
            .bearer_auth(jeton)
            .multipart(form)
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        (status, resp.json().await.unwrap_or(Value::Null))
    }

    async fn importer(&self, fichiers: &[(&str, &[u8])]) -> (u16, Value) {
        self.importer_avec(&self.jwt, fichiers, Some(WISE)).await
    }

    /// (entrées d'import, transactions, traces d'import).
    async fn ecrit(&self) -> (i64, i64, i64) {
        (
            self.count("SELECT COUNT(*) FROM bank_imports").await,
            self.count("SELECT COUNT(*) FROM bank_transactions").await,
            self.count(
                "SELECT COUNT(*) FROM audit_log WHERE action IN ('bank_import.created', 'wise_import.created')",
            )
            .await,
        )
    }
}

fn par_fichier<'a>(b: &'a Value, nom: &str) -> &'a Value {
    b["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["filename"] == nom)
        .unwrap_or_else(|| panic!("fichier {nom} absent : {b}"))
}

// ============================================================
// AC-3 à AC-5 — l'import
// ============================================================

/// AC-4, AC-5 — les trois relevés de 39-11a, un par devise, s'importent d'un
/// geste : trois entrées, une transaction par ligne, soldes posés, somme =
/// clôture − ouverture, contrepartie et IBAN ; relus, refusés.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_de_trois_releves_puis_doublon_de_fichier(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (c, e, u) = ctx.trois_soldes().await;
    let fichiers: [(&str, &[u8]); 3] = [("chf.csv", CHF), ("eur.csv", EUR), ("usd.csv", USD)];

    let (s, b) = ctx.importer(&fichiers).await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["account"], WISE);
    assert_eq!(b["files"].as_array().unwrap().len(), 3);
    for (nom, devise, id, n, ouverture, cloture) in [
        ("chf.csv", "CHF", c, 10, "0.00", "2433.55"),
        ("eur.csv", "EUR", e, 5, "0.00", "160.00"),
        ("usd.csv", "USD", u, 3, "0.00", "317.01"),
    ] {
        let f = par_fichier(&b, nom);
        assert_eq!(f["currency"], devise, "{nom}");
        assert_eq!(f["bankAccountId"], id, "{nom}");
        assert_eq!(f["inserted"], n, "{nom}");
        assert_eq!(f["duplicates"], 0, "{nom}");
        assert!(f["importId"].is_i64(), "{nom}");
        assert_eq!(
            f["openingBalance"]
                .as_str()
                .unwrap()
                .parse::<Decimal>()
                .unwrap(),
            ouverture.parse::<Decimal>().unwrap(),
            "{nom}"
        );
        assert_eq!(
            f["closingBalance"]
                .as_str()
                .unwrap()
                .parse::<Decimal>()
                .unwrap(),
            cloture.parse::<Decimal>().unwrap(),
            "{nom}"
        );
        assert_eq!(f["warnings"], json!([]), "{nom}");
        // Somme des transactions = clôture − ouverture.
        let somme: Decimal = sqlx::query_scalar(
            "SELECT SUM(amount) FROM bank_transactions WHERE bank_account_id = ?",
        )
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(somme, cloture.parse::<Decimal>().unwrap(), "{nom}");
    }
    assert_eq!(ctx.ecrit().await, (3, 18, 4));

    // L'entrée d'import CHF : source CSV, période, soldes, nombre.
    let (format, du, au, ouverture, cloture, nombre, nom): (
        String,
        NaiveDate,
        NaiveDate,
        Option<Decimal>,
        Option<Decimal>,
        i32,
        String,
    ) = sqlx::query_as(
        "SELECT source_format, period_from, period_to, opening_balance, closing_balance, \
         transaction_count, filename FROM bank_imports WHERE bank_account_id = ?",
    )
    .bind(c)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(format, "CSV");
    assert_eq!(du, NaiveDate::from_ymd_opt(2020, 1, 10).unwrap());
    assert_eq!(au, NaiveDate::from_ymd_opt(2020, 3, 15).unwrap());
    assert_eq!(ouverture, Some(dec!(0.00)));
    assert_eq!(cloture, Some(dec!(2433.55)));
    assert_eq!(nombre, 10);
    assert_eq!(nom, "chf.csv");

    // Un virement vers un IBAN, et un numéro de compte qui n'en a pas la forme.
    let (iban, nom, reference, provider_type, details): (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
    ) = sqlx::query_as(
        "SELECT counterparty_iban, counterparty_name, reference, provider_type, details \
         FROM bank_transactions WHERE transaction_id = 'TRANSFER-90000005'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(iban.as_deref(), Some("CH5604835012345678009"));
    assert_eq!(nom.as_deref(), Some("Atelier Exemple Sàrl"));
    assert_eq!(reference.as_deref(), Some("Facture 2020-001"));
    assert_eq!(provider_type.as_deref(), Some("TRANSFER"));
    assert_eq!(
        details,
        "Envoi de 120\u{202f}CHF à Atelier Exemple Sàrl · TRANSFER"
    );
    let (iban, details): (Option<String>, String) = sqlx::query_as(
        "SELECT counterparty_iban, details FROM bank_transactions \
         WHERE transaction_id = 'TRANSFER-90000006'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(iban, None, "numéro national : pas d'IBAN");
    assert!(details.ends_with(" · 0123456789"), "{details}");
    // Le même identifiant sur deux soldes (une conversion) : deux transactions.
    assert_eq!(
        ctx.count(
            "SELECT COUNT(*) FROM bank_transactions WHERE transaction_id = 'BALANCE-90000003'"
        )
        .await,
        2
    );

    // Les traces : une par fichier, et celle de l'import.
    let details: Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'wise_import.created'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(details["provider"], "wise");
    assert_eq!(details["provider_account"], WISE);
    assert_eq!(details["files"].as_array().unwrap().len(), 3);
    let modifiers: Vec<Value> = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'bank_import.created'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(
        modifiers
            .iter()
            .all(|d| d["provider"] == "wise" && d["modifiers"] == json!(["wise"])),
        "{modifiers:?}"
    );

    // Relus tels quels : refusés (empreinte), rien d'écrit.
    let (s, b) = ctx.importer(&fichiers[..1]).await;
    assert_eq!((s, code(&b)), (422, "BANK_IMPORT_DUPLICATE_FILE"), "{b}");
    assert_eq!(ctx.ecrit().await, (3, 18, 4));
}

/// AC-5 — continuité, jugée sur le premier jour : un relevé qui chevauche le
/// précédent n'insère que ses lignes neuves, sans avertissement ; un relevé
/// discontinu passe avec `gap` ; un relevé rattrapé après coup dont la fin
/// chevauche un relevé plus récent, mais dont le début ne prolonge pas le
/// précédent, aussi ; un relevé qui prolonge, sans avertissement.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_continuite_et_doublons_de_lignes(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (c, _, _) = ctx.trois_soldes().await;
    let (s, b) = ctx.importer(&[("chf.csv", CHF)]).await;
    assert_eq!(s, 201, "{b}");
    let premier = par_fichier(&b, "chf.csv")["importId"].as_i64().unwrap();

    // A — chevauche (premier jour = dernier jour du relevé précédent) : la
    // ligne déjà connue est ignorée, deux neuves.
    let a = releve(&[
        ligne("CARD-90000021", "25-03-2020", "100.00", "CHF", "2500.00"),
        ligne("CARD-90000020", "20-03-2020", "-33.55", "CHF", "2400.00"),
        "BALANCE-90000008,15-03-2020,15-03-2020 08:00:00.000,1500.00,CHF,Argent reçu de Stripe,,2433.55,,,,Stripe Payments Europe,,,,,,,,,,CREDIT,DEPOSIT".to_string(),
    ]);
    let (s, b) = ctx.importer(&[("a.csv", a.as_slice())]).await;
    assert_eq!(s, 201, "{b}");
    let f = par_fichier(&b, "a.csv");
    assert_eq!(
        (f["inserted"].clone(), f["duplicates"].clone()),
        (json!(2), json!(1))
    );
    assert_eq!(f["warnings"], json!([]), "chevauchement : pas d'écart");
    let import_a = f["importId"].as_i64().unwrap();
    assert_ne!(import_a, premier);

    // B — discontinu : ouverture 2600 pour une clôture précédente de 2500.
    let b_releve = releve(&[ligne(
        "CARD-90000030",
        "01-05-2020",
        "-10.00",
        "CHF",
        "2590.00",
    )]);
    let (s, b) = ctx.importer(&[("b.csv", b_releve.as_slice())]).await;
    assert_eq!(s, 201, "{b}");
    let f = par_fichier(&b, "b.csv");
    assert_eq!(f["inserted"], 1);
    assert_eq!(
        f["warnings"],
        json!([{ "code": "gap", "expected": "2500.00", "found": "2600.00", "previousImportId": import_a }])
    );

    // C — rattrapé après coup : du 01-04 au 05-05, sa fin chevauche B, son
    // début ne prolonge pas A (ouverture 2550 pour 2500) → `gap`.
    let c_releve = releve(&[
        ligne("CARD-90000041", "05-05-2020", "-5.00", "CHF", "2540.00"),
        ligne("CARD-90000040", "01-04-2020", "-5.00", "CHF", "2545.00"),
    ]);
    let (s, b) = ctx.importer(&[("c.csv", c_releve.as_slice())]).await;
    assert_eq!(s, 201, "{b}");
    let f = par_fichier(&b, "c.csv");
    assert_eq!(f["warnings"][0]["code"], "gap", "{b}");
    assert_eq!(f["warnings"][0]["previousImportId"], import_a);

    // D — prolonge le plus tardif (C, clôture 2540) : pas d'avertissement.
    let d = releve(&[ligne(
        "CARD-90000050",
        "10-06-2020",
        "-40.00",
        "CHF",
        "2500.00",
    )]);
    let (s, b) = ctx.importer(&[("d.csv", d.as_slice())]).await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(par_fichier(&b, "d.csv")["warnings"], json!([]));

    // E — un relevé tout en doublons (autres octets : une autre description) :
    // aucune entrée créée.
    let e_releve = String::from_utf8(d)
        .unwrap()
        .replace("Mouvement", "Mvt")
        .into_bytes();
    let avant = ctx.ecrit().await;
    let (s, b) = ctx.importer(&[("e.csv", e_releve.as_slice())]).await;
    assert_eq!(s, 201, "{b}");
    let f = par_fichier(&b, "e.csv");
    assert_eq!(f["importId"], Value::Null);
    assert_eq!(
        (f["inserted"].clone(), f["duplicates"].clone()),
        (json!(0), json!(1))
    );
    assert_eq!(
        ctx.ecrit().await,
        (avant.0, avant.1, avant.2 + 1),
        "aucune entrée ; la trace de l'import seule"
    );
    let total: Decimal =
        sqlx::query_scalar("SELECT SUM(amount) FROM bank_transactions WHERE bank_account_id = ?")
            .bind(c)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        total,
        dec!(2433.55) + dec!(66.45) - dec!(10) - dec!(10) - dec!(40)
    );
}

/// Story 39-11a2 — un même identifiant au débit puis au crédit (réserve de
/// carte expirée puis recréditée, vérification de carte) : les deux lignes
/// s'importent ; la clé de doublon est (identifiant, sens), si bien qu'un
/// relevé qui ne porte que le recrédit l'importe après le débit, et que le
/// réimport du même contenu n'insère rien. Identifiants et montants inventés.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_meme_identifiant_au_debit_puis_au_credit(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (c, _, _) = ctx.trois_soldes().await;
    let virement = ligne("TRANSFER-92000001", "01-12-2020", "150.00", "CHF", "150.00");
    let debit = ligne("CARD-92000002", "20-12-2020", "-32.10", "CHF", "117.90");
    let recredit = ligne("CARD-92000002", "27-12-2020", "32.10", "CHF", "150.00");
    let verif_debit = ligne("CARD-92000003", "28-12-2020", "-0.50", "CHF", "149.50");
    let verif_credit = ligne("CARD-92000003", "28-12-2020", "0.50", "CHF", "150.00");
    let rapport = |b: &Value, nom: &str| {
        let f = par_fichier(b, nom);
        (f["inserted"].clone(), f["duplicates"].clone())
    };

    // 1 — jusqu'au débit de la réserve.
    let un = releve(&[debit.clone(), virement.clone()]);
    let (s, b) = ctx.importer(&[("1.csv", un.as_slice())]).await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(rapport(&b, "1.csv"), (json!(2), json!(0)));

    // 2 — le recrédit seul : même identifiant, l'autre sens → importé.
    let deux = releve(std::slice::from_ref(&recredit));
    let (s, b) = ctx.importer(&[("2.csv", deux.as_slice())]).await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(rapport(&b, "2.csv"), (json!(1), json!(0)));
    assert_eq!(par_fichier(&b, "2.csv")["warnings"], json!([]), "{b}");

    // 3 — tout le mois, plus une vérification de carte : seules les deux
    // lignes de la vérification sont neuves.
    let tout = [verif_credit, verif_debit, recredit, debit, virement];
    let trois = releve(&tout);
    let (s, b) = ctx.importer(&[("3.csv", trois.as_slice())]).await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(rapport(&b, "3.csv"), (json!(2), json!(3)));

    // 4 — le même contenu (autres octets) : rien d'inséré, aucune entrée.
    let quatre = String::from_utf8(trois)
        .unwrap()
        .replace("Mouvement", "Mvt")
        .into_bytes();
    let avant = ctx.ecrit().await;
    let (s, b) = ctx.importer(&[("4.csv", quatre.as_slice())]).await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(rapport(&b, "4.csv"), (json!(0), json!(5)));
    assert_eq!(par_fichier(&b, "4.csv")["importId"], Value::Null);
    assert_eq!(ctx.ecrit().await, (avant.0, avant.1, avant.2 + 1));

    // Les deux sens de chaque identifiant sont en base, et le solde est juste.
    let lignes: Vec<(String, Decimal)> = sqlx::query_as(
        "SELECT transaction_id, amount FROM bank_transactions \
         WHERE bank_account_id = ? AND transaction_id LIKE 'CARD-%' \
         ORDER BY transaction_id, amount",
    )
    .bind(c)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        lignes,
        vec![
            ("CARD-92000002".to_string(), dec!(-32.10)),
            ("CARD-92000002".to_string(), dec!(32.10)),
            ("CARD-92000003".to_string(), dec!(-0.50)),
            ("CARD-92000003".to_string(), dec!(0.50)),
        ]
    );
    let total: Decimal =
        sqlx::query_scalar("SELECT SUM(amount) FROM bank_transactions WHERE bank_account_id = ?")
            .bind(c)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(total, dec!(150.00));
}

/// AC-3 — ordre des refus, rien d'écrit : champ manquant ou aucun fichier
/// (400), compte inconnu (404), fichier illisible (422, nommé), deux fichiers
/// de même devise (422), devise sans compte (422), deux comptes pour une
/// devise (409), compte clos (409) ; rôle Consultation et clé `read` refusés
/// (403) ; clé `read-write` admise, la trace porte la clé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_refus_dans_l_ordre_roles_et_cles(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let chf = ctx.solde(ctx.chf, None).await["id"].as_i64().unwrap();
    ctx.solde(ctx.eur, None).await;

    let (s, b) = ctx.importer_avec(&ctx.jwt, &[("chf.csv", CHF)], None).await;
    assert_eq!((s, code(&b)), (400, "VALIDATION_ERROR"), "{b}");
    let (s, b) = ctx.importer_avec(&ctx.jwt, &[], Some(WISE)).await;
    assert_eq!((s, code(&b)), (400, "VALIDATION_ERROR"), "{b}");
    let (s, b) = ctx
        .importer_avec(&ctx.jwt, &[("chf.csv", CHF)], Some("Wise inconnu"))
        .await;
    assert_eq!((s, code(&b)), (404, "PROVIDER_ACCOUNT_NOT_FOUND"), "{b}");
    assert_eq!(b["error"]["details"]["account"], "Wise inconnu");
    // Un compte PayPal du même libellé n'est pas un compte Wise.
    let (s, b) = ctx
        .post(
            "/api/v1/bank-accounts",
            json!({ "bankName": "PayPal", "journalAccountId": ctx.usd, "paypal": { "account": "Homonyme" } }),
        )
        .await;
    assert_eq!(s, 201, "{b}");
    let (s, b) = ctx
        .importer_avec(&ctx.jwt, &[("chf.csv", CHF)], Some("Homonyme"))
        .await;
    assert_eq!((s, code(&b)), (404, "PROVIDER_ACCOUNT_NOT_FOUND"), "{b}");

    let illisible: &[u8] = b"Date,Amount\n01-01-2020,1.00\n";
    let (s, b) = ctx
        .importer(&[("chf.csv", CHF), ("mauvais.csv", illisible)])
        .await;
    assert_eq!((s, code(&b)), (422, "WISE_CSV_INVALID"), "{b}");
    assert_eq!(b["error"]["details"]["filename"], "mauvais.csv");
    assert_eq!(b["error"]["details"]["reason"], "WISE_CSV_MISSING_COLUMNS");
    assert!(b["error"]["details"]["columns"].as_array().unwrap().len() > 3);

    let (s, b) = ctx
        .importer(&[("a.csv", CHF), ("b.csv", EUR), ("c.csv", CHF)])
        .await;
    assert_eq!((s, code(&b)), (422, "WISE_DUPLICATE_CURRENCY_FILE"), "{b}");
    assert_eq!(
        b["error"]["details"],
        json!({ "currency": "CHF", "filenames": ["a.csv", "c.csv"] })
    );

    let (s, b) = ctx.importer(&[("chf.csv", CHF), ("usd.csv", USD)]).await;
    assert_eq!(
        (s, code(&b)),
        (422, "PROVIDER_CURRENCY_ACCOUNT_MISSING"),
        "{b}"
    );
    assert_eq!(b["error"]["details"]["currencies"], json!(["USD"]));
    assert_eq!(ctx.ecrit().await, (0, 0, 0));

    // Deux comptes EUR de ce libellé (écrits en base : la course de l'AC-2).
    let doublon = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, is_primary, journal_account_id, provider, provider_account) \
         VALUES (?, 'Wise bis', FALSE, ?, 'wise', 'wise FICTIF')",
    )
    .bind(ctx.seeded.company_id)
    .bind(ctx.eur2)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let (s, b) = ctx.importer(&[("eur.csv", EUR)]).await;
    assert_eq!((s, code(&b)), (409, "PROVIDER_CURRENCY_DUPLICATE"), "{b}");
    assert_eq!(b["error"]["details"]["bankAccountId"], doublon);
    sqlx::query("UPDATE bank_accounts SET archived = TRUE WHERE id = ?")
        .bind(doublon)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("UPDATE bank_accounts SET closed_on = '2026-01-31' WHERE id = ?")
        .bind(chf)
        .execute(&pool)
        .await
        .unwrap();
    let (s, b) = ctx.importer(&[("chf.csv", CHF)]).await;
    assert_eq!((s, code(&b)), (409, "BANK_ACCOUNT_CLOSED"), "{b}");
    assert_eq!(ctx.ecrit().await, (0, 0, 0));
    sqlx::query("UPDATE bank_accounts SET closed_on = NULL WHERE id = ?")
        .bind(chf)
        .execute(&pool)
        .await
        .unwrap();

    let consultation = forge_jwt(
        ctx.seeded.admin_user_id,
        "Consultation",
        ctx.seeded.company_id,
    );
    let (s, _) = ctx
        .importer_avec(&consultation, &[("chf.csv", CHF)], Some(WISE))
        .await;
    assert_eq!(s, 403);
    let (_, lecture) =
        common::create_key_via_http(&ctx.app.client, &ctx.app.base_url, &ctx.jwt, "ro", "read")
            .await;
    let (s, _) = ctx
        .importer_avec(&lecture, &[("chf.csv", CHF)], Some(WISE))
        .await;
    assert_eq!(s, 403);
    assert_eq!(ctx.ecrit().await, (0, 0, 0));

    let (cle_id, ecriture) = common::create_key_via_http(
        &ctx.app.client,
        &ctx.app.base_url,
        &ctx.jwt,
        "rw",
        "read-write",
    )
    .await;
    // Le libellé se compare sans casse.
    let (s, b) = ctx
        .importer_avec(
            &ecriture,
            &[("chf.csv", CHF), ("eur.csv", EUR)],
            Some("wise fictif"),
        )
        .await;
    assert_eq!(s, 201, "{b}");
    let acteurs: Vec<(String, Option<i64>)> = sqlx::query_as(
        "SELECT actor_type, actor_api_key_id FROM audit_log \
         WHERE action IN ('wise_import.created', 'bank_import.created')",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(acteurs.len(), 3);
    assert!(
        acteurs
            .iter()
            .all(|(t, k)| t == "api_key" && *k == Some(cle_id)),
        "{acteurs:?}"
    );
}

/// Story 38-11c1, étendue à Wise — **l'import, victime d'un interblocage avec
/// un geste de compte bancaire, est rejoué** par le cœur commun (même montage
/// que `import_rejoue_un_interblocage_avec_un_geste_de_compte` de PayPal).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_wise_rejoue_un_interblocage_avec_un_geste_de_compte(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (c, _, _) = ctx.trois_soldes().await;
    let capture = common::capture_rejeu::CaptureRejeu::installer();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS lest_rejeu (id INT AUTO_INCREMENT PRIMARY KEY, x INT) \
         ENGINE=InnoDB",
    )
    .execute(&pool)
    .await
    .unwrap();
    let mut lourde = pool.begin().await.unwrap();
    for _ in 0..10 {
        sqlx::query("INSERT INTO lest_rejeu (x) SELECT seq FROM seq_1_to_50")
            .execute(&mut *lourde)
            .await
            .unwrap();
    }
    sqlx::query("SELECT id FROM companies WHERE id = ? FOR UPDATE")
        .bind(ctx.seeded.company_id)
        .fetch_all(&mut *lourde)
        .await
        .unwrap();

    let task = {
        let client = ctx.app.client.clone();
        let url = format!("{}/api/v1/wise-imports", ctx.app.base_url);
        let jwt = ctx.jwt.clone();
        tokio::spawn(async move {
            let form = multipart::Form::new()
                .part(
                    "file",
                    multipart::Part::bytes(CHF.to_vec())
                        .file_name("chf.csv")
                        .mime_str("text/csv")
                        .unwrap(),
                )
                .text("account", WISE);
            let resp = client
                .post(url)
                .bearer_auth(jwt)
                .multipart(form)
                .send()
                .await
                .unwrap();
            let status = resp.status().as_u16();
            (status, resp.json::<Value>().await.unwrap_or(Value::Null))
        })
    };
    assert!(
        attendre_une_requete_en_cours(&pool, &["INSERT INTO bank_imports"], || {
            task.is_finished()
        })
        .await,
        "l'import doit attendre `companies` à son INSERT, après sa relecture verrouillante"
    );
    sqlx::query("SELECT id FROM bank_accounts WHERE id = ? FOR UPDATE")
        .bind(c)
        .fetch_all(&mut *lourde)
        .await
        .expect("la transaction de test doit survivre à l'interblocage");
    lourde.rollback().await.unwrap();

    let (s, b) = task.await.unwrap();
    assert_eq!(s, 201, "⛔ l'interblocage doit être rejoué : {b}");
    capture.exiger_un_rejeu("wise_imports::create");
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM audit_log WHERE action = 'wise_import.created'")
            .await,
        1,
        "une seule trace"
    );
}

// ============================================================
// AC-2 — les réglages du compte Wise
// ============================================================

/// AC-2 — un solde Wise se crée sans IBAN, lié à un compte en devise ; la
/// réponse porte `wise` ; chaque garde de 38-11b vaut pour Wise ; `paypal` et
/// `wise` ensemble sont refusés ; un compte PayPal et un compte Wise de même
/// libellé et de même devise coexistent.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn creer_un_solde_wise_et_ses_refus(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let b = ctx.solde(ctx.usd, Some(ctx.frais_usd)).await;
    assert_eq!(b["provider"], "wise");
    assert_eq!(b["iban"], Value::Null);
    assert_eq!(b["paypal"], Value::Null);
    assert_eq!(
        b["wise"],
        json!({ "account": WISE, "feeAccountId": ctx.frais_usd })
    );
    let usd = b["id"].as_i64().unwrap();

    let c = &ctx;
    let creer = |corps: Value| async move { c.post("/api/v1/bank-accounts", corps).await };
    // Garde 2 : compte du plan obligatoire.
    let (s, b) = creer(json!({ "bankName": "Wise", "wise": { "account": WISE } })).await;
    assert_eq!((s, code(&b)), (400, "PROVIDER_LEDGER_REQUIRED"), "{b}");
    // Garde 3 : un compte bancaire par devise et par compte Wise.
    let (s, b) = creer(json!({ "bankName": "Wise", "journalAccountId": ctx.usd,
        "wise": { "account": "wise fictif" } }))
    .await;
    assert_eq!((s, code(&b)), (409, "PROVIDER_CURRENCY_DUPLICATE"), "{b}");
    assert_eq!(b["error"]["details"]["bankAccountId"], usd);
    // … du même prestataire : un compte PayPal homonyme en USD est admis.
    let (s, b) = creer(json!({ "bankName": "PayPal", "journalAccountId": ctx.usd,
        "paypal": { "account": WISE } }))
    .await;
    assert_eq!(s, 201, "PayPal homonyme : {b}");
    // Garde 4 : compte de frais — un produit est refusé.
    let (s, b) = creer(json!({ "bankName": "Wise", "journalAccountId": ctx.eur,
        "wise": { "account": WISE, "feeAccountId": ctx.produit } }))
    .await;
    assert_eq!((s, code(&b)), (400, "PROVIDER_FEE_ACCOUNT_INVALID"), "{b}");
    assert_eq!(b["error"]["details"]["reason"], "not_expense");
    // Garde 1 : libellé vide, numéro de carte.
    let (s, b) = creer(json!({ "bankName": "Wise", "journalAccountId": ctx.eur,
        "wise": { "account": "   " } }))
    .await;
    assert_eq!((s, code(&b)), (400, "VALIDATION_ERROR"), "{b}");
    let (s, b) = creer(json!({ "bankName": "Wise", "journalAccountId": ctx.eur,
        "wise": { "account": "Wise 4111 1111 1111 1111" } }))
    .await;
    assert_eq!((s, code(&b)), (400, "CARD_NUMBER_NOT_ALLOWED"), "{b}");
    assert_eq!(b["error"]["details"]["field"], "wise.account");
    // Ni QR-IBAN, ni principal ; une carte n'est jamais un solde Wise.
    let (s, b) = creer(
        json!({ "bankName": "Wise", "journalAccountId": ctx.eur, "isPrimary": true,
        "wise": { "account": WISE } }),
    )
    .await;
    assert_eq!(
        (s, code(&b)),
        (400, "BANK_ACCOUNT_FIELD_NOT_ALLOWED"),
        "{b}"
    );
    let (s, b) = creer(
        json!({ "bankName": "Wise", "accountKind": "credit_card", "cardLast4": "1234",
        "journalAccountId": ctx.seeded.accounts["2000"], "wise": { "account": WISE } }),
    )
    .await;
    assert_eq!((s, code(&b)), (400, "CREDIT_CARD_FIELD_NOT_ALLOWED"), "{b}");
    assert_eq!(b["error"]["details"]["field"], "wise");
    // `paypal` et `wise` ensemble.
    let (s, b) = creer(json!({ "bankName": "Wise", "journalAccountId": ctx.eur,
        "paypal": { "account": WISE }, "wise": { "account": WISE } }))
    .await;
    assert_eq!((s, code(&b)), (400, "VALIDATION_ERROR"), "{b}");
    // Un IBAN donné est contrôlé.
    let (s, b) = creer(
        json!({ "bankName": "Wise", "journalAccountId": ctx.eur, "iban": "CH00 0000",
        "wise": { "account": WISE } }),
    )
    .await;
    assert_eq!((s, code(&b)), (400, "VALIDATION_ERROR"), "{b}");
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM bank_accounts WHERE provider = 'wise'")
            .await,
        1,
        "un seul solde Wise créé"
    );
}

/// AC-2 — `PUT` : `wise` absent = inchangé, `null` = retiré (compte sans
/// transaction), `paypal: null` sans effet sur un compte Wise, `paypal` et
/// `wise` posés ensemble refusés ; une fois une transaction importée, changer
/// le libellé ou passer à PayPal → `409 PROVIDER_ACCOUNT_IN_USE`, changer le
/// compte de frais reste admis ; `PATCH` du lien refusé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn modifier_un_solde_wise(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let b = ctx.solde(ctx.chf, None).await;
    let id = b["id"].as_i64().unwrap();
    let chemin = format!("/api/v1/bank-accounts/{id}");
    let corps = |version: &Value, extra: Value| {
        let mut c = json!({ "bankName": "Wise", "journalAccountId": ctx.chf, "version": version });
        c.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        c
    };

    let (s, b) = ctx.put(&chemin, corps(&b["version"], json!({}))).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["wise"]["account"], WISE, "absent = inchangé");
    let (s, b) = ctx
        .put(&chemin, corps(&b["version"], json!({ "paypal": null })))
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        b["provider"], "wise",
        "`paypal: null` sans effet sur un compte Wise"
    );
    let (s, e) = ctx
        .put(
            &chemin,
            corps(
                &b["version"],
                json!({ "paypal": { "account": "P" }, "wise": { "account": "W" } }),
            ),
        )
        .await;
    assert_eq!((s, code(&e)), (400, "VALIDATION_ERROR"), "{e}");
    let (s, b) = ctx
        .put(
            &chemin,
            corps(
                &b["version"],
                json!({ "wise": { "account": WISE, "feeAccountId": ctx.frais_chf } }),
            ),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["wise"]["feeAccountId"], ctx.frais_chf);

    // Une transaction importée fige le libellé, le compte lié et le prestataire.
    let (s, r) = ctx.importer(&[("chf.csv", CHF)]).await;
    assert_eq!(s, 201, "{r}");
    let (s, e) = ctx
        .put(
            &chemin,
            corps(&b["version"], json!({ "wise": { "account": "Autre" } })),
        )
        .await;
    assert_eq!((s, code(&e)), (409, "PROVIDER_ACCOUNT_IN_USE"), "{e}");
    let (s, e) = ctx
        .put(
            &chemin,
            corps(&b["version"], json!({ "paypal": { "account": WISE } })),
        )
        .await;
    assert_eq!(
        (s, code(&e)),
        (409, "PROVIDER_ACCOUNT_IN_USE"),
        "passer à PayPal : {e}"
    );
    let (s, e) = ctx
        .put(&chemin, corps(&b["version"], json!({ "wise": null })))
        .await;
    assert_eq!(
        (s, code(&e)),
        (409, "PROVIDER_ACCOUNT_IN_USE"),
        "retirer : {e}"
    );
    let (s, b) = ctx
        .put(
            &chemin,
            corps(&b["version"], json!({ "wise": { "account": WISE } })),
        )
        .await;
    assert_eq!(s, 200, "compte de frais retiré, admis : {b}");
    assert_eq!(b["wise"]["feeAccountId"], Value::Null);

    let (s, e) = ctx
        .req(
            reqwest::Method::PATCH,
            &chemin,
            Some(json!({ "journalAccountId": ctx.chf, "version": b["version"] })),
        )
        .await;
    assert_eq!(
        (s, code(&e)),
        (400, "BANK_ACCOUNT_FIELD_NOT_ALLOWED"),
        "{e}"
    );

    // Un second solde, sans transaction : `wise: null` le rend ordinaire (avec
    // son IBAN), puis il redevient Wise.
    let b2 = ctx.solde(ctx.eur, None).await;
    let id2 = b2["id"].as_i64().unwrap();
    let (s, b2) = ctx
        .put(
            &format!("/api/v1/bank-accounts/{id2}"),
            json!({ "bankName": "Wise", "iban": "CH9300762011623852957", "version": b2["version"],
                "wise": null }),
        )
        .await;
    assert_eq!(s, 200, "{b2}");
    assert_eq!(
        (b2["provider"].clone(), b2["wise"].clone()),
        (Value::Null, Value::Null)
    );
}

/// AC-2, AC-6 — un compte Wise ne reçoit ni relevé bancaire (CAMT, relevé
/// saisi) ni règlement de facture : `409 BANK_ACCOUNT_IS_PROVIDER`, comme un
/// compte PayPal (les gardes lisent `provider IS NOT NULL`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn les_autres_chemins_refusent_un_compte_wise(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let chf = ctx.solde(ctx.chf, None).await["id"].as_i64().unwrap();
    let jour = common::jour::aujourd_hui_test().to_string();

    let camt = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../kesh-import/tests/fixtures/camt053/v04_characterization.xml"),
    )
    .expect("fixture CAMT");
    for route in ["/api/v1/bank-imports/preview", "/api/v1/bank-imports"] {
        let form = multipart::Form::new()
            .text("bankAccountId", chf.to_string())
            .part(
                "file",
                multipart::Part::bytes(camt.clone())
                    .file_name("releve.xml")
                    .mime_str("application/xml")
                    .unwrap(),
            );
        let resp = ctx
            .app
            .client
            .post(format!("{}{route}", ctx.app.base_url))
            .bearer_auth(&ctx.jwt)
            .multipart(form)
            .send()
            .await
            .unwrap();
        let s = resp.status().as_u16();
        let b: Value = resp.json().await.unwrap();
        assert_eq!(
            (s, code(&b)),
            (409, "BANK_ACCOUNT_IS_PROVIDER"),
            "{route} : {b}"
        );
    }
    let releve = json!({
        "bankAccountId": chf, "periodFrom": "2026-09-01", "periodTo": "2026-09-30",
        "openingBalance": "0.00", "closingBalance": "50.00",
        "lines": [{ "bookingDate": "2026-09-15", "valueDate": null, "details": "Vente",
            "counterpartyName": null, "reference": null, "amount": "50.00", "direction": "credit" }]
    });
    let (s, b) = ctx.post("/api/v1/bank-imports/manual", releve).await;
    assert_eq!((s, code(&b)), (409, "BANK_ACCOUNT_IS_PROVIDER"), "{b}");
    assert_eq!(ctx.ecrit().await, (0, 0, 0));

    let client = kesh_db::repositories::contacts::create(
        &ctx.pool,
        ctx.seeded.admin_user_id,
        kesh_db::entities::contact::NewContact {
            company_id: ctx.seeded.company_id,
            contact_type: kesh_db::entities::contact::ContactType::Entreprise,
            name: "Client".into(),
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
            address: Some("Rue 2\n1000 Lausanne".into()),
            address_street: None,
            address_building: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            email: None,
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: None,
            salutation: kesh_db::entities::contact::Salutation::Neutre,
        },
    )
    .await
    .unwrap()
    .id;
    let (s, b) = ctx
        .post("/api/v1/invoices", json!({ "contactId": client, "date": jour,
            "lines": [{"description": "Conseil", "quantity": "1", "unitPrice": "100.00", "vatRate": "8.10"}] }))
        .await;
    assert_eq!(s, 201, "facture : {b}");
    let invoice = b["id"].as_i64().unwrap();
    let (s, b) = ctx
        .req(
            reqwest::Method::POST,
            &format!("/api/v1/invoices/{invoice}/validate"),
            None,
        )
        .await;
    assert_eq!(s, 200, "validation : {b}");
    let (s, b) = ctx
        .post(
            &format!("/api/v1/invoices/{invoice}/settlements"),
            json!({ "settlementType": "bank_transfer", "bankAccountId": chf, "amount": "108.10",
                "settledOn": jour, "version": b["version"] }),
        )
        .await;
    assert_eq!((s, code(&b)), (409, "BANK_ACCOUNT_IS_PROVIDER"), "{b}");

    // Paiement d'une facture fournisseur, puis lot pain.001 (revue P1, L3) :
    // refusés, rien d'écrit.
    let fournisseur = kesh_db::repositories::contacts::create(
        &ctx.pool,
        ctx.seeded.admin_user_id,
        kesh_db::entities::contact::NewContact {
            company_id: ctx.seeded.company_id,
            contact_type: kesh_db::entities::contact::ContactType::Entreprise,
            name: "Fournisseur SA".into(),
            first_name: None,
            last_name: None,
            is_client: false,
            is_supplier: true,
            address: Some("Rue 2\n1000 Lausanne".into()),
            address_street: None,
            address_building: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            email: None,
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: None,
            salutation: kesh_db::entities::contact::Salutation::Neutre,
        },
    )
    .await
    .unwrap()
    .id;
    sqlx::query(
        "UPDATE company_invoice_settings SET default_payable_account_id = ? WHERE company_id = ?",
    )
    .bind(ctx.seeded.accounts["2000"])
    .bind(ctx.seeded.company_id)
    .execute(&ctx.pool)
    .await
    .unwrap();
    let ff = kesh_db::repositories::supplier_invoices::create_validated(
        &ctx.pool,
        kesh_db::entities::NewSupplierInvoice {
            company_id: ctx.seeded.company_id,
            contact_id: fournisseur,
            supplier_invoice_number: Some("FF-1".into()),
            invoice_date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            due_date: None,
            creditor_iban: None,
            creditor_qr_iban: None,
            payment_reference: None,
            expected_payment_amount: None,
            project_id: None,
            lines: vec![kesh_db::entities::NewSupplierInvoiceLine {
                description: "Prestation".into(),
                quantity: dec!(1),
                unit_price: dec!(100.00),
                vat_rate: dec!(0),
                expense_account_id: ctx.seeded.accounts["4000"],
            }],
        },
        ctx.seeded.admin_user_id,
    )
    .await
    .expect("facture fournisseur")
    .invoice
    .id;
    let ecritures = ctx.count("SELECT COUNT(*) FROM journal_entries").await;
    let (s, b) = ctx
        .post(
            &format!("/api/v1/supplier-invoices/{ff}/pay"),
            json!({ "settlementType": "bank_transfer", "bankAccountId": chf, "paymentDate": "2026-06-20" }),
        )
        .await;
    assert_eq!((s, code(&b)), (409, "BANK_ACCOUNT_IS_PROVIDER"), "{b}");
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM journal_entries").await,
        ecritures,
        "paiement : rien d'écrit"
    );
    let (s, b) = ctx
        .post(
            "/api/v1/payment-batches",
            json!({ "bankAccountId": chf, "requestedExecutionDate": "2026-06-25",
                "supplierInvoiceIds": [ff] }),
        )
        .await;
    assert_eq!((s, code(&b)), (409, "BANK_ACCOUNT_IS_PROVIDER"), "{b}");
    assert_eq!(ctx.count("SELECT COUNT(*) FROM payment_batches").await, 0);
}

// ============================================================
// Story 39-11e (D5) — le relevé exporté sans les frais séparés
// ============================================================

/// Le relevé CHF de 39-11a exporté **sans** l'option « frais indiqués
/// séparément » : lignes `FEE-` retirées, virement et conversion frais compris
/// (les soldes courants des lignes restantes ne changent pas).
fn chf_sans_frais() -> Vec<u8> {
    let texte = std::str::from_utf8(CHF).unwrap();
    let mut s = String::new();
    for l in texte.lines().filter(|l| !l.starts_with("FEE-")) {
        s.push_str(l);
        s.push('\n');
    }
    for (avant, apres) in [
        (",-300.00,CHF,", ",-301.20,CHF,"),
        (",-500.00,CHF,", ",-500.35,CHF,"),
    ] {
        assert_eq!(s.matches(avant).count(), 1, "{avant}");
        s = s.replace(avant, apres);
    }
    s.into_bytes()
}

/// AC-4 — `422 WISE_CSV_FEES_NOT_SEPARATED`, code propre (non
/// `WISE_CSV_INVALID`), le fichier nommé, `archive` nul pour un fichier à
/// part, les lignes à frais et leur nombre ; rien d'écrit, d'aucun fichier ;
/// le premier fichier refusé dans l'ordre de la liste.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_refuse_un_releve_exporte_sans_les_frais_separes(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    ctx.trois_soldes().await;
    let sans = chf_sans_frais();

    let (s, b) = ctx.importer(&[("eur.csv", EUR), ("chf.csv", &sans)]).await;
    assert_eq!((s, code(&b)), (422, "WISE_CSV_FEES_NOT_SEPARATED"), "{b}");
    assert_eq!(
        b["error"]["details"],
        json!({ "filename": "chf.csv", "archive": null, "lines": [6, 7], "count": 2 })
    );
    let message = b["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("« chf.csv »") && message.contains("en cochant cette option"),
        "{message}"
    );
    assert_eq!(ctx.ecrit().await, (0, 0, 0));

    // Un fichier illisible placé après : le relevé sans frais, premier refusé
    // dans l'ordre de la liste, l'emporte ; placé avant, l'illisible.
    let illisible: &[u8] = b"Date,Amount\n01-01-2020,1.00\n";
    let (s, b) = ctx
        .importer(&[("chf.csv", &sans), ("mauvais.csv", illisible)])
        .await;
    assert_eq!((s, code(&b)), (422, "WISE_CSV_FEES_NOT_SEPARATED"), "{b}");
    let (s, b) = ctx
        .importer(&[("mauvais.csv", illisible), ("chf.csv", &sans)])
        .await;
    assert_eq!((s, code(&b)), (422, "WISE_CSV_INVALID"), "{b}");
    assert_eq!(ctx.ecrit().await, (0, 0, 0));

    // Le bon relevé passe.
    let (s, b) = ctx.importer(&[("chf.csv", CHF)]).await;
    assert_eq!(s, 201, "{b}");
}
