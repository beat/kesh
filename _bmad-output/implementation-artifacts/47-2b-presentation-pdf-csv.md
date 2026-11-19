# Story 47-2b : présentation au bilan et au compte de résultat — PDF et CSV

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md` — D6, D9 ; Q9 ; FR-47-4 (PDF, CSV), FR-47-6
(mention « présentation compensée »), FR-47-8. Née du découpage de l'esquisse 47-2 (2026-11-19). Dépend de la
**47-2a** et de la fusion dans `main` de **#1327** (`fix/pdf-montants-alignes`, réécriture de `pdf.rs`) et de
**`fix/bilan-resultat-reporte`** (#1346, `pdf.rs`, `csv.rs`) ; si la **46-3** est fusionnée, ses rendus groupés
reçoivent la même marque.

## Story

En tant que comptable,
je veux que les exports PDF et CSV du bilan et du compte de résultat suivent la présentation choisie, en disant
clairement quand une ligne est présentée ou compensée,
afin de remettre un document lisible sans masquer ce qui a été regroupé.

## Critères d'acceptation

1. **PDF présenté** (`render_balance_sheet_pdf`, `render_income_statement_pdf`) : une ligne qui porte des `sources`
   reçoit une **marque** (appel de note : `*` pour un miroir, `‡` pour une paire) ; une **note de bas de rapport**
   liste, par ligne marquée, ses sources (numéro, libellé, solde brut) — p. ex. « 1179 Créances sociales :
   comprend 2270 Dettes sociales, solde −1 200.00 ».
2. **« Présentation compensée »** (Q9, D9) : quand le rapport porte `compensated`, le PDF affiche en clair, sous le
   titre, la mention **« Présentation compensée »** suivie d'une phrase : les paires marquées ‡ sont présentées
   nettes ; CO 958c al. 1 ch. 7 ; la présentation brute est disponible pour les comptes annuels officiels. Absente
   sinon.
3. **PDF brut** : avec `presentation=brute`, le PDF est **identique** à celui d'aujourd'hui (mêmes lignes de texte,
   `pdftotext`), sans marque ni mention.
4. **CSV** (`render_balance_sheet_csv`, `render_income_statement_csv`) : seulement si au moins une ligne porte des
   `sources`, deux colonnes s'ajoutent en fin d'en-tête — `Presentation` (`miroir` | `compense`) et `Sources`
   (`2270:-1200.00|…`) — comme `Devise;SoldeDevise` (37-4) ; sinon, et toujours en `brute`, le CSV est **identique
   octet pour octet** à aujourd'hui (BOM, `;`, CRLF).
5. **Libellés** : les textes du PDF (marque, note, mention) viennent de `SectionLabels` / du contexte i18n
   (`load_pdf_context`), quatre locales ; le CSV garde ses en-têtes FR-CH en dur (limitation L1/L4/L11, D3 de la
   9-2a), valeurs `miroir` / `compense` sans accent.
6. **Montants** : arrondis au centime comme le reste du rapport (règle de la 48-11 si fusionnée, sinon celle en
   vigueur) ; une ligne présentée nulle après arrondi ne paraît pas.
7. **Groupes (si la 46-3 est fusionnée)** : la marque et la note s'appliquent aux lignes de compte des rendus groupés ;
   un sous-total de groupe n'est jamais marqué (il agrège des lignes déjà présentées).
8. **Documentation (exigence de Guy, 2026-11-19)** : `docs/api-external.md` — colonnes CSV `Presentation`, `Sources`,
   mention « présentation compensée » du PDF ; manuel d'administration — exports présentés et bruts. `.tex`
   seulement ; **le prompt de chaque passe de revue nomme ces documents** et exige le contrôle du PDF compilé
   localement, ainsi que celui d'un PDF de rapport réel (`pdftotext … | tr '\n' ' ' | tr -s ' '`).

## Tâches / sous-tâches

- [ ] **T0** : reprise de `main` (#1327 et #1346 fusionnés — sinon la story attend) ; lire la mise en page de #1327
  (colonnes de montants alignées, en-têtes de devise) avant de placer la marque.
- [ ] **T1 — PDF** (AC 1–3, 5, 6) : marque, note, mention ; libellés.
- [ ] **T2 — CSV** (AC 4).
- [ ] **T3 — groupes** (AC 7), si la 46-3 est là.
- [ ] **T4 — tests** : unitaires de rendu (texte du PDF, octets du CSV) en présenté, en brut et sans lien ;
  `reports_e2e.rs` (export de l'exemple chiffré, les deux présentations).
- [ ] **T5 — documentation** (AC 8). **T6 — gate complet**, E2E complet au dernier commit de code.

## Dev Notes

### Code lu (à `origin/main` `fd04099f`)

- `crates/kesh-report/src/pdf.rs:66` (`PdfContext`), `:84` (`SectionLabels`), `:636` (`render_balance_sheet_pdf`),
  `:766` (`render_income_statement_pdf`) — **réécrit par #1327 (+1117 lignes)** : les numéros de ligne seront à
  relire après sa fusion.
- `crates/kesh-report/src/csv.rs:135-175` (`render_balance_sheet_csv`, colonnes de devise conditionnelles
  `write_fx_record`), `:271` (`render_income_statement_csv`).
- `crates/kesh-api/src/routes/reports.rs:374-420` (`export_balance_sheet` → `rapport.exporter`), `:928`
  (`load_pdf_context`).
- `fix/bilan-resultat-reporte` (non fusionné) : `BalanceSheet::shows_retained_earnings_line` utilisé par PDF et CSV —
  conflit de texte attendu dans les deux rendus.

### Choix proposés

- **C-47-2b-1** : deux symboles distincts (miroir `*`, paire `‡`) pour que la mention CO 958c ne vise que les paires
  (Guy : « le miroir ne compense rien »).
- **C-47-2b-2** : colonnes CSV conditionnelles plutôt qu'une ligne de commentaire (`#`), qu'un tableur lirait comme
  une donnée.

### Stratégie de tests

Rendus comparés : brut = actuel (PDF : lignes de texte ; CSV : octets) ; présenté : marque, note, mention ; sans lien :
identique. Mutations : mention affichée pour un miroir seul ; colonnes CSV présentes sans ligne présentée.

### Risques

Dépendance à deux correctifs non fusionnés qui réécrivent les mêmes fonctions ; taille de la note sur un rapport à
nombreuses sources (saut de page à tester).

### References

- Fiche 47-2a ; release 0.20.0 § *Correctifs de Beat* (#1327 avant la 48-11) ; `CLAUDE.md` § *Le prompt d'une passe
  doit NOMMER le manuel*.

## Notes de version

Les exports PDF et CSV du bilan et du compte de résultat suivent la présentation : les lignes présentées sont
marquées et leurs sources notées ; un rapport où une paire s'applique porte la mention **« Présentation compensée »**.
L'export brut reste identique à celui d'avant.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
