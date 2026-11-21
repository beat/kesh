# Story 39-12a4b : paiements par carte partiellement convertis dans le téléchargement Wise — cours, lignes déjà dans Kesh, compte rendu « à vérifier », refus

Status: drafted

*Fiche rédigée le 2026-11-21 (amendements du 21.11 de 39-12a2 et 39-12a3 ; suivi de la décision (a) du 21.11 du
responsable de l'epic, journal des décisions de signature des epics du 21.11, sous `kesh-gate-logs/`) ; amendée le même
jour à la validation P1. Branche `epic/39-wise`. Epic 39, **phase 2**. **Dépend de** : 39-12a4a1 (continuité) et
39-12a4a2 (estimation au cours), 39-12a3 (téléchargement), 39-12a3b (doublons `CARD-`/`FEE-CARD-`), 39-11a2 (lecture
`existing_transactions_by_ids`, étendue ici), 39-12a1 (client), 39-11c (écriture groupée, amendement A-1). **Suivie
par** : 39-12b (écran et manuels : la liste « à vérifier » et les codes neufs y sont décrits, amendement de 39-12b du
21.11).*

## Story

En tant que **Comptable**, je veux que le téléchargement Wise importe les paiements par carte payés avec plusieurs
soldes quand Kesh en établit exactement la répartition, me propose les autres à vérifier, avec ce qu'il en sait, et me
dise clairement quand il faut passer par le relevé CSV.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-12a4b-1` | Le téléchargement lit le cours de Wise de chaque conversion par carte et les lignes déjà dans Kesh, puis appelle le moteur de 39-12a4a1 et 39-12a4a2. | AC-1 à AC-3 |
| `FR-39-12a4b-2` | Le compte rendu liste les partielles « à vérifier » et la somme exacte de leur groupe ; rien n'en est écrit. | AC-4 |
| `FR-39-12a4b-3` | Un trou de solde, ou une partielle que la continuité ne résout pas, refuse la tâche avec un code propre et un renvoi au CSV. | AC-5 |
| `NFR-39-12a4b-1` | Aucun secret ni appel réel en test ; aucune migration. | AC-7 |

## Critères d'acceptation

1. **AC-1 — Cours de Wise.** Le client de 39-12a1 gagne `rate(source, target, at)` : `GET
   /v1/rates?source=<X>&target=<Y>&time=<instant ISO-8601 UTC>` (doc publique de Wise ; ouvert au jeton personnel sans
   authentification forte, constaté le 2026-11-21) ; la réponse est une liste d'un élément `{ rate, source, target, time }`
   dont `source` et `target` doivent être ceux demandés ; `rate` se lit en `Decimal` depuis son texte exact (comme les
   montants, 39-12a1 revue P1 M1) et doit être **> 0** ; toute autre forme (liste vide ou de plusieurs éléments, paire
   différente, cours nul ou négatif) → `PROVIDER_RESPONSE_INVALID`, `details.reason = "rate"`. Reprises et pause commune
   du socle (429, `Retry-After`) ; le serveur simulé de 39-12a1 sert la route.
2. **AC-2 — Ordre de lecture** (39-12a3 AC-3, complété). Après les devis, une étape par **cours**, dans la liste que rend
   `rates_needed` de `kesh-import` (39-12a4a2 AC-1 : un cours par devise convertie de chaque partielle `COMPLETED`, un par
   totale par carte ; aucune règle recopiée ici), à l'**instant** de l'activité qu'elle désigne (`created_on`, ou
   `updated_on` quand `at_updated`) pris **tel que Wise le rend**, en UTC, avant la conversion à l'heure de Zurich. Le
   `total` des étapes vaut 4 + devis + cours, connu à la fin de l'étape « transferts ». Les cours se lisent sous
   `KESH_WISE_PARALLEL`, comme les devis (39-12a3 AC-5) ; résultat identique à 1 et à 3.
3. **AC-3 — Lignes déjà dans Kesh, appel du moteur.** Avant l'appel du moteur et avant toute écriture, la lecture
   `existing_transactions_by_ids` de 39-11a2 (réutilisée par 39-12a3b) gagne **deux champs** de
   `ExistingTransaction` — `import_id` (`bank_transactions.import_id`) et `booked` (statut autre qu'en attente) — et
   **aucune lecture neuve** n'est écrite ; elle rend, pour le compte bancaire de chaque devise liée, les transactions dont
   l'identifiant est lu (ou est `FEE-<identifiant lu>`) ; le sens se lit au signe du montant (39-11a2). Elles forment
   `known_lines` (39-12a4a1 AC-1) ; les points de contrôle portent déjà l'identifiant de leur relevé (raccord de
   39-12a4a1). La route passe `split: Some(…)` avec ces lignes et les cours. Lecture hors de la transaction d'écriture :
   une ligne importée entre-temps (CSV concurrent) est écartée sous les verrous par les doublons (39-12a3b) ; une
   partielle retenue à tort de ce fait n'écrit rien.
4. **AC-4 — Compte rendu.** `result` (39-12a3 AC-2) gagne `cardSplitsToVerify: [{ cardId, date, merchantAmount,
   merchantCurrency, parts: [{ currency, amount?, exact, cover? }], reason }]` (`reason` ∈ `split_group`,
   `rate_mismatch`, `no_calibration`, `part_already_booked`, `unsupported_shape`) et `splitGroups: [{ currency, from,
   to, sum, cardIds }]` (39-12a4a1 AC-5 ; listes toujours présentes, vides le cas échéant). Une partielle exacte s'importe
   comme toute ligne (une par solde) ; le rapprochement la propose en écriture groupée (39-11c, A-1). Une partielle « à
   vérifier » n'a **aucune** transaction importée : elle s'importe par le relevé CSV de sa période (`POST
   /api/v1/wise-imports`, 39-11b : ses lignes y sont neuves, puis le rapprochement les groupe) — un relevé saisi à la
   main est refusé sur un compte Wise (`409 BANK_ACCOUNT_IS_PROVIDER`) ; `part_already_booked` : annuler d'abord le
   rapprochement de la ligne en base, puis relancer. Décision (c) du 21.11 : tout cela se fait aussi par l'API seule, en
   plusieurs appels (état de la tâche, import CSV, rapprochement). La trace `wise_download.created` porte le nombre de
   partielles importées et à vérifier, jamais leurs montants. *(Si la décision ouverte de 39-12a4a2 retient la
   validation d'un geste, son écran et sa route viennent d'une story suivante, 39-12a4c.)*
5. **AC-5 — Refus et codes.** `WISE_API_BALANCE_GAP` (39-12a4a1) devient le code **`WISE_BALANCE_GAP`**, `details: {
   currency, from, to }` : entre ces dates, les soldes de cette devise ne se suivent pas ; le message dit les trois
   causes (39-12a4a1 AC-6) — un mouvement absent de l'API de Wise (p. ex. une conversion), un mouvement **en cours** déjà
   porté au solde (attendre sa fin et relancer), un ordre de prélèvement des soldes réglé autrement chez Wise — et la
   sortie : importer le relevé CSV de cette période. `WISE_API_SPLIT_UNRESOLVED` devient
   **`WISE_CARD_SPLIT_UNRESOLVED`**, `details: { currency, date, reason }` : `unanchored` — importer d'abord un relevé de
   cette devise qui se termine avant `date` (ou le CSV de la période) ; `period_boundary` — choisir une fin de période à
   aujourd'hui, ou importer le CSV. `WISE_API_MISSING_RATE` (39-12a4a2) → `PROVIDER_RESPONSE_INVALID`, `details.reason`
   = ce code. Rien n'est écrit (39-12a3 AC-3). Le code `WISE_CARD_PARTIAL_CONVERSION` de 39-12a3 (amendement A-1) est
   **retiré** de `kesh-api` (code, messages, test remplacé) : la route ne passe plus `split: None` ; la variante
   `PartialCardConversion` reste dans `kesh-import` pour cet appel (fonction publique, sans moteur). Messages aux quatre
   locales, `errors.rs`.
6. **AC-6 — Volume d'appels.** Un cours par devise convertie de chaque partielle et un par totale par carte de la
   lecture : quelques appels pour un mois (2026 : 16 paiements par carte convertis sur l'année). Le manuel le dit
   (39-12b, amendement du 21.11).
7. **AC-7 — Tests** (`crates/kesh-api/tests/wise_download_e2e.rs`, serveur simulé, base `#[sqlx::test]`, données
   **fictives**) : partielle exacte → deux transactions `CARD-x` (une par solde), segment de conversion `X→CHF <couverture>
   au cours <cours>` lu par le rapprochement, proposition `card_group` acceptée → une écriture, charge = montant du
   marchand ; partielle « à vérifier » → `cardSplitsToVerify`, `splitGroups` à somme exacte, aucune de ses transactions
   importée, soldes d'ouverture et de clôture exacts, puis import du CSV fictif de la période → ses lignes insérées et
   groupées ; relevé CSV en base qui connaît une partielle → exacte au téléchargement suivant, doublons ignorés
   (39-12a3b) ; fichier CSV d'un seul solde importé et sa ligne rapprochée, puis téléchargement → `part_already_booked`,
   rien d'écrit pour cette partielle ; point de contrôle d'un CSV arrêté en cours de journée → placé, partielle du
   lendemain exacte ; **deux téléchargements le même jour puis un troisième** → aucune erreur (39-12a4a1 AC-2) ; trou →
   `WISE_BALANCE_GAP`, rien d'écrit ; segment ouvert → `WISE_CARD_SPLIT_UNRESOLVED` (`unanchored`) ; fin de période dans
   un groupe → (`period_boundary`) ; cours absent, de mauvaise paire, à `rate ≤ 0`, liste vide → `PROVIDER_RESPONSE_INVALID`
   (`rate`) ; 429 sur un cours puis reprise ; parallélisme 3 = 1 ; étapes au compte ; jeton absent des journaux. Le
   comportement de Wise pour un `time` très ancien n'est pas relevé : le serveur simulé rend la forme documentée.

## Tâches

- [ ] T1 — `rate` au client et au serveur simulé (AC-1).
- [ ] T2 — Étape des cours par `rates_needed`, total des étapes (AC-2) ; `import_id` et `booked` à la lecture de 39-11a2,
  `split: Some` (AC-3).
- [ ] T3 — Compte rendu, audit (AC-4) ; codes, messages, retrait de `WISE_CARD_PARTIAL_CONVERSION` (AC-5).
- [ ] T4 — Tests (AC-7).

## Stratégie de tests

Gate ciblé : `binary(wise_download_e2e)`, `binary(wise_client_e2e)`, les binaires de 39-11a2, 39-11b et 39-11c ; **gate
complet** au patch qui touche le dépôt de `kesh-db` (règle d'exception de `CLAUDE.md`) et au dernier commit. Aucune
migration.

## Modules touchés (3)

`kesh-api` (client Wise, tâche de téléchargement, `errors.rs`), `kesh-db` (deux champs de la lecture de 39-11a2),
`kesh-i18n`.

## Fichiers communs

`crates/kesh-api/src/wise/mod.rs` (39-12a1), `errors.rs`, catalogues i18n (leur bloc), dépôt des transactions bancaires
(`existing_transactions_by_ids`, 39-11a2 ; 39-12a3b) : à relire à la reprise de `main`.

## Note de version

*Ajouté* — Le téléchargement Wise importe un paiement par carte payé avec plusieurs soldes quand Kesh en établit
exactement la répartition, et le propose à vérifier sinon ; un trou dans les soldes renvoie au relevé CSV de la période.

## Notes de développement

- Aucun appel réel en test ni en CI. Le budget d'appels réels de la recette (39-12a4a2 AC-7) est nul : elle rejoue des
  réponses déjà sauvées.
- Le cours est lu à l'instant UTC de Wise : le convertir d'abord à l'heure de Zurich, puis le renvoyer, ferait lire un
  autre cours.
- Limite assumée (finding M12, voie CSV seule) : un relevé CSV d'**un seul** solde importé, et sa ligne `CARD-x`
  acceptée comme une dépense ordinaire avant l'arrivée du fichier de l'autre solde, ne se regroupe plus ; le manuel
  conseille d'importer ensemble les fichiers des trois soldes d'une même période (39-12b). La garde au rapprochement
  (ligne `CARD-x` sans segment dont un solde du même libellé porte un `CARD-x` converti → « à examiner ») relève de
  39-11c : à suivre par une issue.

## Change Log

- 2026-11-21 — **remédiation de la validation P1 des fiches 39-12a** (Claude Sonnet 5.5, contexte frais ; rapport
  `kesh-gate-logs/s39-12a-specs/validation-p1-rapport.md` ; ici 0 C, 0 H, 2 M, 3 L, plus des propagations). **M11** — plus
  de « relevé saisi » (refusé sur un compte Wise) : sortie par le CSV de la période ; **M12** — part déjà rapprochée
  (`booked`) → `part_already_booked`, test, limite de la voie CSV seule ; **M13** — aucune lecture neuve : deux champs
  (`import_id`, `booked`) de la lecture de 39-11a2 ; **H1** — `import_id` remplace `period_to` (placement des points de
  contrôle, 39-12a4a1) ; **M9** — `split: Some` ici, `WISE_CARD_PARTIAL_CONVERSION` retiré de `kesh-api` seulement ;
  **L9** — trois causes au message de `WISE_BALANCE_GAP` ; **L12** — liste des cours par `rates_needed` ; **L14** —
  `rate ≤ 0`, liste vide ; dépend de 39-12a4a1 et 39-12a4a2 (39-12a4a scindée). Statut `drafted`, validation P2 à
  lancer.
- 2026-11-21 — fiche rédigée (amendements du 21.11 de 39-12a2 et 39-12a3). Statut `drafted`, validation P1 à lancer.
