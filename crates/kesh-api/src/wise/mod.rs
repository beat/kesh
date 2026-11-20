//! Client de l'API Wise, voie « activités », **lecture seule** (Story 39-12a1).
//!
//! Ce qui est propre à Wise, le socle faisant le reste ([`crate::http_client`],
//! [`crate::provider_download`], [`crate::provider_env`]) :
//!
//! - **jeton personnel en lecture seule** (`Authorization: Bearer`), rien
//!   d'autre : aucune signature, aucun en-tête `x-2fa-approval` ni
//!   `X-Signature` — la voie des relevés de solde, fermée par Wise aux jetons
//!   personnels de ce compte, est abandonnée ;
//! - **requêtes `GET` seulement** : aucune route de Wise qui écrit n'est
//!   appelée ;
//! - le client **n'assemble rien** : il rend le profil, les soldes, les
//!   activités d'une période, les transferts et le devis d'un transfert ;
//!   l'assemblage en lignes est celui de 39-12a2, l'ordre des appels et la
//!   progression ceux de 39-12a3.
//!
//! Appels (relevés réels du 2026-11-20, sur 2024, jeton seul, sans défi) :
//!
//! | Étape (`step`) | Appel |
//! |---|---|
//! | `profiles` | `GET /v1/profiles` |
//! | `balances` | `GET /v4/profiles/{p}/balances?types=STANDARD` |
//! | `activities` | `GET /v1/profiles/{p}/activities?since&until&size=100[&nextCursor]` |
//! | `transfers` | `GET /v1/transfers?profile&createdDateStart&createdDateEnd&limit&offset`, `GET /v1/transfers/{id}` |
//! | `quote` | `GET /v3/profiles/{p}/quotes/{quoteUuid}` |
//!
//! Erreurs (jamais le texte de Wise, jamais l'URL ni le jeton) : 401
//! `PROVIDER_AUTH_FAILED` ; 403 **avec ou sans** défi d'authentification forte
//! `PROVIDER_PERMISSION_DENIED`, sans rejeu ; 429 et 5xx repris par
//! [`with_retry`] (`Retry-After` entier borné à 120 s, sinon 5, 10, 20, 40,
//! 60 s ; six essais ; pause commune) puis `PROVIDER_RATE_LIMITED` ou
//! `PROVIDER_UNAVAILABLE` ; délai dépassé `PROVIDER_UNAVAILABLE` ; corps
//! illisible ou champ attendu absent `PROVIDER_RESPONSE_INVALID` ; tout autre
//! statut `PROVIDER_API_ERROR`.
//!
//! **Écart connu avec la fiche, à raccorder à la fusion** : les erreurs du
//! socle (`AppError::Provider*`) portent `details: { provider, window?, page? }`
//! et non `{ provider, step, from?, to? }`. Le nom de l'étape ne s'y exprime que
//! pour `PROVIDER_RESPONSE_INVALID` (`details.step`) ; ailleurs, `window` porte
//! la fenêtre (`2024-01-01…2024-12-31`) et `page` la page. De même
//! `WISE_PROFILE_AMBIGUOUS` n'est pas encore une variante d'`AppError` :
//! [`WiseProfileError::Ambiguous`] la porte, 39-12a3 la rend (variante et
//! textes aux quatre locales).

use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Days, NaiveDate, Utc};
use kesh_core::horloge::{debut_du_jour_suisse, fin_du_jour_suisse};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::Value;
use serde_json::value::RawValue;

use crate::errors::AppError;
use crate::http_client::{BodyError, build_client, read_body_bounded};
use crate::provider_download::{
    Attempt, ProviderGate, RetryError, Transient, Wait, Window, windows, with_retry,
};
use crate::provider_env::{ProviderApiConfig, ProviderSeries, ProviderSpec};

/// Le prestataire dans l'API et les journaux.
pub const PROVIDER: &str = "wise";

/// Wise : un jeton personnel en lecture seule par compte Wise, et un
/// identifiant de profil **facultatif** (sans lui, le seul profil `business`
/// du jeton). Inscrit à [`crate::provider_env::ALL_PROVIDERS`].
pub const WISE: ProviderSpec = ProviderSpec {
    key: "WISE",
    provider: PROVIDER,
    fields: &["TOKEN"],
    optional_fields: &["PROFILE_ID"],
    default_base_url: "https://api.wise.com",
};

/// Jours de Zurich par fenêtre d'activités : Wise ne borne pas la période, la
/// fenêtre ne sert qu'à découper la progression (366 couvre une année civile
/// bissextile).
pub const MAX_DAYS: u32 = 366;
/// Activités par page (maximum de l'API).
pub const ACTIVITIES_PAGE_SIZE: u32 = 100;
/// Transferts par page. La fiche annonçait 200 ; les relevés réels ont été
/// faits à 100, et une page plus courte que `limit` terminant la lecture, une
/// limite que Wise plafonnerait en deçà arrêterait la lecture à la première
/// page sans rien dire : 100 est la valeur éprouvée.
pub const TRANSFERS_PAGE_SIZE: usize = 100;
/// Plafond de garde, en pages, d'une lecture (activités d'une fenêtre ou
/// transferts d'une période).
pub const MAX_PAGES: u32 = 200;
/// Jours de recul de la lecture des transferts : un transfert se crée avant de
/// se terminer.
pub const TRANSFER_LOOKBACK_DAYS: u64 = 31;
/// Borne du corps d'une réponse.
pub const MAX_BODY_BYTES: usize = 8 * 1024 * 1024;
/// Délai d'établissement de la connexion.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Délai total d'une requête.
pub const TOTAL_TIMEOUT: Duration = Duration::from_secs(60);

/// Étape d'un téléchargement (`details.step`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Liste des profils.
    Profiles,
    /// Soldes d'un profil.
    Balances,
    /// Activités d'une fenêtre.
    Activities,
    /// Transferts.
    Transfers,
    /// Devis d'un transfert.
    Quote,
}

impl Step {
    /// Le nom de l'étape tel qu'il se lit dans `details.step`.
    pub fn as_str(self) -> &'static str {
        match self {
            Step::Profiles => "profiles",
            Step::Balances => "balances",
            Step::Activities => "activities",
            Step::Transfers => "transfers",
            Step::Quote => "quote",
        }
    }
}

/// Où une requête se situe, pour nommer l'échec : étape, fenêtre, page.
#[derive(Debug, Clone)]
struct Locus {
    step: Step,
    window: Option<String>,
    page: Option<u32>,
}

impl Locus {
    fn new(step: Step) -> Self {
        Self {
            step,
            window: None,
            page: None,
        }
    }
}

/// Un profil du jeton.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WiseProfile {
    /// Identifiant du profil (nombre de Wise, gardé en texte).
    pub id: String,
    /// Type (`business`, `personal`), en minuscules.
    pub profile_type: String,
}

/// Un solde `STANDARD`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WiseBalance {
    /// Identifiant du solde.
    pub id: String,
    /// Devise.
    pub currency: String,
    /// Le champ de solde `amount.value` (égal à `cashAmount.value` sur les
    /// relevés de 2024).
    pub amount: Decimal,
}

/// Une activité, **telle que rendue** par Wise : les textes ne sont pas
/// analysés ici (c'est 39-12a2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WiseActivity {
    /// Identifiant de l'activité.
    pub id: String,
    /// `TRANSFER`, `CARD_PAYMENT`, `INTERBALANCE`, `AUTO_CONVERSION`…
    pub activity_type: String,
    /// Type de la ressource (`resource.type`).
    pub resource_type: String,
    /// Identifiant de la ressource (`resource.id`).
    pub resource_id: String,
    /// Titre.
    pub title: String,
    /// Description.
    pub description: String,
    /// Montant principal, texte balisé de Wise.
    pub primary_amount: String,
    /// Montant secondaire, s'il y en a un.
    pub secondary_amount: Option<String>,
    /// Statut (`COMPLETED`, `CANCELLED`…).
    pub status: String,
    /// Création (`createdOn`), tel que reçu.
    pub created_on: String,
    /// Dernière mise à jour (`updatedOn`), tel que reçu.
    pub updated_on: String,
}

/// Un transfert de la liste.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WiseTransfer {
    /// Identifiant du transfert.
    pub id: String,
    /// Identifiant du devis, s'il y en a un.
    pub quote_uuid: Option<String>,
    /// Devise source.
    pub source_currency: Option<String>,
    /// Référence du paiement (`Payment Reference` du CSV).
    pub reference: Option<String>,
}

/// Les frais d'un transfert payé par le solde.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WiseQuoteFee {
    /// Devise source du devis.
    pub currency: String,
    /// `fee.total`, zéro quand le virement est sans frais.
    pub total: Decimal,
}

/// Échec de la résolution du profil.
#[derive(Debug)]
pub enum WiseProfileError {
    /// Zéro ou plusieurs profils `business`, ou un `PROFILE_ID` absent de la
    /// liste : `WISE_PROFILE_AMBIGUOUS`, `details: { count }`.
    Ambiguous {
        /// Nombre de profils qui conviennent (0 si `PROFILE_ID` n'en désigne
        /// aucun).
        count: usize,
    },
    /// Toute autre erreur (réseau, droits, réponse).
    App(AppError),
}

impl From<AppError> for WiseProfileError {
    fn from(e: AppError) -> Self {
        WiseProfileError::App(e)
    }
}

/// Le client d'un compte Wise : un jeton, un profil facultatif.
pub struct WiseClient {
    http: reqwest::Client,
    base_url: String,
    token: String,
    profile_id: Option<String>,
    gate: Arc<ProviderGate>,
    wait: Arc<dyn Wait>,
}

impl std::fmt::Debug for WiseClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WiseClient")
            .field("base_url", &self.base_url)
            .field("token", &"***")
            .field("profile_id", &self.profile_id)
            .finish()
    }
}

/// Une valeur JSON lue comme texte : chaîne, ou nombre (identifiants).
fn text_of(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// Un montant lu en [`Decimal`] par le **texte exact reçu** (`RawValue`) :
/// aucun passage par `f64`, qui altère dès seize chiffres significatifs.
///
/// Un montant Wise est toujours un décimal simple : nombre JSON ou chaîne,
/// la **littérale** doit être de forme stricte `-?chiffres(.chiffres)?` (ni
/// `_`, ni `+`, ni espace, **ni exposant**, `e` ou `E`, positif ou négatif),
/// avec au plus 28 décimales (limite de [`Decimal`]). Toute autre forme,
/// `null` compris, donne `None` : rien n'est arrondi ni réinterprété.
fn decimal_of_raw(raw: &RawValue) -> Option<Decimal> {
    let text = raw.get().trim();
    let litterale = text
        .strip_prefix('"')
        .and_then(|t| t.strip_suffix('"'))
        .unwrap_or(text);
    let digits = litterale.strip_prefix('-').unwrap_or(litterale);
    let (entier, frac) = digits
        .split_once('.')
        .map_or((digits, None), |(e, f)| (e, Some(f)));
    let chiffres = |t: &str| !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit());
    if !chiffres(entier) || frac.is_some_and(|f| !chiffres(f) || f.len() > 28) {
        return None;
    }
    litterale.parse::<Decimal>().ok()
}

/// Encode un composant de requête : tout octet hors `A-Za-z0-9-._~` en `%XX`.
fn encode_component(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for b in raw.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Un instant au format de l'API : ISO 8601 UTC **à la milliseconde**.
fn api_instant(t: DateTime<Utc>) -> String {
    encode_component(&t.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
}

/// L'en-tête `Retry-After`, tel que reçu.
fn retry_after(response: &reqwest::Response) -> Option<String> {
    response
        .headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
}

/// `PROVIDER_RESPONSE_INVALID` d'une étape, avec sa raison.
fn invalid(step: Step, reason: &'static str) -> AppError {
    AppError::ProviderResponseInvalid {
        provider: PROVIDER,
        details: serde_json::json!({ "step": step.as_str(), "reason": reason }),
    }
}

/// L'échec après six essais passagers.
fn exhausted(kind: Transient, locus: &Locus) -> AppError {
    let (window, page) = (locus.window.clone(), locus.page);
    match kind {
        Transient::RateLimited => AppError::ProviderRateLimited {
            provider: PROVIDER,
            window,
            page,
        },
        Transient::Unavailable => AppError::ProviderUnavailable {
            provider: PROVIDER,
            window,
            page,
        },
    }
}

impl WiseClient {
    /// Le client de la série `series`. `timeouts` remplace les délais (tests).
    pub fn new(
        config: &ProviderApiConfig,
        series: &ProviderSeries,
        gate: Arc<ProviderGate>,
        wait: Arc<dyn Wait>,
        timeouts: Option<(Duration, Duration)>,
    ) -> Result<Self, AppError> {
        let (connect, total) = timeouts.unwrap_or((CONNECT_TIMEOUT, TOTAL_TIMEOUT));
        let https_only = config.base_url.starts_with("https://");
        let http = build_client(https_only, connect, total).map_err(|_| {
            AppError::Internal("client HTTP de l'API du prestataire impossible à construire".into())
        })?;
        let token = series.field("TOKEN").map(str::to_string).ok_or_else(|| {
            AppError::ProviderApiNotConfigured {
                provider: PROVIDER,
                account: series.account.clone(),
            }
        })?;
        Ok(Self {
            http,
            base_url: config.base_url.clone(),
            token,
            profile_id: series.field("PROFILE_ID").map(str::to_string),
            gate,
            wait,
        })
    }

    /// Un essai d'une requête `GET` : statut classé **avant** toute lecture du
    /// corps (un refus 401 ou 403 ne se rejoue jamais, quel que soit son
    /// corps), puis, sur 2xx seulement, corps lu borné et vérifié comme JSON.
    async fn once(&self, path_and_query: &str, locus: &Locus) -> Attempt<Vec<u8>, AppError> {
        let transient = |kind, retry_after| Attempt::Transient { kind, retry_after };
        let sent = self
            .http
            .get(format!("{}{path_and_query}", self.base_url))
            .bearer_auth(&self.token)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await;
        // Le texte d'une erreur de `reqwest` porte l'URL : il ne sort jamais.
        let response = match sent {
            Ok(r) => r,
            Err(_) => return transient(Transient::Unavailable, None),
        };
        let status = response.status().as_u16();
        let retry = retry_after(&response);
        match status {
            200..=299 => {
                let body = match read_body_bounded(response, MAX_BODY_BYTES).await {
                    Ok(b) => b,
                    Err(BodyError::TooLarge) => {
                        return Attempt::Fatal(invalid(locus.step, "too_large"));
                    }
                    Err(BodyError::Transport(_)) => return transient(Transient::Unavailable, None),
                };
                match serde_json::from_slice::<serde::de::IgnoredAny>(&body) {
                    Ok(_) => Attempt::Done(body),
                    Err(_) => Attempt::Fatal(invalid(locus.step, "json")),
                }
            }
            401 => Attempt::Fatal(AppError::ProviderAuthFailed { provider: PROVIDER }),
            403 => {
                tracing::warn!(
                    provider = PROVIDER,
                    etape = locus.step.as_str(),
                    "API : droits insuffisants"
                );
                Attempt::Fatal(AppError::ProviderPermissionDenied { provider: PROVIDER })
            }
            429 => transient(Transient::RateLimited, retry),
            500..=599 => transient(Transient::Unavailable, retry),
            code => {
                tracing::warn!(
                    provider = PROVIDER,
                    etape = locus.step.as_str(),
                    statut = code,
                    "API : refus"
                );
                Attempt::Fatal(AppError::ProviderApiError {
                    provider: PROVIDER,
                    window: locus.window.clone(),
                    page: locus.page,
                })
            }
        }
    }

    /// Une requête `GET`, reprise comprise (six essais sur 429, 5xx et délai).
    /// Le corps reçu, JSON vérifié, non interprété (les montants s'y lisent
    /// par leur texte).
    async fn get_bytes(&self, path_and_query: &str, locus: &Locus) -> Result<Vec<u8>, AppError> {
        match with_retry(&self.gate, self.wait.as_ref(), || {
            self.once(path_and_query, locus)
        })
        .await
        {
            Ok(v) => Ok(v),
            Err(RetryError::Fatal(e)) => Err(e),
            Err(RetryError::Exhausted(kind)) => Err(exhausted(kind, locus)),
        }
    }

    /// Une requête `GET` dont la réponse n'a pas de montant : lue en [`Value`].
    async fn get(&self, path_and_query: &str, locus: &Locus) -> Result<Value, AppError> {
        let body = self.get_bytes(path_and_query, locus).await?;
        serde_json::from_slice(&body).map_err(|_| invalid(locus.step, "json"))
    }

    /// `GET /v1/profiles`.
    pub async fn profiles(&self) -> Result<Vec<WiseProfile>, AppError> {
        let v = self
            .get("/v1/profiles", &Locus::new(Step::Profiles))
            .await?;
        let array = v
            .as_array()
            .ok_or_else(|| invalid(Step::Profiles, "shape"))?;
        array
            .iter()
            .map(|p| {
                Some(WiseProfile {
                    id: text_of(p.get("id")?)?,
                    profile_type: p.get("type")?.as_str()?.to_ascii_lowercase(),
                })
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| invalid(Step::Profiles, "shape"))
    }

    /// Le profil du compte : celui de `KESH_WISE_<n>_PROFILE_ID`, qui doit
    /// figurer dans la liste, sinon le **seul** profil `business` (casse
    /// ignorée) ; zéro ou plusieurs → [`WiseProfileError::Ambiguous`].
    pub async fn resolve_profile(&self) -> Result<WiseProfile, WiseProfileError> {
        let all = self.profiles().await?;
        Self::pick_profile(all, self.profile_id.as_deref())
    }

    /// Le choix du profil dans une liste (fonction pure).
    pub fn pick_profile(
        all: Vec<WiseProfile>,
        wanted: Option<&str>,
    ) -> Result<WiseProfile, WiseProfileError> {
        let mut chosen: Vec<WiseProfile> = match wanted {
            Some(id) => all.into_iter().filter(|p| p.id == id.trim()).collect(),
            None => all
                .into_iter()
                .filter(|p| p.profile_type == "business")
                .collect(),
        };
        match chosen.len() {
            1 => Ok(chosen.remove(0)),
            count => Err(WiseProfileError::Ambiguous { count }),
        }
    }

    /// `GET /v4/profiles/{p}/balances?types=STANDARD`.
    pub async fn balances(&self, profile: &WiseProfile) -> Result<Vec<WiseBalance>, AppError> {
        let path = format!(
            "/v4/profiles/{}/balances?types=STANDARD",
            encode_component(&profile.id)
        );
        let body = self.get_bytes(&path, &Locus::new(Step::Balances)).await?;
        parse_balances(&body).ok_or_else(|| invalid(Step::Balances, "shape"))
    }

    /// Les activités d'une fenêtre, **toutes ses pages** : `since` = premier
    /// instant du jour de Zurich `window.from`, `until` = dernier instant du
    /// jour `window.to` borné à `now`, **à la milliseconde** (`Window.start` et
    /// `Window.end` sont tronqués à la seconde et ne servent pas ici : une
    /// activité de la dernière seconde n'appartiendrait à aucune fenêtre).
    /// Une activité (même `id` d'activité) vue dans deux pages n'est gardée qu'une
    /// fois, dans sa dernière version lue ([`dedup_activities`]).
    pub async fn activities_window(
        &self,
        profile: &WiseProfile,
        window: &Window,
        now: DateTime<Utc>,
    ) -> Result<Vec<WiseActivity>, AppError> {
        let since = api_instant(debut_du_jour_suisse(window.from));
        let until = api_instant(fin_du_jour_suisse(window.to).min(now));
        let base = format!(
            "/v1/profiles/{}/activities?since={since}&until={until}&size={ACTIVITIES_PAGE_SIZE}",
            encode_component(&profile.id)
        );
        let mut out: Vec<WiseActivity> = Vec::new();
        let mut cursor: Option<String> = None;
        for page in 1..=MAX_PAGES {
            let locus = Locus {
                step: Step::Activities,
                window: Some(window.label()),
                page: Some(page),
            };
            let path = match &cursor {
                None => base.clone(),
                Some(c) => format!("{base}&nextCursor={}", encode_component(c)),
            };
            let v = self.get(&path, &locus).await?;
            let list = v
                .get("activities")
                .and_then(Value::as_array)
                .ok_or_else(|| invalid(Step::Activities, "shape"))?;
            for item in list {
                out.push(parse_activity(item).ok_or_else(|| invalid(Step::Activities, "shape"))?);
            }
            match v.get("cursor") {
                None | Some(Value::Null) => return Ok(dedup_activities(out)),
                Some(c) => match text_of(c).filter(|c| !c.is_empty()) {
                    Some(c) => cursor = Some(c),
                    None => return Ok(dedup_activities(out)),
                },
            }
        }
        Err(AppError::ProviderResponseInvalid {
            provider: PROVIDER,
            details: serde_json::json!({
                "step": Step::Activities.as_str(),
                "reason": "too_many_pages",
            }),
        })
    }

    /// Les activités de la période `from..=to` (jours de Zurich) : fenêtres du
    /// socle de [`MAX_DAYS`] jours, lues l'une après l'autre ; une activité vue
    /// dans deux fenêtres n'est gardée qu'une fois ([`dedup_activities`]). 39-12a3 emploie
    /// [`Self::activities_window`] pour lire plusieurs fenêtres de front.
    pub async fn activities(
        &self,
        profile: &WiseProfile,
        from: NaiveDate,
        to: NaiveDate,
        now: DateTime<Utc>,
    ) -> Result<Vec<WiseActivity>, AppError> {
        let mut all = Vec::new();
        for w in windows(from, to, MAX_DAYS, now) {
            all.extend(self.activities_window(profile, &w, now).await?);
        }
        Ok(dedup_activities(all))
    }

    /// Les transferts de la période `from..=to`, **élargie de 31 jours avant
    /// son début** (un transfert se crée avant de se terminer) : pages par
    /// `offset` jusqu'à une page de moins de [`TRANSFERS_PAGE_SIZE`] éléments.
    pub async fn transfers(
        &self,
        profile: &WiseProfile,
        from: NaiveDate,
        to: NaiveDate,
        now: DateTime<Utc>,
    ) -> Result<Vec<WiseTransfer>, AppError> {
        let start = from
            .checked_sub_days(Days::new(TRANSFER_LOOKBACK_DAYS))
            .unwrap_or(from);
        let base = format!(
            "/v1/transfers?profile={}&createdDateStart={}&createdDateEnd={}&limit={TRANSFERS_PAGE_SIZE}",
            encode_component(&profile.id),
            api_instant(debut_du_jour_suisse(start)),
            api_instant(fin_du_jour_suisse(to).min(now)),
        );
        let mut out: Vec<WiseTransfer> = Vec::new();
        for page in 1..=MAX_PAGES {
            let offset = (page as usize - 1) * TRANSFERS_PAGE_SIZE;
            let locus = Locus {
                step: Step::Transfers,
                window: None,
                page: Some(page),
            };
            let v = self.get(&format!("{base}&offset={offset}"), &locus).await?;
            let list = v
                .as_array()
                .ok_or_else(|| invalid(Step::Transfers, "shape"))?;
            for item in list {
                out.push(parse_transfer(item).ok_or_else(|| invalid(Step::Transfers, "shape"))?);
            }
            if list.len() < TRANSFERS_PAGE_SIZE {
                return Ok(out);
            }
        }
        Err(AppError::ProviderResponseInvalid {
            provider: PROVIDER,
            details: serde_json::json!({
                "step": Step::Transfers.as_str(),
                "reason": "too_many_pages",
            }),
        })
    }

    /// `GET /v1/transfers/{id}` : un transfert sortant d'une activité qui
    /// manque à la liste se lit seul.
    pub async fn transfer(&self, id: &str) -> Result<WiseTransfer, AppError> {
        let path = format!("/v1/transfers/{}", encode_component(id));
        let v = self.get(&path, &Locus::new(Step::Transfers)).await?;
        parse_transfer(&v).ok_or_else(|| invalid(Step::Transfers, "shape"))
    }

    /// `GET /v3/profiles/{p}/quotes/{quoteUuid}` : les frais du transfert payé
    /// par le solde. L'option retenue est celle dont `payIn` vaut `BALANCE`
    /// **et** dont `payOut` est celui du devis ; zéro ou plusieurs options
    /// retenues → `PROVIDER_RESPONSE_INVALID` (`reason = "quote_fee"`).
    pub async fn quote_fee(
        &self,
        profile: &WiseProfile,
        quote_uuid: &str,
    ) -> Result<WiseQuoteFee, AppError> {
        let path = format!(
            "/v3/profiles/{}/quotes/{}",
            encode_component(&profile.id),
            encode_component(quote_uuid)
        );
        let body = self.get_bytes(&path, &Locus::new(Step::Quote)).await?;
        pick_quote_fee(&body).ok_or_else(|| invalid(Step::Quote, "quote_fee"))
    }
}

/// Une même activité lue deux fois (deux pages, ou deux fenêtres dont l'une la
/// voit mise à jour : le filtre porte sur `updatedOn`) n'est gardée qu'une
/// fois, **dans sa dernière version lue**, à la place de sa première lecture.
/// La clé est l'`id` **d'activité** : deux activités distinctes d'une même
/// ressource (un virement et son remboursement) restent deux, 39-12a2 les
/// distingue.
pub fn dedup_activities(activities: Vec<WiseActivity>) -> Vec<WiseActivity> {
    let mut place: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut out: Vec<WiseActivity> = Vec::new();
    for a in activities {
        match place.get(&a.id) {
            Some(&i) => out[i] = a,
            None => {
                place.insert(a.id.clone(), out.len());
                out.push(a);
            }
        }
    }
    out
}

/// Une activité de la liste ; `None` si un champ attendu manque.
fn parse_activity(v: &Value) -> Option<WiseActivity> {
    let resource = v.get("resource")?;
    let opt = |key: &str| {
        v.get(key)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    };
    Some(WiseActivity {
        id: text_of(v.get("id")?)?,
        activity_type: v.get("type")?.as_str()?.to_string(),
        resource_type: resource.get("type")?.as_str()?.to_string(),
        resource_id: text_of(resource.get("id")?)?,
        title: opt("title").unwrap_or_default(),
        description: opt("description").unwrap_or_default(),
        primary_amount: opt("primaryAmount").unwrap_or_default(),
        secondary_amount: opt("secondaryAmount"),
        status: v.get("status")?.as_str()?.to_string(),
        created_on: v.get("createdOn")?.as_str()?.to_string(),
        updated_on: v.get("updatedOn")?.as_str()?.to_string(),
    })
}

/// Un transfert ; `None` si son identifiant manque.
fn parse_transfer(v: &Value) -> Option<WiseTransfer> {
    let opt = |key: &str| v.get(key).and_then(text_of).filter(|s| !s.is_empty());
    Some(WiseTransfer {
        id: text_of(v.get("id")?)?,
        quote_uuid: opt("quoteUuid"),
        source_currency: opt("sourceCurrency"),
        reference: opt("reference"),
    })
}

/// Les soldes d'une réponse `balances` (fonction pure) : chaque `amount.value`
/// est lu par son texte exact ; `None` si un champ attendu manque.
fn parse_balances(body: &[u8]) -> Option<Vec<WiseBalance>> {
    #[derive(Deserialize)]
    struct Amount {
        value: Box<RawValue>,
    }
    #[derive(Deserialize)]
    struct Balance {
        id: Value,
        currency: String,
        amount: Amount,
    }
    let all: Vec<Balance> = serde_json::from_slice(body).ok()?;
    all.into_iter()
        .map(|b| {
            Some(WiseBalance {
                id: text_of(&b.id)?,
                currency: b.currency,
                amount: decimal_of_raw(&b.amount.value)?,
            })
        })
        .collect()
}

/// L'option de devis retenue (fonction pure) : le devis porte un `payOut`,
/// `payIn` vaut `BALANCE` et `payOut` est celui du devis ; exactement une
/// option, avec `fee.total` lu par son texte exact.
fn pick_quote_fee(body: &[u8]) -> Option<WiseQuoteFee> {
    #[derive(Deserialize)]
    struct Fee {
        total: Option<Box<RawValue>>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct QuoteOption {
        pay_in: Option<String>,
        pay_out: Option<String>,
        source_currency: Option<String>,
        fee: Option<Fee>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Quote {
        pay_out: Option<String>,
        source_currency: Option<String>,
        payment_options: Vec<QuoteOption>,
    }
    let quote: Quote = serde_json::from_slice(body).ok()?;
    let pay_out = quote.pay_out?;
    let mut chosen = quote.payment_options.into_iter().filter(|o| {
        o.pay_in.as_deref() == Some("BALANCE") && o.pay_out.as_deref() == Some(pay_out.as_str())
    });
    let option = chosen.next()?;
    if chosen.next().is_some() {
        return None;
    }
    // La devise des frais est celle du devis (devise source) ; l'option ne
    // sert qu'en repli.
    // Si les deux sont présentes et diffèrent, on ne choisit pas en silence.
    if let (Some(q), Some(o)) = (&quote.source_currency, &option.source_currency)
        && q != o
    {
        return None;
    }
    let currency = quote.source_currency.or(option.source_currency)?;
    Some(WiseQuoteFee {
        currency,
        total: decimal_of_raw(option.fee?.total?.as_ref())?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider_env::{ProviderApiConfig, ProviderSeries, ProvidersConfig, read};
    use serde_json::json;
    use std::collections::HashMap;

    fn lire(pairs: &[(&str, &str)]) -> Result<ProviderApiConfig, crate::config::ConfigError> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let names: Vec<String> = map.keys().cloned().collect();
        read(
            WISE,
            |k| {
                map.get(k)
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty())
            },
            &names,
        )
    }

    fn var_refusee(pairs: &[(&str, &str)]) -> String {
        match lire(pairs) {
            Err(e) => e.to_string(),
            Ok(_) => panic!("la configuration aurait dû être refusée"),
        }
    }

    #[test]
    fn wise_est_inscrit_a_la_liste_des_prestataires() {
        assert!(
            crate::provider_env::ALL_PROVIDERS
                .iter()
                .any(|p| p.key == "WISE" && p.provider == "wise")
        );
        assert_eq!(WISE.fields, &["TOKEN"]);
        assert_eq!(WISE.optional_fields, &["PROFILE_ID"]);
        assert_eq!(WISE.default_base_url, "https://api.wise.com");
        // Aucune clé de signature : aucune variable SCA n'est lue.
        assert!(WISE.all_fields().all(|f| !f.contains("SCA")));
    }

    #[test]
    fn serie_complete_avec_et_sans_profil() {
        let c = lire(&[
            ("KESH_WISE_1_ACCOUNT", "Wise CHF"),
            ("KESH_WISE_1_TOKEN", "jeton-1"),
        ])
        .unwrap();
        let s = c.series_for("wise chf").unwrap();
        assert_eq!(s.field("TOKEN"), Some("jeton-1"));
        assert_eq!(s.field("PROFILE_ID"), None);
        assert_eq!(c.base_url, "https://api.wise.com");
        assert_eq!(c.parallel, 1);

        let c = lire(&[
            ("KESH_WISE_2_ACCOUNT", "Wise EUR"),
            ("KESH_WISE_2_TOKEN", "jeton-2"),
            ("KESH_WISE_2_PROFILE_ID", "4242"),
        ])
        .unwrap();
        assert_eq!(
            c.series_for("Wise EUR").unwrap().field("PROFILE_ID"),
            Some("4242")
        );
    }

    #[test]
    fn rien_de_wise_le_demarrage_passe() {
        let c = lire(&[]).unwrap();
        assert_eq!(c.series_count(), 0);
        // Une valeur vide vaut absente.
        let c = lire(&[
            ("KESH_WISE_1_ACCOUNT", ""),
            ("KESH_WISE_1_TOKEN", "  "),
            ("KESH_WISE_1_PROFILE_ID", ""),
        ])
        .unwrap();
        assert_eq!(c.series_count(), 0);
    }

    #[test]
    fn serie_incomplete_refusee_nommant_la_variable_sans_sa_valeur() {
        let msg = var_refusee(&[("KESH_WISE_1_ACCOUNT", "Wise CHF")]);
        assert!(msg.contains("KESH_WISE_1_TOKEN"), "{msg}");
        let msg = var_refusee(&[("KESH_WISE_1_TOKEN", "TOKEN-MARQUEUR-1")]);
        assert!(msg.contains("KESH_WISE_1_ACCOUNT"), "{msg}");
        assert!(!msg.contains("TOKEN-MARQUEUR-1"), "{msg}");
        // `PROFILE_ID` seul ne fait pas une série.
        let msg = var_refusee(&[("KESH_WISE_1_PROFILE_ID", "4242")]);
        assert!(msg.contains("KESH_WISE_1_ACCOUNT"), "{msg}");
    }

    #[test]
    fn gabarit_et_libelle_en_double_refuses() {
        let msg = var_refusee(&[
            ("KESH_WISE_1_ACCOUNT", "Wise CHF"),
            ("KESH_WISE_1_TOKEN", "<EDIT: jeton>"),
        ]);
        assert!(msg.contains("KESH_WISE_1_TOKEN"), "{msg}");
        assert!(!msg.contains("EDIT"), "{msg}");
        let msg = var_refusee(&[
            ("KESH_WISE_1_ACCOUNT", "Wise CHF"),
            ("KESH_WISE_1_TOKEN", "a"),
            ("KESH_WISE_2_ACCOUNT", "WISE chf"),
            ("KESH_WISE_2_TOKEN", "b"),
        ]);
        assert!(msg.contains("KESH_WISE_2_ACCOUNT"), "{msg}");
    }

    #[test]
    fn hote_et_parallelisme() {
        let c = lire(&[
            ("KESH_WISE_BASE_URL", "http://127.0.0.1:9999/"),
            ("KESH_WISE_PARALLEL", "4"),
        ])
        .unwrap();
        assert_eq!(c.base_url, "http://127.0.0.1:9999");
        assert_eq!(c.parallel, 4);
        assert!(lire(&[("KESH_WISE_BASE_URL", "http://example.com")]).is_err());
        assert!(lire(&[("KESH_WISE_PARALLEL", "7")]).is_err());
        assert!(lire(&[("KESH_WISE_1_SCA_KEY_FILE", "/x")]).is_err());
    }

    #[test]
    fn jeton_masque_dans_debug() {
        let series = ProviderSeries::for_tests(
            1,
            "Wise CHF",
            &[("TOKEN", "TOKEN-MARQUEUR-DEBUG"), ("PROFILE_ID", "1")],
        );
        let cfg = ProvidersConfig::default().with(ProviderApiConfig::for_tests(
            WISE,
            "http://127.0.0.1:1",
            1,
            vec![series.clone()],
        ));
        let rendu = format!("{series:?} {cfg:?}");
        assert!(!rendu.contains("TOKEN-MARQUEUR-DEBUG"), "{rendu}");
        let client = WiseClient::new(
            &cfg.get(WISE),
            &series,
            Arc::new(ProviderGate::new(1)),
            Arc::new(crate::provider_download::TokioWait),
            None,
        )
        .unwrap();
        assert!(!format!("{client:?}").contains("TOKEN-MARQUEUR-DEBUG"));
    }

    #[test]
    fn instants_a_la_milliseconde_encodes() {
        use chrono::TimeZone;
        let t = Utc.with_ymd_and_hms(2024, 1, 30, 22, 59, 59).unwrap()
            + chrono::TimeDelta::milliseconds(999);
        assert_eq!(api_instant(t), "2024-01-30T22%3A59%3A59.999Z");
    }

    /// Le texte brut d'un nombre, tel qu'un serveur l'écrit (jamais passé par
    /// `json!`, qui le ferait transiter par `f64`).
    fn brut(texte: &str) -> Box<RawValue> {
        serde_json::from_str(texte).unwrap()
    }

    #[test]
    fn nombres_lus_par_le_texte_recu() {
        let lire = |t: &str| decimal_of_raw(&brut(t)).map(|d| d.to_string());
        assert_eq!(lire("1234.50").as_deref(), Some("1234.50"));
        assert_eq!(lire("0").as_deref(), Some("0"));
        assert_eq!(lire("0.0").as_deref(), Some("0.0"));
        assert_eq!(lire("\"12.30\"").as_deref(), Some("12.30"));
        // Aucune forme à exposant, nombre JSON ou chaîne (revue P3, M1, L1, L3).
        for refus in [
            "1e-7",
            "1.5e-28",
            "1e-29",
            "1.5e-3",
            "1E3",
            "1.0E+2",
            "1e3",
            "\"1e-7\"",
            "\"1.5E+2\"",
        ] {
            assert_eq!(lire(refus), None, "{refus}");
        }
        assert_eq!(lire("-0").as_deref(), Some("0"));
        // Les cas que `f64` altère (revue P1, M1).
        assert_eq!(
            lire("99999999999999.99").as_deref(),
            Some("99999999999999.99")
        );
        assert_eq!(
            lire("123456789012345.67").as_deref(),
            Some("123456789012345.67")
        );
        assert_eq!(
            lire("0.123456789012345678").as_deref(),
            Some("0.123456789012345678")
        );
        assert_eq!(lire("null"), None);
        assert_eq!(lire("\"abc\""), None);
        // Chaînes hors de la forme `-?chiffres(.chiffres)?` (revue P2, L4).
        assert_eq!(lire("\"-3.50\"").as_deref(), Some("-3.50"));
        for refus in [
            "\"1_000\"",
            "\"+5\"",
            "\" 5\"",
            "\"5 \"",
            "\"1e3\"",
            "\"\"",
            "\"-\"",
            "\".5\"",
            "\"5.\"",
            "\"1.2.3\"",
            "\"--1\"",
        ] {
            assert_eq!(lire(refus), None, "{refus}");
        }
        // Au-delà de 28 décimales : refusé, jamais arrondi en silence.
        let trente = format!("0.{}", "1".repeat(30));
        assert_eq!(lire(&trente), None);
        assert_eq!(lire(&format!("\"{trente}\"")), None);
        let vingt_huit = format!("0.{}", "1".repeat(28));
        assert_eq!(lire(&vingt_huit).as_deref(), Some(vingt_huit.as_str()));
    }

    #[test]
    fn soldes_au_texte_brut_sans_perte() {
        let corps =
            br#"[{"id":1,"currency":"CHF","amount":{"value":99999999999999.99,"currency":"CHF"}},
            {"id":2,"currency":"EUR","amount":{"value":123456789012345.67}},
            {"id":3,"currency":"USD","amount":{"value":0.123456789012345678}}]"#;
        let s = parse_balances(corps).unwrap();
        assert_eq!(s[0].amount.to_string(), "99999999999999.99");
        assert_eq!(s[1].amount.to_string(), "123456789012345.67");
        assert_eq!(s[2].amount.to_string(), "0.123456789012345678");
        assert!(parse_balances(br#"[{"id":1,"currency":"CHF","amount":{}}]"#).is_none());
        assert!(parse_balances(br#"[{"id":1,"currency":"CHF"}]"#).is_none());
    }

    #[test]
    fn composant_de_requete_encode() {
        assert_eq!(encode_component("a+b/c=d"), "a%2Bb%2Fc%3Dd");
        assert_eq!(encode_component("Az09-._~"), "Az09-._~");
    }

    fn profil(id: &str, t: &str) -> WiseProfile {
        WiseProfile {
            id: id.into(),
            profile_type: t.into(),
        }
    }

    #[test]
    fn choix_du_profil() {
        let liste = vec![profil("1", "personal"), profil("2", "business")];
        assert_eq!(
            WiseClient::pick_profile(liste.clone(), None).unwrap().id,
            "2"
        );
        assert_eq!(
            WiseClient::pick_profile(liste.clone(), Some("1"))
                .unwrap()
                .id,
            "1"
        );
        assert!(matches!(
            WiseClient::pick_profile(liste.clone(), Some("9")),
            Err(WiseProfileError::Ambiguous { count: 0 })
        ));
        assert!(matches!(
            WiseClient::pick_profile(vec![profil("1", "personal")], None),
            Err(WiseProfileError::Ambiguous { count: 0 })
        ));
        assert!(matches!(
            WiseClient::pick_profile(vec![profil("1", "business"), profil("2", "business")], None),
            Err(WiseProfileError::Ambiguous { count: 2 })
        ));
    }

    fn activite_id(id: &str, rt: &str, rid: &str, statut: &str) -> WiseActivity {
        parse_activity(&json!({
            "id": id, "type": "TRANSFER",
            "resource": {"type": rt, "id": rid},
            "status": statut, "createdOn": "2024-01-02T10:00:00Z",
            "updatedOn": "2024-01-02T10:00:01.123Z"
        }))
        .unwrap()
    }

    #[test]
    fn deduplication_par_identifiant_d_activite_derniere_version() {
        let v = dedup_activities(vec![
            activite_id("A1", "TRANSFER", "1", "PENDING"),
            activite_id("A2", "TRANSFER", "2", "COMPLETED"),
            activite_id("A1", "TRANSFER", "1", "COMPLETED"),
        ]);
        assert_eq!(v.len(), 2);
        // La place de la première lecture, le contenu de la dernière.
        assert_eq!(v[0].id, "A1");
        assert_eq!(v[0].status, "COMPLETED");
        assert_eq!(v[1].id, "A2");
    }

    #[test]
    fn deux_activites_de_meme_ressource_restent_deux() {
        // Un virement remboursé : seconde activité de même `resource_id`.
        let v = dedup_activities(vec![
            activite_id("A1", "TRANSFER", "7", "COMPLETED"),
            activite_id("A2", "TRANSFER", "7", "COMPLETED"),
        ]);
        assert_eq!(v.len(), 2);
    }

    fn octets(v: &Value) -> Vec<u8> {
        v.to_string().into_bytes()
    }

    #[test]
    fn devis_sans_pay_out_refuse() {
        let q = json!({"paymentOptions":[
            {"payIn":"BALANCE","sourceCurrency":"CHF","fee":{"total":1.0}}]});
        assert!(pick_quote_fee(&octets(&q)).is_none());
    }

    #[test]
    fn frais_du_devis_au_texte_brut_sans_perte() {
        let corps = br#"{"payOut":"BANK_TRANSFER","paymentOptions":[
            {"payIn":"BALANCE","payOut":"BANK_TRANSFER","sourceCurrency":"CHF","fee":{"total":99999999999999.99}}]}"#;
        assert_eq!(
            pick_quote_fee(corps).unwrap().total.to_string(),
            "99999999999999.99"
        );
    }

    #[test]
    fn devis_une_option_balance() {
        let q = json!({"payOut":"BANK_TRANSFER","sourceCurrency":"CHF","paymentOptions":[
            {"payIn":"BALANCE","payOut":"BANK_TRANSFER","sourceCurrency":"CHF","fee":{"total":1.25}},
            {"payIn":"BANK_TRANSFER","payOut":"BANK_TRANSFER","sourceCurrency":"EUR","fee":{"total":9.0}},
            {"payIn":"BALANCE","payOut":"SWIFT","sourceCurrency":"EUR","fee":{"total":7.0}}]});
        let f = pick_quote_fee(&octets(&q)).unwrap();
        assert_eq!(f.currency, "CHF");
        assert_eq!(f.total, "1.25".parse::<Decimal>().unwrap());
        // Devises du devis et de l'option présentes et différentes : refus,
        // aucun choix silencieux (revue P3, L2).
        let diff = json!({"payOut":"BANK_TRANSFER","sourceCurrency":"CHF","paymentOptions":[
            {"payIn":"BALANCE","payOut":"BANK_TRANSFER","sourceCurrency":"EUR","fee":{"total":1.25}}]});
        assert!(pick_quote_fee(&octets(&diff)).is_none());
        // Repli : le devis sans devise source, l'option fait foi.
        let repli = json!({"payOut":"BANK_TRANSFER","paymentOptions":[
            {"payIn":"BALANCE","payOut":"BANK_TRANSFER","sourceCurrency":"EUR","fee":{"total":1.25}}]});
        assert_eq!(pick_quote_fee(&octets(&repli)).unwrap().currency, "EUR");
        // Frais nuls : l'option est présente, le total vaut zéro.
        let q0 = json!({"payOut":"BANK_TRANSFER","paymentOptions":[
            {"payIn":"BALANCE","payOut":"BANK_TRANSFER","sourceCurrency":"CHF","fee":{"total":0.0}}]});
        assert_eq!(pick_quote_fee(&octets(&q0)).unwrap().total, Decimal::ZERO);
    }

    #[test]
    fn devis_sans_option_ou_a_deux_options_refuse() {
        let sans = json!({"payOut":"BANK_TRANSFER","paymentOptions":[
            {"payIn":"BANK_TRANSFER","payOut":"BANK_TRANSFER","sourceCurrency":"CHF","fee":{"total":1.0}}]});
        assert!(pick_quote_fee(&octets(&sans)).is_none());
        let deux = json!({"payOut":"BANK_TRANSFER","paymentOptions":[
            {"payIn":"BALANCE","payOut":"BANK_TRANSFER","sourceCurrency":"CHF","fee":{"total":1.0}},
            {"payIn":"BALANCE","payOut":"BANK_TRANSFER","sourceCurrency":"CHF","fee":{"total":2.0}}]});
        assert!(pick_quote_fee(&octets(&deux)).is_none());
        assert!(pick_quote_fee(&octets(&json!({"payOut":"X"}))).is_none());
    }

    #[test]
    fn activite_sans_champ_attendu_refusee() {
        assert!(parse_activity(&json!({"id":"a","type":"TRANSFER"})).is_none());
        let a = parse_activity(&json!({
            "id": "a", "type": "CARD_PAYMENT", "resource": {"type":"CARD","id": 77},
            "primaryAmount": "<positive>+ 1 CHF</positive>", "secondaryAmount": "",
            "status": "COMPLETED", "createdOn": "x", "updatedOn": "y"
        }))
        .unwrap();
        assert_eq!(a.resource_id, "77");
        assert_eq!(a.secondary_amount, None);
    }
}
