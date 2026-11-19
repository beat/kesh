# Story 47-4 : compte miroir — écrans des rapports, parcours E2E, manuel utilisateur

Status: ready-for-dev

Epic : `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md` — § *Décision du 2026-11-19 : miroir seul*,
règles R5 à R7 ; D4 ; FR-47-3 (écran), FR-47-4, FR-47-5 (manuel utilisateur) ; NFR-47-4. **Refonte du 2026-11-19
(miroir seul, KISS)** de l'ancienne 47-3b (fiche renommée). **Dépend des 47-2 et 47-3.**

## Story

En tant que comptable,
je veux voir, au bilan et au compte de résultat, de quel compte vient le solde présenté sous un compte miroir,
afin de comprendre pourquoi un compte sans écriture y paraît.

## Modules touchés (règle de splitting préventif)

Cinq : `frontend/src/lib/features/reports` (deux vues et les types), les quatre catalogues i18n, le manuel utilisateur,
l'E2E (`frontend/tests/e2e/`), la garde `crates/kesh-api/tests/textes_coherents.rs` (G26).

## Critères d'acceptation

1. **Mention sous la ligne du miroir** : dans `BalanceSheetView.svelte` (actifs, passifs) et
   `IncomeStatementView.svelte` (produits, charges), une ligne qui porte `mirrorOf` affiche sous son libellé, en petit
   texte discret, « Solde de 2270 AVS/AI/APG » (plusieurs sources : « Soldes de … », séparées par
   des virgules ; numéros et noms en variables Fluent, **texte seul, sans lien** : les sources n'ont pas de ligne à
   ce rapport, et le compte de résultat ne lie aucun numéro ; une clé Fluent à pluriel sur `$count`, les sources déjà
   jointes en une chaîne). Le numéro de la ligne du miroir reste lié au grand livre comme aujourd'hui (vue du bilan),
   sans traitement propre.
   Une ligne **sans** `mirrorOf` est rendue exactement comme aujourd'hui. Pas de bascule, pas de bandeau, pas de
   dépliage (R6, R7). La mention est rendue par un **petit composant unique** (`MirrorOfNote.svelte`), placé dans la
   cellule du nom (à côté d'`AccountCurrencyBalance`) aux quatre endroits, non par quatre copies.
2. **Types** : `reports.types.ts` — `mirrorOf?: MirrorSource[]` sur `AccountBalance`, type `MirrorSource` (`accountId:
   number`, `accountNumber: string`, `accountName: string`, `balance: string` — chaîne décimale, comme
   `AccountBalance.balance`). `reports.api.ts` et `isReportEmpty` inchangés (un rapport présenté est
   vide si et seulement si le brut l'est : aucune compensation).
3. **Vitest** (dans `BalanceSheetView.test.ts` et `IncomeStatementView.test.ts`, existants) : vues (mention présente sur une ligne à `mirrorOf`, absente sinon, une et plusieurs sources, fonds
   propres jamais concernés), `MirrorOfNote` seul. Mutation à faire rougir : mention rendue sur une ligne sans
   `mirrorOf`.
4. **Sélecteurs d'E2E** (`data-testid`) : `report-row-{numéro}` — **n'existe pas** (aucun `data-testid` aux lignes des
   quatre tableaux) : à poser sur les `<tr>` d'actifs, de passifs, de produits et de charges —, `mirror-of-{numéro}` (la
   mention, numéro du **miroir**) ; la balance garde `tb-row`, sans numéro : l'E2E filtre par le texte du numéro
   (`getByTestId('tb-row').filter({ hasText: … })`).
5. **E2E du parcours** (NFR-47-4, `frontend/tests/e2e/reports-mirror.spec.ts`, `seedTestState('with-company')`,
   `describe.configure({ mode: 'serial' })` comme `reports-currency.spec.ts`) — **l'exemple du fichier d'epic**, semé
   par l'API. ⚠️ Le seed `with-company` ne charge aucun plan livré (cinq comptes ad hoc, `test_fixtures.rs:149-166` :
   1000 « Caisse CI », 1100, 2000 « Capital CI » (passif, sans rôle), 3000, 4000) : l'E2E **crée par l'API** 1010,
   1020, 2270, 2800 (rôle capital), 3200, 6900, 6950.1, et les miroirs 6900.1, 1040.1, 2100.1 (non imputables), tous
   **sans parent** (un parent deviendrait non imputable) ; il
   **n'asserte jamais un libellé de compte semé** ; écritures contre 2800 à la date du jour de Zurich (`todayInZurich()`).
   Parcours : miroirs **posés à l'écran** (dialogue de la 47-2 : 1020 → 2100.1, 2270 → 1040.1, 6950.1 → 6900.1) →
   *Générer* le bilan (onglet `#reports-tab-balance-sheet`, bouton *Générer*) : **aucune cellule de montant négative**
   (cellules `td.text-right` des lignes `[data-testid^="report-row-"]`, AC-4 — non de toute la page, où le tableau des
   fonds propres a les siennes —, au bilan puis au compte de résultat ; aucune ne commence par `-`), 1040.1 à `1’200.00` (actif) avec la mention « Solde de
   2270 », 2100.1 à `800.00` (passif), total actif `13’700.00`, total passif `3’800.00` → compte de résultat : 6900.1 à
   `250.00` en charges avec la mention « Solde de 6950.1 », 6950.1 absent des produits, total produits `2’000.00`, total
   charges `300.00`, résultat `1’700.00` → balance de vérification : 1020 à `-800.00`, 2270 à `-1’200.00`, 6950.1 à
   `-250.00` (soldes réels). **Exports** déclenchés par les boutons (*Export PDF*, *Export CSV*, par leur nom, comme
   `reports-export-pdf.spec.ts`) : le PDF et le CSV du bilan portent 1040.1 et 2100.1 avec leurs montants présentés,
   **dans leur forme propre** — PDF `1'200.00` (apostrophe ASCII U+0027, `format_swiss_amount`, Latin-1 des `Tj`),
   CSV `1200.00` (ISO, sans séparateur, `format_amount_iso`) —, et ni 1020 ni 2270, cherchés comme **jetons entiers**
   (au PDF, la chaîne `<…>` entière d'un `Tj` ; au CSV, un champ entier), non comme sous-chaînes (texte du PDF **extrait** : flux Flate, `Tj` hexadécimal
   en Latin-1 — `contenuDuPdf` et `hexWinAnsi` d'`invoice-qr-caracteres.spec.ts:13-37`, **déplacés** dans `helpers/`
   plutôt que recopiés). ⚠️ Formes **à l'écran seulement** : apostrophe U+2019 des milliers et signe `-` ASCII
   (`formatSwissAmount`) ; ni le PDF ni le CSV n'emploient l'U+2019.
6. **Captures d'écran** (thème clair ; `KESH_CAPTURES_DIR`, patron de `reports-currency.spec.ts`) : bilan présenté,
   compte de résultat présenté, balance ; les figures du manuel en sont tirées (PNG seulement,
   `docs/manual/fr/screenshots/`, patron de l'Epic 37 ; aucun PDF commité).
7. **Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, section *Rapports comptables* `:3908`, sous-sections
   *Bilan* `:3910`, *Compte de résultat* `:3941`) : nouvelle sous-section « Comptes miroirs dans les rapports »
   (`\label{sec:rapports-miroirs}`, renvoyée depuis la sous-section *Compte miroir* de la 47-2, `\ref{sec:compte-miroir}`
   — c'est ici que le renvoi s'écrit dans les deux sens) : ce que montrent bilan et compte de résultat (solde de sens
   inverse présenté sous le miroir, aucune compensation, comme la présentation du bilan de l'art. 959a CO le prévoit), la mention « Solde de … », les
   exports, la balance et le grand livre restés sur les soldes réels, l'accueil sur les soldes réels ; les phrases
   d'ouverture à **corriger** (« les comptes d'actif et de passif à solde non nul, triés par numéro », `:3914-3915`, et son
   pendant du compte de résultat, `:3945-3946`), **et les deux affirmations que le miroir rend fausses** : *Balance des
   comptes*, « La balance concorde avec le bilan et le grand livre » (`:3968-3972` : vrai ligne à ligne sans miroir ;
   avec un miroir, la balance garde le solde de la source, le bilan le présente sous le miroir — les totaux restent
   égaux au résultat près) et l'accueil, « Le total des barres égale le compte de résultat de l'exercice » (`:308` :
   l'accueil reste sur les soldes réels, Q11, le résultat net seul est le même). **Sans dire que le bilan de Kesh est
   « conforme » à l'art. 959a CO** : l'avertissement qui ouvre *Bilan* (« pas encore structuré selon l'art. 959a CO »,
   `:3918`) et celui du compte de résultat (`:3948`) restent vrais ; le miroir ne règle que le signe d'un solde de sens
   inverse, comme la présentation du CO l'attend. **L'exemple chiffré est celui du fichier d'epic, sans autre jeu** : un
   tableau avant/après sur une société au plan **PME** (qu'il nomme), comptes à créer en sous-comptes **sans compte
   parent** (6950.1, 6900.1, 1040.1, 2100.1 ; 47-2 AC-9), comptes livrés sous leur nom (1000 « Caisse », 1010 « Poste », 1020 « Banque », 2000
   « Créanciers », 2270 « AVS/AI/APG », 2800 « Capital social », 3200 « Prestations de services », 6900 « Charges
   financières »), montants au format du manuel (apostrophe : `13'700.00`, jamais un montant de quatre chiffres sans
   apostrophe, que G4-bis lirait comme un compte). ⚠️ G4-bis et G4-ter lisent ce manuel : les exécuter.
8. **Garde G26** (`crates/kesh-api/tests/textes_coherents.rs`, prochain numéro libre à `origin/main` — à recontrôler à
   l'intégration —, avec sa rangée au tableau des gardes de l'en-tête du module ; reprise de l'ancienne 47-3b AC-10,
   validée) : l'exemple du manuel est écrit entre `% exemple-presentation:debut` et `% exemple-presentation:fin` (lu
   par `bloc_marque`, sur le source **brut** lu par `lire`, jamais le texte normalisé) ; le test de la 47-3
   (`reports_e2e.rs`) porte `EXEMPLE_PRESENTATION` entre `// exemple-presentation:debut` et `// …:fin`, lu par une
   **fonction pure sœur** `bloc_marque_rs(source, nom)` (mêmes erreurs : absent, non fermé ; corps commun permis avec
   `bloc_marque`, dont la signature, le comportement et le témoin G23 ne changent pas) ; la garde extrait des deux
   blocs les montants **comme jetons entiers** (`-?\d+(?:['’]\d{3})*\.\d{2}` en `find_iter`, apostrophes retirées
   ensuite, signe ignoré ; jamais une sous-chaîne, que `800.00` satisferait dans `3'800.00`) et exige que **chaque
   montant de la liste figure dans le bloc du manuel** ; témoins sur textes fabriqués : bloc absent ou non fermé en
   erreur, liste d'au moins quatorze montants, un montant retiré du manuel rougit.
9. **i18n** : quatre locales ; `sitesTotal` recompté ; `lint-i18n-ownership` (clés `reports-…`) et
   `i18n-libelle-en-dur` verts ; aucun numéro de compte écrit en dur dans un catalogue ou un repli.
10. **Documentation** : `.tex` seulement ; **le prompt de chaque passe de revue nomme le manuel** et exige le contrôle
    du PDF compilé localement (aplati : `pdftotext … | tr '\n' ' ' | tr -s ' '`).

## Tâches / sous-tâches

- [ ] **T0** : reprise de `main` ; `fix/bilan-resultat-reporte` (#1346) touche `BalanceSheetView.svelte` ; si la 46-2
  (niveau de détail) est fusionnée, la mention vaut aussi pour les lignes `kind: account` des groupes.
- [ ] **T1** : types, `MirrorOfNote`, vues (AC 1, 2, 4). **T2** : Vitest (AC 3).
- [ ] **T3** : E2E (assistant PDF déplacé dans `helpers/`), captures (AC 5, 6).
- [ ] **T4** : manuel, garde G26 et ses témoins, i18n (AC 7–10).
- [ ] **T5** : gates frontend, **backend** (`textes_coherents`) et E2E complets au dernier commit de code.

## Dev Notes

### Code lu (à `origin/main` `fd04099f`, relevé de l'ancienne 47-3b)

- `frontend/src/lib/features/reports/BalanceSheetView.svelte` (trois répétitions de ligne, `ledgerHref`),
  `IncomeStatementView.svelte` (deux répétitions, numéro sans lien), `reports.types.ts:14-60`, `reports.api.ts:138`
  (`isReportEmpty`), `:260` (`formatReportAmount`).
- `crates/kesh-api/tests/textes_coherents.rs:48` (`lire`), `:283` (`nombres_isoles`), `:414` (G4-bis), `:582`
  (G4-ter), `:2167` (`bloc_marque`, préfixe `% ` codé en dur), `:2215` (`g23_les_fonctions_pures`).
- `frontend/tests/e2e/invoice-qr-caracteres.spec.ts:13-37`, `reports-currency.spec.ts`,
  `crates/kesh-db/src/test_fixtures.rs:149-166`.

- Formes des montants : écran `formatSwissAmount` (U+2019, `-`) ; PDF `kesh-report/src/pdf.rs:274`
  (`format_swiss_amount`, apostrophe U+0027) ; CSV `csv.rs:50` (`{amount:.2}`) ; même bibliothèque `printpdf 0.7` pour le
  PDF des factures et celui des rapports (le texte se lit en `Tj` hexadécimal dans les deux). Testids existants :
  `account-fx-{numéro}`, `tb-row`, `tb-fx-row-{numéro}`. Manuel : figures `docs/manual/fr/screenshots/` (préfixes `md-`,
  `rd-` de l'Epic 37 ; préfixe `mi-` ici).

### Choix KISS (refonte du 2026-11-19)

- **C-47-4-1** : une mention en petit texte sous la ligne du miroir. *Écarté* : composant `PresentedRow` à pastille
  « miroir / compensé », chevron et ligne dépliée des sources, bandeaux « présentation compensée » et « présentation
  brute », bascule *Présentée · Brute* persistée en `localStorage`, transmission aux exports.
- **C-47-4-3** (validation P1) : mention en texte seul, sans lien vers le grand livre (les sources n'ont pas de ligne
  au rapport ; le compte de résultat ne lie aucun numéro). *Écarté* : numéro de source lié, prop de lien, deux
  comportements selon la vue.
- **C-47-4-2** : l'E2E vérifie la présentation à l'écran, aux exports et la balance réelle. *Écarté* : les contrôles
  croisés présenté / brut des exports (plus de bascule).

### Risques

Conflits avec #1346 et la 46-2 dans les mêmes vues ; extraction du texte du PDF ; G4-bis sur l'exemple du manuel.

### References

- Fiches 47-2 (dialogue, label `sec:compte-miroir`), 47-3 (AC-10, `EXEMPLE_PRESENTATION`) ; fichier d'epic (§ *Exemple
  chiffré*) ; `CLAUDE.md` § *Recette visuelle*, § *Le prompt d'une passe doit NOMMER le manuel*.
- Registre de la dette du fichier d'epic : à la clôture de la boucle de revue, tout finding non remédié y est consigné ;
  la 47-5 en part (« on trace toujours »).

## Notes de version

Au bilan et au compte de résultat, la ligne d'un compte miroir dit de quel compte vient son solde (« Solde de … ») ;
le manuel utilisateur décrit les comptes miroirs, avec un exemple chiffré.

## Change Log

- 2026-11-19 — validation P3 (Sonnet 5.5, ciblée sur la remédiation P2) : **0 C / 0 H / 0 M / 1 L, remédié**. Relu au
  code d'`origin/main` `fd04099f` : AC-7 sans « conforme à l'art. 959a CO » (la parenthèse ne dit plus que « comme la
  présentation du bilan de l'art. 959a CO le prévoit », les avertissements `:3918` et `:3948` intacts), sous-comptes
  **sans parent** (E2E AC-5, manuel AC-7, d'accord avec le fichier d'epic et la 47-2 AC-9), `MirrorSource.balance: string`,
  test « aucune cellule négative » borné aux `td.text-right` des lignes `report-row-` (les `<tr>` de `BalanceSheetView`
  `:131`, `:170`, `:225` n'ont aucun testid ; les lignes de fonds propres n'en reçoivent pas), exports : le numéro de
  compte est un `use_text` propre dans `pdf.rs` (`:565`, `:1030`) et un champ propre dans `csv.rs` (`:111`, `:410`), donc
  « jeton entier » est testable ; formes des montants (`format_swiss_amount`, `format_amount_iso`), `contenuDuPdf` /
  `hexWinAnsi` (`invoice-qr-caracteres.spec.ts:24`, `:40`), G26 libre (G25 dernière). L : lignes citées des phrases
  d'ouverture, `:3914` et `:3945` (début de la phrase) corrigées en `:3914-3915` et `:3945-3946` (« d'actif et de passif
  à solde non nul », « et les comptes de charges à solde non nul »). Rapport :
  `kesh-gate-logs/epic-47-validation/47-4-miroir-ecrans-rapports-manuel-e2e-P3.md`.
- 2026-11-19 — validation P2 (Opus 5.5, complète, contexte frais) : **0 C / 0 H / 1 M / 4 L, remédiés**. Remédiation P1
  relue au code d'`origin/main` `fd04099f` : manuel `:308`, `:3908`-`:3918`, `:3941`-`:3948`, `:3968` (lignes exactes) ;
  `format_swiss_amount` (`kesh-report/src/pdf.rs:274`, apostrophe U+0027), `format_amount_iso` (`csv.rs:50`) ;
  `contenuDuPdf` et `hexWinAnsi` (`invoice-qr-caracteres.spec.ts:24`, `:40`), absents de `helpers/` ; aucun
  `data-testid` aux lignes de `BalanceSheetView`/`IncomeStatementView`, `tb-row` sans numéro (`TrialBalanceView.svelte:65`),
  `#reports-tab-…` (`reports/+page.svelte:598`, `reports-currency.spec.ts:167`) ; G26 libre (G25 dernière) ; exemple
  chiffré du fichier d'epic recalculé, montants de l'E2E exacts. M1 : l'AC-7 disait encore « conforme à l'art. 959a CO »
  dans sa parenthèse, contre sa propre phrase « sans dire que le bilan de Kesh est conforme » (résidu de la P1). L :
  comptes à créer **sans parent** (E2E et manuel ; propagation de la 47-2 P2 M1 : un parent devient non imputable) ;
  types de `MirrorSource` (`balance: string`) ; « aucune cellule négative » bornée aux lignes `report-row-` ; 1020 et
  2270 absents cherchés en jetons entiers. Concordance : 47-3 à HEAD (exports par le service, `mirrorOf`, quatorze
  montants entre marqueurs `//`) et 47-2 (dialogue, label) concordantes. Rapport :
  `kesh-gate-logs/epic-47-validation/47-4-miroir-ecrans-rapports-manuel-e2e-P2.md`.
- 2026-11-19 — validation P1 (Sonnet 5.5) : 0 C / 0 H / 4 M / 4 L, remédiés. M : deux affirmations du manuel que le miroir
  rend fausses, hors AC-7 (`:3968` « la balance concorde avec le bilan », `:308` « le total des barres égale le compte de
  résultat ») ; « conforme à l'art. 959a CO » contre les avertissements `:3918` et `:3948` ; formes des montants aux
  exports non dites (PDF `1'200.00`, CSV `1200.00`, U+2019 à l'écran seulement) ; sélecteurs d'E2E (`report-row-` n'existe
  pas, `tb-row` sans numéro, test « aucune cellule négative »). L : mention sans lien au grand livre (KISS) ; Vitest dans
  les fichiers existants ; place du composant ; préfixe des figures. Rapport :
  `kesh-gate-logs/epic-47-validation/47-4-miroir-ecrans-rapports-manuel-e2e-P1.md`.
- 2026-11-19 — **Refonte du 2026-11-19 (miroir seul, KISS)** : fiche renommée de `47-3b-ecrans-rapports-e2e-manuel`.
  Retirés : bascule *Présentée · Brute* et sa persistance, `PresentedRow` à pastille et dépliage, bandeaux compensé et
  brut, changement d'`isReportEmpty`, mention « Présentation compensée », colonnes `Presentation` / `Sources` des
  exports, ancien exemple (paire). Gardés : E2E du parcours sur le seed `with-company` avec comptes créés par l'API,
  extraction du texte du PDF déplacée dans `helpers/`, captures et figures du manuel, sous-section du manuel et
  phrases d'ouverture à corriger, garde G26 (`bloc_marque_rs`, jetons entiers, témoins), contraintes G4-bis / G4-ter.
  Dix critères (onze avant). **Statut remis à `drafted`** en attendant la revalidation.

### Historique — Change Log de la 47-3b (avant la refonte, laissé tel quel)

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


## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
