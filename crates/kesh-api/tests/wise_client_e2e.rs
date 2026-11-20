//! Story 39-12a1 (Epic 39, phase 2) — le client de l'API Wise, contre un
//! **serveur Wise simulé** lancé dans le processus (AC-5, socle S5 : un seul
//! schéma de test, aucun appel au vrai Wise).
//!
//! Le serveur simulé (`axum` sur `127.0.0.1:0`) joue les profils, les soldes,
//! des activités sur trois pages (curseur), des transferts par `offset`, des
//! devis (une option `BALANCE` juste, aucune, deux), 429 (avec et sans
//! `Retry-After`, et un `Retry-After` de 3600 borné à 120), 5xx, 401, 403
//! portant `x-2fa-approval`, un corps illisible et une réponse qui ne vient
//! pas. Toutes les données sont **fictives**, écrites à la main ; le jeton
//! porte le marqueur du socle S9.
//!
//! Le serveur note chaque requête : le test exige qu'aucune ne porte
//! `X-Signature` ni `x-2fa-approval`, qu'aucune n'emploie un verbe autre que
//! `GET`, et que l'en-tête `Authorization` soit exactement `Bearer <jeton>`.
//! L'attente des pauses est notée et ne dort pas. Aucune base de données.

mod common;

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use kesh_api::errors::AppError;
use kesh_api::provider_download::{ProviderGate, Wait, error_body, windows};
use kesh_api::provider_env::{ProviderApiConfig, ProviderSeries};
use kesh_api::wise::{MAX_DAYS, WISE, WiseClient, WiseProfile, WiseProfileError, dedup_activities};
use serde_json::{Value, json};

use common::secrets_prestataire::{aucun_secret, secret};

fn jour(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// L'instant « maintenant » des tests : après toutes les périodes lues.
fn maintenant() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 20, 12, 0, 0).unwrap()
}

// ---------------------------------------------------------------------------
// Serveur Wise simulé
// ---------------------------------------------------------------------------

/// Une réponse du serveur simulé.
#[derive(Clone, Debug)]
enum Rep {
    /// 200 et ce corps JSON.
    Json(Value),
    /// Statut, `Retry-After`, `x-2fa-approval`, corps.
    Statut(u16, Option<&'static str>, bool, String),
    /// La réponse ne vient pas (au-delà du délai du client).
    Lente,
}

/// Une requête reçue.
#[derive(Clone, Debug)]
struct Req {
    method: String,
    path: String,
    query: HashMap<String, String>,
    raw_query: String,
    authorization: String,
    interdits: Vec<String>,
}

type Repondeur = Arc<dyn Fn(&Req) -> Rep + Send + Sync>;

#[derive(Clone)]
struct Sim {
    /// Réponses forcées, consommées une à une avant le cours normal.
    forcees: Arc<Mutex<VecDeque<Rep>>>,
    repondeur: Arc<Mutex<Repondeur>>,
    requetes: Arc<Mutex<Vec<Req>>>,
}

impl Sim {
    fn forcer(&self, r: Rep) {
        self.forcees.lock().unwrap().push_back(r);
    }
    fn repondre(&self, f: impl Fn(&Req) -> Rep + Send + Sync + 'static) {
        *self.repondeur.lock().unwrap() = Arc::new(f);
    }
    fn requetes(&self) -> Vec<Req> {
        self.requetes.lock().unwrap().clone()
    }
    fn vider(&self) {
        self.requetes.lock().unwrap().clear();
    }
    /// Le nombre de requêtes dont le chemin commence par `prefixe`.
    fn compte(&self, prefixe: &str) -> usize {
        self.requetes()
            .iter()
            .filter(|r| r.path.starts_with(prefixe))
            .count()
    }
}

async fn gerer(
    State(sim): State<Sim>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let interdits = ["x-signature", "x-2fa-approval", "x-2fa-approval-result"]
        .iter()
        .filter(|h| headers.contains_key(**h))
        .map(|h| h.to_string())
        .collect();
    let req = Req {
        method: method.to_string(),
        path: uri.path().to_string(),
        query,
        raw_query: uri.query().unwrap_or_default().to_string(),
        authorization: headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string(),
        interdits,
    };
    sim.requetes.lock().unwrap().push(req.clone());
    let forcee = sim.forcees.lock().unwrap().pop_front();
    let rep = match forcee {
        Some(r) => r,
        None => {
            let f = sim.repondeur.lock().unwrap().clone();
            f(&req)
        }
    };
    match rep {
        Rep::Json(v) => (StatusCode::OK, v.to_string()).into_response(),
        Rep::Statut(code, retry, defi, corps) => {
            let mut r = (StatusCode::from_u16(code).unwrap(), corps).into_response();
            if let Some(v) = retry {
                r.headers_mut().insert("retry-after", v.parse().unwrap());
            }
            if defi {
                r.headers_mut()
                    .insert("x-2fa-approval", "OTT-FICTIF".parse().unwrap());
            }
            r
        }
        Rep::Lente => {
            tokio::time::sleep(Duration::from_secs(3)).await;
            StatusCode::OK.into_response()
        }
    }
}

async fn simuler() -> (String, Sim) {
    let sim = Sim {
        forcees: Arc::default(),
        repondeur: Arc::new(Mutex::new(Arc::new(|_: &Req| Rep::Json(json!({}))))),
        requetes: Arc::default(),
    };
    let app = Router::new().fallback(gerer).with_state(sim.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), sim)
}

/// Attente notée, qui ne dort pas.
#[derive(Default)]
struct Notee(Mutex<Vec<Duration>>);

impl Wait for Notee {
    fn wait(&self, d: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        self.0.lock().unwrap().push(d);
        Box::pin(async {})
    }
}

impl Notee {
    fn secondes(&self) -> Vec<u64> {
        self.0
            .lock()
            .unwrap()
            .iter()
            // Arrondi : la pause restante, relue un instant plus tard, perd quelques microsecondes.
            .map(|d| d.as_secs_f64().round() as u64)
            .collect()
    }
}

struct Banc {
    client: WiseClient,
    sim: Sim,
    attente: Arc<Notee>,
    jeton: String,
}

async fn banc_avec(profile_id: Option<&str>, timeouts: Option<(Duration, Duration)>) -> Banc {
    let (url, sim) = simuler().await;
    let jeton = secret("jeton");
    let mut champs: Vec<(&'static str, &str)> = vec![("TOKEN", jeton.as_str())];
    if let Some(p) = profile_id {
        champs.push(("PROFILE_ID", p));
    }
    let serie = ProviderSeries::for_tests(1, "Wise CHF", &champs);
    let config = ProviderApiConfig::for_tests(WISE, &url, 1, vec![serie.clone()]);
    let attente = Arc::new(Notee::default());
    let client = WiseClient::new(
        &config,
        &serie,
        Arc::new(ProviderGate::new(1)),
        attente.clone(),
        timeouts,
    )
    .unwrap();
    Banc {
        client,
        sim,
        attente,
        jeton,
    }
}

async fn banc() -> Banc {
    banc_avec(None, None).await
}

impl Banc {
    /// Contrôles communs à toute requête reçue : verbe, en-têtes, jeton.
    fn controles_communs(&self) {
        for r in self.sim.requetes() {
            assert_eq!(r.method, "GET", "verbe interdit : {r:?}");
            assert!(r.interdits.is_empty(), "en-tête interdit : {r:?}");
            assert_eq!(r.authorization, format!("Bearer {}", self.jeton));
        }
    }
}

fn profil() -> WiseProfile {
    WiseProfile {
        id: "1001".into(),
        profile_type: "business".into(),
    }
}

/// Une activité fictive de la liste de Wise.
fn activite(rid: u64, status: &str) -> Value {
    json!({
        "id": format!("ACT-FICTIVE-{rid}"),
        "type": "TRANSFER",
        "resource": { "type": "TRANSFER", "id": rid.to_string() },
        "title": format!("<strong>Fictif {rid}</strong>"),
        "description": "Référence fictive",
        "primaryAmount": "<positive>+ 12.50 CHF</positive>",
        "secondaryAmount": "",
        "status": status,
        "createdOn": "2024-03-04T09:00:00.000Z",
        "updatedOn": "2024-03-04T09:00:05Z",
    })
}

// ---------------------------------------------------------------------------
// Profil et soldes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn profil_business_unique_sans_identifiant_configure() {
    let b = banc().await;
    b.sim.repondre(|r| {
        assert_eq!(r.path, "/v1/profiles");
        Rep::Json(json!([
            {"id": 1000, "type": "PERSONAL"},
            {"id": 1001, "type": "BUSINESS"},
        ]))
    });
    let p = b.client.resolve_profile().await.unwrap();
    assert_eq!(p.id, "1001");
    assert_eq!(p.profile_type, "business");
    b.controles_communs();
}

#[tokio::test]
async fn profil_zero_ou_plusieurs_business_est_ambigu() {
    let b = banc().await;
    b.sim
        .repondre(|_| Rep::Json(json!([{"id": 1000, "type": "personal"}])));
    assert!(matches!(
        b.client.resolve_profile().await,
        Err(WiseProfileError::Ambiguous { count: 0 })
    ));
    b.sim.repondre(|_| {
        Rep::Json(json!([{"id": 1, "type": "business"}, {"id": 2, "type": "business"}]))
    });
    assert!(matches!(
        b.client.resolve_profile().await,
        Err(WiseProfileError::Ambiguous { count: 2 })
    ));
}

#[tokio::test]
async fn profil_designe_doit_figurer_dans_la_liste() {
    let b = banc_avec(Some("1000"), None).await;
    b.sim.repondre(|_| {
        Rep::Json(json!([{"id": 1000, "type": "personal"}, {"id": 1001, "type": "business"}]))
    });
    assert_eq!(b.client.resolve_profile().await.unwrap().id, "1000");
    let b = banc_avec(Some("9999"), None).await;
    b.sim
        .repondre(|_| Rep::Json(json!([{"id": 1001, "type": "business"}])));
    assert!(matches!(
        b.client.resolve_profile().await,
        Err(WiseProfileError::Ambiguous { count: 0 })
    ));
}

#[tokio::test]
async fn trois_soldes_lus_en_decimal_par_le_champ_amount() {
    let b = banc().await;
    b.sim.repondre(|r| {
        assert_eq!(r.path, "/v4/profiles/1001/balances");
        assert_eq!(r.query.get("types").map(String::as_str), Some("STANDARD"));
        Rep::Json(json!([
            {"id": 11, "currency": "CHF", "amount": {"value": 1234.5, "currency": "CHF"},
             "cashAmount": {"value": 1234.5, "currency": "CHF"}},
            {"id": 12, "currency": "EUR", "amount": {"value": 0.0, "currency": "EUR"},
             "cashAmount": {"value": 0.0, "currency": "EUR"}},
            {"id": 13, "currency": "USD", "amount": {"value": 99.07, "currency": "USD"},
             "cashAmount": {"value": 99.07, "currency": "USD"}},
        ]))
    });
    let s = b.client.balances(&profil()).await.unwrap();
    assert_eq!(s.len(), 3);
    assert_eq!(s[0].currency, "CHF");
    assert_eq!(s[0].amount.to_string(), "1234.5");
    assert_eq!(s[1].amount.to_string(), "0.0");
    assert_eq!(s[2].amount.to_string(), "99.07");
    assert_eq!(s[2].id, "13");
    b.controles_communs();
}

#[tokio::test]
async fn solde_sans_montant_est_une_reponse_invalide() {
    let b = banc().await;
    b.sim
        .repondre(|_| Rep::Json(json!([{"id": 11, "currency": "CHF"}])));
    match b.client.balances(&profil()).await {
        Err(AppError::ProviderResponseInvalid { provider, details }) => {
            assert_eq!(provider, "wise");
            assert_eq!(details["step"], "balances");
        }
        autre => panic!("{autre:?}"),
    }
}

// ---------------------------------------------------------------------------
// Activités
// ---------------------------------------------------------------------------

#[tokio::test]
async fn activites_sur_trois_pages_par_curseur() {
    let b = banc().await;
    // Le curseur porte `+`, `/` et `=` : il doit arriver intact.
    b.sim.repondre(|r| {
        assert_eq!(r.path, "/v1/profiles/1001/activities");
        assert_eq!(r.query.get("size").map(String::as_str), Some("100"));
        match r.query.get("nextCursor").map(String::as_str) {
            None => Rep::Json(json!({"cursor": "c1+/=", "activities": [activite(1, "COMPLETED"), activite(2, "COMPLETED")]})),
            Some("c1+/=") => Rep::Json(json!({"cursor": "c2", "activities": [activite(3, "COMPLETED"), activite(2, "COMPLETED")]})),
            Some("c2") => Rep::Json(json!({"cursor": null, "activities": [activite(4, "CANCELLED")]})),
            autre => panic!("curseur inattendu : {autre:?}"),
        }
    });
    let w = &windows(jour(2024, 1, 1), jour(2024, 1, 31), MAX_DAYS, maintenant())[0];
    let v = b
        .client
        .activities_window(&profil(), w, maintenant())
        .await
        .unwrap();
    // Quatre activités : la deuxième, vue sur deux pages, n'est gardée qu'une fois.
    let ids: Vec<&str> = v.iter().map(|a| a.resource_id.as_str()).collect();
    assert_eq!(ids, ["1", "2", "3", "4"]);
    assert_eq!(v[3].status, "CANCELLED");
    // Textes tels quels ; secondaryAmount vide = absent ; les deux formes de date.
    assert_eq!(v[0].title, "<strong>Fictif 1</strong>");
    assert_eq!(v[0].primary_amount, "<positive>+ 12.50 CHF</positive>");
    assert_eq!(v[0].secondary_amount, None);
    assert_eq!(v[0].updated_on, "2024-03-04T09:00:05Z");
    assert_eq!(b.sim.compte("/v1/profiles/1001/activities"), 3);
    b.controles_communs();
}

#[tokio::test]
async fn bornes_a_la_milliseconde_en_jours_de_zurich() {
    let b = banc().await;
    b.sim
        .repondre(|_| Rep::Json(json!({"cursor": null, "activities": []})));
    let w = &windows(jour(2024, 1, 1), jour(2024, 1, 31), MAX_DAYS, maintenant())[0];
    b.client
        .activities_window(&profil(), w, maintenant())
        .await
        .unwrap();
    let r = &b.sim.requetes()[0];
    // Hiver : Zurich = UTC+1 ; la fin est la dernière milliseconde du 31.
    assert_eq!(r.query["since"], "2023-12-31T23:00:00.000Z");
    assert_eq!(r.query["until"], "2024-01-31T22:59:59.999Z");
    assert!(r.raw_query.contains("since=2023-12-31T23%3A00%3A00.000Z"));
    b.sim.vider();

    // Été : UTC+2 ; la fin de la dernière fenêtre est bornée à « maintenant ».
    let now = Utc.with_ymd_and_hms(2024, 7, 10, 8, 30, 15).unwrap();
    let w = &windows(jour(2024, 7, 1), jour(2024, 7, 31), MAX_DAYS, now)[0];
    b.client.activities_window(&profil(), w, now).await.unwrap();
    let r = &b.sim.requetes()[0];
    assert_eq!(r.query["since"], "2024-06-30T22:00:00.000Z");
    assert_eq!(r.query["until"], "2024-07-10T08:30:15.000Z");
}

#[tokio::test]
async fn deux_fenetres_une_activite_a_la_frontiere_gardee_une_fois() {
    let b = banc().await;
    // Une année bissextile tient en une fenêtre de 366 jours : 2024 puis 2025.
    let ws = windows(jour(2024, 1, 1), jour(2025, 12, 31), MAX_DAYS, maintenant());
    assert_eq!(ws.len(), 2);
    b.sim.repondre(|r| {
        if r.query["since"].starts_with("2023-12-31") {
            Rep::Json(json!({"cursor": null, "activities": [activite(1, "COMPLETED"), activite(2, "COMPLETED")]}))
        } else {
            Rep::Json(json!({"cursor": null, "activities": [activite(2, "COMPLETED"), activite(3, "COMPLETED")]}))
        }
    });
    let v = b
        .client
        .activities(
            &profil(),
            jour(2024, 1, 1),
            jour(2025, 12, 31),
            maintenant(),
        )
        .await
        .unwrap();
    let ids: Vec<&str> = v.iter().map(|a| a.resource_id.as_str()).collect();
    assert_eq!(ids, ["1", "2", "3"]);
    assert_eq!(b.sim.compte("/v1/profiles/1001/activities"), 2);
    assert_eq!(dedup_activities(v.clone()).len(), 3);
}

#[tokio::test]
async fn plafond_de_pages_une_pagination_sans_fin_est_refusee() {
    let b = banc().await;
    // Un curseur qui ne finit jamais.
    b.sim.repondre(|_| {
        Rep::Json(json!({"cursor": "toujours", "activities": [activite(1, "COMPLETED")]}))
    });
    let w = &windows(jour(2024, 1, 1), jour(2024, 1, 31), MAX_DAYS, maintenant())[0];
    match b.client.activities_window(&profil(), w, maintenant()).await {
        Err(AppError::ProviderResponseInvalid { details, .. }) => {
            assert_eq!(details["step"], "activities");
            assert_eq!(details["reason"], "too_many_pages");
        }
        autre => panic!("{autre:?}"),
    }
    assert_eq!(b.sim.compte("/v1/profiles/1001/activities"), 200);
}

#[tokio::test]
async fn activite_sans_champ_attendu_ou_liste_absente_refusee() {
    let b = banc().await;
    b.sim
        .repondre(|_| Rep::Json(json!({"cursor": null, "activities": [{"id": "x"}]})));
    let w = &windows(jour(2024, 1, 1), jour(2024, 1, 31), MAX_DAYS, maintenant())[0];
    assert!(matches!(
        b.client.activities_window(&profil(), w, maintenant()).await,
        Err(AppError::ProviderResponseInvalid { .. })
    ));
    b.sim.repondre(|_| Rep::Json(json!({"cursor": null})));
    assert!(matches!(
        b.client.activities_window(&profil(), w, maintenant()).await,
        Err(AppError::ProviderResponseInvalid { .. })
    ));
}

// ---------------------------------------------------------------------------
// Transferts et devis
// ---------------------------------------------------------------------------

fn transfert(id: u64) -> Value {
    json!({"id": id, "quoteUuid": format!("00000000-0000-4000-8000-{id:012}"),
           "sourceCurrency": "CHF", "reference": format!("Réf fictive {id}")})
}

#[tokio::test]
async fn transferts_par_offset_periode_elargie_de_31_jours() {
    let b = banc().await;
    b.sim.repondre(|r| {
        assert_eq!(r.path, "/v1/transfers");
        assert_eq!(r.query["profile"], "1001");
        assert_eq!(r.query["limit"], "100");
        match r.query["offset"].as_str() {
            "0" => Rep::Json(Value::Array((1..=100).map(transfert).collect())),
            "100" => Rep::Json(Value::Array((101..=130).map(transfert).collect())),
            autre => panic!("offset inattendu : {autre}"),
        }
    });
    let v = b
        .client
        .transfers(&profil(), jour(2024, 2, 1), jour(2024, 2, 29), maintenant())
        .await
        .unwrap();
    assert_eq!(v.len(), 130);
    assert_eq!(v[0].id, "1");
    assert_eq!(v[0].reference.as_deref(), Some("Réf fictive 1"));
    assert_eq!(v[0].source_currency.as_deref(), Some("CHF"));
    assert!(v[0].quote_uuid.as_deref().unwrap().starts_with("00000000-"));
    let reqs = b.sim.requetes();
    assert_eq!(reqs.len(), 2);
    // 31 jours avant le 1er février, en heure de Zurich : le 1er janvier.
    assert_eq!(
        reqs[0].query["createdDateStart"],
        "2023-12-31T23:00:00.000Z"
    );
    assert_eq!(reqs[0].query["createdDateEnd"], "2024-02-29T22:59:59.999Z");
    b.controles_communs();
}

#[tokio::test]
async fn transferts_page_pleine_puis_page_vide_termine() {
    let b = banc().await;
    b.sim.repondre(|r| match r.query["offset"].as_str() {
        "0" => Rep::Json(Value::Array((1..=100).map(transfert).collect())),
        _ => Rep::Json(json!([])),
    });
    let v = b
        .client
        .transfers(&profil(), jour(2024, 2, 1), jour(2024, 2, 29), maintenant())
        .await
        .unwrap();
    assert_eq!(v.len(), 100);
    assert_eq!(b.sim.compte("/v1/transfers"), 2);
}

#[tokio::test]
async fn transfert_absent_de_la_liste_lu_par_son_identifiant() {
    let b = banc().await;
    b.sim.repondre(|r| {
        assert_eq!(r.path, "/v1/transfers/777");
        Rep::Json(transfert(777))
    });
    let t = b.client.transfer("777").await.unwrap();
    assert_eq!(t.id, "777");
    b.controles_communs();
}

fn devis(options: Value) -> Value {
    json!({"id": "00000000-0000-4000-8000-000000000001", "sourceCurrency": "CHF",
           "payOut": "BANK_TRANSFER", "paymentOptions": options})
}

#[tokio::test]
async fn devis_option_balance_juste_et_virement_sans_frais() {
    let b = banc().await;
    b.sim.repondre(|r| {
        assert!(r.path.starts_with("/v3/profiles/1001/quotes/"), "{}", r.path);
        if r.path.ends_with("-000000000001") {
            Rep::Json(devis(json!([
                {"payIn": "BANK_TRANSFER", "payOut": "BANK_TRANSFER", "sourceCurrency": "CHF", "fee": {"total": 8.0}},
                {"payIn": "BALANCE", "payOut": "BANK_TRANSFER", "sourceCurrency": "CHF", "fee": {"total": 1.37}},
                {"payIn": "BALANCE", "payOut": "SWIFT", "sourceCurrency": "CHF", "fee": {"total": 9.99}},
            ])))
        } else {
            Rep::Json(devis(json!([
                {"payIn": "BALANCE", "payOut": "BANK_TRANSFER", "sourceCurrency": "EUR", "fee": {"total": 0.0}},
            ])))
        }
    });
    let f = b
        .client
        .quote_fee(&profil(), "00000000-0000-4000-8000-000000000001")
        .await
        .unwrap();
    assert_eq!(f.currency, "CHF");
    assert_eq!(f.total.to_string(), "1.37");
    let f0 = b
        .client
        .quote_fee(&profil(), "00000000-0000-4000-8000-000000000002")
        .await
        .unwrap();
    assert_eq!(f0.currency, "EUR");
    assert!(f0.total.is_zero());
    b.controles_communs();
}

#[tokio::test]
async fn devis_sans_option_balance_ou_a_deux_options_refuse() {
    let b = banc().await;
    for options in [
        json!([{"payIn": "BANK_TRANSFER", "payOut": "BANK_TRANSFER", "sourceCurrency": "CHF", "fee": {"total": 1.0}}]),
        json!([
            {"payIn": "BALANCE", "payOut": "BANK_TRANSFER", "sourceCurrency": "CHF", "fee": {"total": 1.0}},
            {"payIn": "BALANCE", "payOut": "BANK_TRANSFER", "sourceCurrency": "CHF", "fee": {"total": 2.0}},
        ]),
    ] {
        b.sim.repondre(move |_| Rep::Json(devis(options.clone())));
        match b.client.quote_fee(&profil(), "q").await {
            Err(AppError::ProviderResponseInvalid { details, .. }) => {
                assert_eq!(details["step"], "quote");
                assert_eq!(details["reason"], "quote_fee");
            }
            autre => panic!("{autre:?}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Erreurs et reprise
// ---------------------------------------------------------------------------

fn ok_profils() -> Rep {
    Rep::Json(json!([{"id": 1001, "type": "business"}]))
}

#[tokio::test]
async fn limitation_de_debit_avec_et_sans_retry_after() {
    let b = banc().await;
    b.sim.repondre(|_| ok_profils());
    b.sim
        .forcer(Rep::Statut(429, Some("3"), false, "{}".into()));
    b.client.profiles().await.unwrap();
    assert_eq!(b.attente.secondes(), vec![3]);

    let b = banc().await;
    b.sim.repondre(|_| ok_profils());
    b.sim.forcer(Rep::Statut(429, None, false, "{}".into()));
    b.client.profiles().await.unwrap();
    assert_eq!(b.attente.secondes(), vec![5]);

    // Un `Retry-After` démesuré est borné à 120 s.
    let b = banc().await;
    b.sim.repondre(|_| ok_profils());
    b.sim
        .forcer(Rep::Statut(429, Some("3600"), false, "{}".into()));
    b.client.profiles().await.unwrap();
    assert_eq!(b.attente.secondes(), vec![120]);
}

#[tokio::test]
async fn erreur_5xx_puis_succes() {
    let b = banc().await;
    b.sim.repondre(|_| ok_profils());
    b.sim.forcer(Rep::Statut(500, None, false, "oups".into()));
    b.client.profiles().await.unwrap();
    assert_eq!(b.sim.compte("/v1/profiles"), 2);
    assert_eq!(b.attente.secondes(), vec![5]);
    b.controles_communs();
}

#[tokio::test]
async fn six_essais_puis_limite_de_debit_ou_indisponible() {
    let b = banc().await;
    b.sim
        .repondre(|_| Rep::Statut(429, None, false, "{}".into()));
    match b.client.profiles().await {
        Err(AppError::ProviderRateLimited { provider, .. }) => assert_eq!(provider, "wise"),
        autre => panic!("{autre:?}"),
    }
    assert_eq!(b.sim.compte("/v1/profiles"), 6);
    assert_eq!(b.attente.secondes(), vec![5, 10, 20, 40, 60]);

    let b = banc().await;
    b.sim
        .repondre(|_| Rep::Statut(503, None, false, "{}".into()));
    let w = &windows(jour(2024, 1, 1), jour(2024, 1, 31), MAX_DAYS, maintenant())[0];
    match b.client.activities_window(&profil(), w, maintenant()).await {
        Err(AppError::ProviderUnavailable { window, page, .. }) => {
            assert_eq!(window.as_deref(), Some("2024-01-01…2024-01-31"));
            assert_eq!(page, Some(1));
        }
        autre => panic!("{autre:?}"),
    }
}

#[tokio::test]
async fn delai_depasse_indisponible() {
    let b = banc_avec(
        None,
        Some((Duration::from_secs(1), Duration::from_millis(80))),
    )
    .await;
    b.sim.repondre(|_| Rep::Lente);
    assert!(matches!(
        b.client.profiles().await,
        Err(AppError::ProviderUnavailable { .. })
    ));
    assert_eq!(b.sim.compte("/v1/profiles"), 6);
}

#[tokio::test]
async fn jeton_refuse_401_sans_rejeu() {
    let b = banc().await;
    b.sim
        .repondre(|_| Rep::Statut(401, None, false, r#"{"error":"unauthorized"}"#.into()));
    assert!(matches!(
        b.client.profiles().await,
        Err(AppError::ProviderAuthFailed { provider: "wise" })
    ));
    assert_eq!(b.sim.compte("/v1/profiles"), 1);
    assert!(b.attente.secondes().is_empty());
}

#[tokio::test]
async fn refus_403_avec_defi_de_authentification_forte_jamais_rejoue() {
    let b = banc().await;
    b.sim
        .repondre(|_| Rep::Statut(403, None, true, r#"{"error":"REJECTED"}"#.into()));
    assert!(matches!(
        b.client.balances(&profil()).await,
        Err(AppError::ProviderPermissionDenied { provider: "wise" })
    ));
    assert_eq!(b.sim.compte("/v4/profiles"), 1, "aucun rejeu du défi");
    // Et le client n'a jamais répondu au défi : aucun en-tête interdit.
    b.controles_communs();
}

#[tokio::test]
async fn autre_statut_api_error_sans_texte_de_wise() {
    let b = banc().await;
    b.sim.repondre(|_| {
        Rep::Statut(
            404,
            None,
            false,
            r#"{"message":"TEXTE-DE-WISE-A-NE-PAS-RENDRE"}"#.into(),
        )
    });
    match b.client.transfer("5").await {
        Err(e @ AppError::ProviderApiError { .. }) => {
            let rendu = format!("{e:?} {e}");
            assert!(!rendu.contains("TEXTE-DE-WISE"), "{rendu}");
        }
        autre => panic!("{autre:?}"),
    }
}

#[tokio::test]
async fn reponse_illisible_ou_tronquee_reponse_invalide() {
    let b = banc().await;
    b.sim
        .repondre(|_| Rep::Statut(200, None, false, r#"[{"id": 1001, "type": "bus"#.into()));
    match b.client.profiles().await {
        Err(AppError::ProviderResponseInvalid { details, .. }) => {
            assert_eq!(details["step"], "profiles");
            assert_eq!(details["reason"], "json");
        }
        autre => panic!("{autre:?}"),
    }
    // Une réponse bien formée qui n'est pas une liste de profils.
    b.sim.repondre(|_| Rep::Json(json!({"id": 1})));
    assert!(matches!(
        b.client.profiles().await,
        Err(AppError::ProviderResponseInvalid { .. })
    ));
}

#[tokio::test]
async fn aucune_requete_ne_porte_de_signature_ni_de_verbe_d_ecriture() {
    let b = banc().await;
    b.sim.repondre(|r| match r.path.as_str() {
        "/v1/profiles" => ok_profils(),
        p if p.ends_with("/balances") => Rep::Json(json!([])),
        p if p.ends_with("/activities") => Rep::Json(json!({"cursor": null, "activities": []})),
        "/v1/transfers" => Rep::Json(json!([])),
        _ => Rep::Json(devis(json!([
            {"payIn": "BALANCE", "payOut": "BANK_TRANSFER", "sourceCurrency": "CHF", "fee": {"total": 0}}
        ]))),
    });
    let p = b.client.resolve_profile().await.ok().unwrap();
    b.client.balances(&p).await.unwrap();
    b.client
        .activities(&p, jour(2024, 1, 1), jour(2024, 1, 31), maintenant())
        .await
        .unwrap();
    b.client
        .transfers(&p, jour(2024, 1, 1), jour(2024, 1, 31), maintenant())
        .await
        .unwrap();
    b.client.quote_fee(&p, "q").await.unwrap();
    assert_eq!(b.sim.requetes().len(), 5);
    b.controles_communs();
}

// ---------------------------------------------------------------------------
// Aucun secret nulle part (socle S9)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn le_jeton_n_apparait_dans_aucune_erreur_reponse_ni_journal() {
    let capture = common::capture_trace::CaptureTrace::installer();
    let mut rendus: Vec<String> = Vec::new();

    for (statut, defi) in [(401u16, false), (403, true), (404, false), (500, false)] {
        let b = banc().await;
        b.sim
            .repondre(move |_| Rep::Statut(statut, None, defi, "{}".into()));
        let e = b.client.profiles().await.unwrap_err();
        rendus.push(format!("{e:?} / {e}"));
        rendus.push(error_body(e, "wise").await.to_string());
        // Le jeton du banc porte bien le marqueur : le contrôle n'est pas vide.
        assert!(b.jeton.contains("KESHSECRETDETEST"));
        rendus.push(format!("{:?}", b.client));
    }
    let b = banc().await;
    b.sim
        .repondre(|_| Rep::Statut(200, None, false, "pas du json".into()));
    let e = b.client.profiles().await.unwrap_err();
    rendus.push(error_body(e, "wise").await.to_string());

    // Témoin : la capture voit bien les évènements du client.
    capture.exiger("API : droits insuffisants");
    capture.exiger("API : refus");
    let evenements = capture.evenements();
    aucun_secret(rendus.iter().map(String::as_str));
    aucun_secret(evenements.iter().map(String::as_str));
}
