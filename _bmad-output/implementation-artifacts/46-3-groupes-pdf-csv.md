# Story 46-3 : PDF et CSV du bilan et du compte de résultat par groupes

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-46-rapports-groupes.md` — D8, D9 (étendue), Q2, Q3 ; FR-46-3, FR-46-8,
FR-46-9, FR-46-11 ; NFR-46-1, NFR-46-2. Release visée **0.21.0** (prévision). Approuvée par Beat et par Guy (revue 1,
2026-11-19) ; fiche écrite le 2026-11-20, à valider en boucle. **Suit la 46-1 ; commence après la fusion de la 48-11
dans `main`** (arrondi au centime, Epic 48, release 0.20.0) — la PR #1327 (PDF aligné) est déjà fusionnée. Aucune
migration. **Corrigée le 2026-11-21, avant développement**, sur l'API réelle de la 46-1 (relevée sur `story/46-1` à
`de0a582f` : § *L'API de la 46-1*, Dev Notes) et sur la présentation miroir de la 47-3 (`story/47-3` à `b4d17d70`) ;
validation à reprendre (nouvelle boucle).

## Story

En tant que comptable,
je veux imprimer et exporter le bilan et le compte de résultat **tels que je les vois à l'écran** — même niveau, mêmes
groupes ouverts ou fermés, mêmes sous-totaux —,
afin de remettre un bilan par groupes à ma fiduciaire ou de le retravailler dans un tableur.

## Modules touchés (règle de splitting préventif)

Cinq : `kesh-report/pdf`, `kesh-report/csv`, `kesh-api/routes/reports` (paramètres des deux exports),
la documentation (`docs/api-external.md`, `docs/manual/fr/user-manual.tex`), les tests d'intégration
(`crates/kesh-api/tests/reports_export_e2e.rs`). Pas de `kesh-i18n` : les libellés du PDF sont ceux de
`PdfContext::fr_ch_default` (`crates/kesh-report/src/pdf.rs:294`, et `SectionLabels::fr_ch_defaults` `:204`, FR-CH en
dur, i18n des PDF différée).

## Critères d'acceptation

1. **AC-1 — Paramètres de l'export.** `GET /reports/balance-sheet/export` et `/income-statement/export`
   (`export_balance_sheet`, `export_income_statement`, `routes/reports.rs`) acceptent `detailLevel`, `subtotalPosition`,
   `open`, `closed`, pour `format=pdf` comme `format=csv`, **lus comme aux routes JSON de la 46-1** : un second
   extracteur `Query<Vec<(String, String)>>` et `groupes_de(&paires)` (même module, donc sans changement de visibilité),
   qui délègue à `DemandeGroupes::depuis_requete` puis `depuis_valeurs` du service — aucune validation recopiée.
   **Aucun champ n'est ajouté à `ExportQuery`** (revue P2 de la 46-1, L-6) : partagé avec les exports de la balance,
   des journaux et de la TVA, un champ `Option<String>` y ferait d'un paramètre répété un rejet de désérialisation
   d'Axum, sans `VALIDATION_ERROR`. Mêmes refus `400 VALIDATION_ERROR` que la 46-1, **paramètre répété compris**.
   Ordre des contrôles : `format` (`validate_format`, comme aujourd'hui), puis l'accès
   (`acces.exiger(rapports::EXIGENCES_BILAN.export)`, resp. `EXIGENCES_COMPTE_DE_RESULTAT.export`), puis les groupes —
   l'ordre des routes JSON (revue P1 de la 46-1, L-5) ; inobservable aujourd'hui (l'export est ouvert à tout rôle,
   `EXPORT_SANS_MCP`), donc sans test d'ordre. Les deux handlers appellent `rapports::bilan_par_groupes` et
   `rapports::compte_de_resultat_par_groupes` (au lieu de `bilan` et `compte_de_resultat`) avec cette demande : la
   présentation miroir de la 47-3, insérée dans ces deux fonctions avant le regroupement (46-1 AC-9), vaut donc pour
   l'export comme pour l'écran. Les exports de la balance, des journaux et de la TVA ne lisent pas ces paramètres
   (inchangés ; la balance groupée exportée est la 46-6b).
2. **AC-2 — Sans paramètre (NFR-46-1).** CSV **identique octet pour octet** et PDF aux **mêmes lignes de texte**
   (texte extrait, `pdftotext` ou l'extracteur des tests existants) qu'avant la story, comparés à des fixtures écrites
   **au commit de base** (plan PME, compte en devise compris, deux exercices). Les sous-titres de rôle des fonds propres
   restent en mode à plat seulement. Sans `detailLevel`, `groupes_de` rend `None` : le service n'ajoute aucune requête
   (46-1 AC-10) et `grouping` est absent, ce qui fait choisir le rendu à plat.
3. **AC-3 — PDF fidèle à l'écran (responsable de l'epic ; remarque 4 du Project Lead).** Le PDF rend les lignes de
   `grouped_rows` (`SectionsBilan { assets, liabilities, equity }` ou `SectionsResultat { revenues, expenses }`, des
   `Vec<LigneRapport>`) dans l'ordre reçu, **avec la règle de rendu de la 46-1**, lue sur les champs réels de
   `LigneRapport` (`kind`, `depth`, `account_number`, `account_name`, `balance`, `currency`, `balance_currency`,
   `open`) : retrait par `depth` (pas fixe, borné pour que le nom ne recouvre pas les montants,
   troncature existante `truncate_with_ellipsis`) ; ligne `GenreLigne::Groupe` en gras, montant sur le titre si
   `grouping.subtotal_position` vaut `PositionSousTotal::Titre` ou si `open == Some(false)`, sinon sur sa ligne
   `GenreLigne::SousTotal` (« Total <nom du groupe> », gras ; même `account_name` et `balance` que la ligne du groupe) ;
   `GenreLigne::Propre` « <nom> (solde propre) » ; `GenreLigne::Compte` comme un compte d'aujourd'hui (la ligne d'un
   compte miroir de la 47-3 comprise : rien de propre au miroir, 47-3 C-47-3-3) ; devise et
   solde en devise d'une ligne de groupe à la place prévue par 37-4 (colonnes en devise si une ligne en porte) ;
   **garder-avec-la-suite** : un titre de groupe ouvert ne reste pas seul en bas de page
   (`ensure_space_for_rows(2.0)`, comme les sous-titres de rôle, `pdf.rs:877`). En-tête : une ligne sous la période
   dit le niveau (`grouping.detail_level` : `NiveauDetail::Niveau(2)` → « Niveau 2 », `NiveauDetail::Tous` → « Tous
   les comptes ») et, si `grouping.open` ou `grouping.closed` — les listes **retenues** par l'écho de la 46-1,
   identifiants ignorés retirés — n'est pas vide, ajoute « , détail ajusté à la main » — sans liste de numéros
   (C-46-3-1). Totaux de section, lignes calculées et
   équation : inchangés. **Rapport vide** : les fonctions groupées reprennent la garde « rapport vide » des fonctions à
   plat (bilan : `BalanceSheet::is_empty`, message « rapport vide » au PDF, garde par section au CSV ; compte de résultat : `revenues` et `expenses` tous deux vides, `pdf.rs:967`, `csv.rs:296`), car l'écran
   groupé étant celui par défaut (46-2 AC-1), l'export d'un bilan vide part désormais avec `detailLevel=all`.
4. **AC-4 — Fidélité prouvée.** Un test d'intégration demande le JSON (46-1) et le PDF avec **les mêmes paramètres**
   (niveau 2, un groupe ouvert à la main, un fermé, `footer`), et affirme que, **section par section**, la suite des
   lignes de texte du PDF entre le titre de la section et sa ligne de total est exactement celle des
   `groupedRows.<section>` (numéro, nom, montant formaté selon la règle de rendu) ; pour **Capitaux propres**, la
   suite attendue est `groupedRows.equity` **suivie** des lignes calculées que le rendu à plat écrit déjà entre les
   comptes et le total (`pdf.rs:896-925` : « Résultat reporté (calculé) » si `shows_retained_earnings_line()`, puis
   « Résultat de l'exercice », toujours ; hors de `groupedRows`, 46-1 D4), puis le total. Comparaison sur le **texte
   normalisé** : retrait de début de ligne et espaces répétés réduits ; le jeu d'essai n'a que des noms de comptes
   plus courts que la limite de troncature (`ACCOUNT_NAME_MAX_CHARS_FX`, 34 caractères, `pdf.rs:718`), ce que le test
   affirme d'abord, si bien qu'aucun « … » n'entre dans la comparaison — sonde négative :
   ignorer `open` dans le PDF rougit le test. **Si la 47-3 est dans la branche d'epic au T0**, le même test pose aussi
   un compte miroir (sans parent) qui reçoit un solde, et un second miroir rangé sous un groupe d'**un autre type** que
   le sien (état admis, avertissement non bloquant : décision du responsable de l'Epic 47 du 2026-11-21) : le groupe et
   ses ancêtres sont dissous dans le JSON (46-1 AC-4) et le PDF les rend de même, sans mention propre (C-46-3-3) ;
   sinon ce cas revient au recâblage par le second arrivé (fiche 47-3, § *Recâblage*).
5. **AC-5 — CSV groupé.** En mode groupé seulement, deux colonnes après `Section` : `Niveau` (la `depth`) et
   `TypeLigne` (`groupe`, `compte`, `propre`, `soustotal`, pour `kind` = `group`, `account`, `own`, `subtotal`) ; une ligne par élément de `groupedRows`, montant dans
   `Solde` selon la règle de rendu (vide sur un titre de groupe ouvert en `footer`) ; colonnes en devise comme
   aujourd'hui (`Devise;SoldeDevise` si une ligne en porte) ; lignes de total de section et lignes calculées
   inchangées ; ni `mirrorOf` ni `role` ne sont exportés (47-3 C-47-3-3). En-tête de colonnes en mode groupé documenté
   dans `docs/api-external.md`.
6. **AC-6 — Arrondi (contrainte de Guy, 48-11).** Tout montant écrit passe par les fonctions de `kesh-core` posées
   par la 48-11 (`round_for_display`, `format_two_decimals`, ou les formateurs de `pdf.rs`/`csv.rs` qu'elle aura
   rebranchés) ; **aucun `{:.2}` neuf** dans `kesh-report` : la garde de la 48-11 reste verte. Les sous-totaux arrivent
   exacts de la 46-1 et ne s'arrondissent qu'à l'écriture ; un test sur un groupe de deux comptes à `10.005` et
   `5.006` affirme le sous-total imprimé `15.01` (arrondi de la somme, non somme des arrondis — écart assumé avec la
   somme visuelle des lignes, écrit au manuel).
7. **AC-7 — Pilotable sans écran (FR-46-11).** Tout ce que montre le PDF de l'écran s'obtient par les paramètres de
   l'export seuls ; le MCP (46-4) n'exporte pas de fichier et n'est pas concerné.
8. **AC-8 — Tests et documentation.** Tests unitaires de rendu (texte extrait) pour `header`, `footer`, groupe fermé,
   `own`, devise de groupe, saut de page avec titre gardé, rapport vide groupé ; intégration (`reports_export_e2e.rs`)
   des refus (ceux de la 46-1, sur les deux exports et les deux formats, **paramètre répété compris**), d'un paramètre
   de groupes répété **ignoré** par l'export de la balance (inchangé jusqu'à la 46-6b), des fixtures sans paramètre
   (AC-2) et de l'AC-4 ; `docs/api-external.md` (paramètres de l'export, colonnes CSV) ;
   manuel utilisateur FR, § *Visualisation et exports* (`user-manual.tex:4188`) : export groupé, fidélité à l'écran,
   affichage de travail (lecture (a) de l'art. 959a CO), `.tex` seul ; la phrase « Aucun des deux ne perd de ligne,
   même quand l'écran en limite le nombre » (`user-manual.tex:4200-4202`) se restreint à l'export **à plat** ou nomme
   l'exception de l'export groupé (les comptes d'un groupe fermé n'y ont pas de ligne, leur somme est sur celle du
   groupe), et la liste des textes abrégés d'un « … » reste juste (aucun nom de groupe n'y entre si le retrait ne
   tronque pas ; sinon, l'y ajouter).

## Tâches / sous-tâches

- [ ] **T0** — Reprise de `main` et de la branche d'epic : la **46-1** doit y être fusionnée (sinon arrêt) ; relever
      au Change Log, ligne lue citée, les noms du § *L'API de la 46-1* (Dev Notes) tels qu'ils sont dans la branche, et
      si la **47-3** est fusionnée (`grep -rn "present_balance_sheet" crates/kesh-api/src/services/rapports.rs`). La
      48-11 doit être dans `main` (sinon **arrêt**, le dire au Change Log) ; relever ses noms de
      fonctions exacts (annoncés `round_for_display`, `format_two_decimals`, garde `arrondi_centime_garde.rs` ; non
      relevés à l'écriture de la fiche : la fiche 48-11 était « à écrire » au 2026-11-20 et aucune branche ne la
      portait). Si la 48-11 sort de la 0.20.0 (ordre de sortie de `epic-48-corrections-0-20-0.md`), la 46-3 attend
      quand même sa fusion dans `main` — le responsable de l'epic et le Project Lead décident alors de l'ordre.
- [ ] **T1 — fixtures** (AC-2) au commit de base, avant toute modification.
- [ ] **T2 — route** (AC-1) : dans `export_balance_sheet` et `export_income_statement`, un second extracteur
      `Query(paires): Query<Vec<(String, String)>>`, `acces.exiger(… .export)?` puis `groupes_de(&paires)?`, et l'appel
      de `bilan_par_groupes` / `compte_de_resultat_par_groupes` ; **`ExportQuery` inchangé** (revue P2 de la 46-1,
      L-6). Le rendu choisi dans la closure d'`exporter` selon `report.grouped_rows` (`Some` : rendu groupé ; `None` :
      rendu à plat de `&report.rapport`, comme aujourd'hui).
- [ ] **T3 — PDF** (AC 3, 4, 6) : deux fonctions **neuves** de rendu groupé, qui reçoivent l'enveloppe
      `&AvecGroupes<BalanceSheet, SectionsBilan>` / `&AvecGroupes<IncomeStatement, SectionsResultat>` de la 46-1
      (C-46-1-4 ; `kesh_report::groupes`, même crate) — `rapport` pour l'en-tête, les totaux, les lignes calculées et la
      garde « rapport vide », `grouping` pour la ligne de niveau et la position des sous-totaux, `grouped_rows` pour le
      corps ; `render_balance_sheet_pdf`
      (`pdf.rs:816`) et `render_income_statement_pdf` (`:954`) gardent **leur comportement**, leurs appelants et leurs
      tests : la route choisit l'une ou l'autre selon que `grouping` est présent (AC-2 tenu par construction, et
      prouvé par les fixtures). Libellés neufs dans `SectionLabels` et `fr_ch_defaults` ; une fonction de rendu de
      ligne groupée partagée par les deux rapports **et** par la balance (46-6b) ; l'en-tête, les totaux, les lignes
      calculées, l'équation et la garde « rapport vide » se **partagent** avec les fonctions à plat (extraits en
      fonctions communes, comportement inchangé ; zone de la 48-11, voir le tableau des fichiers communs de l'epic), seul le corps des sections diffère (DRY).
- [ ] **T4 — CSV** (AC 5, 6) : de même, deux fonctions neuves à côté de `render_balance_sheet_csv` (`csv.rs:143`) et
      `render_income_statement_csv` (`:277`), inchangées.
- [ ] **T5 — tests, documentation** (AC 8).
- [ ] **T6 — gate complet** ; E2E complet au dernier commit de code.

## Dev Notes

### État du code (relevé le 2026-11-20 à `29b9ec27`, après #1327 et #1346)

- `pdf.rs:84-97` `PdfContext`, `:102` `SectionLabels` (libellés FR-CH en dur), `:316` `format_swiss_amount`
  (contient aujourd'hui un `{amount:.2}`, `:318` — c'est le site que la 48-11 corrige), `:349` `round_for_display`
  (`pub(crate)`, déplacé par la 48-11), `:739` `any_foreign`, `:744` `draw_account_row` (nom tronqué si devise),
  `:816-950` bilan (sous-titres de rôle avec garder-avec-la-suite `:870-893`, `ensure_space_for_rows(2.0)` à `:877`,
lignes calculées, équation en pied),
  `:954` compte de résultat. Montants alignés à droite (`AMOUNT_RIGHT_MM`, `FX_AMOUNT_RIGHT_MM`, #1327).
- `csv.rs:42` `make_writer` (`;`, CRLF, BOM), `:50` `format_amount_iso` (`{amount:.2}` — site de la 48-11), `:101`
  `write_account_balance`, `:143-270` bilan (garde par section, `Total actifs;;;…`), `:277` compte de résultat.
- `routes/reports.rs`, à `29b9ec27` : `:174` `ExportQuery`, `:374` `export_balance_sheet`, `:504`
  `export_income_statement`, `:942` `PdfContext::fr_ch_default` ; sur `story/46-1` à `de0a582f` : `:203`
  `ExportQuery`, `:427` `export_balance_sheet`, `:559` `export_income_statement`, `:999` `PdfContext::fr_ch_default`. Les
  numéros de ligne de `pdf.rs` et `csv.rs` bougeront avec la 48-11 : se fier aux noms, relevés à nouveau au T0.

### L'API de la 46-1 (relevée sur `story/46-1` à `de0a582f`, à revérifier au T0 dans la branche d'epic)

- `kesh_report::groupes` : `AvecGroupes<T, S> { rapport: T, grouping: Option<Grouping>, grouped_rows: Option<S> }`
  (`#[serde(flatten)]` du rapport ; `sans_groupes`, `bilan`, `compte_de_resultat`) ; `SectionsBilan { assets,
  liabilities, equity }`, `SectionsResultat { revenues, expenses }`, chacune un `Vec<LigneRapport>` ; `LigneRapport {
  kind: GenreLigne, depth: u32, account_id, account_number, account_name, balance: Decimal, currency:
  Option<String>, balance_currency: Option<Decimal>, open: Option<bool>, active: Option<bool>, role:
  Option<Option<AccountRole>> }` (`mirror_of` s'y ajoute au recâblage de la 47-3, JSON seulement) ; `GenreLigne::{Groupe,
  Compte, Propre, SousTotal}` (sérialisés `group`, `account`, `own`, `subtotal`) ; `Grouping { detail_level:
  NiveauDetail::{Niveau(u8), Tous}, subtotal_position: PositionSousTotal::{Titre, Pied}, max_depth, open: Vec<i64>,
  closed: Vec<i64> }`. Les montants sont des `Decimal` exacts, jamais arrondis par la 46-1.
- `services/rapports.rs` : `DemandeGroupes { options }` et ses deux entrées `depuis_requete` (chaînes REST) et
  `depuis_valeurs` (valeurs typées, refus du contrat) ; `bilan_par_groupes(acces, lecture, demande,
  Option<DemandeGroupes>, usage)` et `compte_de_resultat_par_groupes(…)` rendent `Rapport<AvecGroupes<…>, U>` ;
  `bilan` et `compte_de_resultat` gardent leurs quatre arguments et délèguent avec `None` (écart E1 de la 46-1) ;
  aujourd'hui les deux exports appellent `bilan` / `compte_de_resultat` et rendent `&report.rapport`.
- `routes/reports.rs` : `PARAMETRES_GROUPES`, `groupes_de(&[(String, String)]) -> Result<Option<DemandeGroupes>,
  AppError>` (paramètre répété → 400 qui le nomme) ; les handlers JSON prennent `Query<ReportQuery>` **et**
  `Query<Vec<(String, String)>>`, contrôlent l'accès, puis lisent les groupes.
- Point d'insertion de la 47-3 : dans `bilan_par_groupes` et `compte_de_resultat_par_groupes`, entre le générateur et
  le `match groupes` — `present_balance_sheet(raw, &links)` / `present_income_statement` (47-3 AC-1) y présentent le
  rapport brut, et les groupes se calculent sur les sections présentées. Cette story n'y touche pas : en appelant les
  fonctions `*_par_groupes`, l'export hérite de la présentation et du regroupement tels quels.

### Ce qui doit rester

Les exports sans paramètre (AC-2), les exports des autres rapports, la ligne « Résultat reporté (calculé) » omise à
zéro (#1346), l'alignement (#1327).

### Choix et écarts

- **C-46-3-1** : le PDF imprime le **niveau**, et signale seulement qu'il y a des écarts manuels ; les groupes ouverts
  ou fermés se voient dans le corps (un groupe fermé porte son sous-total sans ses comptes). Simplifié à la validation
  P1 (KISS) : la liste des numéros, tronquée à une ligne, n'apprenait rien que le corps ne montre, et coûtait une
  troncature et un test de plus. Lecture retenue de la remarque 4 de Guy (« le PDF imprime le niveau et les groupes
  ouverts ou fermés ») : il imprime les groupes **dans l'état** ouvert ou fermé de l'écran.
- **C-46-3-2** : sous-total arrondi une fois, sur la somme exacte (AC-6), comme les totaux de section aujourd'hui.
- **C-46-3-3** (correction du 2026-11-21) : **aucun avertissement imprimé** pour un compte miroir rangé sous un groupe
  d'un autre type. La décision du responsable de l'Epic 47 (2026-11-21) admet cet état avec un avertissement **non
  bloquant**, dont le porteur n'est pas encore désigné (fichier d'epic 47, § *Décision … point e* : « personne à ce
  jour », issue `enhancement` à ouvrir) ; sa place naturelle est l'édition du plan, non un rapport remis à une
  fiduciaire. Le PDF et le CSV rendent l'effet tel que le JSON le donne (groupe dissous, totaux justes) et rien de
  plus. *Écarté* : une note de bas de page « groupe dissous par un compte miroir », qui demanderait au rendu une
  information que `groupedRows` ne porte pas.

### References

- Fiche 46-1 (§ *Le contrat*, Dev Agent Record : écarts E1 à E4, point d'insertion de la 47-3) ; rapports de revue de la
  46-1 (P1 L-4 et L-5, P2 L-6) ; fiche 47-3 (AC-1, AC-3, C-47-3-3, § *Recâblage*) et fichier de l'Epic 47 (§ *Décision
  … du 2026-11-21, point e*) ; fiche 48-11 et `epic-48-corrections-0-20-0.md` § *48-11* (garde `{:.2}`) ;
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
- 2026-11-20 — Validation P1 (Claude Opus 5.5, contexte frais, au code de `origin/main` `d29aa557`) : 0 C / 0 H / 1 M
  / 4 L, remédiés. M : les rendus groupés greffés dans `render_*_pdf`/`render_*_csv` lisaient des champs de
  `BalanceSheet` que la 46-1 n'ajoute plus (enveloppe `AvecGroupes<T, S>`, C-46-1-4) → fonctions neuves, rendus à plat
  inchangés avec leurs tests (T3, T4). L : en-tête du PDF simplifié (niveau et « détail ajusté à la main », sans liste
  tronquée, C-46-3-1, AC-3) ; fidélité de l'AC-4 vérifiée section par section ; noms de la 48-11 annoncés, non
  relevés, et cas d'une 48-11 sortie de la 0.20.0 (T0) ; lignes citées corrigées (`fr_ch_default` `:294`,
  garder-avec-la-suite `:877`). Rapport : `kesh-gate-logs/epic-46/validation/46-3-P1.md`.
- 2026-11-20 — Validation P2 (Claude Sonnet 5.5, contexte frais, au code de `origin/main` `d29aa557`) : 0 C / 0 H / 1 M
  / 1 L, remédiés. Lignes vérifiées : `pdf.rs:294`, `:399`, `:487`, `:816`, `:877`, `:954`, `csv.rs:143`, `:277`,
  `routes/reports.rs:174` (`ExportQuery` : ni `deny_unknown_fields` ni les quatre champs), `user-manual.tex:4188`. M :
  les fonctions groupées neuves omettaient la garde « rapport vide », alors que l'export d'un bilan vide part avec
  `detailLevel=all` depuis la 46-2 (AC-3, test `reports-export-pdf.spec.ts:40`). L : en-tête, totaux, équation et garde
  partagés avec les rendus à plat (T3, DRY) ; nom de l'enveloppe à deux paramètres. Rapport :
  `kesh-gate-logs/epic-46/validation/46-3-P2.md`.
- 2026-11-20 — Validation P3 ciblée (Claude Haiku 5.5, contexte frais, lentille sur le seul commit `8a7f0f73`, au code de `origin/main` `d29aa557`) : 0 C / 0 H / 0 M / 2 L, remédiés. L : « `BalanceSheet::is_empty` et son pendant » ne désignait aucune méthode pour le compte de résultat (garde précisée : `revenues` et `expenses` vides, `pdf.rs:967`, `csv.rs:296`) ; T3 disait les fonctions à plat « inchangées » tout en extrayant leurs parties communes (comportement, appelants et tests inchangés ; zone de la 48-11). Rapport : `kesh-gate-logs/epic-46/validation/46-123-P3-ciblee.md`.
- 2026-11-21 — **Corrections avant développement** (orchestrateur de l'epic, sur les rapports de revue de la 46-1 et
  l'API relevée sur `story/46-1` à `de0a582f`, et la fiche 47-3 à `b4d17d70`). AC-1 et T2 : paramètres lus par
  `groupes_de` et un second extracteur `Query<Vec<(String, String)>>`, `ExportQuery` inchangé (revue P2 de la 46-1,
  L-6 — la consigne posée sur `story/46-1` par `de0a582f` disait encore « `ExportQuery` gagne les quatre champs » en
  tête de T2 et « à passer en `pub(crate)` », inutile : les exports sont dans le même module), paramètre répété refusé,
  ordre format → accès → groupes (P1 L-5), appel de `bilan_par_groupes` / `compte_de_resultat_par_groupes` ; AC-3 et
  T3 : champs réels de `LigneRapport`, `GenreLigne`, `Grouping` (`NiveauDetail`, `PositionSousTotal`, listes retenues)
  ; AC-4 : cas des comptes miroirs, dont un miroir sous un groupe d'un autre type (décision du responsable de l'Epic 47
  du 2026-11-21) ; C-46-3-3 : aucun avertissement imprimé ; Dev Notes : § *L'API de la 46-1*, lignes de
  `routes/reports.rs` à la 46-1 ; T0 : 46-1 fusionnée exigée, 47-3 relevée. Validation à reprendre (P1 d'une nouvelle
  boucle, prompt `kesh-gate-logs/s46-specs/validation-p1-prompt.md`).
- 2026-11-21 — **Validation P1 de la nouvelle boucle** (Claude Sonnet 5.5, contexte frais, à `fe8f0eeb`) : 0 C / 0 H /
  1 M / 3 L sur cette fiche, remédiés. M (F1) : l'AC-4 comparait la section Capitaux propres à `groupedRows.equity`
  seul, alors que le PDF écrit les deux lignes calculées entre les comptes et le total (`pdf.rs:896-925`) → suite
  attendue `groupedRows.equity` puis les lignes calculées. L : texte normalisé et jeu d'essai à noms courts, sans
  troncature (F3, AC-4) ; phrase du manuel « aucun des deux ne perd de ligne » à restreindre à l'export à plat (F4,
  AC-8) ; lignes de `routes/reports.rs` séparées par commit (F5). Rapport :
  `kesh-gate-logs/s46-specs/validation-p1-rapport.md`.
