# Story 39-12a1 : client de l'API Wise — configuration, signature SCA, profil, soldes et relevés

Status: drafted

*Fiche rédigée le 2026-11-20 (ex-39-12a, scindée le même jour en 39-12a1 et 39-12a2 par la règle de splitting préventif : la
recette réelle a imposé la signature SCA, qui s'ajoute à la configuration, au client et à la route). Branche
`epic/39-wise`, numéros de ligne relevés au code de `origin/main` (`fd04099f`). Epic 39, **phase 2** (H1 à H5 du fichier
d'epic ; **socle commun S1 à S9** de la phase 2 des Epics 38, 39 et 40, § *Socle commun* du fichier d'epic). **Dépend de** :
39-11a (lecteur), du **socle S1, S2 et S5**, écrit par 38-12a (ou, à défaut, par la première des fiches de phase 2
développée, générique : C-40-K6). **Suivie par** : 39-12a2.*

## Story

En tant qu'**Administrateur**, je veux poser dans l'environnement du serveur le jeton Wise en lecture seule d'un compte
Wise et la clé privée qui signe le défi d'authentification forte (SCA) de Wise, afin que Kesh puisse lire les relevés de
chaque solde sans que ni le jeton ni la clé soient jamais en base ni dans une sauvegarde.

## Ce que la recette réelle du 2026-11-20 a établi

*Trois appels réels en lecture seule (`data-not-committed/wise-recette-ac0.md`, métadonnées seulement).*

| Point | État |
|---|---|
| `GET /v1/profiles` avec le jeton personnel | **constaté** : 200, un profil `personal` et un profil `business` (champs `details`, `id`, `type`) |
| `GET /v4/profiles/{profileId}/balances?types=STANDARD` | **constaté** : 200 ; champs `id`, `currency`, `type`, `name`, `amount`, `cashAmount`, `groupId`, `primary`, `visible`… |
| `GET /v1/profiles/{profileId}/balance-statements/{balanceId}/statement.csv?currency=…&intervalStart=…&intervalEnd=…&type=FLAT` | **constaté** : **403**, corps vide, en-têtes `x-2fa-approval: <uuid>` et `x-2fa-approval-result: REJECTED` : le jeton seul ne suffit pas, **Wise exige la signature SCA pour la Suisse** |
| limite de débit | **aucune visible** (ni `x-ratelimit-*` ni `Retry-After`) ; la forme d'un 429 : **non confirmée** |
| rejeu signé (`x-2fa-approval` et `X-Signature`, RSA-SHA256 en base64 du jeton à usage unique, clé publique déposée dans les réglages API de Wise) | **non constaté** : procédure documentée par Wise (guide SCA, de mémoire ; la page `…/sca-over-api/personal-tokens` n'a pas été relue : 404 au 2026-11-20) ; à constater à la **recette 2** (AC-0) |
| colonnes du `statement.csv` de l'API ; `statement.json` | **non obtenus** (SCA) : recette 2 |
| profondeur d'historique servie, durée de validité du jeton à usage unique, interface où se dépose la clé publique | **non confirmées** |

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-12a1-0` | La **recette 2** (rejeu signé) constate colonnes du CSV de l'API et signature acceptée. | AC-0 |
| `FR-39-12a1-1` | Jeton, identifiant de profil facultatif et fichier de la clé SCA se posent par variables d'environnement, une série par compte Wise, rattachée à son libellé ; rien en base ni en sauvegarde. | AC-1 |
| `FR-39-12a1-2` | Un défi SCA (403 avec `x-2fa-approval`) est relevé par une signature RSA-SHA256 de son jeton, rejeu unique. | AC-2 |
| `FR-39-12a1-3` | Le client lit le profil d'entreprise, ses soldes `STANDARD` et le relevé d'un solde sur une fenêtre, rendu en `WiseRow`. | AC-3 |
| `FR-39-12a1-4` | Chaque erreur de Wise devient un code `PROVIDER_*` ou `WISE_*` lisible, sans texte de Wise, jeton, signature ni clé. | AC-4 |
| `NFR-39-12a1-1` | Aucun appel réseau en CI ni par un agent : serveur Wise simulé dans le processus (socle S5). | AC-5 |
| `NFR-39-12a1-2` | Jeton, clé privée, jeton de défi et signature absents de tout journal, erreur, réponse, audit, export et sauvegarde (test au marqueur, socle S9). | AC-1, AC-5 |
| `NFR-39-12a1-3` | Aucune migration ; aucun paquet neuf au `Cargo.lock`. | AC-2 |

## Critères d'acceptation

0. **AC-0 — Recette 2 (non bloquante pour le développement, bloquante pour le choix CSV ou JSON de l'AC-3).** Une fois la
   **clé publique** déposée par Beat dans les réglages API de Wise, un appel réel signé, **à la main, hors de Kesh**, par
   l'orchestrateur, **chaque appel précédé de `kesh-gate-logs/appel-reel.sh wise "<motif>"`** (1 par minute, 40 au total ;
   **3 consommés** à la recette 1) : relevé CSV d'un mois pour un solde (403 puis rejeu signé = **2 appels**), puis le
   relevé JSON du même mois (2 appels), soit **4 appels de plus**. Constats écrits au fichier d'epic (Q-39-13, Q-39-14) :
   rejeu accepté ou non ; colonnes du CSV de l'API comparées à celles de 39-11a (relevé du site, au rapport de recette 1) ;
   forme d'un 429 s'il survient. **Rejeu refusé** malgré une clé correctement déposée → la story s'arrête et Beat décide.
   **Colonnes identiques** → `parse_wise_csv` de 39-11a sert tel quel (AC-3, branche CSV) ; **différentes** →
   `kesh_import::wise::rows_from_statement_json` (pure, une fonction, module de 39-11a) rend les mêmes `WiseRow`, frais à
   part, solde courant contrôlé. Aucun développement de la branche JSON avant ce constat.
1. **AC-1 — Configuration**, par le **lecteur unique du socle S1** (`provider_env`, écrit par 38-12a ; patron
   `KESH_SMTP_PASSWORD`, `crates/kesh-api/src/config.rs:329-332`, `Debug` masqué `:380`, accesseur `:399`,
   `is_template_placeholder` `:1538`) : `KESH_WISE_<n>_ACCOUNT` (libellé du compte Wise de 39-11b), `KESH_WISE_<n>_TOKEN`,
   `n` de 1 à 9 ; facultatifs `KESH_WISE_<n>_PROFILE_ID` (sinon le seul profil `business` du jeton, AC-3) et
   **`KESH_WISE_<n>_SCA_KEY_FILE`** (chemin d'un fichier PEM, clé privée RSA **non chiffrée**, 2048 bits au moins, PKCS#8
   `BEGIN PRIVATE KEY` ou PKCS#1 `BEGIN RSA PRIVATE KEY`) ; `KESH_WISE_BASE_URL` (défaut `https://api.wise.com` ; bac à sable
   `https://api.sandbox.transferwise.tech`, de mémoire ; `http` admis vers le seul bouclage, S5) ; `KESH_WISE_PARALLEL`
   (S1 : 1 à 6, défaut 1 ; employé par 39-12a2). Paire ACCOUNT/TOKEN incomplète, libellé en double (sans casse), valeur vide
   ou de la forme `<…>`, **fichier de clé illisible, vide, chiffré, non RSA ou de moins de 2048 bits** → refus au
   **démarrage**, message qui nomme la variable (et le chemin du fichier, jamais son contenu). La clé est lue **une fois**,
   au démarrage, et gardée en mémoire dans un champ privé de `Config` (masqué dans `Debug`) ; la remplacer exige un
   redémarrage, comme tout secret. Le fichier se trouve **hors** des dossiers qu'emporte la sauvegarde (rien dans
   `.keshbackup`) ; en conteneur, il se monte en lecture seule (manuel, 39-12b). Sans `SCA_KEY_FILE`, un défi rencontré donne
   `WISE_SCA_KEY_NOT_CONFIGURED` (AC-4) : un compte hors Suisse et hors Union européenne peut ne jamais en avoir besoin.
2. **AC-2 — Signature SCA.** Module `crates/kesh-api/src/wise/sca.rs`, **sans réseau** : `sign_challenge(key, token: &str) ->
   Result<String, ScaError>` = signature **RSA PKCS#1 v1.5 / SHA-256** des octets UTF-8 du jeton `x-2fa-approval`, en
   **base64 standard**. **Aucun paquet neuf** : `ring` (**0.17.14**, déjà au `Cargo.lock` par `rustls`) pour
   `RsaKeyPair::from_pkcs8` / `from_der` et `sign(&RSA_PKCS1_SHA256, …)`, `base64` (**0.22.1**, déjà au `Cargo.lock`) ; les deux
   se déclarent en dépendances directes de `kesh-api` (`Cargo.toml`, à côté de `sha2 = "0.10"` `:55`), versions du lock ;
   le décodage PEM par `pem` (3.0.6, déjà au lock) ou à la main (retrait des lignes d'en-tête, base64). ⚠️ **Pourquoi pas
   `rsa` 0.9.10**, pourtant au lock par `sqlx-mysql` : il porte l'avis RUSTSEC-2023-0071 (attaque par chronométrage, sans
   correctif ; **numéro cité de mémoire, à relire**) ; `ring` n'en porte pas. Procédure du client (AC-3) : une requête `GET`
   reçoit **403 avec `x-2fa-approval`** → signer le jeton → **rejouer la même requête une seule fois** avec les en-têtes
   `x-2fa-approval: <jeton>` et `X-Signature: <signature>` ; la réponse du rejeu est **définitive** (un second 403 →
   `WISE_SCA_REJECTED`, jamais de troisième essai : chaque tentative consomme un jeton et un appel). Chaque requête de
   relevé coûte donc **deux** appels HTTP (progression et délais en tiennent compte, AC-5). Le jeton de défi, la signature
   et la clé ne sont jamais journalisés. **Test** : signature vérifiée avec la clé publique correspondante
   (`ring::signature::UnparsedPublicKey`, `RSA_PKCS1_2048_8192_SHA256`) sur une clé **de test sans valeur**, versionnée
   (`crates/kesh-api/tests/fixtures/wise/sca-test-key.pem`, en-tête qui dit qu'elle ne sert qu'aux tests ; si la protection
   de dépôt de GitHub la refuse, elle se reconstruit au test depuis un DER en constante) ; la **conformité à ce que Wise
   accepte** n'est prouvée que par la recette 2.
3. **AC-3 — Client** (`crates/kesh-api/src/wise/`, client HTTP du socle S5 : rustls, aucune redirection — le jeton ne suit
   jamais une redirection —, délais bornés, `User-Agent` sans donnée personnelle ; **`reqwest` n'a pas la fonctionnalité
   `json` en dépendance normale** (`Cargo.toml:112`, `default-features = false, features = ["rustls-tls"]`) : lire par
   `.bytes()` puis `serde_json::from_slice`, comme la source des cours lit son XML). `Authorization: Bearer <jeton>`.
   - **Profil** : `GET /v1/profiles` (constaté) ; sans `KESH_WISE_<n>_PROFILE_ID`, le **seul** profil de type `business`
     (casse ignorée) ; zéro ou plusieurs → `WISE_PROFILE_AMBIGUOUS` (`details: { count }`) ; avec l'identifiant, il doit
     figurer dans la liste.
   - **Soldes** : `GET /v4/profiles/{profileId}/balances?types=STANDARD` (constaté) ; un solde = `id` et `currency`.
   - **Relevé** : `GET /v1/profiles/{profileId}/balance-statements/{balanceId}/statement.csv` (ou `.json`, AC-0) avec
     `currency`, `intervalStart`, `intervalEnd`, `type=FLAT` (frais sur une ligne à part). **Fenêtres** : fonction du socle
     S2, `windows(from, to, max_days = 469)` (borne de Wise, doc publique), jours de **Zurich** ; `intervalStart` =
     `kesh_core::horloge::debut_du_jour_suisse(début de fenêtre)` et `intervalEnd` = `fin_du_jour_suisse(fin de fenêtre)`
     (`crates/kesh-core/src/horloge.rs:95`, `:110`), en UTC à la milliseconde (`…T23:59:59.999Z`) : fenêtres contiguës, sans
     chevauchement d'une milliseconde.
   - **Plusieurs fenêtres d'une même devise** : lignes concaténées dans l'ordre ; solde d'ouverture de la **première**,
     de clôture de la **dernière** (colonnes de 39-11a AC-5) ; la clôture de chaque fenêtre doit égaler l'ouverture de la
     suivante, sinon `PROVIDER_RESPONSE_INVALID` (rien n'est importé, 39-12a2).
   - Un solde Wise sans compte bancaire de la devise dans Kesh est listé (`unlinkedCurrencies`, 39-12a2) et **non
     téléchargé** : le client expose la liste des soldes, le choix est fait par 39-12a2.
4. **AC-4 — Erreurs**, codes du socle S4 (génériques `PROVIDER_*`, jamais de texte de Wise) : 401 →
   `PROVIDER_AUTH_FAILED` ; **403 sans `x-2fa-approval`** → `PROVIDER_PERMISSION_DENIED` ; 403 avec défi et sans clé →
   `WISE_SCA_KEY_NOT_CONFIGURED` ; **403 au rejeu signé → `WISE_SCA_REJECTED`** (le message dit : clé publique non déposée
   chez Wise, ou qui ne correspond pas à la clé privée du fichier) ; 429 et 5xx → reprise du socle S2 (`Retry-After` entier
   borné à 120 s, sinon 5, 10, 20, 40, 60 s ; six essais ; pause commune), puis `PROVIDER_RATE_LIMITED` ou
   `PROVIDER_UNAVAILABLE` ; délai dépassé → `PROVIDER_UNAVAILABLE` ; réponse illisible ou colonnes inattendues →
   `PROVIDER_RESPONSE_INVALID` ; autre statut → `PROVIDER_API_ERROR` (code HTTP seul). Chaque erreur nomme l'étape (devise
   et fenêtre) dans `detail`. Messages aux quatre locales par 39-12a2.
5. **AC-5 — Tests sans réseau** (socle S5 et S9). Serveur `axum` **dans le processus** sur `127.0.0.1:0`
   (`KESH_WISE_BASE_URL`), données **fictives** : `profiles` (un `business`, un `personal`), `balances` (trois soldes),
   `statement.csv` qui joue : le **défi** (403 avec `x-2fa-approval: TEST-…`) puis, au rejeu, la **vérification de la
   signature avec la clé publique de test** (200 si juste, 403 `x-2fa-approval-result: REJECTED` sinon), un 429 avec puis
   sans `Retry-After`, un 401, un 403 sans défi, une réponse tronquée, deux fenêtres dont la clôture de l'une ≠ l'ouverture de
   l'autre. Unitaires : configuration (AC-1 : paire incomplète, gabarit, fichier de clé absent, chiffré, trop court ;
   patron de `smtp_password_masked_in_debug`, `config.rs:3484`), `sign_challenge` (AC-2), `windows` à 469 jours (socle).
   **Test au marqueur** : `TOKEN-MARQUEUR-…`, le contenu de la clé, `x-2fa-approval` et la signature n'apparaissent dans
   aucun journal capturé, erreur ni réponse.

## Tâches

- [ ] T1 — Configuration par le lecteur du socle, clé SCA lue au démarrage (AC-1).
- [ ] T2 — `wise/sca.rs` : signature et clé de test (AC-2).
- [ ] T3 — Client : profil, soldes, relevé, fenêtres, rejeu signé, erreurs (AC-3, AC-4).
- [ ] T4 — Serveur simulé et tests (AC-5).
- [ ] T5 — Après la recette 2 : branche CSV ou JSON (AC-0).

## Stratégie de tests

Gate ciblé : tests de `kesh-api` (`config`, `wise`) et du binaire de test du serveur simulé ; **gate complet** au dernier
commit. Aucune migration : P5 à P8 sans objet. `Cargo.lock` : seules les lignes de dépendances de `kesh-api` changent (aucun
paquet ajouté) — se contrôle par `git diff Cargo.lock`.

## Modules touchés (2)

`kesh-api` (configuration, client, signature), `kesh-import` (JSON → `WiseRow`, **si** la recette 2 l'exige).

## Fichiers communs

`crates/kesh-api/src/config.rs` et `provider_env` du socle (38-12a), `crates/kesh-api/Cargo.toml`, `Cargo.lock` (se régénère
après fusion) : à relire à la reprise de `main`.

## Note de version

*Ajouté* — Configuration du téléchargement des relevés Wise : jeton en lecture seule et clé privée de signature SCA, posés
dans l'environnement du serveur (`KESH_WISE_<n>_…`), jamais en base ni dans la sauvegarde.

## Notes de développement

- Doc publique de Wise consultée le 2026-11-20 : relevé de solde (`docs.wise.com/api-reference/balance-statement`, **469
  jours**, `type` `COMPACT` ou `FLAT`, `currency` obligatoire, formats JSON, CSV, PDF, XLSX, CAMT.053, MT940, QIF, SCA
  exigée hors des États-Unis, de l'Australie, de la Nouvelle-Zélande, de Singapour, du Canada et de la Malaisie « une fois tous
  les 90 jours »), jetons personnels. Les en-têtes `x-2fa-approval` et `X-Signature` et l'algorithme sont **constatés pour le
  défi** (recette 1) et **de mémoire pour la réponse** (non relus).
- **Non confirmé** : acceptation du rejeu (recette 2), colonnes du CSV de l'API, durée de vie du jeton de défi, forme d'un 429,
  chemin de dépôt de la clé publique dans l'interface de Wise.
- Anciennes 39-1 et 39-3 (`superseded`) : connexion, coffre et relevé JSON remplacés par la variable d'environnement et le
  cœur de l'import de la phase 1. **L'ancienne 39-2 (authentification forte) est reprise ici** (AC-1, AC-2), avec la clé
  privée en **fichier désigné par une variable d'environnement**, non au coffre.

## Change Log

- 2026-11-20 — fiche rédigée (phase 2), non validée.
- 2026-11-20 — validation P1 (Claude Sonnet 5.5, contexte frais) : voir le Change Log de 39-12a2 (commun aux deux fiches) ;
  la recette 1 impose la signature SCA : fiche scindée, AC-0 recentré sur la recette 2, AC-1 et AC-2 neufs, socle S1 à S9.
