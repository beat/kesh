# Story 39-11d2 : la revue et l'acceptation Wise à l'écran

Status: drafted

*Fiche rédigée le 2026-11-22, branche `story/39-11d2`, partie de `story/39-11d` (`440ae42c`, en revue). Epic 39, phase 1
KISS. **Reprend l'AC-3 de la 39-11d**, reporté par son amendement A3 (la revue Wise, Story 39-11c, n'était pas dans la
branche). **Dépend de** : 39-11c (`done`, fusionnée dans `epic/39-wise` en `ab38871d`), 39-11d (page Wise sur la page
commune `ProviderReviewPage.svelte` de la 40-7d). C'est le seul écran où se rapprochent les transactions Wise importées :
sans lui, elles restent en attente, et le responsable de l'epic ne peut pas passer ses comptes 2025.*

## Story

En tant que **Comptable**, je veux, sur la page Wise, voir ce que Kesh propose pour chaque transaction importée — frais,
conversion, rattachement à l'autre côté, règle, paiement par carte groupé avec ses lignes —, comprendre en une phrase
pourquoi une ligne est « à examiner » ou en écart, et accepter les propositions choisies d'un geste, afin de tenir mes
comptes Wise sans passer par l'API.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-11d2-1` | La revue Wise montre les lignes d'un paiement par carte groupé (proposition ou « à examiner »). | AC-1, AC-2 |
| `FR-39-11d2-2` | Page Wise : exercice, propositions à cocher par genre, « ce qui sera fait » en clair. | AC-2 |
| `FR-39-11d2-3` | Chaque motif « à examiner » et chaque écart de la revue Wise s'explique à l'écran en une phrase, quatre locales. | AC-3 |
| `FR-39-11d2-4` | Acceptation par lot (*Pattern batch*, `{ accepted, failed }`), refus lisibles, revue relue. | AC-4 |
| `NFR-39-11d2-1` | Manuel utilisateur (`.tex` seul) et `docs/api-external.md` ; limites du rapprochement au manuel. | AC-5 |
| `NFR-39-11d2-2` | E2E de l'acceptation ; captures de chaque état touché (*Recette visuelle*). | AC-6 |

## Avant le code — base de la branche

`git merge epic/39-wise` dans `story/39-11d2` (apporte la 39-11c :
`routes/wise.rs`, `kesh-reconciliation/src/wise.rs`, `wise_reconciliation_e2e.rs`). Conflit attendu sur la fiche
`39-11d-ecrans-manuel-e2e-wise.md` (la version d'`epic/39-wise` porte la section « Limites du rapprochement à reprendre
au manuel » et l'entrée de Change Log de la P3 de la 39-11c ; celle de `story/39-11d` l'amendement A1–A5 et le Dev Agent
Record) : **garder les deux côtés** ; `sprint-status.yaml` : garder les deux côtés, chaque story à son statut le plus
avancé ; compteurs (`sitesTotal`, gardes i18n, registres) recomptés depuis la source. Relire au code, et citer au Change
Log, la ligne de chaque nom repris ci-dessous (`build_wise_review`, `card_group_blocker`, `ReviewProposal`,
`PayoutProposal` de `stripe.rs`). Si la 39-11d a changé entre-temps par sa revue P1, la reprendre aussi (`git merge
story/39-11d`).

## Critères d'acceptation

1. **AC-1 — Les lignes d'un paiement groupé dans la revue** (seul changement serveur, **additif**, `kesh-api` seulement).
   La revue ne montre aujourd'hui d'un `card_group` que sa ligne principale (`transaction`) et des identifiants
   (`bankTransactionIds`, `details.bankTransactionIds`) : l'écran ne peut pas en montrer les lignes. Dans
   `routes/wise.rs`, sur le patron de la proposition `payout` de Stripe (type de **revue** sérialisé seul,
   `PayoutProposal`/`StripeReviewProposal`, `kesh-reconciliation/src/stripe.rs`) : la proposition `card_group` de la
   revue porte en plus `lines` (le `TransactionSummary` de chaque ligne, dans l'ordre de `bankTransactionIds`, principale
   en tête) et `merchantAmount` (montant du marchand en CHF, deux décimales, chaîne) ; les quatre motifs `card_group_*`
   portent `details.lines` (mêmes résumés). Le **corps de l'acceptation ne change pas** (`WiseProposal`, champs
   d'affichage ignorés, déjà admis par `POST /wise/accept`) ; aucune migration ; `ReviewProposal` commune non touchée.
   Test HTTP ajouté à `wise_reconciliation_e2e.rs` (paiement groupé de `monter_partielle` : `lines` de trois résumés, la
   ligne CHF en tête, `merchantAmount` ; un `card_group_statements_missing` porte `details.lines`) ; un corps
   d'acceptation qui reprend `lines` et `merchantAmount` est accepté.

2. **AC-2 — La page Wise avec sa revue** (`frontend/src/routes/(app)/reconciliation/wise/+page.svelte`, page commune
   **étendue, non recopiée**). `wise.api.ts` gagne `getWiseReview`, `acceptWiseProposals` (sur `getProviderReview`,
   `acceptProviderProposals`), passés à `api` : la page affiche l'exercice, les propositions, les écarts et « à
   examiner » (la page commune le fait dès que `getReview` est là). `wise.types.ts` : `WiseProposal` (`fee`,
   `conversion`, `link_entry`, `rule`, `card_group` avec `lines` et `merchantAmount`), `WiseAcceptProposal` (champs du
   corps seuls). `wise-review.ts`, sur le patron de `stripe-review.ts` : clé (`<type>-<transaction>`, la débitée d'une
   conversion, la principale d'un groupe), corps d'acceptation, `selectedInDateOrder`. Genre et « ce qui sera fait »
   (`wise-labels.ts`) :
   - `fee` : « Frais » — « Écriture au compte de frais <n° — nom> » ;
   - `conversion` : « Conversion » — « Au cours de Wise : 100.00 EUR → 109.00 USD » (montants de `transaction` et
     `otherTransaction`), suivi de « une écriture » si un côté est en CHF, sinon « deux écritures par le compte de
     conversion » ;
   - `link_entry` : « Rattachement » — « Rattachée à l'écriture n° <n> déjà passée, sans nouvelle écriture » (`entry`) ;
   - `rule` : « Règle » — « Règle <libellé> → <n° — nom> » ;
   - `card_group` : « Paiement par carte » — « Une écriture : <merchantAmount> CHF au compte <n° — nom> (règle
     <libellé>), chaque solde crédité de sa part, la différence de change sur son compte » ; ses **lignes** affichées
     sous la proposition, une par ligne (date, montant et devise, identifiant Wise, « frais » pour une `FEE-CARD-…`).
   Libellés des règles et des comptes lus au chargement (`onLoad`, best-effort) ; la lecture paginée des règles de la page
   PayPal (`allRules`) **sort dans un module partagé** (`rules.api.ts` ou `lib/features/provider/`) et sert aux deux pages
   (DRY). Page commune : deux propriétés facultatives neuves, `proposalLines(p)` et `findingLines(f)` (rendent des
   `ProviderReviewTransaction[]`), qui affichent les lignes en sous-lignes sous la proposition ou la ligne « à examiner »
   — chaque sous-ligne d'un « à examiner » avec son propre bouton « Rattacher à une écriture » (même modale, sur cette
   transaction) ; sans elles, PayPal et Stripe rendent ce qu'ils rendent (tests existants verts sans changement
   d'assertion). Rien n'est coché d'office ; « Tout cocher » ne coche que des propositions. Le commentaire d'en-tête de la
   page Wise et celui de `ProviderReviewPage` (« jusqu'à la revue Wise ») sont récrits.

3. **AC-3 — Chaque motif en une phrase** (`wise-labels.ts`, clés `wise-` ; communs par `providerToReviewLabel`). Listes
   **fermées**, exportées (`WISE_GAP_CODES`, `WISE_TO_REVIEW_CODES`, `WISE_CARD_GROUP_UNSUPPORTED_REASONS`) et **comparées à
   la source Rust** par un test Vitest (constantes `GAP_*`, `REVIEW_*` de `kesh-reconciliation/src/wise.rs` et celles de la
   revue commune que Wise sert ; raisons de `judge_card_group`), sur le patron de `STRIPE_TO_REVIEW_CODES`. Un code
   inconnu s'affiche brut, jamais une ligne vide. Phrases (à reprendre telles quelles ou mieux dites, une phrase chacune) :
   - `fee_account_missing` : la phrase commune existante ;
   - `no_rule` : « Aucune règle ne reconnaît cette ligne : créez une règle d'affectation (filtre de type `CARD`,
     `TRANSFER`…), ou passez l'écriture à la main puis rattachez la ligne. » ;
   - `ambiguous_link` : « Plusieurs écritures déjà passées peuvent être l'autre côté de ce virement (n° …) : rattachez la
     ligne à la bonne. » (numéros tirés de `entries`) ;
   - `stripe_payout_unmatched` : « Versement Stripe dont l'écriture de versement n'est pas encore passée : importez le
     rapport Stripe et acceptez le versement ; aucune règle ne s'y applique. » (lien vers la page Stripe) ;
   - `card_group_no_rule` : « Paiement par carte de { merchantAmount } CHF : aucune règle ne lui donne un compte en CHF ;
     créez-en une. » ;
   - `card_group_unsupported` : « Les lignes de ce paiement par carte n'ont pas la forme attendue (<raison>) : traitez
     chaque ligne à la main. », la raison traduite par sa liste fermée (`not_a_debit`, `merchant_scale`,
     `exchange_missing`, `merchant_not_chf`, `exchange_currency_mismatch`, `exchange_amount_missing`,
     `card_line_missing`) ;
   - `card_group_statements_missing` : « Une ligne de ce paiement peut manquer : le relevé du solde <devises> ne couvre
     pas cette date ; importez-le, ou traitez les lignes à la main. » (devises tirées de `details.missingBankAccountIds`
     par les comptes bancaires chargés, l'identifiant à défaut) ;
   - `card_group_partly_reconciled` : « Une part de ce paiement est déjà rapprochée (écriture n° …) : annulez ce
     rapprochement puis acceptez le paiement entier, ou finissez à la main. » (liens vers `details.journalEntryIds`) ;
   - écart `conversion_half_missing` : « L'autre moitié de cette conversion manque : importez le relevé de l'autre devise,
     ou elle est déjà rapprochée. » ;
   - écart `fee_without_origin` : « Frais sans la ligne dont il vient sur ce solde : importez le relevé complet de la
     période. » ;
   - écart `balance_gap` : la phrase de l'avertissement `gap` de l'import (`wiseWarningLabel`, **réutilisée**), avec la
     date et la devise de `details` (l'écart n'a pas de transaction : la colonne date lit `details.date`).
   Textes « proposer », jamais « comptabiliser automatiquement ». Clés aux quatre locales, chacune en toutes lettres ;
   `sitesTotal` de `i18n-keys.test.ts` et garde des libellés en dur **recomptés** depuis la source.

4. **AC-4 — Acceptation par lot.** « Accepter la sélection » envoie les corps dans l'ordre des dates, par paquets de 500
   (`chunkProposals`), compte figé ; réponse `{ accepted, failed }` (*Pattern batch*, HTTP 200) : compte rendu « n
   acceptée(s), m refusée(s) », chaque refus par sa transaction et sa phrase (`wiseFailedLabel` : délégation à
   `providerFailedLabel`, puis `failedProposalLabel`) ; `PROVIDER_PROPOSAL_STALE` dont `details.reason` est
   `card_group_statements_missing` ou `card_group_partly_reconciled` dit la phrase du motif (AC-3) ; les autres raisons
   (`card_group_changed`, `rule_no_longer_matches`, `conversion_pair_changed`, …) la phrase commune « Rechargez la
   revue » ; `RATE_UNAVAILABLE`, `FX_ACCOUNT_INVALID`, `COUNTERPARTY_CURRENCY_MISMATCH`, `PERIOD_LOCKED` par les libellés
   existants. Liste des codes de `POST /wise/accept` testée contre leur source, comme `PROVIDER_COMMON_FAILED_CODES`. Un
   `400` global (corps refusé) s'affiche par `errorMessageOf`. La revue se relit après l'acceptation. Vitest : corps d'un
   groupe (identifiants, règle, contrepartie ; sans `lines`), ordre, paquets, refus d'un groupe périmé.

5. **AC-5 — Manuel et API.** `docs/manual/fr/user-manual.tex` (`.tex` seul, PDF à la release ; DE, IT, EN « à traduire »
   selon la règle du dépôt), sous-section `sec:comptes-wise` : le paragraphe « Le rapprochement » (« n'est pas encore dans
   cette version ») et la phrase du contrôle de clôture (« la page Wise ne fait qu'importer ») **récrits** ; ce que Kesh
   propose (frais, conversions au cours de Wise — une écriture si un côté est en francs, deux par le compte de conversion
   sinon —, rattachement à l'autre côté déjà passé, règles, paiement par carte converti en une écriture avec ses lignes) ;
   chaque motif « à examiner » et chaque écart en termes d'utilisateur ; l'acceptation par lot, rien d'office ; une ligne
   non proposée se traite à la main (écriture, puis *Rattacher à une écriture*) ; **les trois limites de la section
   « Limites du rapprochement à reprendre au manuel » de la fiche 39-11d** (continuité des relevés, solde en devise peu
   utilisé, part arrivée après coup), et l'annulation d'une ligne d'un paiement groupé qui remet **toutes** ses lignes en
   attente (amendement A-2 de la 39-11c). Gardes de `textes_coherents.rs` connues avant d'écrire (G2, G4-bis/G4-ter :
   aucun numéro de compte en dur, G18, G25). Contrôle du PDF local, aplati (`pdftotext … | tr '\n' ' ' | tr -s ' '`), non
   commité. `docs/api-external.md` : `lines` et `merchantAmount` du `card_group`, `details.lines` des motifs
   `card_group_*` ; phrase « La revue et l'acceptation des transactions Wise ne sont pas encore servies » (§ *L'écran
   Wise*) remplacée par ce que l'écran appelle.

6. **AC-6 — E2E** `frontend/tests/e2e/wise-import.spec.ts` étendue (même société, même parcours, reseed en fin) : cours
   mensuels de 2020 posés par l'API (patron de `paypal-import.spec.ts`), compte de conversion et comptes de change de la
   société, règles par l'API (`counterparty_contains` filtrée `CARD`). Après l'import du zip : exercice 2020, revue à
   l'écran — une proposition de frais, une conversion, une règle ; « à examiner » `no_rule` et `stripe_payout_unmatched`
   avec leur phrase. Un **paiement par carte groupé** sur un **second compte Wise** (libellé distinct, soldes CHF et USD,
   relevés CSV faits par la spec sur le patron de `releves_partielle` de `wise_reconciliation_e2e.rs`, identifiants et
   noms fictifs) : proposition `card_group` avec ses trois lignes affichées. Acceptation par l'écran d'un frais, d'une
   conversion et du groupe : compte rendu « 3 acceptée(s), 0 refusée(s) », propositions disparues à la relecture ; le
   groupe en **une** écriture (vérifiée par l'API : trois transactions rapprochées sur le même `journalEntryId`). Captures
   (thème clair, `KESH_CAPTURES_DIR`, hors dépôt) : revue, groupe et ses lignes, « à examiner », écarts s'il y en a,
   compte rendu de l'acceptation ; jointes au Dev Agent Record et au prompt de revue.

## Tâches

- [ ] T0 — Fusion d'`epic/39-wise`, conflits gardés des deux côtés, noms relus au code (*Avant le code*).
- [ ] T1 — `lines`, `merchantAmount`, `details.lines` ; test HTTP (AC-1).
- [ ] T2 — API, types, `wise-review.ts`, propriétés de la page commune, règles partagées (AC-2).
- [ ] T3 — Libellés des genres, motifs, écarts et refus ; listes contre la source ; i18n (AC-3, AC-4).
- [ ] T4 — Manuel, `docs/api-external.md` (AC-5).
- [ ] T5 — E2E, captures (AC-6).

## Gates

Règle des stories de l'epic : `nextest` complet (changement de `kesh-api`), frontend complet (`check`,
`lint-i18n-ownership`, Vitest, `build`), E2E de la seule spec `wise-import.spec.ts` au dernier commit de code ;
`gate-ci.sh` et E2E complet à la fin de l'epic. Le Dev Agent Record déclare exactement ce qui a tourné.

## Hors périmètre

- La vue d'annulation d'une transaction (`GET /reconciliation/transactions/{id}`, écran de la réconciliation) ne dit pas
  que les autres lignes d'un paiement groupé reviennent en attente (relevé par la 39-11c, « l'écran le dira avec
  39-11d ») : le manuel le dit (AC-5) ; l'écran d'annulation reste à faire, **à ouvrir en issue**.
- Choisir à l'écran une écriture parmi celles d'un `ambiguous_link` : la modale « Rattacher à une écriture » existante y
  suffit.

## Modules touchés (4)

`kesh-api` (`routes/wise.rs`, tests), `frontend` (page Wise, page commune, features `wise`, `provider`, règles, E2E),
`kesh-i18n` (libellés), `docs` (manuel `.tex`, `docs/api-external.md`).

## Note de version

*Ajouté* — Rapprochement Wise à l'écran : la page Wise propose, pour chaque transaction importée, les frais, les
conversions au cours de Wise, le rattachement à l'autre côté déjà passé, les règles et le paiement par carte converti
en une écriture, avec ses lignes ; chaque ligne « à examiner » ou en écart dit pourquoi ; les propositions choisies
s'acceptent d'un geste.

## Change Log

- 2026-11-22 — fiche rédigée (AC-3 de la 39-11d, amendement A3), non validée.
