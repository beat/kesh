# Story 39-12a : téléchargement des relevés par l'API Wise (configuration, client, séquentiel, route)

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/39-wise`, numéros de ligne relevés au code de `origin/main` (`fd04099f`).
Epic 39, **phase 2** (H1 à H4 du fichier d'epic ; décisions de Beat du 2026-11-20, sur le modèle de la phase 2 de
l'Epic 38, `kesh-gate-logs/epic-38-kiss/phase2-decisions.md`). **Dépend de** : 39-11b (cœur de l'import), la
**recette de Beat** (AC-0). **Suivie par** : 39-12b.*

## Story

En tant que **Comptable**, je veux que Kesh télécharge lui-même, pour une période que je choisis, les relevés de
chaque solde de mon compte Wise, et les importe comme un relevé CSV, afin de ne plus exporter les fichiers à la main.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-12a-0` | La story ne se développe qu'après une recette réelle, hors de Kesh, qui obtient un relevé avec un jeton lecture seule. | AC-0 |
| `FR-39-12a-1` | Jetons Wise lus de variables d'environnement, une par compte Wise, rattachées à son libellé ; rien en base ni en sauvegarde. | AC-1 |
| `FR-39-12a-2` | Route et téléchargement d'une période : pour chaque compte bancaire du libellé, le relevé de sa devise, importé par le cœur de 39-11b (doublons par identifiant, CSV et API mélangés). | AC-2, AC-3 |
| `FR-39-12a-3` | Téléchargement séquentiel, pause entre requêtes, reprise sur `429`, progression. | AC-4 |
| `FR-39-12a-4` | Chaque erreur de Wise devient un code lisible, sans texte de Wise ni jeton. | AC-5 |
| `NFR-39-12a-1` | Aucun appel réseau en CI ni par un agent : transport factice. | AC-6 |
| `NFR-39-12a-2` | Jeton absent de tout journal, erreur, réponse, audit, export et sauvegarde (test au marqueur). | AC-1, AC-6 |
| `NFR-39-12a-3` | Aucune migration. | — |

## Critères d'acceptation

0. **AC-0 — Recette de Beat (bloquante).** Beat, hors de Kesh et hors du dépôt, avec un jeton personnel **lecture
   seule** de son profil d'entreprise : `GET /v1/profiles`, liste des soldes, `statement.csv` et `statement.json`
   (`type=FLAT`) d'un mois pour un solde. Constats écrits au fichier d'epic (Q-39-13, Q-39-14) : relevé servi ou
   refusé (la documentation publique le dit non servi au jeton personnel hors des États-Unis, du Canada, de
   l'Australie, de la Nouvelle-Zélande, de Singapour et de la Malaisie) ; défi SCA ou non ; colonnes du CSV de l'API
   comparées à celles de 39-11a ; limite de débit observée. **Refus** → la story passe `deferred`, l'epic s'arrête à
   la phase 1. **Défi SCA** → la story s'arrête et l'ancienne 39-2 est reprise (clé privée en **fichier désigné par
   une variable d'environnement**, `KESH_WISE_<n>_SCA_KEY_FILE`, jamais en base), à Beat avant reprise.
1. **AC-1 — Configuration**, sur le patron de `KESH_SMTP_PASSWORD` (`crates/kesh-api/src/config.rs:329-332`, champ
   privé, `Debug` masqué `:380`, accesseur `:399`) : `KESH_WISE_<n>_ACCOUNT` (libellé du compte Wise des réglages de
   39-11b) et `KESH_WISE_<n>_TOKEN`, `n` de 1 à 9 ; facultatif `KESH_WISE_<n>_PROFILE_ID` (sinon le seul profil
   d'entreprise du jeton ; plusieurs → refus au téléchargement, `WISE_PROFILE_AMBIGUOUS`). Paire incomplète, libellé
   en double, valeur vide ou de la forme `<…>` (`is_template_placeholder`, même fichier) → refus au **démarrage**,
   nommant la variable (jamais la valeur). **Un seul lecteur** pour les identifiants prestataires (`KESH_PAYPAL_<n>_…`
   de la phase 2 de l'Epic 38, `KESH_WISE_<n>_…`) : si la phase 2 de l'Epic 38 est écrite avant, Wise s'y ajoute ;
   sinon 39-12a pose ce lecteur générique et PayPal s'y branchera.
2. **AC-2 — Route** `POST /api/v1/wise-imports/download` `{ account, from, to }` (Comptable et plus, session ; clés
   d'API admises en `read-write` comme l'import CSV, à confirmer en validation), au registre des routes auditées.
   Refus, rien d'écrit : 400 (dates, `from > to`, plus de 469 jours) → 404 `PROVIDER_ACCOUNT_NOT_FOUND` → 409
   `WISE_TOKEN_NOT_CONFIGURED` (aucun jeton pour ce libellé). Réponse : le compte rendu de 39-11b AC-5, plus
   `downloaded: [{ currency, from, to }]`.
3. **AC-3 — Téléchargement.** Base constante `https://api.wise.com`, `Authorization: Bearer <jeton>`, client HTTP
   borné du dépôt (délais, taille de réponse) ; liste des soldes `STANDARD` du profil (chemin constaté à l'AC-0) ;
   pour **chaque compte bancaire du libellé** (devise), `GET
   /v1/profiles/{profileId}/balance-statements/{balanceId}/statement.csv` (ou `.json`) `?currency=…&intervalStart=…&
   intervalEnd=…&type=FLAT`, en UTC, bornes du jour à Zurich. Lecture par `parse_wise_csv` de 39-11a si l'AC-0 a
   constaté les mêmes colonnes ; sinon `kesh_import::wise::rows_from_statement_json` (pure, une fonction) rend les
   mêmes `WiseRow`, frais à part, solde courant contrôlé. Solde Wise sans compte bancaire → listé au compte rendu
   (`unlinkedCurrencies`), non téléchargé. Import par le cœur de 39-11b (empreinte calculée sur le contenu reçu ;
   doublons par identifiant).
4. **AC-4 — Séquentiel et 429.** Une requête à la fois, pause fixe entre deux (constante, p. ex. 500 ms) ; `429` →
   attente de `Retry-After` s'il est donné (borné), sinon 30 s, cinq essais au plus, puis `PROVIDER_RATE_LIMITED`
   sans rien importer de la devise en cours (les devises déjà importées restent, chacune tout ou rien) ; progression
   tenue en mémoire (« solde 2/3 ») et lisible par `GET /api/v1/wise-imports/download/status` (39-12b l'affiche).
   Un seul téléchargement à la fois par libellé (verrou nommé ; second → 409 `PROVIDER_DOWNLOAD_RUNNING`).
5. **AC-5 — Erreurs.** 401 → `PROVIDER_AUTH_FAILED` ; 403 sans défi → `PROVIDER_PERMISSION_DENIED` ; 403 avec défi
   SCA → `PROVIDER_SCA_REQUIRED` (sans tentative de signature, AC-0) ; 5xx, délai → `PROVIDER_UNAVAILABLE` ; réponse
   illisible → `WISE_STATEMENT_INVALID`. Statut HTTP de Kesh : 409 ou 502, jamais le texte de Wise. Codes partagés
   avec la phase 2 PayPal (préfixe `PROVIDER_`).
6. **AC-6 — Tests sans réseau.** Transport derrière un trait ; en mode test, réponses factices (fixtures fictives de
   39-11a servies comme `statement.csv`, un `statement.json` fictif, un `429` puis un succès, un 401, un 403 avec
   en-tête de défi) ; test au marqueur : un jeton `TOKEN-MARQUEUR-…` n'apparaît dans aucun journal, erreur, réponse,
   trace d'audit ni sauvegarde.

## Modules touchés (3)

`kesh-api` (configuration, client, route), `kesh-import` (conversion JSON → `WiseRow`, si l'AC-0 l'exige),
`kesh-i18n`.

## Note de version

*Ajouté* — Kesh télécharge les relevés Wise d'une période par l'API de Wise (jeton lecture seule en variable
d'environnement, posé par l'administrateur) et les importe comme un relevé CSV, sans doublon.

## Notes de développement

- Documentation publique consultée le 2026-11-20 : relevé de solde (`docs.wise.com/api-reference/balance-statement`,
  borne de 469 jours, formats, SCA hors de six pays « une fois tous les 90 jours »), jetons personnels
  (`docs.wise.com/guides/developer/auth-and-security/personal-api-token` : relevés non servis hors des mêmes six
  pays). Non confirmé : droits d'un jeton lecture seule, colonnes du CSV de l'API, chemin de la liste des soldes,
  limites de débit (aucune documentée).
- Anciennes 39-1 et 39-3 (`superseded`) : connexion, coffre et relevé JSON remplacés par la variable d'environnement
  et le cœur de l'import de la phase 1.

## Change Log

- 2026-11-20 — fiche rédigée (phase 2), non validée.
