# Story 39-11d : écrans, manuel et E2E de l'import Wise

Status: ready-for-dev

*Fiche rédigée le 2026-11-20, branche `epic/39-wise`, revérifiée à la validation P1. Epic 39, **phase 1 KISS**.
**Dépend de** : 39-11a à 39-11c, **38-11d** (écrans PayPal, validée P1 : case « Compte PayPal » du formulaire des
comptes bancaires, `paypal.ts` et `withoutPaypal`, filtres de règle, page `reconciliation/paypal`, libellés des refus,
E2E `paypal-import.spec.ts`), dont les composants sont **généralisés**, non recopiés (G10).*

## Story

En tant que **Comptable**, je veux régler mes comptes Wise, importer mes relevés et traiter les propositions et les
écarts depuis l'écran, et trouver au manuel comment exporter le relevé de Wise et ce que Kesh en fait.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-11d-1` | Formulaire d'un compte bancaire : prestataire « Wise » à côté de « PayPal » (un seul choix), libellé, compte de frais. | AC-1 |
| `FR-39-11d-2` | Page Wise : import d'un ou plusieurs relevés (un par devise), compte rendu par fichier (insérées, doublons, soldes, trou). | AC-2 |
| `FR-39-11d-3` | Propositions Wise (frais, conversion, rattachement à l'autre côté, règle), écarts et « à examiner » lisibles, quatre locales. | AC-3 |
| `FR-39-11d-4` | Filtre de règle « type » devenu générique : aide qui liste les types PayPal et les préfixes Wise. | AC-1 |
| `NFR-39-11d-1` | Manuel utilisateur et `docs/api-external.md` à jour (`.tex` seulement, PDF à la release) ; *Note de version*. | AC-4 |
| `NFR-39-11d-2` | E2E sur les fixtures fictives ; captures d'écran de chaque état touché (*Recette visuelle*). | AC-5 |

## Critères d'acceptation

1. **AC-1 — Comptes bancaires et règles.** Le formulaire de 38-11d AC-1 (`frontend/src/routes/(app)/bank-accounts/
   +page.svelte`, `submitCreate`, `submitUpdateFull`) remplace la case « Compte PayPal » par un **choix de
   prestataire** (« Aucun », « PayPal », « Wise »), mêmes champs et mêmes règles d'affichage (IBAN facultatif,
   QR-IBAN et « principal » masqués, compte du plan en devise offert, compte de frais filtré comme au serveur) ; charge
   `paypal: {…}` ou `wise: {…}` (39-11b AC-2), `null` pour retirer. `paypal.ts` devient `provider.ts` : `providerOf(a)`,
   `isProvider(a)`, `withoutProviders(accounts)`, appliqués aux **quatre sites** de `withoutPaypal` (38-11d AC-1 :
   `BankImportUpload.svelte`, `ManualStatementForm.svelte`, `SettleInvoiceDialog.svelte`, page
   `reconciliation/+page.svelte`) ; `BankAccountSummary` gagne `wise` ; la liste dit « Wise · <libellé> · <devise> ».
   Le filtre « Type PayPal (exact) » de `RuleFormModal.svelte` devient « Type (prestataire) », aide : types PayPal
   (`Website Payment`…) et préfixes Wise (`TRANSFER`, `CARD`, `BALANCE`, `CONVERSION_ORDER`, `FEE-TRANSFER`…). Refus du
   serveur affichés par leur code (`errorMessageOf`). Vitest : choix du prestataire (charges, champs masqués), les
   quatre sites, mappage `wise`.
2. **AC-2 — Page Wise** `frontend/src/routes/(app)/reconciliation/wise/+page.svelte`, sur le **composant de la page
   PayPal généralisé** (38-11d AC-3 à AC-5 : sélecteurs compte et exercice, bouton d'import, revue rechargée après import
   et acceptation, garde de génération, propositions par paquets de `MAX_IN_CLAUSE_IDS`, écarts, « à examiner »,
   modale « Rattacher à une écriture » en devise) paramétré par le prestataire (routes, libellés, forme du compte rendu),
   entrée « Wise » du menu à côté de « PayPal », dans `comptableOnly`. Bouton **« Importer des relevés Wise »** :
   sélection multiple (`accept=".csv,text/csv"`, `multiple`), envoi à `POST /api/v1/wise-imports` ; compte rendu par
   fichier ; refus traduits par les codes fermés de 39-11a (colonnes manquantes nommées, ligne et champ, rupture du
   solde courant avec la ligne) et de 39-11b (devise sans compte, fichier de même devise) ; avertissement `gap`
   expliqué (« le relevé ne commence pas où finissait le précédent »).
3. **AC-3 — Propositions, écarts, à examiner.** Genres : « Frais », « Conversion au cours de Wise : 100.00 EUR → 95.00
   CHF » (exemple fictif ; « deux écritures par le compte de conversion » entre deux devises étrangères), « Rattachement
   à l'écriture n° … » (`link_entry`), « Règle <libellé> » ; écarts (`conversion_half_missing`, `fee_without_origin`,
   `balance_gap`) ; motifs « à examiner » (`fee_account_missing`, `ambiguous_link`, `stripe_payout_unmatched`,
   `no_rule`). Textes « proposer », jamais « comptabiliser automatiquement ». Libellés des refus d'acceptation par le
   module de 38-11d (`paypal-labels.ts` généralisé, délégation à `failedProposalLabel`), test des codes de la route Wise
   contre leur source. Clés aux quatre locales, préfixe `wise-` pour ce qui est propre à Wise, `provider-` pour le
   commun (`docs/i18n-key-ownership-pattern.md`), chaque clé en toutes lettres ; `sitesTotal` de
   `frontend/src/lib/shared/i18n-keys.test.ts` **recompté** depuis la source.
4. **AC-4 — Manuel et API** (`docs/manual/fr/user-manual.tex`, `.tex` seul ; DE, IT, EN « à traduire » selon la règle
   du dépôt) : sous-section « Comptes Wise » à côté de « Comptes PayPal » (38-11d AC-6) — un compte bancaire par devise,
   compte de frais, compte de conversion requis entre deux devises étrangères ; exporter de Wise le relevé CSV de chaque
   solde (période, une devise par fichier ; le zip n'est pas lu) ; l'importer ; ce que Kesh propose ligne par ligne ;
   les conversions au cours de Wise ; le rattachement à l'autre côté (virement depuis votre banque, versement Stripe de
   l'Epic 40 : jamais compté deux fois, jamais affecté par une règle) ; un virement vers sa banque par une règle quand
   l'autre côté n'est pas encore passé ; les écarts ; les limites (date de l'export sans conversion de fuseau, aucun
   code TVA, pas de téléchargement automatique ; **celles de 38-11b AC-3 reprises pour Wise** : une facture fournisseur ne se paie
   pas depuis un compte Wise — régler par un compte interne ou une écriture à la main, puis rattacher la transaction —, et la
   réconciliation générale ne refuse pas un compte Wise **en francs** : ses lignes se traitent par la seule page Wise, sans quoi une
   conversion ou des frais pourraient s'écrire deux fois). Gardes de `textes_coherents.rs` à connaître avant d'écrire (38-11d
   AC-6 : G2 « Réglages », G4-bis/G4-ter numéros de compte — ni « 1020 », « 4830 » ni « 9900 » —, G25, G18). La phrase
   du compte des routes du manuel d'administration est tenue par 39-11b et 39-11c. `docs/api-external.md` (section de
   38-11d AC-7) : `POST /api/v1/wise-imports`, `GET /api/v1/wise/review`, `POST /api/v1/wise/accept`, champ `wise`
   des comptes bancaires, codes `WISE_*` à la table des erreurs. Contrôle local du PDF permis (aplati, `pdftotext … |
   tr '\n' ' ' | tr -s ' '`), PDF non commité.
5. **AC-5 — E2E** `frontend/tests/e2e/wise-import.spec.ts`, patron de `paypal-import.spec.ts` (38-11d AC-8 : société
   `with-company`, exercice 2020, multi-devises activé par l'API, reseed en fin de parcours) : comptes du plan EUR et
   USD, trois comptes bancaires Wise d'un libellé, compte de frais, compte de conversion, cours mensuels de 2020 ;
   import par l'interface des trois fixtures de 39-11a, lues par chemin (`crates/kesh-import/tests/fixtures/`, ni
   copie ni seconde fixture) ; compte rendu ; acceptation d'un frais et d'une conversion CHF↔EUR (une écriture) ; ligne de versement Stripe « à examiner » (`stripe_payout_unmatched`) affichée ; réimport → refus doublon. Captures (thème clair) de chaque état touché (formulaire du compte Wise,
   compte rendu d'import, propositions, écarts, à examiner, acceptation), dans `KESH_CAPTURES_DIR`, hors dépôt
   (`kesh-gate-logs/`), jointes au Dev Agent Record et au prompt de revue. Suite E2E complète au dernier commit de code.

## Tâches

- [ ] T1 — Choix du prestataire, `provider.ts`, filtre de type générique (AC-1).
- [ ] T2 — Composant de la page PayPal généralisé, page Wise, menu (AC-2, AC-3).
- [ ] T3 — Manuel utilisateur, `docs/api-external.md` (AC-4).
- [ ] T4 — E2E, captures (AC-5).

## Modules touchés (4)

`frontend`, `kesh-i18n` (libellés), `docs/manual` (`.tex`) et `docs/api-external.md`, E2E.

## Note de version

*Ajouté* — Écrans de l'import Wise : réglage des comptes, import des relevés, propositions et écarts ; section Wise au
manuel utilisateur.

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
- 2026-11-20 — validation P1 (Claude Opus 5.5, contexte frais) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 2 LOW, remédiés dans la
  fiche (rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-11d-ecrans-manuel-e2e-wise-P1.md`). MEDIUM : composants
  de 38-11d nommés sans dire lesquels ni comment les généraliser (case PayPal, `withoutPaypal` et ses quatre sites,
  page PayPal, libellés des refus) ; E2E sans exercice d'accueil ni multi-devises (patron de `paypal-import.spec.ts`) ;
  `docs/api-external.md` absent. LOW : gardes du manuel (numéros de compte), clés i18n et `sitesTotal`.
- 2026-11-20 — validation P2 (Claude Sonnet 5.5, contexte frais) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 1 LOW, remédiés (rapport
  `kesh-gate-logs/epic-39-40-kiss/validation/39-11d-ecrans-manuel-e2e-wise-P2.md`). MEDIUM : le manuel ne reprenait pas les limites
  de la phase 1 de 38-11b (facture fournisseur, angle mort de la réconciliation générale, plus lourd pour un solde Wise en francs
  qui reçoit conversions et frais). LOW : « écart Stripe » est un motif « à examiner », non un écart.
