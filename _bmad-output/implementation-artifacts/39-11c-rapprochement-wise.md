# Story 39-11c : rapprochement des lignes Wise — frais, conversions, autre côté déjà passé (virements, versements Stripe), règles, écarts

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/39-wise`, numéros de ligne relevés au code de `origin/main` (`fd04099f`),
revérifiés à la validation P1. Epic 39, **phase 1 KISS** (G5 à G10, § *Ce que la phase 1 attend de l'Epic 38*).
**Dépend de** : 39-11b, **38-11c** (rapprochement PayPal : rattachement en devise, routes de revue et d'acceptation,
écritures de frais, de règle et de conversion ; fiche `drafted` lue dans le worktree de l'Epic 38 le 2026-11-20).
**Suivie par** : 39-11d.*

## Story

En tant que **Comptable**, je veux qu'après l'import des relevés Wise Kesh me **propose** l'écriture ou le
rattachement de chaque ligne — frais au compte de frais, conversions au cours de Wise, rattachement à l'écriture déjà
passée de l'autre côté (virement depuis ma banque, versement Stripe), achats et paiements par mes règles — et me liste
ce qu'il ne sait pas rapprocher, afin de tenir mon compte Wise sans saisie et sans rien compter deux fois.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-11c-1` | Une ligne de frais (`FEE-…`) se propose au compte de frais du compte Wise ; sans compte de frais, elle reste « à examiner ». | AC-2 |
| `FR-39-11c-2` | Une conversion avec un côté en CHF se propose en **une** écriture (ligne CHF au montant de Wise, ligne en devise), ses deux transactions rattachées ; entre deux devises étrangères, en deux écritures par le compte de conversion, au même CHF. | AC-3 |
| `FR-39-11c-3` | Une ligne dont l'autre côté est déjà passé (virement entre ses comptes, versement Stripe de l'Epic 40) se propose au rattachement à cette écriture. | AC-4 |
| `FR-39-11c-4` | Un versement Stripe n'est jamais écrit ni affecté par une règle ; sans écriture de versement, il reste « à examiner ». | AC-4 |
| `FR-39-11c-5` | Les autres lignes passent par les règles d'affectation (filtres de 38-11b et 38-11c) ; le reste « à examiner ». | AC-5 |
| `FR-39-11c-6` | Les écarts sont listés avec leur motif. | AC-6 |
| `NFR-39-11c-1` | Aucune écriture d'office ; acceptation en lot (*Pattern batch*, `FailedProposal` par proposition). | AC-1, AC-7 |
| `NFR-39-11c-2` | Aucun code TVA ; aucune migration ; moteur de 38-11c réutilisé, non recopié. | AC-1, AC-8 |

## Critères d'acceptation

1. **AC-1 — Où vivent les propositions.** 38-11c **ne passe pas** par le rapprochement CHF existant (`get_proposals`
   `crates/kesh-api/src/routes/reconciliation.rs:1091`, saut hors CHF ; `accept_one_rule` `:4191`, refus hors CHF
   `:4385-4396` : 38-11c AC-1 les laisse CHF) : elle a ses routes `GET /api/v1/paypal/review` et `POST
   /api/v1/paypal/accept`, une revue pure et un moteur d'acceptation (verrous des comptes touchés par identifiant
   croissant, point de sauvegarde par proposition, rejugement sous verrou, écritures `fee`, `rule`, `conversion`,
   rattachement par `link_entry_in_tx`, `mark_reconciled_in_tx`, `fx_failed_proposal`). Wise suit **le même
   chemin** : `GET /api/v1/wise/review?account=<libellé>&fiscalYearId=<id>` (Comptable et plus, clés d'API admises en
   lecture, lecture seule) et `POST /api/v1/wise/accept` (Comptable et plus, clés `read-write`), **gestionnaires
   minces** sur le moteur de 38-11c **paramétré par le prestataire** (classement et revue propres à Wise, AC-2 à AC-6 ;
   écritures et acceptation communes) — si 38-11c est développée sous une forme propre à PayPal, cette story la
   généralise (G10), tests PayPal inchangés en assertions. Réponses de la forme de 38-11c (`proposals`, `gaps`,
   `toReview`, `counts` ; `{ accepted, failed }`), *Pattern batch* (`CLAUDE.md`, `FailedProposal {
   bank_transaction_id, error_code, details }`, `routes/reconciliation.rs:576`, HTTP 200 ; 400 global seulement pour
   un corps mal formé, vide, de plus de `MAX_IN_CLAUSE_IDS` propositions ou portant deux fois la même transaction) ;
   proposition périmée → `failed[]` sous le code de 38-11c (générique `PROVIDER_PROPOSAL_STALE` demandé, Q-39-10 ;
   sinon `WISE_PROPOSAL_STALE`). `POST /api/v1/wise/accept` à `LIB_ROUTES` (`audit_route_registry.rs:440`, clé `wise::post_accept`) en
   **`Traced, Rejouee`**, comme `paypal/accept` (38-11c AC-7) : le handler appelle l'enveloppe `retry_app_on_deadlock`
   (`crates/kesh-api/src/retry.rs:40`), **jamais `retry_with`** (`RETRY_WITH_AUTORISE = ["post_accept"]`, le test refuse
   tout autre appel), et un interblocage (`is_app_deadlock`) ou un échec de `ROLLBACK TO SAVEPOINT` **remonte** en
   `AppError` au lieu d'entrer dans `failed[]` — le moteur de 38-11c le fait une fois, la route Wise en hérite, et un test
   la prend en défaut sur le modèle de `rejeu_interblocage_e2e.rs` ; compte de la partition et phrase du manuel
   d'administration recomptés comme en 39-11b AC-3 ; trace par proposition
   acceptée (`wise.proposal_accepted`, ou la trace générique de 38-11c), sous l'acteur, inscrite à
   `audit_acteur_garde.rs`. La revue porte sur les transactions **pending** des comptes bancaires `provider = 'wise'`
   du libellé, datées dans l'exercice ; refus 404 compte Wise introuvable (39-11b), 404 `FISCAL_YEAR_NOT_FOUND`.
2. **AC-2 — Classement et frais** (fonction pure neuve `crates/kesh-reconciliation/src/wise.rs`, `classify(tx) ->
   WiseKind`, sur `provider_type` — le préfixe d'identifiant posé par 39-11b —, le signe et `counterparty_name`) :
   `Fee` (préfixe commençant par `FEE-`) ; `Conversion` (préfixe `BALANCE` ou `CONVERSION_ORDER`) ; `StripePayout`
   (montant > 0 et `counterparty_name` contenant « stripe », sans casse) ; `Other`. Chaque ligne n'a qu'un genre, dans
   cet ordre. **Frais** : proposition `fee` de 38-11c (débit compte de frais `fee_account_id`, crédit compte du plan
   du compte Wise, montant `-amount` en devise, ligne en devise au cours mensuel de la société, libellé « Frais Wise
   <devise> · <identifiant> ») ; compte de frais absent → « à examiner », motif `fee_account_missing`.
3. **AC-3 — Conversions** (décision (1) de l'orchestrateur, règle commune aux trois prestataires, Q-39-12 tranchée).
   Paire = deux transactions **pending** de comptes bancaires `wise` **du même libellé**, même `transaction_id`, genre
   `Conversion`, l'une débit, l'autre crédit, devises différentes (les relevés 2024 : chaque conversion est dans deux
   fichiers, frais à part dans la devise source). Proposition `conversion { debitTransactionId, creditTransactionId }`,
   exécutée par le **plan et le constructeur de conversion de 38-11c AC-8** — `plan_conversion` (`kesh-reconciliation/src/conversion.rs`) et
  `journal_entry_conversions::create_provider_conversion_in_tx` (le même code pour PayPal, Wise et Stripe ; relire leurs noms au T0) :
   - **un côté en CHF** → **une** écriture du journal *Banque* : ligne CHF au montant de la transaction CHF, ligne du
     compte en devise au montant en devise avec `foreign = { amount, rate = rate_from_chf(amount, CHF) }`
     (`crates/kesh-core/src/accounting/fx.rs:266`, provenance `manual`) — le CHF de Wise, jamais un cours de l'AFC ;
     écriture équilibrée sans différence de change ; `rate_from_chf` en échec → `failed[]` ; à l'acceptation, **les
     deux** transactions passent `reconciled` sur cette écriture (`mark_reconciled_in_tx` de 38-11c, une fois chacune).
   - **deux devises étrangères** (EUR↔USD) → deux écritures par le compte de conversion de la société
     (`companies.fx_transit_account_id`, `crates/kesh-db/src/entities/company.rs:205`), par `create_conversion_in_tx`
     (`crates/kesh-db/src/repositories/journal_entry_conversions.rs:261`, aujourd'hui privée : appelée par le constructeur de
     38-11c, `complete_difference = false`) : CHF = montant source au cours mensuel retenu de la société
     (`find_company_rate_in_tx`, `repositories/exchange_rates.rs:365`), cible au même CHF par `rate_from_chf` ; cours
     absent → `failed[]` `RATE_UNAVAILABLE` ; compte de conversion absent → `failed[]` `FX_ACCOUNT_INVALID` ; chaque
     transaction rattachée à son écriture.
   - ⚠️ **L'autre côté d'une conversion à une écriture.** Une écriture qui porte deux transactions de comptes
     différents doit être reconnue « virement entre deux comptes propres » par l'annulation, sinon annuler un côté
     contre-passerait l'écriture et laisserait l'autre rapprochée d'une écriture annulée. Le prédicat unique
     `other_side_predicate` (`crates/kesh-db/src/repositories/reconciliation_link.rs:698-709`, `bt.amount =
     -(movement)`, de même devise seulement) gagne **une** alternative : transaction d'un autre compte bancaire, de
     **devise différente** de `t` et de **signe opposé** au mouvement (`SIGN(bt.amount) = -SIGN(mouvement)`), exclusions des
     règlements inchangées ; la condition 3 (mouvement de l'écriture sur le grand livre de `t`) borne toujours le rattachement.
     Toutes ses dérivées en héritent (`linkable_side` `:739`, `transfer_other_side` `:773`, `find_matched_elsewhere` `:896`,
     décompte de page) : annuler un côté ne retire que son lien (34-11b). La devise de `t` devient un paramètre de plus,
     **commun à toute la page** (devise du compte du plan, comme `t.bank_account_id`). **Inerte pour l'existant** : tant qu'aucun compte
     bancaire n'est en devise (seuls les comptes prestataires peuvent l'être, C-37-9), toute transaction liée est en CHF
     (condition 1) et la seconde branche ne s'active jamais — tests existants verts sans changement d'assertion.
     ⚠️ **Texte unique avec 40-7c** : 40-7c AC-1 ajoute au même prédicat `AND bt.currency = <devise de t>` (« aucune autre paire
     ne passe ») ; les deux changements s'écrivent **dans le même texte** : `(bt.bank_account_id <> ? AND ((bt.currency = ?
     AND bt.amount = -(M)) OR (bt.currency <> ? AND SIGN(bt.amount) = -SIGN(M))) AND exclusions)` — première branche : même
     devise et montant opposé exact (virement, versement Stripe) ; seconde : les deux côtés d'une conversion. Le premier des
     deux epics qui arrive écrit les deux branches, le second reprend `main`, relit le texte et garde les tests de l'autre ;
     le test d'égalité de la 34-4 couvre un compte en devise et une paire de devises différentes.
     **Le plafond « une transaction liée au plus »** de `linkable_side` (doc `:730-737`) reste inerte : chaque branche
     impose `SIGN(bt.amount) = -SIGN(M)` ; deux transactions liées à l'écriture sont de signes opposés, une troisième ne
     peut l'être des deux (démonstration à écrire au Change Log de développement) ; un test de comportement (troisième
     transaction d'un compte qui partage le grand livre, refusée) le garde quand même. Porté par **38-11c** avec le
     constructeur — **c'est 38-11c qui livre la première conversion à une écriture (PayPal)** : sans l'alternative,
     annuler un côté d'une conversion PayPal contre-passe l'écriture et laisse l'autre côté rapproché d'une écriture annulée
     (demande à l'Epic 38, au rapport P2) ; si 38-11c est développée sans, cette story le porte. Changement de `kesh-db` : gate complet en boucle.
   - Moitié manquante (l'autre devise pas encore importée, ou déjà rapprochée) → écart `conversion_half_missing`.
4. **AC-4 — Autre côté déjà passé : virements entre ses comptes, versements Stripe** (G7, G8 ; décision (4) de
   l'orchestrateur). Pour une ligne `StripePayout`, ou `Other` de préfixe `TRANSFER` (virement, dépôt : seuls genres qui peuvent être l'autre côté
   d'un virement entre ses comptes ; un achat par carte ne l'est jamais et n'ouvre pas la lecture) : `find_matched_elsewhere`
   (`reconciliation_link.rs:896` : les écritures rattachables à `t` **déjà rapprochées** d'une transaction d'un autre
   compte bancaire, l'autre côté d'un virement, 34-4 et 34-11b), en devise par la levée de 38-11c AC-1 →
   **exactement une** candidate à 5 jours au plus (`days_apart`) → proposition `link_entry { bankTransactionId,
   journalEntryId }`, exécutée par `link_entry_in_tx` (`:1156`) comme le `link_sale` de 38-11c (une seule fonction) ;
   coût **mesuré** au développement (une lecture par transaction concernée, sur 20 000 écritures et 300 transactions `TRANSFER`, au Change Log) ;
   au-delà d'une seconde, lecture par page sur le patron de `count_already_booked_for_page` (`:959`) ; plusieurs → « à examiner », motif `ambiguous_link` (candidates nommées, choix à l'écran par `POST …/link-entry`,
   `lib.rs:1046`). Cas couverts sans code propre : le virement passé depuis la banque (relevé CAMT rapproché) et
   l'écriture du versement de l'Epic 40 (rapprochée de la transaction du compte Stripe, C-40-K5) — la transaction Wise
   en devient l'autre côté, jamais une seconde écriture. 40-7c propose le même rattachement depuis l'autre côté : le
   premier accepté rend l'autre périmé (la transaction n'est plus `pending`). Sans candidate : `StripePayout` →
   « à examiner », motif `stripe_payout_unmatched` (**jamais** de règle ni d'écriture de vente, écrit au manuel par
   39-11d) ; `Other` → AC-5.
5. **AC-5 — Règles et reste.** `first_matching_rule` (`crates/kesh-reconciliation/src/rules.rs:85-95`, `rule_matches`
   `:46-70`) avec les **filtres facultatifs** de 38-11b et 38-11c (`provider_type`, `counterparty_email`, `direction`,
   décision (3) : aucun type de critère neuf) → proposition `rule` de 38-11c (contrepartie CHF, ligne Wise en devise au
   cours mensuel, **sans code TVA**, `increment_applied_count_in_tx`). Les critères existants suffisent à Wise :
   `counterparty_contains` / `counterparty_exact` (marchand, bénéficiaire, payeur), `iban_exact` (IBAN du
   bénéficiaire), `reference_contains`, avec le filtre `provider_type` (`CARD`, `TRANSFER`…) et le sens. Un virement
   vers sa propre banque dont l'autre côté n'est pas encore passé s'affecte **par une règle** (IBAN ou nom → compte du
   plan de la banque) ; l'autre côté s'y rattachera à son import (34-11b) — Q-39-15 tranchée ainsi. Aucune règle →
   « à examiner » (motif `no_rule`).
6. **AC-6 — Écarts** (dans `gaps` de la revue, calculés, rien de stocké) : `conversion_half_missing` ;
   `fee_without_origin` (ligne `FEE-x` sans transaction `x` sur le même compte bancaire) ; `balance_gap` (pour chaque
   import des comptes du libellé dont la période tombe dans l'exercice et dont `opening_balance` n'est pas `NULL`, **la règle
   de 39-11b AC-5, telle quelle** : aucun autre relevé du compte ne chevauche sa période (`overlapping_statements`, hors lui-même)
   et `previous_statement_closing` (`crates/kesh-db/src/repositories/bank_imports.rs:358`, relevé antérieur à `period_from`) en
   rend un dont la clôture diffère de l'ouverture ; les doublons ne se relisent pas, ils ne sont pas stockés). Chaque écart et chaque « à examiner » : transaction, date, montant, devise, motif.
7. **AC-7 — Acceptation.** Corps `{ account, proposals: [ {type, …} ] }` (`fee`, `conversion`, `link_entry`, `rule`) ;
   chaque proposition **rejuge** son classement sous verrou (transaction encore `pending`, même genre, même paire, même
   candidate, règle qui correspond toujours), sinon `failed[]` périmée ; refus du contrôle central des lignes en devise
   → `failed[]` par `fx_failed_proposal` (`routes/reconciliation.rs:763`) ; refus du rattachement (`LinkRefusal`) →
   `failed[]` sous leurs codes existants. Réponse `{ accepted: [{ type, bankTransactionIds, journalEntryIds }], failed
   }`.
8. **AC-8 — Aucune migration, i18n.** Genres et motifs calculés ; libellés des motifs et codes neufs aux quatre
   locales (`crates/kesh-i18n/locales/*`), `crates/kesh-api/src/errors.rs`.

## Tâches

- [ ] T0 — Lire 38-11c telle que développée ; noter chaque fonction réutilisée ou généralisée (G10) ; constater si le
  constructeur de conversion à une écriture et le prédicat étendu y sont.
- [ ] T1 — `wise.rs` : `classify`, revue pure (AC-2, AC-4 à AC-6), tests purs.
- [ ] T2 — Conversions : paire, constructeur commun, prédicat de l'autre côté s'il manque (AC-3).
- [ ] T3 — Routes minces de revue et d'acceptation sur le moteur de 38-11c ; registre, trace (AC-1, AC-7).
- [ ] T4 — Codes, i18n (AC-8) ; non-régression.

## Stratégie de tests

- **Purs** (`wise.rs`) : chaque genre sur des transactions construites ; préséance (une ligne de frais ou un versement
  Stripe n'est jamais une règle) ; paire de conversion (même ID, devises, sens, même libellé ; moitié manquante) ;
  candidate unique, aucune, plusieurs, à plus de 5 jours ; écarts.
- **HTTP** (`crates/kesh-api/tests/wise_reconciliation_e2e.rs`) sur les fixtures de 39-11a importées par 39-11b, cours
  mensuels de 2020 posés : frais acceptés au bon CHF ; conversion CHF→EUR → **une** écriture, CHF = montant de Wise,
  deux transactions rapprochées ; annulation d'un côté → seul son lien retiré, l'écriture reste ; second côté
  rattachable à nouveau ; troisième transaction refusée ; EUR→USD → deux écritures, même CHF,
  compte de conversion soldé ; virement vers l'IBAN fictif de la société après import et rapprochement d'un relevé CAMT
  de ce compte → `link_entry` accepté ; versement Stripe avec une écriture fictive de versement rapprochée d'une
  transaction d'un compte bancaire `stripe` posé en base → `link_entry` accepté, sans → `stripe_payout_unmatched` ;
  paiement par règle `counterparty_contains` filtrée `CARD` (jamais de lecture `find_matched_elsewhere` pour une ligne `CARD`) ; virement par règle `iban_exact` ; écarts listés ;
  proposition périmée ; interblocage injecté dans une proposition → rejeu, aucune ligne dans `failed[]` ; import chevauchant un autre
  → aucun `balance_gap` à la revue, import discontinu → un seul ; clé `read` → 403 au `POST`. Somme des écritures par compte Wise = variation du solde Wise.
  Tests PayPal de 38-11c et de l'annulation 34-11b verts sans changement d'assertion.
- Gate complet en boucle (prédicat de `kesh-db`).

## Modules touchés (4)

`kesh-reconciliation` (`wise.rs`), `kesh-api` (routes `wise` minces, erreurs), `kesh-db` (prédicat de l'autre côté,
seulement si 38-11c ne le porte pas), `kesh-i18n`.

## Note de version

*Ajouté* — Les lignes d'un relevé Wise se rapprochent sur proposition : frais au compte de frais, conversions au cours
de Wise (une écriture quand un côté est en francs), rattachement à l'écriture déjà passée de l'autre côté (virement
depuis votre banque, versement Stripe), le reste par vos règles ; ce qui ne se rapproche pas est listé avec son motif.

## Notes de développement

- Ancienne 39-4 (classement des types Wise du relevé JSON, `superseded`) : la reconnaissance se fait ici par le
  préfixe de l'identifiant, langue de l'export indifférente (les descriptions du relevé sont traduites par Wise).
- La reconnaissance d'un virement interne par IBAN ou nom de la société (première rédaction) est remplacée par
  l'autre côté déjà passé, puis la règle : aucune heuristique propre, aucun motif `own_transfer_target_unknown`.
- ⚠️ **Taille** : la plus lourde des quatre ; tenable en une passe seulement si 38-11c livre un moteur et un plan de conversion génériques
  (T0) ; sinon la généralisation lui revient et la story se découpe (11c1 : classement, revue, routes ; 11c2 : conversions et prédicat),
  à trancher à l'ouverture du développement, pas en cours de route.
- ⚠️ **Risque principal** : l'alternative du prédicat de l'autre côté (AC-3) touche un invariant de 34-11b qu'aucune
  contrainte ne défend ; ses tests (annulation d'un côté, troisième transaction refusée) sont la condition de sa clôture.

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
- 2026-11-20 — validation P1 (Claude Opus 5.5, contexte frais) : 0 CRITICAL, 3 HIGH, 4 MEDIUM, 2 LOW, remédiés dans la
  fiche (rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-11c-rapprochement-wise-P1.md`). HIGH : propositions
  placées dans `GET /reconciliation/proposals` et `POST /reconciliation/accept`, que 38-11c laisse CHF (elle a ses
  routes) → routes `wise` minces sur le moteur de 38-11c ; conversion EUR↔USD « au cours AFC du mois » et repli à deux
  écritures pour un côté CHF → décision (1) appliquée ; annulation d'une conversion à une écriture qui contre-passait l'écriture en laissant l'autre côté rapproché
  (prédicat de l'autre côté étendu, testé). MEDIUM : virements internes par IBAN et nom (heuristique, `normalize_iban` privée,
  `rules.rs:112`) → `find_matched_elsewhere` puis règle ; versement Stripe par `find_entry_candidates` à ± 5 jours sans
  l'autre côté → décision (4), même chemin ; écarts dans une route à part → `gaps` de la revue ; `balance_gap` sans
  définition → règle de 39-11b. LOW : registre et trace d'audit ; motifs réduits (`own_transfer_target_unknown`,
  `fx_transit_missing` retirés, ce dernier rendu par `FX_ACCOUNT_INVALID` à l'acceptation).
- 2026-11-20 — validation P2 (Claude Sonnet 5.5, contexte frais) : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 1 LOW, remédiés (rapport
  `kesh-gate-logs/epic-39-40-kiss/validation/39-11c-rapprochement-wise-P2.md`). Remédiation P1 relue au code d'`origin/main`
  (`fd04099f`), à 38-11c et 40-7c lues dans leurs worktrees. MEDIUM : (1) `balance_gap` définie par un chevauchement que la
  revue ne peut pas lire (doublons non stockés) et qui, relue à la revue, signale à tort l'import qui en chevauche un autre →
  règle de 39-11b AC-5 (`overlapping_statements`) ; (2) `POST /wise/accept` sans statut `Rejouee` ni enveloppe de rejeu au
  registre, contre 38-11c AC-7 et 40-7c ; (3) l'alternative du prédicat de l'autre côté et le `AND bt.currency = …` de 40-7c AC-1
  réécrivent le même texte en sens contraires → un texte à deux branches, signal à 38 (38-11c livre la première conversion à une
  écriture sans l'alternative) et à 40 ; (4) `find_matched_elsewhere` lu pour toute ligne `Other` (une lecture lourde par
  transaction) → seulement `StripePayout` et `TRANSFER`, coût mesuré. LOW : constructeur de conversion nommé (`plan_conversion`,
  `create_provider_conversion_in_tx`) au lieu de `create_conversion_in_tx` « rendue publique ».
