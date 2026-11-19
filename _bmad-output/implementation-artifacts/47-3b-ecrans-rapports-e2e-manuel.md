# Story 47-3b : écrans des rapports présentés, parcours E2E, manuel utilisateur

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md` — D4, D6, D9 ; Q9 ; FR-47-3 (écran), FR-47-4,
FR-47-6, FR-47-8 (manuel utilisateur) ; NFR-47-4. Née du découpage de l'esquisse 47-3 (2026-11-19). Dépend des
**47-2a**, **47-2b** et **47-3a**. Maquettes : § M4 et M5 du formulaire d'approbation.

## Story

En tant que comptable,
je veux voir, au bilan et au compte de résultat, quelles lignes sont présentées et d'où elles viennent, et basculer
d'un clic vers la présentation brute (écran et exports),
afin de lire mes rapports comme dans mon ancien logiciel sans perdre la vue des soldes réels.

## Critères d'acceptation

1. **Bascule « Présentation »** dans la barre de commandes des onglets *Bilan* et *Compte de résultat*
   (`routes/(app)/reports/+page.svelte`) : **Présentée** (défaut) · **Brute** ; elle recharge le rapport avec
   `presentation=…` et passe aux exports PDF et CSV de l'onglet (`getReportExportUrl`). Retenue par visiteur
   (`localStorage`, lecture et écriture sous `try/catch`), jamais pour les autres onglets.
2. **Marque d'une ligne présentée** (`BalanceSheetView.svelte`, `IncomeStatementView.svelte`) : une ligne avec
   `sources` porte une pastille « miroir » ou « compensé » après le libellé ; un bouton (chevron, `aria-expanded`)
   **déplie** ses sources sous la ligne : numéro (lien vers le grand livre, `ledgerHref`), libellé, solde brut signé.
   Accessible au clavier ; au survol, `title` résumant les sources.
3. **Bandeau « Présentation compensée »** (Q9) : quand le rapport porte `compensated`, un avis au-dessus des tableaux
   (style d'information) — « Présentation compensée : les paires de comptes sont présentées nettes. Pour les comptes
   annuels officiels, utilisez la présentation brute (CO 958c al. 1 ch. 7). » — avec un lien qui bascule en brute.
4. **Présentation brute** : rendu identique à l'écran d'aujourd'hui (aucune pastille, aucun bandeau) ; un bandeau
   discret rappelle « Présentation brute : soldes signés, compte par compte. ».
5. **Types** : `reports.types.ts` (`sources`, `compensated`, `PresentationSource`), `reports.api.ts`
   (`presentation` dans la requête et l'URL d'export) ; `isReportEmpty` inchangé dans son sens (un rapport présenté
   vide l'est aussi en brut, à tester).
6. **Vitest** : vues (marque, dépliage, bandeaux, brute), API (paramètre), page (bascule persistée, exports).
7. **E2E du parcours** (NFR-47-4, `frontend/tests/e2e/`) : jeu de l'exemple chiffré du fichier d'epic semé par l'API ;
   lien posé à l'écran (47-3a) → bilan sans montant négatif (1179 à 1 200.00, 2120 à 800.00) → dépliage des sources
   → bascule brute (2270 à −1 200.00) → export PDF en présenté contenant « Présentation compensée » quand la paire
   6960/6961 est posée.
8. **Captures d'écran** (thème clair) : bilan présenté, sources dépliées, compte de résultat compensé avec bandeau,
   présentation brute.
9. **Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, section *Rapports comptables* `:3908`, sous-sections
   *Bilan* `:3910`, *Compte de résultat* `:3941`, *Visualisation et exports* `:4171`) : ce que montrent le bilan et le
   compte de résultat présentés, la marque et les sources, la mention « présentation compensée » et l'avertissement
   CO 958c, comment obtenir la présentation brute (écran, PDF, CSV), et **l'exemple chiffré du fichier d'epic**
   (rejoué par le test de la 47-2a AC-11 : le manuel et le test portent les mêmes nombres) ; renvoi à la sous-section
   du plan comptable (47-3a). Les sections des rapports en devise (`:4051`) disent la règle Q3. `.tex` seulement ;
   **le prompt de chaque passe de revue nomme le manuel** et exige le contrôle du PDF compilé localement.
10. **i18n** : quatre locales ; `sitesTotal` recompté ; gardes de libellés vertes.

## Tâches / sous-tâches

- [ ] **T0** : reprise de `main` ; `fix/bilan-resultat-reporte` touche `BalanceSheetView.svelte` et `reports.api.ts`
  (`showsRetainedEarningsLine`) ; si l'Epic 46 est fusionnée, la bascule cohabite avec le sélecteur de niveau (46-2).
- [ ] **T1** : types et API (AC 5). **T2** : bascule et persistance (AC 1). **T3** : marque, dépliage, bandeaux
  (AC 2–4).
- [ ] **T4** : Vitest, E2E, captures (AC 6–8). **T5** : manuel, i18n (AC 9, 10). **T6** : gates frontend et E2E
  complets au dernier commit de code.

## Dev Notes

### Code lu (à `origin/main` `fd04099f`)

- `frontend/src/routes/(app)/reports/+page.svelte:622-715` (barre de commandes, boutons PDF/CSV, patron de la
  bascule).
- `frontend/src/lib/features/reports/BalanceSheetView.svelte` (tableaux par section, `ledgerHref`,
  `AccountCurrencyBalance`), `IncomeStatementView.svelte`, `reports.types.ts:14-60`, `reports.api.ts:36-50`,
  `:138` (`isReportEmpty`), `:303` (`getReportExportUrl`).

### Interaction avec l'Epic 46

Si la 46-2 (sélecteur de niveau) est fusionnée, les deux réglages sont indépendants et passent tous deux aux exports ;
les lignes de compte groupées portent la même marque (47-2a § *Recâblage*).

### Stratégie de tests

Vitest, E2E du parcours, captures ; mutations : bandeau compensé affiché pour un miroir seul, bascule non transmise à
l'export.

### Risques

Conflits avec `fix/bilan-resultat-reporte` et la 46-2 dans les mêmes vues.

### References

- Fiches 47-2a, 47-2b, 47-3a ; maquettes M4, M5 ; `CLAUDE.md` § *Recette visuelle*.

## Notes de version

Le bilan et le compte de résultat marquent les lignes présentées et en montrent les sources ; une bascule passe à la
**présentation brute**, à l'écran comme dans les exports ; un bandeau signale une présentation compensée.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
