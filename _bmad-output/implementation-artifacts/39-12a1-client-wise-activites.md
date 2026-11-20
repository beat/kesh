# Story 39-12a1 : client de l'API Wise — configuration, profil, soldes, activités, transferts, devis

Status: review

*Fiche réécrite le 2026-11-20 sur la **voie activités** (recherche du 2026-11-20, `data-not-committed/wise-recherche.md`,
métadonnées seulement) ; elle remplace la fiche « client de l'API Wise — configuration, signature SCA, profil, soldes et
relevés » (`39-12a1-client-wise-et-sca`, renommée). Branche `epic/39-wise`, numéros de ligne relevés au code de
`origin/main` (`fd04099f`). Epic 39, **phase 2** (H1 à H6 du fichier d'epic ; **socle commun S1 à S9**). **Dépend de** :
la **recette préalable (AC-0 a)**, du **socle S1, S2 et S5**, écrit par 38-12a (ou, à défaut, par la première des fiches
de phase 2 développée, générique : C-40-K6). **Suivie par** : 39-12a2 (activités → lignes), 39-12a3 (route et import).*

## Story

En tant qu'**Administrateur**, je veux poser dans l'environnement du serveur le jeton Wise **en lecture seule** d'un
compte Wise, afin que Kesh puisse lire les mouvements de chaque solde — sans authentification forte, sans clé de
signature, et sans que le jeton soit jamais en base ni dans une sauvegarde.

## Ce que la recherche du 2026-11-20 a établi (métadonnées)

| Point | État |
|---|---|
| relevé de solde (`balance-statements`) au jeton personnel, compte tenu par Wise Payments Limited (Royaume-Uni) | **fermé par règle** : 403 `REJECTED` avec ou sans signature valide (Wise n'évalue pas la signature) ; doc des jetons personnels : relevés hors US, CA, AU, NZ, SG, MY non pris en charge. **Voie abandonnée** (H5) |
| `GET /v1/profiles` ; `GET /v4/profiles/{p}/balances?types=STANDARD` | **constatés** (200) au jeton personnel |
| `GET /v1/profiles/{p}/activities?since&until&size=100[&nextCursor]` | **constaté** (200) au jeton seul, sans défi : 2024 entière en 2 pages (182 activités), 2025 en 3 pages (222) ; pagination par `cursor` → `nextCursor`, `cursor` nul en fin |
| `GET /v1/transfers?profile=…&createdDateStart&createdDateEnd&limit&offset` | **constaté** (200) : 132 transferts 2024, tous avec `quoteUuid` |
| `GET /v3/profiles/{p}/quotes/{quoteUuid}` | **constaté** (200), devis de 2024 lisibles en 2026 ; `paymentOptions[payIn=BALANCE].fee.total` = frais du CSV, 22 sur 22 |
| borne de période de `activities` | **aucune** dans la doc (`docs.wise.com/api-reference/activity/activitylist`, relue le 2026-11-20 : « no maximum period », `size` 1 à 100, défaut 10) |
| limite de débit, forme d'un 429 | **aucune vue** (ni `x-ratelimit-*` ni `Retry-After`) ; forme d'un 429 **non confirmée** |
| comparaison à un CSV | **faite pour 2024** (176 identifiants sur 177, soldes au centime hors frais) ; les points ouverts ont été tranchés sur **2024**, dont le CSV du site est en local : AC-0 (a), faite le 2026-11-20 ; **2025 non comparée** (pas de CSV 2025 local), comparaison plus tard, non bloquante (AC-0 c) |

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-12a1-0` | Une recette préalable compare l'API au CSV 2024 du site, en local, et tranche les points ouverts ; une recette finale rejoue la conversion de Kesh sur 2024, ligne à ligne ; un mois 2025 se compare plus tard. | AC-0 |
| `FR-39-12a1-1` | Jeton et identifiant de profil facultatif se posent par variables d'environnement, une série par compte Wise, rattachée à son libellé ; rien en base ni en sauvegarde. | AC-1 |
| `FR-39-12a1-2` | Le client lit le profil d'entreprise, ses soldes `STANDARD`, les activités d'une période (toutes les pages), les transferts de la période et le devis d'un transfert. | AC-2, AC-3 |
| `FR-39-12a1-3` | Chaque erreur de Wise devient un code `PROVIDER_*` ou `WISE_PROFILE_AMBIGUOUS`, lisible, sans texte de Wise ni jeton. | AC-4 |
| `NFR-39-12a1-1` | Aucun appel réseau en CI ni par un agent : serveur Wise simulé dans le processus (socle S5). | AC-5 |
| `NFR-39-12a1-2` | Jeton absent de tout journal, erreur, réponse, audit, export et sauvegarde (test au marqueur, socle S9). | AC-1, AC-5 |
| `NFR-39-12a1-3` | Aucune migration ; aucun paquet neuf au `Cargo.lock`. | AC-2 |

## Critères d'acceptation

0. **AC-0 — Recettes réelles, hors de Kesh puis par Kesh, sur 2024** (décision de l'orchestrateur : la recette n'attend
   pas le CSV 2025 de Beat). Chaque appel réel est précédé de `kesh-gate-logs/appel-reel.sh wise "<motif>"` (plafond 200
   pour Wise, rafales de 4 ; **relire le journal `appels-reels/wise.log` avant** : 65 appels comptés au quota à la fin de
   la recherche du 2026-11-20, le fichier portant 73 lignes dont 8 `REJEU`, qui ne comptent pas). Jeton, réponses brutes
   et CSV restent dans `data-not-committed/` ; seules des **métadonnées** (comptes, égalités, formes) s'écrivent au
   fichier d'epic et aux fiches. **Matière** : le CSV 2024 du site des trois soldes, déjà en local
   (`data-not-committed/2024/OK_11-wise/Extraits-de-compte-Wise-CSV/`, CHF, EUR, USD, du 2024-01-01 au 2024-12-31), et la
   recherche du 2026-11-20 l'a déjà comparé (176 des 177 identifiants, soldes au centime hors frais, 22 frais de
   virement par les devis) ; 2024 porte des entrants, des paiements par carte, des conversions `INTERBALANCE` et
   `AUTO_CONVERSION`, des virements à frais : **tous les types à trancher y sont**.
   - **(a) Recette préalable, à la main, avant toute ligne de 39-12a1, 39-12a2, 39-12a3 ou 39-12b** (budget **45 appels,
     80 au plus** ; au-delà, s'arrêter et rendre compte) : profils (1), soldes (2, à deux instants), activités de
     2024-01-01 à l'instant de l'appel par les mêmes fenêtres que Kesh (≈ 8 pages), transferts de 2023-12-01 à
     l'instant (≈ 3 pages), devis des **22 virements à frais** de 2024 et de **cinq virements sans frais** (≈ 27). Les
     réponses brutes sont **sauvées** dans `data-not-committed/wise-recette-2024/` (non versionné) : la recette finale
     (b) les relit, sans appel de plus. Elle **tranche et écrit** au fichier d'epic (Q-39-17) : (1) la **date** d'une
     ligne — `createdOn` ou `updatedOn`, en UTC ou en heure de Zurich — qui redonne la colonne `Date` du CSV
     pour chacune des lignes de 2024 (y compris celles qui tombent près de minuit, et la date d'une ligne de
     frais face à celle de son origine) — **et le choix suit l'achèvement** : relever les lignes dont `createdOn` et
     `updatedOn` tombent sur des jours différents (virements) et dire lequel le CSV suit ; un mouvement terminé après la fin
     d'un relevé ne doit jamais porter une date antérieure à cette fin, sans quoi il est perdu et le contrôle refuse
     (39-12a2, notes) ; (2) l'**identifiant** d'une conversion `AUTO_CONVERSION` : le numéro
     `CONVERSION_ORDER-…` du CSV se lit-il quelque part dans l'API (tranché : non, `AUTO_CONVERSION-<resource_id>` définitif, 39-12a2 AC-3) ;
     (3) le **champ de solde** de `/v4/…/balances` : `amount.value` et `cashAmount.value` sont relevés tous deux sur les
     trois soldes ; **égaux**, on lit `amount.value` ; **différents**, la différence s'explique ou la phase 2 attend
     (l'effet des activités non `COMPLETED` sur le champ n'est pas mesurable sur 2024 : il reste couvert, de façon sûre,
     par le contrôle de 39-12a2 AC-5, qui refuse en cas d'écart) ; (4) la **règle du sens** des montants
     (39-12a2 AC-3) sur les paiements par carte en devise étrangère **dont la devise est aussi un solde du profil**, sur les
     virements sortants dont le bénéficiaire est payé dans une devise que le profil détient aussi (la devise débitée est
     celle du devis, 39-12a2 AC-3) et sur les entrants de 2024 ; (5) le **type de
     détail** (`DEPOSIT`, `TRANSFER`, `CARD`, `CONVERSION`) que porte le CSV pour chaque type d'activité ; (6) la
     `reference` des transferts face à la colonne `Payment Reference` ; (7) **quand les frais d'un virement sont nuls**,
     ce que rend le devis (`fee.total` = 0 ; tranché : option `BALANCE` présente, non absente, voir le statut de (a)) : les cinq virements sans frais le disent ; (8) un **virement remboursé** (s'il s'en trouve en 2024, sinon noté « non relevé », la règle de suffixe de 39-12a2 AC-3 valant alors) : comment l'API le rend — seconde activité de même `resource_id`, autre `activity_type`, ou statut du virement d'origine —, sous quel identifiant et quel `Payment Reference` le CSV porte la ligne de remboursement, et que le solde reconstruit reste juste ; **sans cette ligne relevée, un téléchargement qui couvre un remboursement reste possible** (suffixe provisoire), mais le manuel avertit de ne pas mélanger CSV et téléchargement sur ce virement. Une règle
     **contredite** par 2024 se corrige dans la fiche concernée avant le développement ; un écart qui ne se règle pas va
     à Beat, et la phase 2 attend.
   - **Statut de (a) : faite le 2026-11-20** (profil business, jeton personnel lecture seule, 34 appels sur 80 permis, activités 2024 rejouées sans appel ; décisions : `kesh-gate-logs/epic-39-40-kiss/wise-ac0-decisions.md`, détail hors dépôt dans `data-not-committed/wise-recette-ac0-activites.md`). Concordance exacte sur 2024 : 176 identifiants sur 177 (le 177e est la conversion automatique, absente sous son numéro), 22 frais de virement sur 22 par le devis, soldes reconstruits égaux à ceux du CSV en clôture dans les trois devises, 12 fins de mois sur 12, 163 fins de jour sur 163, 230 soldes courants sur 230. Les huit points sont tranchés et reportés dans 39-12a2 (règles définitives), 39-12a3 et l'epic (Q-39-17). **Restent AC-0 (b) (après 39-12a2) et AC-0 (c) (un mois 2025, non bloquant).**
- **(b) Recette finale, après 39-12a2, avant que Beat s'y fie** : les réponses brutes sauvées en (a) passent par
     `rows_from_activities` (39-12a2), **dans un test local non versionné**, **comme si l'on était le 2024-12-31** :
     `period` = 2024, soldes actuels = clôtures 2024-12-31 du CSV de chaque devise, activités de 2024
     seules, `transfer_fees` = les devis sauvés en (a) pour les 27 virements qu'ils couvrent et, **pour les autres
     virements sortants de 2024**, les frais du CSV (ligne `FEE-TRANSFER-…` ou zéro ; `fee_currency` = la devise du fichier CSV qui porte le débit `TRANSFER-…`, celle qui départage le montant, 39-12a2 AC-3) — seule entrée qui ne vient pas de
     l'API, écrite au compte rendu : 39-12a2 exige un frais par virement sortant (`MissingTransferFee`), et lire le devis
     des quelque 130 virements de 2024 dépasserait le quota (65 + ≈ 150 > 200) ; les 27 devis éprouvent la correspondance
     dans les deux cas (avec et sans frais), `checkpoints` = clôtures du CSV à chaque fin de mois (le CSV porte un solde courant à chaque ligne). Chaque
     ligne rendue est comparée à la ligne du CSV de même identifiant (date ; montant, devise, sens, **solde
     courant** ; **l'heure n'est pas comparée** : Kesh l'écrit à l'heure de Zurich, le CSV en UTC+1 toute l'année, d'où un écart prévu d'une heure l'été) ; seules les différences **prévues** sont admises (frais de conversion inclus dans le débit, H2 ;
     conversion automatique si (2) n'a pas trouvé le numéro) ; le solde d'ouverture et de clôture de chaque devise égale
     celui du CSV. Le compte rendu (métadonnées) s'écrit au fichier d'epic. Aucun appel de plus.
   - **(c) Comparaison d'un mois 2025, plus tard, non bloquante** : quand Beat dépose le CSV 2025 du site d'un mois, la
     même comparaison (a)(1)(4) et (b) se rejoue sur ce mois, avec les soldes **réels** d'aujourd'hui ; aucune ligne de
     la phase 2 n'attend. Tant qu'elle n'est pas faite, le manuel (39-12b) dit que la voie n'a été éprouvée que sur 2024.
1. **AC-1 — Configuration**, par le **lecteur unique du socle S1** (`provider_env`, écrit par 38-12a ; patron
   `KESH_SMTP_PASSWORD`, `crates/kesh-api/src/config.rs:329-332`, `Debug` masqué `:380`, accesseur `:399`,
   `is_template_placeholder` `:1538`) : `KESH_WISE_<n>_ACCOUNT` (libellé du compte Wise de 39-11b), `KESH_WISE_<n>_TOKEN`
   (jeton personnel **lecture seule**), `n` de 1 à 9 ; facultatif `KESH_WISE_<n>_PROFILE_ID` (sinon le seul profil
   `business` du jeton, AC-2) ; `KESH_WISE_BASE_URL` (défaut `https://api.wise.com` ; `http` admis vers le seul bouclage,
   S5) ; `KESH_WISE_PARALLEL` (S1 : 1 à 6, défaut 1 ; employé par 39-12a3). **Champ facultatif** : le socle en développement (38-12a, `provider_env.rs`) porte déjà `ProviderSpec.optional_fields`
   (vide pour PayPal ; une série reste incomplète sans `ACCOUNT` ou sans `TOKEN`, jamais sans `PROFILE_ID` ; test
   `champ_facultatif`) ; si le socle fusionné ne l'a pas, cette story l'y ajoute ainsi, au lieu de lire `PROFILE_ID` à
   part ; la spécification de Wise est `key: "WISE"`, `provider: "wise"`, `fields: ["TOKEN"]`, `optional_fields: ["PROFILE_ID"]`, hôte par défaut `https://api.wise.com`, inscrite à `ALL_PROVIDERS`. **Aucune clé SCA** : la variable
   `KESH_WISE_<n>_SCA_KEY_FILE` des versions précédentes de cette fiche n'existe pas et n'est lue nulle part. Série
   ACCOUNT/TOKEN incomplète, libellé en double (sans casse), valeur de la forme `<…>` (une valeur **vide vaut absente**,
   `env_nonempty`, `config.rs:1406` : sans aucune variable Wise, Kesh démarre) → refus au **démarrage**, message qui nomme
   la variable, jamais sa valeur. **Transmission au conteneur** (socle S1) : les séries `KESH_WISE_1_*` à
   `KESH_WISE_3_*` (ACCOUNT, TOKEN, PROFILE_ID), `KESH_WISE_BASE_URL` et `KESH_WISE_PARALLEL` s'ajoutent à
   `docker-compose.yml` et `docker-compose.prod.yml` en `${NOM:-}` (patron `KESH_SMTP_*`, `docker-compose.yml:117-122`)
   et à `.env.example` (commentées, avec `<EDIT: …>`, patron `:302-340`). **Garde `configuration_transmise.rs` (#550)** :
   la famille indexée générique de 38-12a **dérive ses noms d'`ALL_PROVIDERS`** (`noms_familles`,
   `noms_familles_exiges` : séries 1 à 9 connues, 1 à 3 exigées, tous les champs dont les facultatifs, et les deux noms
   sans numéro `KESH_WISE_BASE_URL` et `KESH_WISE_PARALLEL`) : inscrire `WISE` à `ALL_PROVIDERS` suffit, sans relâcher
   aucune liste ni inscription à la main (si le socle fusionné ne les dérivait pas, les deux noms sans numéro
   s'inscriraient en forme exacte à `EMPLACEMENTS_AUTORISES`) ; leurs lignes de `.env.example` sont commentées ou vides, jamais
   `<…>`.
2. **AC-2 — Client, profil et soldes** (`crates/kesh-api/src/wise/`, client HTTP du socle S5 : rustls, aucune redirection
   — le jeton ne suit jamais une redirection —, délais bornés, `User-Agent` sans donnée personnelle ; **`reqwest` n'a pas
   la fonctionnalité `json`** en dépendance normale, `Cargo.toml:112` : `.bytes()` puis `serde_json::from_slice`, comme
   la source des cours). `Authorization: Bearer <jeton>`, rien d'autre (aucun en-tête `x-2fa-approval` ni `X-Signature`).
   Requêtes `GET` seulement : aucune route de Wise qui écrit n'est appelée.
   - **Profil** : `GET /v1/profiles` ; sans `KESH_WISE_<n>_PROFILE_ID`, le **seul** profil de type `business` (casse
     ignorée) ; zéro ou plusieurs → `WISE_PROFILE_AMBIGUOUS` (`details: { count }`) ; avec l'identifiant, il doit figurer
     dans la liste.
   - **Soldes** : `GET /v4/profiles/{p}/balances?types=STANDARD` ; un solde = `id`, `currency` et le **champ de solde**
     `amount.value` (AC-0 (a)(3), égal à `cashAmount.value`), lu comme nombre décimal JSON (jamais `f64` : le
     **texte exact reçu**, lu par `serde_json::value::RawValue` puis `Decimal::from_str` ; de même `fee.total` du devis).
3. **AC-3 — Activités, transferts, devis.**
   - **Activités** : `GET /v1/profiles/{p}/activities?since=…&until=…&size=100`, puis `&nextCursor=<cursor>` tant que
     `cursor` n'est pas nul ; `since`/`until` en ISO 8601 UTC à la milliseconde. **Fenêtres** : fonction du socle S2,
     `windows(from, to, max_days = 366, now)` (aucune borne de Wise : la fenêtre ne sert qu'à découper la progression,
     366 couvre une année civile bissextile en une fenêtre), jours de **Zurich** ; `since` =
     `kesh_core::horloge::debut_du_jour_suisse(Window.from)`, `until` = `fin_du_jour_suisse(Window.to)` (`crates/kesh-core/src/horloge.rs:95`,
     `:110`) borné à `now`, **à la milliseconde** : `Window.start` et `Window.end` du socle sont tronqués à la seconde
     (l'API de PayPal les veut ainsi) et **ne servent pas ici**, sans quoi une activité de la dernière seconde d'une
     fenêtre n'appartiendrait à aucune ; un chevauchement éventuel des bornes est sans effet, la déduplication par clé
     (ci-dessous) le résorbe ; une activité vue dans
     deux pages ou deux fenêtres (frontière) n'est gardée qu'une fois, **dans sa dernière version lue** (clé : l'`id` **d'activité**, non `resource.type` + `resource.id` : deux activités distinctes d'une même ressource — un virement et son remboursement, seconde activité de même `resource_id` — restent deux, 39-12a2 AC-3 en tire une ligne chacune ; revue de code P1, M2). Rendu brut :
     `WiseActivity { id, activity_type, resource_type, resource_id, title, description, primary_amount, secondary_amount,
     status, created_on, updated_on }` (textes tels quels ; l'analyse est celle de 39-12a2). Plafond de garde : 200 pages
     par fenêtre, sinon `PROVIDER_RESPONSE_INVALID` (`details.reason = "too_many_pages"`).
   - **Transferts** : `GET /v1/transfers?profile={p}&createdDateStart=…&createdDateEnd=…&limit=200&offset=…` (**`limit=100`**, non 200 : les relevés réels de l'AC-0 ont été faits à 100, et une page plus courte que `limit` terminant la lecture, une limite plafonnée par Wise en deçà de 200 l'arrêterait silencieusement à la première page ; pages par
     `offset` jusqu'à une page de moins de `limit` éléments) sur la période de lecture **élargie de 31 jours avant son
     début** (un transfert se crée avant de se terminer) ; rendu `WiseTransfer { id, quote_uuid, source_currency,
     reference }`. Un transfert sortant d'une activité absent de la liste se lit seul par `GET /v1/transfers/{id}`.
   - **Devis** : `GET /v3/profiles/{p}/quotes/{quoteUuid}` → `paymentOptions[]` ; l'option retenue est celle dont
     `payIn` = `BALANCE` **et** `payOut` = `payOut` du devis ; rendu `WiseQuoteFee { currency, total }` (`fee.total`,
     devise source du devis). Zéro ou plusieurs options retenues → `PROVIDER_RESPONSE_INVALID`
     (`details.reason = "quote_fee"`).
   - Le client **n'assemble rien** : il rend profil, soldes, activités, transferts et devis ; l'assemblage en lignes est
     la fonction pure de 39-12a2, l'ordre des appels et la progression sont ceux de 39-12a3.
4. **AC-4 — Erreurs**, codes du socle S4 (génériques `PROVIDER_*`, jamais de texte de Wise) : 401 →
   `PROVIDER_AUTH_FAILED` ; 403 (avec ou sans `x-2fa-approval` : un défi d'authentification forte n'est jamais relevé) →
   `PROVIDER_PERMISSION_DENIED` ; 429 et 5xx → reprise du socle S2 (`Retry-After` entier borné à 120 s, sinon 5, 10, 20,
   40, 60 s ; six essais ; pause commune), puis `PROVIDER_RATE_LIMITED` ou `PROVIDER_UNAVAILABLE` ; délai dépassé →
   `PROVIDER_UNAVAILABLE` ; réponse illisible, champ attendu absent → `PROVIDER_RESPONSE_INVALID` ; autre statut →
   `PROVIDER_API_ERROR` (code HTTP seul). Chaque erreur porte `details: { provider: "wise", step, from?, to? }` (`step` =
   `profiles`, `balances`, `activities`, `transfers`, `quote` ; un seul nom, `details`, socle S3). Messages aux quatre
   locales par 39-12a3.
5. **AC-5 — Tests sans réseau** (socle S5 et S9). Serveur `axum` **dans le processus** sur `127.0.0.1:0`
   (`KESH_WISE_BASE_URL`), données **fictives** : `profiles` (un `business`, un `personal`), `balances` (trois soldes),
   `activities` sur trois pages (curseur, dernière page à `cursor` nul ; une activité répétée à la frontière de deux
   fenêtres), `transfers` sur deux pages d'`offset`, un transfert absent de la liste lu par son identifiant, `quotes`
   (une option `BALANCE` juste, une sans option `BALANCE`, une à deux options), un 429 avec puis sans `Retry-After`, un
   500 puis 200, un 401, un 403 portant `x-2fa-approval` (→ `PROVIDER_PERMISSION_DENIED`, **aucun** rejeu), une réponse
   tronquée ; le serveur vérifie qu'aucune requête ne porte `X-Signature` ni n'emploie un verbe autre que `GET`.
   Unitaires : configuration (série incomplète, gabarit, libellé en double, valeur vide = absente ; patron de
   `smtp_password_masked_in_debug`, `config.rs:3484`), fenêtres à 366 jours (socle). **Test au marqueur** :
   `TOKEN-MARQUEUR-…` n'apparaît dans aucun journal capturé, erreur ni réponse.

## Tâches

- [x] T0 — Recette préalable constatée sur 2024 (AC-0 a), faite le 2026-11-20 (34 appels), réponses brutes sauvées ; décisions reportées dans 39-12a2, 39-12a3 et l'epic.
- [x] T1 — Configuration par le lecteur du socle, transmission, garde (AC-1).
- [x] T2 — Client : profil, soldes, activités paginées, transferts, devis, erreurs (AC-2 à AC-4).
- [x] T3 — Serveur simulé et tests (AC-5).
- [ ] T4 — Recette finale sur 2024 (AC-0 b), après 39-12a2 ; comparaison d'un mois 2025 (AC-0 c) quand Beat dépose son CSV, hors chemin critique.

## Stratégie de tests

Gate ciblé : tests de `kesh-api` (`config`, `wise`) et du binaire de test du serveur simulé ; **gate complet** au dernier
commit. Aucune migration : P5 à P8 sans objet. `Cargo.lock` : inchangé (se contrôle par `git diff Cargo.lock`).

## Modules touchés (1)

`kesh-api` (configuration, client) ; plus `docker-compose.yml`, `docker-compose.prod.yml`, `.env.example` (transmission des
variables).

## Fichiers communs

`crates/kesh-api/src/config.rs` et `provider_env` du socle (38-12a ; un champ facultatif y est ajouté), `crates/kesh-api/tests/configuration_transmise.rs`,
`docker-compose.yml`, `docker-compose.prod.yml`, `.env.example` (blocs de PayPal et Stripe à côté) : à relire à la reprise
de `main`.

## Note de version

*Ajouté* — Configuration du téléchargement des mouvements Wise : jeton personnel en lecture seule, posé dans
l'environnement du serveur (`KESH_WISE_<n>_…`), jamais en base ni dans la sauvegarde ; aucune authentification forte.

## Notes de développement

- Doc publique de Wise relue le 2026-11-20 : activités (`activitylist`, sécurité `PersonalToken`, `size` ≤ 100, aucune
  borne de période ni d'historique ; `primaryAmount` « not meant to be parsed as a number » : d'où la forme fermée et le
  contrôle de solde de 39-12a2) ; jetons personnels (relevés réservés à six pays).
- **Voie de secours, hors périmètre** : l'Open Banking britannique de Wise par un agrégateur régulé (Enable Banking, mode
  restreint gratuit) donnerait un relevé « officiel » avec les frais de conversion en lignes séparées, au prix d'un
  consentement à renouveler tous les 90 jours (2FA). Option future, aucune fiche (H6 du fichier d'epic).
- Ancienne 39-2 (authentification forte) : **abandonnée**, non reprise (H5).

## Dev Agent Record

*Développée le 2026-11-20 (Claude Sonnet 5.5). Commits de code : `51ae9874` (client, tests, transmission), `84940f05`
(ordre des modules, `cargo fmt`), `53cec570` (remédiation P1), `b7dd76fd` (remédiation P2). Aucun appel réseau réel ; données de test fictives écrites à la main.*

**Ce qui est livré** : `crates/kesh-api/src/wise/mod.rs` (spécification `WISE`, `WiseClient` : `profiles`,
`resolve_profile`, `balances`, `activities_window`, `activities`, `transfers`, `transfer`, `quote_fee`, `dedup_activities`) ;
`crates/kesh-api/tests/wise_client_e2e.rs` (serveur Wise simulé, 36 tests après les remédiations P1 et P2) ; une ligne à `lib.rs` ; séries 1 à 3,
`KESH_WISE_BASE_URL` et `KESH_WISE_PARALLEL` à `docker-compose.yml`, `docker-compose.prod.yml` et `.env.example`.
Aucune migration, aucun paquet neuf (`Cargo.lock` inchangé).

**⚠️ La branche ne compile pas seule : elle dépend du socle S1 à S9, écrit par 38-12a** (commit `d64688fc`, branche
`story/38-12a`, non fusionné à `epic/39-wise`). Il n'a pas été recopié. Code compilé et testé dans un **bac d'essai** (clone
`--shared`, `story/39-12a1` + fusion de `d64688fc` : la fusion produit trois conflits de texte, `.env.example` et les deux `docker-compose*.yml`, blocs Wise et PayPal voisins, résolus en gardant les deux blocs). **À la fusion de `story/38-12a`, se raccordent** :
1. `provider_env.rs` : `ALL_PROVIDERS` devient `&[PAYPAL, crate::wise::WISE]` (une ligne ; sans elle, la série Wise n'est pas
   lue au démarrage, et le test `wise_est_inscrit_a_la_liste_des_prestataires` rougit) ;
2. la garde `configuration_transmise.rs` dérive ses noms d'`ALL_PROVIDERS` : aucune liste à toucher (à confirmer au gate
   complet de la fusion) ;
3. `docker-compose*.yml` et `.env.example` : le bloc Wise est inséré après `KESH_SKIP_PRE_UPGRADE_BACKUP`, comme le bloc
   PayPal du socle : conflit de texte voisin attendu, résolu en gardant les deux blocs ;
4. `tests/common/mod.rs` du socle déclare `secrets_prestataire` (employé par le test de Wise).

**Écarts avec la fiche, dits** : (a) `details: { provider, step, from?, to? }` n'est pas exprimable avec les variantes
`AppError::Provider*` du socle (`window`, `page`) : `step` n'est porté que par `PROVIDER_RESPONSE_INVALID`, ailleurs `window`
(`2024-01-01…2024-12-31`) et `page` situent l'échec ; à reprendre en 39-12a3 si Guy veut `step` partout (variantes du socle à
étendre). (b) `WISE_PROFILE_AMBIGUOUS` n'est pas une variante d'`AppError` : `WiseProfileError::Ambiguous { count }` la porte,
39-12a3 la rend (variante, texte aux quatre locales) ; un `PROFILE_ID` absent de la liste donne `count: 0`. (c) Transferts :
`limit=100` et non 200 (relevés réels faits à 100 ; une page plus courte que `limit` terminant la lecture, une limite plafonnée
par Wise en deçà l'arrêterait silencieusement à la première page). (d) Aucun suivi de progression (`JobHandle`) dans le client :
39-12a3 compose `activities_window` fenêtre par fenêtre.

**Gates réellement exécutés** (bac d'essai, cible `kesh-targets/kesh-wt3912a1`, journaux `kesh-gate-logs/s39-12a1/`) :
- Gates du dev (bac d'essai, avant remédiation) : **15** tests unitaires `wise::` (`cible-1.log`), **25** tests du serveur
  simulé, verts au rejeu (`cible-3.log`) ; les trois pauses arrondies : `Notee` compare en secondes arrondies (premier
  passage rouge, `cible-2.log`, corrigé).
- **Après les remédiations P1 et P2** (copie hors dépôt `lows-3912a1`, socle de `story/38-12a` fusionné, `ALL_PROVIDERS = &[PAYPAL,
  crate::wise::WISE]`, journaux `lows-*.log`) : `cargo test -p kesh-api --lib -- wise::` **19** tests ; `--test wise_client_e2e`
  **36** tests (**38** après la remédiation P3) ; `--test configuration_transmise` **32** tests (comptés à l'exécution : un `grep -c '#[test]'` en rend 33, dont une ligne de doc-comment et une chaîne de code qui ne sont pas des attributs) ; `cargo fmt --all -- --check` et `cargo clippy --workspace
  --all-targets -- -D warnings` verts. Premier passage rouge corrigé : un montant à 30 décimales, arrondi en silence par `Decimal`
  (désormais refusé), et le test de devise du devis.
- Mutations (`mutation-1.log`) : curseur non encodé, 403 repris comme passager, en-tête `X-Signature` ajouté → **10 tests
  rouges** sur 25 (mutations jouées avant les remédiations ; non rejouées depuis), dont `activites_sur_trois_pages_par_curseur`, `refus_403_avec_defi_…` et
  `aucune_requete_ne_porte_de_signature_…` ; restauré, 25 verts. Aucune mutation n'a été jouée sur les remédiations P1 et P2.
- `cargo fmt --all -- --check` vert (après `84940f05`) ; `cargo clippy --workspace --all-targets -- -D warnings` vert, 0
  avertissement (`clippy.log`).
- **Non fait** : `scripts/test-fast.sh --ci` (gate complet) — **il se jouera après la fusion de 38-12a dans `epic/39-wise`**, la branche ne compile pas seule ; `scripts/gate-ci.sh` ; E2E
  Playwright (**sans objet** : aucun écran, route ni texte touché) ; Vitest (sans objet). **Ne rien en déduire pour ce qui n'a pas tourné.**

**Recette AC-0** : (a) faite avant la story ; (b) recette finale **après 39-12a2** (T4) ; (c) mois 2025, plus tard.

## Change Log

- 2026-11-20 — **développée** (Claude Sonnet 5.5, Dev Agent Record ci-dessus) : client, serveur simulé, transmission ; statut `review` ; revue P1 à lancer (`kesh-gate-logs/s39-12a1/revue-p1-prompt.md`).
- 2026-11-20 — **remédiation de la revue de code P1** (Claude Sonnet 5.5 ; rapport `kesh-gate-logs/s39-12a1/revue-p1-rapport.md` : 0 C, 0 H, 2 M, 6 L). M1 : `amount.value` et `fee.total` lus par `RawValue` (fonctionnalité `raw_value` déclarée, `Cargo.lock` inchangé), tests au corps en texte brut (`99999999999999.99`, `123456789012345.67`, `0.123456789012345678`). M2 : déduplication sur l'`id` d'activité, dernière version gardée (aussi L6), AC-3 aligné sur 39-12a2. L1 : statut classé avant la lecture du corps. L2 : repli `quote` retiré. L3 : devis sans `payOut` refusé. L4 : tests du corps trop grand et du corps coupé. L5 : Dev Agent Record corrigé (trois conflits à la fusion). Revue P2 à lancer (`kesh-gate-logs/s39-12a1/revue-p2-prompt.md`).
- 2026-11-20 — **remédiation de la revue de code P2** (Claude Sonnet 5.5, commit `b7dd76fd` ; revue P2 Sonnet : 0 C, 0 H, 0 M, 5 L). L3 (décision de l'orchestrateur) : la devise des frais est la devise source du **devis**, l'option en repli seulement ; test corrigé, cas inverse ajouté. L4 : `decimal_of_raw` refuse les chaînes hors `-?chiffres(.chiffres)?` et tout montant à plus de 28 décimales (arrondi silencieux de `Decimal`), doc-comment dit. L5 : test e2e `value: null` / `fee.total: null` → `PROVIDER_RESPONSE_INVALID`. L2 : phrase ajoutée à 39-12a2 (répétition d'un même `id` déjà fondue par le client). L1 : Dev Agent Record recompté, AC-3 `limit=100` motivé. Boucle de revue : P1 Opus 0 C / 0 H / 2 M / 6 L → P2 Sonnet 0 / 0 / 0 / 5 L. Passe ciblée P3 sur ce seul commit : `kesh-gate-logs/s39-12a1/revue-p3-prompt.md`. Gate complet après la fusion de 38-12a ; statut `review`.
- 2026-11-20 — validation ciblée AC-0 (a) « décisions » P2 sur le commit `b2d272e0` (Claude Sonnet 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-ac0-decisions-ciblee-2.md`) : 0 C, 0 H, 1 M, 2 L, remédiés. Ici : (b) ne compare plus l'heure (l'écart d'une heure l'été entre Zurich et l'UTC+1 du CSV est dit prévu, sans test dessus) ; (a)(1) ne pose plus « Date Time » comme question ouverte, (a)(2) ne parle plus de règle provisoire.
- 2026-11-20 — validation ciblée AC-0 (a) « décisions » sur le commit `d0cfeda7` (Claude Haiku 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-ac0-decisions-ciblee.md`) : 0 C, 0 H, 1 M, 1 L, remédiés. Ici : (a)(7) « option absente » tranché en « option `BALANCE` présente » ; (b) ne compte plus l'heure du CSV (UTC+1 fixe) comme identique à l'heure de Zurich de Kesh, écart d'une heure l'été prévu.
- 2026-11-20 — **décisions de la recette AC-0 (a), 2026-11-20** (recette réelle sur 2024, 34 appels ; source `kesh-gate-logs/epic-39-40-kiss/wise-ac0-decisions.md`) : AC-0 (a) marquée faite et T0 cochée ; concordance exacte (176/177 identifiants, 22/22 frais, soldes égaux en clôture, 12/12 fins de mois, 163/163 fins de jour, 230/230 soldes courants). Règles tranchées, reportées dans 39-12a2 : date = jour de Zurich de `createdOn` (débits, conversions) ou de `updatedOn` (entrants), jamais d'heure comme clé ; heure du CSV en UTC+1 constant ; `AUTO_CONVERSION-<resource_id>` définitif ; solde `amount.value` ; sens et devise débitée confirmés ; table des types de détail ; `reference` = `Payment Reference` ; virement sans frais = option `BALANCE` à 0 ; virement remboursé non relevé (suffixe provisoire maintenu) ; cartes en devise étrangère non relevées. Restent AC-0 (b) et (c). Les mentions « à trancher à la recette » de la fiche sont devenues caduques ; aucun code touché.
- 2026-11-20 — validation P5 ciblée sur le commit `06a8f336` (Claude Sonnet 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P5-ciblee.md`) : 0 C, 0 H, 0 M, 1 L, remédié. Ici : le virement remboursé entre dans la recette AC-0 (a), point (8) ; le message et le manuel du refus de solde, `MissingTransferFee` et `fee_currency` relus, cohérents.
- 2026-11-20 — validation P4 ciblée sur le commit `a98da37a` (Claude Opus 5.5, contexte frais ; commune à 39-12a1, 39-12a2, 39-12a3, 39-12b ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P4-ciblee.md`) : 0 C, 0 H, 1 M, 2 L, remédiés. Ici (AC-0 b, L) : la devise `fee_currency` des frais pris au CSV est celle du fichier qui porte le débit, puisqu'elle départage désormais le montant.
- 2026-11-20 — validation P3 de la voie activités (Claude Sonnet 5.5, contexte frais ; commune à 39-12a2, 39-12a3, 39-12b ;
  rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P3.md`) : 0 C, 0 H, 3 M, 4 L, remédiés. Ici (AC-0 a) : le
  choix de l'horodatage doit suivre l'achèvement, et la règle du sens est éprouvée sur les virements entre soldes détenus et
  les cartes dont la devise est aussi un solde.
- 2026-11-20 — **réécrite sur la voie activités (2026-11-20)** : recherche `data-not-committed/wise-recherche.md`
  (métadonnées). Relevé de solde et signature SCA retirés (Wise refuse par règle, signature non évaluée) ; variable
  `KESH_WISE_<n>_SCA_KEY_FILE`, module `wise/sca.rs`, code `WISE_SCA_REJECTED` et clé de test retirés ; client des
  activités paginées, des transferts et des devis ; `max_days` = 366 (aucune borne documentée) ; AC-0 remplacé par la
  comparaison d'un mois 2025 au CSV 2025 du site (préalable à la main, puis finale par la fonction de 39-12a2).
  Conversion en lignes déplacée dans la nouvelle 39-12a2 ; route et import renumérotés 39-12a3. Fiche renommée
  (`39-12a1-client-wise-et-sca` → `39-12a1-client-wise-activites`), statut `deferred` → `drafted`, à revalider.

- 2026-11-20 — validation P2 de la voie activités (Claude Opus 5.5, contexte frais ; commune à 39-12a2, 39-12a3, 39-12b ;
  rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P2.md`) : 0 C, 1 H, 2 M, 9 L, remédiés. Ici : **M2** — la
  recette finale (b) exige un frais par virement sortant de 2024 (`MissingTransferFee`) et n'en sauve que 27 : frais du CSV
  pour les autres, écrit comme seule entrée hors API (lire tous les devis dépasserait le quota) ; `read_until` (retiré en
  P1) ôté de (b) ; AC-1 mise au niveau du socle en développement (`optional_fields` et famille de la garde dérivée
  d'`ALL_PROVIDERS` déjà dans 38-12a).
- 2026-11-20 — validation P1 de la voie activités (Claude Sonnet 5.5, contexte frais ; commune à 39-12a1, 39-12a2, 39-12a3,
  39-12b ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P1.md`) : 0 C, 2 H, 10 M, 7 L, remédiés. Ici :
  AC-0 réécrit sur **2024** (décision de l'orchestrateur : CSV local, budget 45 appels, réponses brutes sauvées,
  recette finale ligne à ligne « comme au 2024-12-31 », mois 2025 plus tard et non bloquant, point (7) frais nuls) ;
  champ **facultatif** au lecteur du socle (`PROFILE_ID` : le lecteur exige tous les champs d'une série) ;
  bornes en milliseconde de `Window.from`/`to`, non `Window.start`/`end` tronquées à la seconde.

*Historique — entrées de la fiche « client de l'API Wise — configuration, signature SCA, profil, soldes et relevés »,
laissées telles quelles :*


- 2026-11-20 — fiche rédigée (phase 2), non validée.
- 2026-11-20 — validation P1 (Claude Sonnet 5.5, contexte frais) : voir le Change Log de 39-12a2 (commun aux deux fiches) ;
  la recette 1 impose la signature SCA : fiche scindée, AC-0 recentré sur la recette 2, AC-1 et AC-2 neufs, socle S1 à S9.
- 2026-11-20 — validation P2 (Claude Opus 5.5, contexte frais ; commune à 39-12a1, 39-12a2, 39-12b ; rapport
  `kesh-gate-logs/epic-39-40-kiss/validation/39-12-P2.md`) : 0 C, 0 H, 3 M, 7 L, remédiés, plus trois décisions de l'orchestrateur sur le socle. Ici : recette 2 rendue
  **préalable** au développement (doc de Wise : relevés au jeton personnel restreints à six pays) ; unité de reprise =
  couple requête-rejeu (contradiction avec les six essais du socle) ; clé SCA **exigée** (retrait de
  `WISE_SCA_KEY_NOT_CONFIGURED`) ; signature par `jsonwebtoken` RS256, déjà dépendance directe (au lieu de `ring` + PEM à la
  main), borne de 2048 bits contrôlée au démarrage ; RUSTSEC-2023-0071 relu ; réponse sans défi admise (90 jours) ;
  `max_days` = 468 et `details` (décisions de l'orchestrateur sur le socle) ; exemple d'heure UTC de fin de jour corrigé.
- 2026-11-20 — validation P3 ciblée (Claude Sonnet 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-40-phase2-P3-ciblee.md`) : 0 C, 0 H, 1 M, 3 L, remédiés : recette 2 minutée (attente de 60 s de la garde contre jeton périmé), `intervalEnd` borné à l'instant présent, exemple d'heure d'été, « jamais repris » précisé, garde `configuration_transmise.rs` — famille Wise inscrite.
- 2026-11-20 — validation P4 ciblée (Claude Haiku 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-40-phase2-P4-ciblee.md`) : recette 2 : le rejeu passe par `appel-reel.sh` tant que Q-39-16 n'est pas tranchée (Q-39-15 était déjà prise, IBAN) ; registre : six `assert_eq!` dont cinq littéraux, les deux phrases du manuel ; garde : noms sans numéro `KESH_WISE_BASE_URL` et `KESH_WISE_PARALLEL` inscrits ; `payout_total_mismatch` (code exact de 40-7c).
- 2026-11-20 — validation P5 ciblée (Claude Sonnet 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-40-phase2-P5-ciblee.md`) : 0 C, 0 H, 0 M, 2 L, remédiés : casse des noms `KESH_WISE_BASE_URL` et `KESH_WISE_PARALLEL` au Change Log P4 ; titre du « Socle commun » aligné au niveau `##` des deux epics.
- 2026-11-20 — validation P2, complément (Claude Opus 5.5) : corrections du socle relevées par la P2 de la 38-12 appliquées —
  valeur vide = absente, séries 1 à 3 transmises par les deux compose et `.env.example`, fichier de clé monté à la main.

- 2026-11-20 — remédiation de la passe ciblée P3 (rapport `kesh-gate-logs/s39-12a1/revue-p3-rapport.md`, Haiku ; 0 C, 0 H, 1 M, 2 L) :
  **M1 vérifiée avant d'être remédiée** — la prémisse était vraie : sur le code de `b7dd76fd`, le nombre JSON brut `1.5e-28`
  rendait `Some(0.0000000000000000000000000002)` (arrondi en silence par `from_scientific_lossy`) ; `1e-29` rendait `None`.
  Décision de l'orchestrateur : un montant Wise est toujours un décimal simple, **toute forme à exposant est refusée**, nombre
  JSON comme chaîne, par la même forme stricte `-?chiffres(.chiffres)?` (au plus 28 décimales) appliquée à la littérale brute
  (M1, L1, L3 ; `1e-7`, qui était accepté, est désormais refusé). **L2** : devise source du devis et de l'option `BALANCE`
  présentes et différentes → `PROVIDER_RESPONSE_INVALID` (`reason = "quote_fee"`), plus de choix silencieux. Tests :
  `nombres_lus_par_le_texte_recu` (exposants refusés, `-0`), `montants_a_exposant_refuses_en_texte_brut`,
  `devises_du_devis_et_de_l_option_qui_different_refusees` ; deux tests existants, dont la devise d'option différait du devis,
  alignés sur CHF. Gates (copie `lows-3912a1`) : fmt, clippy workspace `-D warnings` verts ; `wise::` **19**, `wise_client_e2e`
  **38**, `configuration_transmise` **32** (le Dev Agent Record disait 32 : juste, le grep en rend 33 par deux lignes qui ne sont
  pas des attributs). Mutation (refus de l'exposant retiré) : `nombres_lus_par_le_texte_recu` et
  `montants_a_exposant_refuses_en_texte_brut` rougissent. Passe ciblée P4 à lancer (`kesh-gate-logs/s39-12a1/revue-p4-prompt.md`).
- 2026-11-20 — **reportée** : recettes réelles 1 et 2 (11 appels) : le relevé de solde par jeton personnel est refusé
  pour le compte suisse (403 `x-2fa-approval` à chaque requête ; rejeu signé par clé publique, signature vérifiée en
  local, toujours `REJECTED` ; endpoint de statut du jeton à usage unique 404). La doc publique des jetons personnels le
  dit : relevés par API non pris en charge hors US, CA, AU, NZ, SG, MY. Reprise seulement si Wise ouvre l'accès (support
  ou accord Wise Platform). Voie retenue : import CSV de la phase 1. Détails : `data-not-committed/wise-recette-ac0*.md`.
