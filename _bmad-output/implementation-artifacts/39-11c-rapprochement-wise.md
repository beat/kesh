# Story 39-11c : rapprochement des lignes Wise — frais, conversions, virements internes, versements Stripe, règles, écarts

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/39-wise`, numéros de ligne relevés au code de `origin/main` (`fd04099f`).
Epic 39, **phase 1 KISS** (G5 à G10, § *Ce que la phase 1 attend de l'Epic 38*). **Dépend de** : 39-11b, **38-11c**
(rapprochement PayPal : propositions et rattachement en devise ; fiche non encore écrite au 2026-11-20). **Suivie
par** : 39-11d.*

## Story

En tant que **Comptable**, je veux qu'après l'import des relevés Wise Kesh me **propose** l'écriture ou le
rattachement de chaque ligne — frais au compte de frais, conversions au cours de Wise, virements entre mes comptes,
versements Stripe rattachés au versement, achats et paiements par mes règles — et me liste ce qu'il ne sait pas
rapprocher, afin de tenir mon compte Wise sans saisie et sans rien compter deux fois.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-11c-1` | Une ligne de frais (`FEE-…`) se propose au compte de frais du compte Wise ; sans compte de frais, elle reste « à examiner ». | AC-2 |
| `FR-39-11c-2` | Une conversion CHF↔devise se propose en **une** écriture au CHF de Wise, ses deux transactions rattachées ; EUR↔USD en deux écritures par le compte de conversion, même CHF. | AC-3 |
| `FR-39-11c-3` | Un virement depuis ou vers un compte bancaire de la société se propose comme virement entre comptes (fonction existante). | AC-4 |
| `FR-39-11c-4` | Un versement Stripe se propose au rattachement à l'écriture du versement de l'Epic 40, jamais en écriture de vente. | AC-5 |
| `FR-39-11c-5` | Les autres lignes passent par les règles d'affectation, critères de 38-11b ; le reste « à examiner ». | AC-6 |
| `FR-39-11c-6` | Les écarts sont listés avec leur motif. | AC-7 |
| `NFR-39-11c-1` | Aucune écriture d'office ; acceptation par le lot existant (*Pattern batch*, `FailedProposal` par proposition). | AC-1 |
| `NFR-39-11c-2` | Aucun code TVA ; aucune migration ; levées en devise de 38-11c étendues, non recopiées. | AC-1, AC-8 |

## Critères d'acceptation

1. **AC-1 — Où vivent les propositions.** Les propositions Wise entrent dans la liste et l'acceptation en lot que
   38-11c étend pour PayPal (`GET /api/v1/reconciliation/proposals`, `POST /api/v1/reconciliation/accept`,
   `crates/kesh-api/src/lib.rs:983-988`), comme genres neufs **calculés** (aucune colonne) : `provider_fee`,
   `provider_conversion`, `own_transfer`, `stripe_payout`, et la proposition par règle existante. Chaque acceptation
   suit le *Pattern batch* (`FailedProposal { bank_transaction_id, error_code, details }`, HTTP 200). Prérequis
   vérifiés au code : la réconciliation n'écrit qu'en CHF aujourd'hui — règle refusée hors CHF
   (`crates/kesh-api/src/routes/reconciliation.rs:4385-4396`), écriture manuelle sans devise
   (`crates/kesh-reconciliation/src/manual.rs:116-118`), rattachement à une écriture réservé au CHF
   (`crates/kesh-db/src/repositories/reconciliation_link.rs:535-537`). 38-11c les lève pour les comptes prestataires
   (ligne du compte en devise : `foreign` au montant de la transaction, CHF au cours de l'Epic 37, sauf AC-3) ;
   39-11c **vérifie** que la levée vaut pour `provider = 'wise'` et, si 38-11c l'a bornée à `'paypal'`, remplace la
   condition par « compte bancaire prestataire » (G10), tests PayPal inchangés.
2. **AC-2 — Frais.** Transaction dont `provider_type` commence par `FEE-` (39-11b) : proposition `provider_fee` =
   débit compte de frais du compte bancaire (`fee_account_id`, 38-11b), crédit compte du plan du compte Wise, montant
   = `-amount` en devise, libellé « Frais Wise · <détails> ». Compte de frais absent → « à examiner », motif
   `fee_account_missing`. La ligne d'origine (`fee_of`) n'est pas requise pour proposer ; son absence au même compte
   est un écart (`fee_without_origin`, AC-7).
3. **AC-3 — Conversions.** Paire = deux transactions **pending** de comptes bancaires **du même libellé Wise**, même
   `transaction_id`, préfixe `BALANCE` ou `CONVERSION_ORDER`, l'une débit, l'autre crédit, devises différentes.
   - **CHF↔devise** : proposition `provider_conversion` = **une** écriture du journal *Banque* : ligne CHF au montant
     de la transaction CHF, ligne du compte en devise au montant en devise (`foreign`), **le CHF de cette ligne étant
     le montant CHF de Wise** (cours de Wise, jamais celui de l'AFC) ; écriture équilibrée en CHF sans différence de
     change. À l'acceptation, **les deux** transactions se rattachent à cette écriture (une de chaque compte). Le
     prédicat de « l'autre côté » (`other_side_predicate`, `reconciliation_link.rs:698-709`, `bt.amount =
     -(movement)`) ne reconnaît que des montants opposés de même devise : il est **étendu** à la conversion — l'autre
     côté est une transaction d'un autre compte bancaire dont la ligne de l'écriture sur son grand livre vaut, **en
     CHF**, l'opposé de celle de `t` — en un seul endroit, utilisé par l'annulation (`reconciliation_cancel`) comme
     aujourd'hui ; annuler un côté ne retire que son lien (34-11b). Changement de `kesh-db` : gate complet en boucle.
   - **EUR↔USD** : deux écritures par le compte de conversion de la société (`companies.fx_transit_account_id`,
     `crates/kesh-db/src/entities/company.rs:205`), par `create_conversion`
     (`crates/kesh-db/src/repositories/journal_entry_conversions.rs:226`) — même CHF des deux côtés, au cours de la
     décision Q-39-12 (défaut : AFC du mois, devise payée) ; chaque transaction se rattache à son écriture. Compte de
     conversion non désigné → « à examiner », motif `fx_transit_missing`.
   - Moitié manquante (l'autre devise pas encore importée) → « à examiner », écart `conversion_half_missing`.
4. **AC-4 — Virements entre ses comptes** (G7). Reconnaissance : `counterparty_iban` normalisé (même normalisation
   que `crates/kesh-reconciliation/src/rules.rs:112`) égal à l'IBAN d'un **autre** compte bancaire non archivé de la
   société, ou `counterparty_name` égal à `companies.name` après `trim`, sans casse. Proposition `own_transfer` :
   - si l'autre compte porte une transaction déjà rapprochée d'une écriture qui peut recevoir l'autre côté (même
     devise, montant opposé, date à ± 5 jours) → rattachement à cette écriture (`link_entry_in_tx`,
     `reconciliation_link.rs:1156`, route `POST …/link-entry`, `lib.rs:1046`) ;
   - sinon → écriture de virement débit/crédit des deux comptes du plan, la transaction Wise rattachée ; l'autre côté
     s'y rattachera à son import (34-11b). Compte destinataire inconnu (nom de la société sans IBAN) → l'écriture
     n'est pas proposée, la ligne reste « à examiner », motif `own_transfer_target_unknown`.
5. **AC-5 — Versements Stripe** (G8). Crédit dont `counterparty_name` contient « stripe » (sans casse) : proposition
   `stripe_payout` = rattachement (`link-entry`) à une écriture **existante** qui porte le versement de l'Epic 40 :
   ligne au débit du compte du plan de ce compte Wise, même devise, même montant en devise, date à ± 5 jours, non
   encore rapprochée de ce compte (requête de candidats de `find_entry_candidates`, `reconciliation_link.rs:870`,
   bornée par ces conditions). Plusieurs candidats → aucun choix d'office : proposés en liste. Aucun candidat, ou Epic
   40 absente → « à examiner », motif `stripe_payout_unmatched` ; **jamais** d'écriture de vente ni de règle sur ces
   lignes (une règle qui les viserait est écartée pour elles, écrit au manuel par 39-11d).
6. **AC-6 — Règles et reste.** Les autres transactions passent par `rule_matches` (`rules.rs:46`) étendu par 38-11b
   (`provider_type`, `direction`) ; la proposition par règle est celle d'aujourd'hui (`accept_one_rule`,
   `reconciliation.rs:4191`), en devise par la levée de 38-11c, **sans code TVA**. Aucune règle → « à examiner »
   (motif `no_rule`).
7. **AC-7 — Liste des écarts.** Lecture `GET` des écarts d'un compte Wise (libellé) ou du rapport de 38-11c s'il en
   expose un (G10) : `conversion_half_missing`, `fee_without_origin`, `own_transfer_target_unknown`,
   `stripe_payout_unmatched`, `fee_account_missing`, `fx_transit_missing`, `no_rule`, et les avertissements `gap` des
   imports (39-11b AC-5). Chaque écart : transaction, date, montant, devise, motif. Calculé, rien de stocké.
8. **AC-8 — Aucune migration, i18n.** Genres et motifs calculés ; libellés des motifs aux quatre locales.

## Tâches

- [ ] T0 — Lire 38-11c telle que développée ; noter chaque levée réutilisée ou généralisée (G10).
- [ ] T1 — Frais et règles en devise pour `wise` (AC-1, AC-2, AC-6).
- [ ] T2 — Conversions : paire, écriture au cours de Wise, rattachement des deux côtés, prédicat étendu ; EUR↔USD
  par `create_conversion` (AC-3).
- [ ] T3 — Virements internes (AC-4), versements Stripe (AC-5).
- [ ] T4 — Écarts, i18n (AC-7, AC-8).

## Stratégie de tests

- **Unitaires** (`kesh-reconciliation`, pur) : reconnaissance de chaque genre sur des transactions construites ;
  paire de conversion (même ID, devises, sens) ; virement interne par IBAN et par nom ; Stripe ; préséance (une ligne
  de frais n'est jamais une règle, un versement Stripe jamais une règle).
- **HTTP** (`crates/kesh-api/tests/wise_reconciliation_e2e.rs`) sur les fixtures de 39-11a importées par 39-11b :
  frais acceptés ; conversion CHF→EUR acceptée → une écriture, CHF = montant de Wise, deux transactions rapprochées ;
  annulation d'un côté → seul son lien retiré ; EUR→USD → deux écritures, même CHF ; virement vers l'IBAN fictif de
  la société après import d'un relevé CAMT de ce compte → rattaché ; versement Stripe avec une écriture fictive de
  versement → rattaché, sans → écart ; paiement par règle `counterparty_contains` ; écarts listés. Somme des
  écritures Wise par compte = variation du solde Wise (test).
- Gate complet en boucle (prédicat de `kesh-db`).

## Modules touchés (4)

`kesh-reconciliation` (genres, reconnaissance), `kesh-api` (propositions, acceptation, écarts), `kesh-db`
(prédicat de l'autre côté, candidats Stripe), `kesh-i18n`.

## Note de version

*Ajouté* — Les lignes d'un relevé Wise se rapprochent sur proposition : frais au compte de frais, conversions en une
écriture au cours de Wise, virements entre vos comptes, versements Stripe rattachés au versement, le reste par vos
règles ; ce qui ne se rapproche pas est listé avec son motif.

## Notes de développement

- Ancienne 39-4 (classement des types Wise du relevé JSON, `superseded`) : la reconnaissance se fait ici par le
  préfixe de l'identifiant, langue de l'export indifférente (les descriptions du relevé sont traduites par Wise).
- ⚠️ **Risque principal** : l'extension du prédicat de « l'autre côté » (AC-3) touche un invariant de 34-11b qu'aucune
  contrainte ne défend ; si la passe de validation la juge trop risquée, repli : conversion CHF↔devise en **deux**
  écritures par le compte de conversion, comme EUR↔USD (contraire au « une écriture » de Beat, à lui soumettre).

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
