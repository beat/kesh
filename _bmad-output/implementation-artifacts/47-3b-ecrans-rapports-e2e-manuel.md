# Story 47-3b : écrans des rapports présentés, parcours E2E, manuel utilisateur

Status: ready-for-dev

Epic : `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md` — D4, D6, D9 ; Q9 ; FR-47-3 (écran), FR-47-4,
FR-47-6, FR-47-8 (manuel utilisateur) ; NFR-47-4. Née du découpage de l'esquisse 47-3 (2026-11-19). Dépend des
**47-2a**, **47-2b** et **47-3a**. Maquettes : § M4 et M5 du formulaire d'approbation — **pour la disposition
seulement** : leurs nombres et comptes (1179, 1021, 2120, 6960, 6961, « Fournisseurs », capital 9 000.00, résultat
900.00), la place de la bascule (« dans la barre de commandes ») et le texte de la note du PDF (« net de … ») sont
antérieurs à la validation ; font foi les critères ci-dessous (bascule sous `ReportSelector`, AC-1), la 47-2b (note
« comprend … ») et l'exemple de la 47-2a AC-11. Les captures et le manuel ne reprennent **aucun** nombre des
maquettes (G4-bis rougirait sur 1179, 1021, 2120, 6960, 6961).

## Story

En tant que comptable,
je veux voir, au bilan et au compte de résultat, quelles lignes sont présentées et d'où elles viennent, et basculer
d'un clic vers la présentation brute (écran et exports),
afin de lire mes rapports comme dans mon ancien logiciel sans perdre la vue des soldes réels.

## Critères d'acceptation

1. **Bascule « Présentation »** : un groupe de deux boutons radio (`role="radiogroup"`) **Présentée** (défaut) ·
   **Brute**, posé par `routes/(app)/reports/+page.svelte` **sous** `ReportSelector` (la barre de commandes des cinq
   rapports classiques est ce composant, partagé : il n'est pas modifié), rendu **seulement** sur les onglets *Bilan* et
   *Compte de résultat* et pour toute société (la société sans lien le voit aussi : le manuel dit qu'il ne change alors
   rien). **Un seul réglage pour les deux onglets** (`presentation` : `'presentee' | 'brute'`), retenu par visiteur sous
   une clé `localStorage` propre (`kesh.reports.presentation`), lecture et écriture sous `try/catch`, toute autre valeur
   lue valant `presentee`, jamais lue ni écrite pour un autre onglet. Le changement :
   a. invalide **les deux** rapports en mémoire (`balanceSheet`, `incomeStatement` à `null`) et toute génération en vol
      (`genSeq++`), comme le changement d'exercice (`+page.svelte` effet `lastFyId`) — sans quoi l'onglet quitté
      garderait un rapport dans l'ancien mode sous un réglage qui dit l'inverse ;
   b. **regénère** le rapport de l'onglet actif **s'il en affichait un** — fait lu **avant** l'invalidation du point a,
      qui le remet à `null` (rien n'est lancé sinon : la page ne génère qu'au clic sur *Générer*) ;
   c. passe `presentation` à `getBalanceSheet`, `getIncomeStatement` **et** à `getReportExportUrl` (PDF et CSV), valeur
      lue **au clic** de l'export comme `query` l'est déjà ; **jamais** envoyé aux autres rapports (trial-balance, TVA,
      journaux, projets, grand livre) ni aux exports de ceux-ci.
   Valeurs d'API : `brute` | `presentee` (47-2a AC-9).
2. **Marque d'une ligne présentée** : composant **unique** `PresentedRow.svelte` (le patron `<tr>` est répété à cinq
   endroits, trois au bilan, deux au compte de résultat : pas de sixième copie), utilisé par `BalanceSheetView.svelte`
   (actifs, passifs ; **pas** les fonds propres, D8) et `IncomeStatementView.svelte` (produits, charges), avec une
   propriété `ledgerLinks` (le bilan lie le numéro au grand livre, le compte de résultat ne le fait pas aujourd'hui et
   ne change pas). Une ligne avec `sources` porte une pastille après le libellé : **« compensé »** si une source est de
   `kind: offset`, **« miroir »** sinon ; un bouton (chevron, `aria-expanded`, `aria-controls`) **déplie** ses sources
   dans une ligne de tableau sous la ligne : numéro (lien vers le grand livre, `ledgerHref`, sur les deux rapports),
   libellé, solde brut signé (`formatReportAmount`), et le genre, **aux libellés de la note du PDF** (47-2b AC-1 et
   AC-5, mêmes mots à l'écran et au PDF) : « comprend » (`mirror`), « solde propre » (`own`, et le membre d'une paire
   qui porte la ligne, premier des `sources`), « déduit » (l'autre membre de la paire). Accessible
   au clavier ; `title` de la pastille résumant les sources. Une ligne **sans** `sources` est rendue exactement comme
   aujourd'hui.
3. **Bandeau « Présentation compensée »** (Q9) : quand le rapport porte `compensated`, un avis (style d'information,
   `role="status"`) au-dessus des tableaux — « Présentation compensée : les paires de comptes sont présentées nettes.
   Pour les comptes annuels officiels, utilisez la présentation brute (CO 958c al. 1 ch. 7). » — avec un bouton-lien
   qui bascule en brute (propriété `onSwitchToRaw` des vues ; même effet que le groupe de l'AC-1). Jamais affiché pour
   un miroir seul.
4. **Présentation brute** : rendu identique à l'écran d'aujourd'hui (aucune pastille, aucun bandeau compensé) ; un
   bandeau discret rappelle « Présentation brute : soldes signés, compte par compte. ». ⚠️ Un rapport brut ne se
   distingue pas, par son JSON, d'un rapport présenté sans lien : **les vues reçoivent `presentation`** en propriété
   (le bandeau brut ne se déduit pas du DTO).
5. **Types** : `reports.types.ts` (`sources?: PresentationSource[]` sur `AccountBalance`, `compensated?: boolean` sur
   `BalanceSheetDto` et `IncomeStatementDto`, `PresentationSource`, `ReportPresentation`), `reports.api.ts`
   (`presentation` facultatif de `ReportQuery`, jamais émis s'il est absent ; ajouté par `getBalanceSheet`,
   `getIncomeStatement` et `getReportExportUrl` pour ces deux types seulement). **`isReportEmpty` change** : un rapport
   présenté dont **tout** s'est compensé (paire à net nul : `revenues` et `expenses` vides, `compensated: true`) n'est
   **pas** vide — il afficherait sinon « Aucune écriture dans la période sélectionnée », faux alors que les écritures
   existent. Règle : `compensated === true` ⇒ jamais vide, pour le bilan comme pour le compte de résultat (le contraire
   — « un rapport présenté vide l'est aussi en brut » — est faux, et le test le démontre). Tests : paire à net nul au
   compte de résultat, brut non vide ; sans lien, inchangé.
6. **Vitest** : vues (marque, genre des sources, dépliage au clavier, bandeaux compensé et brut, absence de pastille en
   brut, fonds propres jamais marqués), `PresentedRow` seul, API (paramètre émis pour les deux rapports et leurs exports,
   **absent** pour les autres types), `isReportEmpty` (cas de l'AC-5), page (bascule persistée, valeur illisible =
   présentée, invalidation des deux rapports, regénération du seul onglet actif, export portant la valeur courante).
   Mutations à faire rougir : bandeau compensé pour un miroir seul ; bascule non transmise à l'export ; bascule
   transmise au rapport TVA ; DTO de l'autre onglet gardé après la bascule.
7. **Sélecteurs d'E2E** (`data-testid`, fixés ici pour que l'E2E de l'AC-8 et les captures y renvoient) :
   `report-presentation` (groupe), `report-presentation-presentee`, `report-presentation-brute`,
   `report-banner-compensated`, `report-banner-compensated-raw` (le lien), `report-banner-raw`,
   `presented-row-{numéro}` (la ligne), `presented-badge-{numéro}`, `presented-toggle-{numéro}`,
   `presented-sources-{numéro}` (la ligne dépliée).
8. **E2E du parcours** (NFR-47-4, `frontend/tests/e2e/reports-presentation.spec.ts`, `seedTestState('with-company')`,
   `describe.configure({ mode: 'serial' })` comme `reports-currency.spec.ts`) — **le jeu de la 47-2a AC-11** (comptes
   des plans livrés, nombres identiques au test d'intégration et au manuel), semé par l'API. ⚠️ Le seed
   `with-company` ne charge **aucun** plan livré (cinq comptes ad hoc, `test_fixtures.rs:149-166`) : l'E2E ne suppose
   donc pas une société au plan `pme` et **crée par l'API** tout compte du jeu absent du seed, 1040 compris (1040
   n'existe qu'au plan `pme` ; le manuel, lui, suppose une société au plan `pme` et le dit, AC-10). Le seed
   porte déjà 1000 « Caisse CI », 1100, 2000 « Capital CI » (passif), 3000, 4000 ; l'E2E **crée** 1010, 1020, 1040,
   2100, 2270, 2800 (rôle capital), 6900, 6950, **n'asserte jamais un libellé de compte semé** (2000 se nomme « Capital
   CI » ici, « Créanciers » au plan), et passe une écriture par compte contre 2800 à la date du jour de Zurich
   (`todayInZurich()`, dans l'exercice du seed). Parcours : liens **posés à l'écran** par la 47-3a (miroirs 1020→2100 et
   2270→1040, paire 6900/6950 avec l'avertissement confirmé) → *Générer* le bilan présenté : **aucune cellule de
   montant négative**, 1040 à `1’200.00` (actif), 2100 à `800.00` (passif), total actif `13’700.00` → dépliage des
   sources de 1040 (2270 à `-1’200.00`, genre « comprend ») → bascule **Brute** : la requête partie porte `presentation=brute`
   (`page.waitForRequest`), le bilan montre 2270 à `-1’200.00` et 1020 à `-800.00`, le bandeau brut paraît, aucune
   pastille → compte de résultat : présenté, 6950 à `570.00`, 6900 absent, bandeau compensé, résultat net `570.00` ;
   brut, deux lignes, même résultat net ; total passif présenté du bilan `3’800.00`. L'assertion « aucune cellule de
   montant négative » porte sur les lignes du rapport **avant tout dépliage** (les lignes `presented-sources-*`
   montrent, à dessein, le solde brut signé de leurs sources). **Exports** déclenchés par les boutons, **rapport par
   rapport** (la mention n'est pas la même aux deux, 47-2b AC-2) :
   - **compte de résultat** (paire posée) : en présenté, le PDF contient « Présentation compensée » et le CSV porte la
     colonne `Presentation` (valeur `compense` sur la ligne 6950) ; en brut, le PDF **ne la contient pas** et le CSV
     n'a pas la colonne ;
   - **bilan** (miroirs seuls) : en présenté, le PDF **ne contient pas** « Présentation compensée » (un miroir ne
     compense rien) mais contient la note des sources (libellé « comprend » de la 47-2b AC-1, suivi de 2270), et le
     CSV porte la colonne `Presentation` (valeur `miroir` sur la ligne 1040, `Sources` `2270:-1200.00`) ; en brut, ni
     note ni colonne.
   C'est la transmission de la bascule à l'export, vérifiée dans les deux sens et sur les deux rapports : un contrôle
   de la mention sur le seul bilan serait faux en présenté et vide en brut. ⚠️ Formes à l'écran : apostrophe U+2019 des
   milliers et signe `-` ASCII (`formatSwissAmount`), non « 1 200.00 » ni « − ». ⚠️ Le PDF n'est pas du texte brut :
   flux Flate, `Tj` hexadécimal en Latin-1 — **extraire** (précédent `contenuDuPdf` et `hexWinAnsi` de
   `invoice-qr-caracteres.spec.ts`, à **déplacer** dans `helpers/` plutôt que recopier) ; une recherche d'octets
   naïve donnerait un faux rouge ou un faux vert. Le test d'une paire à net nul (rapport « compensé » sans ligne) est
   l'objet d'un Vitest, non de l'E2E.
9. **Captures d'écran** (thème clair ; `KESH_CAPTURES_DIR`, patron de `reports-currency.spec.ts`) : bilan présenté,
   sources dépliées, compte de résultat compensé avec bandeau, présentation brute. Les figures du **manuel** en sont
   tirées (PNG seulement, `docs/manual/fr/screenshots/`, patron de l'Epic 37 ; aucun PDF commité).
10. **Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, section *Rapports comptables* `:3908`, sous-sections
    *Bilan* `:3910`, *Compte de résultat* `:3941`, *Visualisation et exports* `:4171`) : ce que montrent le bilan et le
    compte de résultat présentés, la marque et les sources, la mention « présentation compensée » et l'avertissement
    CO 958c, comment obtenir la présentation brute (écran, PDF, CSV), renvoi à la sous-section du plan comptable
    (47-3a) ; les phrases d'ouverture à **corriger** (« les comptes d'actif et de passif à solde non nul, triés par
    numéro », `:3914`, et son pendant du compte de résultat, `:3945`) ; les sections des rapports en devise (`:4051`)
    disent la règle Q3. **L'exemple chiffré est celui de la 47-2a AC-11, sans autre jeu** : un tableau avant/après aux
    **comptes des plans livrés** (plan **PME**, qu'il nomme : 1040 n'existe qu'au plan PME, et 2800 s'y appelle
    « Capital social »), nombres au format du manuel (apostrophe : `1'200.00`, jamais un montant de quatre
    chiffres sans apostrophe, que G4-bis lirait comme un compte). ⚠️ **G4-bis et G4-ter (`textes_coherents.rs`) lisent
    ce manuel** : tout numéro de quatre chiffres doit être un compte d'un plan livré, et, nommé, sous le nom du plan
    (2000 « Créanciers », 2270 « AVS/AI/APG », 6900 « Charges financières », 6950 « Produits financiers »…) ; un compte
    d'un autre jeu — 1179, 2120, 6960… — fait rougir le gate backend. **Le lien entre l'exemple et le test** (NFR-47-4)
    est contrôlé par une garde ajoutée à `textes_coherents.rs` (**G26**, prochain numéro libre à `origin/main` —
    à recontrôler à l'intégration —, avec sa rangée au tableau des gardes de l'en-tête du module), décidable et non
    vide : l'exemple du manuel est écrit
    entre `% exemple-presentation:debut` et `% exemple-presentation:fin` (lu par `bloc_marque`, la fonction de G23,
    sur le source **brut** lu par `lire`, jamais le texte normalisé de `manuels_fr`/`normaliser`, qui efface les sauts
    de ligne et les marqueurs avec eux) ; ⚠️ **`bloc_marque` ne lit que des marqueurs `% <nom>:debut|fin`** (préfixe
    `% ` codé en dur, `textes_coherents.rs:2167`) : les marqueurs du test Rust sont en `//`, qu'elle ne trouverait pas
    (« bloc absent », donc une garde rouge dès le premier passage, ou, pire, contournée) — la garde ajoute une
    fonction pure sœur `bloc_marque_rs(source, nom)` (marqueurs `// <nom>:debut|fin`, mêmes erreurs : absent, non
    fermé), avec son témoin dans `g23_les_fonctions_pures` ou un test voisin ; la signature et le comportement de
    `bloc_marque` ne changent pas (son témoin G23 reste vert), mais les deux fonctions peuvent partager un corps commun
    (une fonction privée prenant le préfixe du marqueur) plutôt que dupliquer ses dix lignes (règle DRY) ; les deux
    fichiers, manuel et test, sont lus bruts par `lire` ;
    le test d'intégration de la 47-2a (`crates/kesh-api/tests/reports_e2e.rs`, sa T5) porte la liste
    `EXEMPLE_PRESENTATION` entre `// exemple-presentation:debut` et
    `// exemple-presentation:fin` (47-2a AC-11, qui fixe la liste) ; la garde extrait des deux blocs les montants
    **comme jetons entiers** (`-?\d+(?:['’]\d{3})*\.\d{2}` en `find_iter`, la correspondance la plus à gauche
    prenant le montant entier ; apostrophes retirées ensuite, si bien que `1'200.00` au manuel et `1200.00` au test
    sont le même jeton ; le signe est ignoré, le manuel pouvant écrire `$-$` ; jamais une recherche de sous-chaîne,
    que `800.00` satisferait dans `3'800.00`) et exige que **chaque montant de la liste figure dans le bloc
    du manuel** ; témoins : un bloc absent ou non fermé est une erreur, la liste lue compte au moins neuf montants,
    et un montant retiré du manuel fait rougir la garde (test de la fonction pure sur un texte fabriqué, comme
    `g23_les_fonctions_pures`). `.tex` seulement ; **le prompt de chaque passe de revue nomme
    le manuel** et exige le contrôle du PDF compilé localement (aplati : `pdftotext … | tr '\n' ' ' | tr -s ' '`).
11. **i18n** : quatre locales ; `sitesTotal` recompté ; `lint-i18n-ownership` et `i18n-libelle-en-dur` verts. Aucun
    numéro de compte écrit en dur dans un catalogue ou un repli (G4-bis lit les quatre catalogues) : les numéros passent
    en variables Fluent.

## Tâches / sous-tâches

- [ ] **T0** : reprise de `main` ; `fix/bilan-resultat-reporte` touche `BalanceSheetView.svelte` et `reports.api.ts`
  (`showsRetainedEarningsLine`) ; si l'Epic 46 est fusionnée, la bascule cohabite avec le sélecteur de niveau (46-2) :
  deux groupes indépendants, tous deux transmis aux exports.
- [ ] **T1** : types et API (AC 5). **T2** : bascule, persistance, invalidation (AC 1). **T3** : `PresentedRow`, marque,
  dépliage, bandeaux (AC 2–4, 7).
- [ ] **T4** : Vitest, E2E (assistant `contenuDuPdf` déplacé dans `helpers/`), captures (AC 6, 8, 9).
- [ ] **T5** : manuel (exemple entre marqueurs, garde `textes_coherents.rs` et son témoin), i18n (AC 10, 11).
- [ ] **T6** : gates frontend, **backend** (`textes_coherents`) et E2E complets au dernier commit de code.

## Dev Notes

### Code lu (à `origin/main` `fd04099f`)

- `frontend/src/routes/(app)/reports/+page.svelte` : `ReportSelector` `:626-638` (la barre des cinq rapports
  classiques, composant `lib/features/reports/ReportSelector.svelte`, props liées) ; les contrôles du grand livre
  `:640-714` (patron de case à cocher) ; `generate` `:245-342` (requête construite **sans** valeur d'onglet à
  l'appel) ; `exportReport` `:391-493` (`query` figée au clic) ; effet de changement d'exercice `:152-175` (patron de
  l'invalidation) ; états `balanceSheet`/`incomeStatement` conservés d'un onglet à l'autre ; vues `:854-857`.
- `frontend/src/lib/features/reports/BalanceSheetView.svelte` (trois répétitions de ligne, `ledgerHref`,
  `AccountCurrencyBalance`), `IncomeStatementView.svelte` (deux répétitions, numéro sans lien), `reports.types.ts:14-60`,
  `reports.api.ts:36-50`, `:138` (`isReportEmpty`), `:303` (`getReportExportUrl`), `:260` (`formatReportAmount`) ;
  `routes/(app)/reports/reports-page.test.ts` existe (la page est testable).
- `crates/kesh-api/tests/textes_coherents.rs:414` (G4-bis), `:582` (G4-ter) : lisent le manuel utilisateur, les
  catalogues et le code ; `crates/kesh-core/assets/charts/{pme,independant,association}.json` (comptes livrés).
- `frontend/tests/e2e/invoice-qr-caracteres.spec.ts:13-37` (extraction du texte d'un PDF), `reports-currency.spec.ts`
  (société, comptes et écritures semés par l'API, captures), `crates/kesh-db/src/test_fixtures.rs:152` (comptes du
  seed `with-company`).

### Interaction avec l'Epic 46

Si la 46-2 (sélecteur de niveau) est fusionnée, les deux réglages sont indépendants et passent tous deux aux exports ;
les lignes de compte groupées portent la même marque (47-2a § *Recâblage*) : `PresentedRow` sert aussi aux lignes
`kind: account` des groupes.

### Stratégie de tests

Vitest (AC 6), E2E du parcours (AC 8), captures (AC 9), garde du manuel (AC 10) ; mutations de l'AC 6.

### Risques

Conflits avec `fix/bilan-resultat-reporte` et la 46-2 dans les mêmes vues ; extraction du texte du PDF (dépend de la
mise en page de la 47-2b : la mention doit tenir sur une ligne de texte).

### References

- Fiches 47-2a (AC-11, source de l'exemple et de la liste `EXEMPLE_PRESENTATION`), 47-2b, 47-3a ; maquettes M4, M5
  (disposition seulement, voir l'en-tête) ; `CLAUDE.md` § *Recette visuelle*, §
  *Le prompt d'une passe doit NOMMER le manuel*.
- Registre de la dette du fichier d'epic (§ *Registre de la dette de l'epic*) : à la clôture de la boucle de revue,
  tout finding non remédié y est consigné, une ligne par problème ; la 47-5 en part (« on trace toujours »).

## Notes de version

Le bilan et le compte de résultat marquent les lignes présentées et en montrent les sources ; une bascule passe à la
**présentation brute**, à l'écran comme dans les exports ; un bandeau signale une présentation compensée.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-11-19 — validation P1 (Sonnet 5.5) : 0 C / 2 H / 6 M / 4 L, remédiés.** H1 : l'exemple chiffré cité au
  manuel (1179, 1021, 2120, 6960, 6961 ; « Fournisseurs », « Dettes sociales ») fait rougir G4-bis et G4-ter, et le
  fichier d'epic ne porte aucun exemple : jeu transposé aux comptes des plans livrés, source unique à la 47-2a AC-11
  (propagé). H2 : « un rapport présenté vide l'est aussi en brut » est faux (paire à net nul) : `isReportEmpty`
  corrigé. M : invalidation des deux rapports et génération en vol à la bascule ; bandeau brut non déductible du DTO ;
  extraction du texte du PDF, formats des montants à l'écran, jeu semé et équilibré de l'E2E, transmission à l'export
  vérifiée dans les deux sens ; composant unique de ligne (DRY) et sélecteurs fixés ; lien manuel/test tenu par une
  garde. L : renvoi à `ReportSelector`, bascule visible pour toute société, valeur `localStorage` illisible, figures du
  manuel. Rapport : `kesh-gate-logs/epic-47-validation/47-3b-P1.md`.
- **2026-11-19 — validation P2 (Opus 5.5) : 0 C / 0 H / 3 M / 6 L, remédiés.** Passe braquée sur la remédiation P1
  et sa propagation. M : (1) contrôle des exports de l'E2E ambigu — « Présentation compensée » n'est écrite qu'au
  PDF du compte de résultat (paire) ; au bilan (miroirs seuls, 47-2b AC-2) elle est absente, si bien que le contrôle
  aurait été faux en présenté et vide en brut : exports vérifiés rapport par rapport (mention, note « comprend »,
  colonnes `Presentation`/`Sources`) ; (2) garde NFR-47-4 indécidable et en partie vide — « figure au manuel » par
  sous-chaîne (`800.00` dans `3'800.00`), « le test affirme les mêmes » sans fichier ni forme, liste « fixée à la
  47-2a AC-11 » que celle-ci ne fixait pas : blocs marqués au manuel (`bloc_marque`) et au test
  (`EXEMPLE_PRESENTATION`, `reports_e2e.rs`), jetons entiers, témoins ; liste écrite à la 47-2a AC-11 (propagé) ;
  (3) les maquettes M4/M5, citées en référence, portent encore l'ancien jeu (1179, 1021, 2120, 6960, 6961, capital
  9 000.00) et une autre place de la bascule : réduites à la disposition, critères et 47-2a AC-11 font foi (le
  formulaire n'est pas modifié). L : plan PME à nommer (1040, « Capital social » ; propagé à la 47-2a AC-11) ;
  « aucune cellule négative » avant dépliage des sources ; regénération décidée avant l'invalidation ; libellés des
  genres de sources alignés sur la note du PDF de la 47-2b (« comprend », « solde propre », « déduit ») ; total passif
  et résultat net affirmés à l'E2E ; résidu « Créances sociales * » de la 47-2b AC-1 (propagé). Vérifié au code :
  G4-bis/G4-ter (`textes_coherents.rs:414`, `:582`, `nombres_isoles`), plans livrés (types et noms des dix comptes du
  jeu, paires et miroirs de types opposés, D2/D3), arithmétique du jeu, seed `with-company`, rôle à la création
  (`CreateAccountRequest.role`), `formatSwissAmount`, `isReportEmpty`, #1351 et #1327 (PR ouvertes).
  Rapport : `kesh-gate-logs/epic-47-validation/47-3b-P2.md`.
- **2026-11-19 — validation P3 (Sonnet 5.5, passe ciblée sur la remédiation P2) : 0 C / 0 H / 1 M / 1 L, remédiés.**
  M : la garde de l'AC-10 lisait le bloc Rust par `bloc_marque`, qui ne connaît que le préfixe `% ` (`textes_coherents.rs:2167`),
  alors que les marqueurs de `reports_e2e.rs` sont en `// ` (47-2a AC-11) : fonction sœur `bloc_marque_rs` et témoin
  prescrits, lecture du source brut (non normalisé) précisée. L : genre des sources de l'E2E dit « miroir » là où l'écran
  affiche « comprend » (AC-2). Vérifié au code de `origin/main` : `bloc_marque` (existence, signature, erreurs),
  `lire` (source brut), seed `with-company` (`test_fixtures.rs:149-166` : 1000, 1100, 2000 Liability, 3000, 4000 ;
  aucun rôle ; exercice 2020-2030), `CreateAccountRequest.role`, `todayInZurich` (`helpers/api-fixtures`),
  `contenuDuPdf` (`invoice-qr-caracteres.spec.ts:24`), lignes du manuel `:3908`, `:3910`, `:3941`, `:4051`, `:4171`,
  liste de neuf montants et blocs `//` de la 47-2a AC-11, mots « comprend / solde propre / déduit » de la 47-2b
  AC-1. Rien à modifier à la 47-2a (à signaler à son validateur : ses marqueurs `//` supposent la fonction sœur de
  la 47-3b). Rapport : `kesh-gate-logs/epic-47-validation/47-3b-P3.md`.
- **2026-11-19 — validation P4 (Opus 5.5, passe ciblée sur la remédiation P3) : 0 C / 0 H / 0 M / 3 L, remédiés.**
  Vérifié au code de `origin/main` (`textes_coherents.rs`) : `lire` (`:48`, lecture brute, panique si absent),
  `manuels_fr` (`:55`) et `normaliser` (`common/manuel.rs`, efface les sauts de ligne), `bloc_marque` (`:2167`,
  préfixe `% ` codé en dur, erreurs « bloc absent » / « bloc non fermé »), structure de G23 et de son témoin
  `g23_les_fonctions_pures` (`:2215`, textes fabriqués) ; `regex` déjà dépendance de dev de `kesh-api` ; le jeton
  `-?\d+(?:['’]\d{3})*\.\d{2}` rend entiers les neuf montants de la 47-2a AC-11 (`13700.00` … `450.00`), au manuel
  comme au test, et aucun montant à apostrophe du manuel (`1'020.00`, `13'700.00`) n'a quatre chiffres contigus que
  `nombres_isoles` (G4-bis, `:283`) lirait comme un compte ; `reports_e2e.rs` existe. Garde implémentable telle que
  décrite. L1 : numéro de garde et rangée du tableau d'en-tête non prescrits (convention du module) → G26. L2 :
  « `bloc_marque` n'est pas modifiée » poussait à dupliquer son corps (DRY) → corps commun permis, signature et
  comportement inchangés. L3 : lecture brute par `lire` dite du seul manuel → des deux fichiers. Rapport :
  `kesh-gate-logs/epic-47-validation/47-3b-P4.md`.
