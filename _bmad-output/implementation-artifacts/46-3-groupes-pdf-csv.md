# Story 46-3 : PDF et CSV du bilan et du compte de résultat par groupes

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-46-rapports-groupes.md` — D8, D9 (étendue), Q2, Q3 ; FR-46-3, FR-46-8,
FR-46-9, FR-46-11 ; NFR-46-1, NFR-46-2. Release visée **0.21.0** (prévision). Approuvée par Beat et par Guy (revue 1,
2026-11-19) ; fiche écrite le 2026-11-20, à valider en boucle. **Suit la 46-1 ; commence après la fusion de la 48-11
dans `main`** (arrondi au centime, Epic 48, release 0.20.0) — la PR #1327 (PDF aligné) est déjà fusionnée. Aucune
migration.

## Story

En tant que comptable,
je veux imprimer et exporter le bilan et le compte de résultat **tels que je les vois à l'écran** — même niveau, mêmes
groupes ouverts ou fermés, mêmes sous-totaux —,
afin de remettre un bilan par groupes à ma fiduciaire ou de le retravailler dans un tableur.

## Modules touchés (règle de splitting préventif)

Cinq : `kesh-report/pdf`, `kesh-report/csv`, `kesh-api/routes/reports` (paramètres des deux exports),
la documentation (`docs/api-external.md`, `docs/manual/fr/user-manual.tex`), les tests d'intégration
(`crates/kesh-api/tests/reports_export_e2e.rs`). Pas de `kesh-i18n` : les libellés du PDF sont ceux de
`PdfContext::fr_ch_default` (`crates/kesh-report/src/pdf.rs`, ~`:200`, FR-CH en dur, i18n des PDF différée).

## Critères d'acceptation

1. **AC-1 — Paramètres de l'export.** `GET /reports/balance-sheet/export` et `/income-statement/export`
   (`routes/reports.rs:374`, `:504`) acceptent `detailLevel`, `subtotalPosition`, `open`, `closed`, avec la **même
   validation** que la 46-1 (même fonction du service, aucun double) et les mêmes refus `400 VALIDATION_ERROR`, pour
   `format=pdf` comme `format=csv`.
2. **AC-2 — Sans paramètre (NFR-46-1).** CSV **identique octet pour octet** et PDF aux **mêmes lignes de texte**
   (texte extrait, `pdftotext` ou l'extracteur des tests existants) qu'avant la story, comparés à des fixtures écrites
   **au commit de base** (plan PME, compte en devise compris, deux exercices). Les sous-titres de rôle des fonds propres
   restent en mode à plat seulement.
3. **AC-3 — PDF fidèle à l'écran (Beat ; remarque 4 de Guy).** Le PDF rend `groupedRows` dans l'ordre reçu, **avec la
   règle de rendu de la 46-1** : retrait par `depth` (pas fixe, borné pour que le nom ne recouvre pas les montants,
   troncature existante `truncate_with_ellipsis`) ; ligne `group` en gras, montant sur le titre en `header` ou si
   fermé, sinon sur sa ligne `subtotal` (« Total <nom du groupe> », gras) ; `own` « <nom> (solde propre) » ; devise et
   solde en devise d'une ligne de groupe à la place prévue par 37-4 (colonnes en devise si une ligne en porte) ;
   **garder-avec-la-suite** : un titre de groupe ouvert ne reste pas seul en bas de page
   (`ensure_space_for_rows(2.0)`, comme les sous-titres de rôle). En-tête : une ligne sous la période dit le niveau
   (« Niveau 2 » ou « Tous les comptes ») et, s'il y a des écarts manuels, « groupes ouverts : 1020, … ; fermés : … »
   (numéros des comptes de groupe, tronqués à une ligne avec « … »). Totaux de section, lignes calculées et équation :
   inchangés.
4. **AC-4 — Fidélité prouvée.** Un test d'intégration demande le JSON (46-1) et le PDF avec **les mêmes paramètres**
   (niveau 2, un groupe ouvert à la main, un fermé, `footer`), et affirme que la suite des lignes de texte du PDF entre
   l'en-tête et les totaux est exactement celle des `groupedRows` (numéro, nom, montant formaté) — sonde négative :
   ignorer `open` dans le PDF rougit le test.
5. **AC-5 — CSV groupé.** En mode groupé seulement, deux colonnes après `Section` : `Niveau` (la `depth`) et
   `TypeLigne` (`groupe`, `compte`, `propre`, `soustotal`) ; une ligne par élément de `groupedRows`, montant dans
   `Solde` selon la règle de rendu (vide sur un titre de groupe ouvert en `footer`) ; colonnes en devise comme
   aujourd'hui (`Devise;SoldeDevise` si une ligne en porte) ; lignes de total de section et lignes calculées
   inchangées. En-tête de colonnes en mode groupé documenté dans `docs/api-external.md`.
6. **AC-6 — Arrondi (contrainte de Guy, 48-11).** Tout montant écrit passe par les fonctions de `kesh-core` posées
   par la 48-11 (`round_for_display`, `format_two_decimals`, ou les formateurs de `pdf.rs`/`csv.rs` qu'elle aura
   rebranchés) ; **aucun `{:.2}` neuf** dans `kesh-report` : la garde de la 48-11 reste verte. Les sous-totaux arrivent
   exacts de la 46-1 et ne s'arrondissent qu'à l'écriture ; un test sur un groupe de deux comptes à `10.005` et
   `5.006` affirme le sous-total imprimé `15.01` (arrondi de la somme, non somme des arrondis — écart assumé avec la
   somme visuelle des lignes, écrit au manuel).
7. **AC-7 — Pilotable sans écran (FR-46-11).** Tout ce que montre le PDF de l'écran s'obtient par les paramètres de
   l'export seuls ; le MCP (46-4) n'exporte pas de fichier et n'est pas concerné.
8. **AC-8 — Tests et documentation.** Tests unitaires de rendu (texte extrait) pour `header`, `footer`, groupe fermé,
   `own`, devise de groupe, saut de page avec titre gardé ; intégration (`reports_export_e2e.rs`) des refus, des
   fixtures sans paramètre (AC-2) et de l'AC-4 ; `docs/api-external.md` (paramètres de l'export, colonnes CSV) ;
   manuel utilisateur FR, § *Visualisation et exports* (`user-manual.tex:4188`) : export groupé, fidélité à l'écran,
   affichage de travail (lecture (a) de l'art. 959a CO), `.tex` seul.

## Tâches / sous-tâches

- [ ] **T0** — Reprise de `main` : la 48-11 doit y être (sinon **arrêt**, le dire au Change Log) ; relever ses noms de
      fonctions exacts.
- [ ] **T1 — fixtures** (AC-2) au commit de base, avant toute modification.
- [ ] **T2 — route** (AC-1) : `ExportQuery` (`routes/reports.rs`) gagne les quatre champs ; validation par la fonction
      du service de la 46-1.
- [ ] **T3 — PDF** (AC 3, 4, 6) : `render_balance_sheet_pdf` (`pdf.rs:816`), `render_income_statement_pdf`
      (`:954`) — branche groupée, branche à plat inchangée ; libellés neufs dans `SectionLabels` et
      `fr_ch_default` ; une fonction de rendu de ligne groupée partagée par les deux rapports **et** par la balance
      (46-6b).
- [ ] **T4 — CSV** (AC 5, 6) : `render_balance_sheet_csv` (`csv.rs:143`), `render_income_statement_csv` (`:277`).
- [ ] **T5 — tests, documentation** (AC 8).
- [ ] **T6 — gate complet** ; E2E complet au dernier commit de code.

## Dev Notes

### État du code (relevé le 2026-11-20 à `29b9ec27`, après #1327 et #1346)

- `pdf.rs:84-97` `PdfContext`, `:102` `SectionLabels` (libellés FR-CH en dur), `:316` `format_swiss_amount`
  (contient aujourd'hui un `{amount:.2}`, `:318` — c'est le site que la 48-11 corrige), `:349` `round_for_display`
  (`pub(crate)`, déplacé par la 48-11), `:739` `any_foreign`, `:744` `draw_account_row` (nom tronqué si devise),
  `:816-950` bilan (sous-titres de rôle avec garder-avec-la-suite `:855-870`, lignes calculées, équation en pied),
  `:954` compte de résultat. Montants alignés à droite (`AMOUNT_RIGHT_MM`, `FX_AMOUNT_RIGHT_MM`, #1327).
- `csv.rs:42` `make_writer` (`;`, CRLF, BOM), `:50` `format_amount_iso` (`{amount:.2}` — site de la 48-11), `:101`
  `write_account_balance`, `:143-270` bilan (garde par section, `Total actifs;;;…`), `:277` compte de résultat.
- `routes/reports.rs:174` `ExportQuery`, `:374` `export_balance_sheet`, `:504` `export_income_statement`,
  `:942` `PdfContext::fr_ch_default`.

### Ce qui doit rester

Les exports sans paramètre (AC-2), les exports des autres rapports, la ligne « Résultat reporté (calculé) » omise à
zéro (#1346), l'alignement (#1327).

### Choix et écarts

- **C-46-3-1** : le PDF imprime l'état d'ouverture (niveau et écarts manuels) pour qu'un lecteur sache ce qui est
  replié — décision 4 de l'orchestrateur, « le PDF imprime le niveau et la liste des groupes ouverts/fermés ».
- **C-46-3-2** : sous-total arrondi une fois, sur la somme exacte (AC-6), comme les totaux de section aujourd'hui.

### References

- Fiche 46-1 (§ *Le contrat*) ; fiche 48-11 et `epic-48-corrections-0-20-0.md` § *48-11* (garde `{:.2}`) ;
  `CLAUDE.md` § *Le prompt d'une passe doit nommer le manuel* (PDF compilé localement, aplati avant `grep`).

## Note de version

*(à écrire au développement)* — Les exports PDF et CSV du bilan et du compte de résultat suivent l'écran : niveau,
groupes ouverts ou fermés, sous-totaux sur le titre ou en pied ; le PDF dit le niveau imprimé. Sans paramètre,
l'export ne change pas.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-20 — Fiche écrite depuis l'esquisse : fidélité à l'écran prouvée par test (remarque 4 de Guy), niveau et
  écarts imprimés, règle de rendu de la 46-1, dépendance à la 48-11 et interdiction du `{:.2}` (contrainte de Guy).
  Huit critères. `drafted`.
