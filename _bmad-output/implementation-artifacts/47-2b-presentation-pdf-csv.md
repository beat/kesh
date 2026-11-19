# Story 47-2b : présentation au bilan et au compte de résultat — PDF et CSV

Status: superseded

> ⚠️ **Superseded le 2026-11-19 — refonte « miroir seul, KISS »** (fichier d'epic, § *Décision du 2026-11-19 : miroir seul*) : PDF et CSV n'ont plus rien de propre au miroir : la ligne du miroir y paraît comme un compte ordinaire, avec son solde présenté, sans marque, note, mention « présentation compensée » ni colonnes `Presentation` / `Sources` (47-3, AC-7 et choix C-47-3-3). Fiche laissée telle quelle pour l'historique ; ne pas développer.

Epic : `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md` — D6, D9 ; Q9 ; FR-47-4 (PDF, CSV), FR-47-6
(mention « présentation compensée »), FR-47-8. Née du découpage de l'esquisse 47-2 (2026-11-19). Dépend de la
**47-2a** et de la fusion dans `main` de **#1327** (`fix/pdf-montants-alignes`, réécriture de `pdf.rs`) et de
**`fix/bilan-resultat-reporte`** (#1346, `pdf.rs`, `csv.rs`) ; **s'il est fusionné avant, la 48-11** (arrondi au
centime des montants des rapports CSV et PDF, mêmes fichiers) en fixe la règle ; si la **46-3** est fusionnée, ses
rendus groupés reçoivent la même marque (et se concertent sur l'ordre des colonnes CSV : AC 4).

## Story

En tant que comptable,
je veux que les exports PDF et CSV du bilan et du compte de résultat suivent la présentation choisie, en disant
clairement quand une ligne est présentée ou compensée,
afin de remettre un document lisible sans masquer ce qui a été regroupé.

## Critères d'acceptation

1. **PDF présenté** (`render_balance_sheet_pdf`, `render_income_statement_pdf`) : une ligne qui porte des `sources`
   (47-2a AC 6 ; sections Actifs, Passifs, Produits, Charges — jamais Capitaux propres, D8) reçoit une **marque** :
   `*` (miroir : ses sources sont de `kind` `own` / `mirror`) ou `‡` (paire : `kind` `offset`), **écrite à la suite du
   libellé** du compte, séparée par une espace (« Avoirs à court terme * »), **jamais dans la colonne des montants**
   (alignée à droite sur le bord de #1327, marge de 15 mm : une marque après le montant sortirait de la page ; avant
   lui, elle décalerait l'alignement). Le libellé d'une ligne marquée est tronqué à `ACCOUNT_NAME_MAX_CHARS_FX` (34)
   **avant** d'ajouter la marque, qui n'est donc jamais coupée, y compris sans devise. Une **note de bas de rapport**,
   après le dernier total, liste par ligne marquée (symbole, numéro, libellé) **une ligne par source**, dans l'ordre
   des `sources` (47-2a AC 6) : numéro, libellé tronqué à `ACCOUNT_NAME_MAX_CHARS_FX`, solde brut formaté par
   `format_swiss_amount`. **Miroir** — p. ex. « * 1040 Avoirs à court terme » puis, en retrait, « comprend 2270
   AVS/AI/APG, solde -1'200.00 » (la source `own`, absente de l'exemple chiffré de la 47-2a AC 11 où 1040 est sans écriture ; si 1040 portait 300.00 : « solde propre 300.00 »). **Paire** — p. ex.
   « ‡ 6950 Produits financiers » puis, en retrait, « solde propre 1'020.00 » (le membre qui porte la ligne, premier
   des `sources`) et « déduit 6900 Charges financières, solde 450.00 » (l'autre membre : la ligne vaut le premier
   **moins** le second, tous deux signés dans leur nature ; « comprend » le dirait ajouté). ⚠️ **Exemples aux comptes
   des plans livrés** (ceux de la 47-2a AC 11) : G4-ter (`textes_coherents.rs`) lit `api-external.md`, le manuel
   d'administration et les doc-comments Rust — « 1179 Créances sociales » ou « 2270 Dettes sociales » y feraient
   rougir le gate. ⚠️ **Le signe est un
   trait d'union ASCII** : le « − » (U+2212) et l'espace fine ne sont pas WinAnsi, `lopdf` les retire en silence
   (`largeurs.rs`, en-tête) et le PDF imprimerait « 1 200.00 », signe perdu. Montants en CHF (la `balance` des sources
   est en devise principale).
2. **« Présentation compensée »** (Q9, D9) : quand le rapport porte `compensated`, le PDF affiche en clair, **sous
   l'en-tête** (après `draw_header`, avant l'éventuel en-tête de colonnes de devise de #1327, **et aussi pour un
   rapport que la compensation a vidé** — la mention précède le test « vide »), la mention **« Présentation
   compensée »** en gras, **seule sur sa ligne** (ligne de texte entière, que l'E2E de la 47-3b cherche dans le PDF
   extrait ; `wrap_text` ne mesure qu'une graisse par appel), suivie, sur les lignes d'après et en maigre, d'une
   phrase : des comptes de types opposés sont présentés nets (les paires marquées ‡
   quand le net n'est pas nul) ; CO 958c al. 1 ch. 7 ; la présentation brute est disponible pour les comptes annuels
   officiels. **Absente** sinon — un rapport à miroirs seuls n'a pas la mention (le miroir ne compense rien) mais a
   marques et note. Une paire **de net nul** supprime ses deux lignes (47-2a AC 3) : aucune ligne marquée ne la
   porte et la note n'en dit rien ; la mention, seule trace, ne doit donc pas renvoyer à une marque (d'où « quand le
   net n'est pas nul »). La mention est écrite **sur la première page seulement** (les pages suivantes n'ont pas
   d'en-tête, comme aujourd'hui).
   **Pas de retour à la ligne automatique** dans `PdfBuilder` : la phrase (≈ 150 caractères) et les lignes de la note
   dépassent 180 mm. Un **helper de césure** (`wrap_text(text, font_size, bold, max_mm) -> Vec<String>`, mesuré par
   `kesh_qrbill::largeur_mm`, comme `right_aligned_x`) coupe aux espaces à la largeur utile
   (`AMOUNT_RIGHT_MM − MARGIN_LEFT_MM`, **moins le retrait** pour les lignes de source de la note, écrites décalées) ; un mot plus long que la ligne est tronqué par `truncate_with_ellipsis`.
   Chaque ligne réserve sa place (`ensure_space_for_row`) ; le titre de la note est gardé avec sa première ligne
   (`ensure_space_for_rows(2.0)`) ; la note **saute de page** au besoin sans redessiner d'en-tête.
3. **PDF brut et sans effet** : avec `presentation=brute`, **et** avec un rapport présenté qui ne porte ni `sources`
   ni `compensated` (aucun lien, ou liens **tous sans effet** : aucune source de sens inverse, paires sans effet —
   47-2a AC 2, 3 ; le rapport présenté **égale** alors le brut, 47-2a AC 5), le PDF est **identique** à
   celui d'aujourd'hui : mêmes lignes de texte, sans marque, note ni mention. La garde existante
   `crates/kesh-report/tests/rapports_mono_devise.rs` (fixtures `tests/fixtures/mono_devise/{chf,multi_sans_devise}/
   bilan.pdf.txt`, `resultat.pdf.txt`, `bilan.csv`, `resultat.csv`, enregistrées au commit de base de la 37-4) **passe
   sans être réenregistrée** (ni `KESH_FIXTURES_ENREGISTRER`) : c'est elle, non un nouveau jeu, qui porte « mêmes
   lignes de texte en mono-devise ». ⚠️ Elle rend des rapports **bruts** (`generate_*` puis `render_*`) : le cas
   « présenté sans effet » s'établit par composition (47-2a AC 5, sortie égale à l'entrée) **et** par un test de rendu
   propre à cette story, sur un rapport présenté aux liens sans effet (T4). Les rendus **présentés** ne s'y ajoutent pas en fixtures (la marque
   `‡` est l'octet 0x87, que le décodeur `char::from` des helpers rend U+0087, illisible) : ils s'affirment par des
   `assert!` sur les lignes extraites, la marque attendue étant `'\u{87}'` pour `‡`.
4. **CSV** (`render_balance_sheet_csv`, `render_income_statement_csv`) : seulement si la présentation a changé le
   rapport — au moins une ligne **des sections Actifs, Passifs, Produits ou Charges** porte des `sources`, **ou** le
   rapport porte `compensated` (une paire de net nul supprime ses deux lignes sans laisser de ligne marquée, 47-2a
   AC 3 et 6 : sans cette clause, un tel CSV différerait du brut sans aucune trace) —, deux colonnes s'ajoutent en fin d'en-tête —
   `Presentation` (`miroir` si les sources sont `own`/`mirror`, `compense` si elles sont `offset`) et `Sources`
   (`numéro:solde` brut, séparés par `|`, **dans l'ordre de `sources`**, 47-2a AC 6 : `2270:-1200.00` pour 1040 de l'exemple chiffré de la 47-2a AC 11 (1040 sans écriture : pas de source `own`) — et `1040:300.00|2270:-1200.00` si 1040 portait un solde propre de 300.00 —,
   `6950:1020.00|6900:450.00` ; la première source est `own` ou le membre porteur quand son numéro égale
   `NumeroCompte`, aucune colonne de `kind`). Lecture : `miroir`, `Solde` = solde propre − somme des soldes des
   sources `mirror` (négatifs dans leur nature) ; `compense`, `Solde` = solde du membre porteur − solde de l'autre —
   écrite à `api-external.md` (AC 8). Sinon, et toujours en `brute`, le CSV est **identique octet pour octet** à
   aujourd'hui (BOM, `;`, CRLF) — en présenté, au CSV brut, puisque le rapport l'égale alors (47-2a AC 5). **Ordre des colonnes** quand plusieurs
   familles s'ajoutent : base, puis `Devise;SoldeDevise` (37-4) **si** un compte est en devise, puis
   `Presentation;Sources` **si** une ligne est présentée, puis les colonnes de groupes de la 46-3 (`Niveau`,
   `TypeLigne`) si elle est fusionnée — ordre à confirmer avec son auteur au premier arrivé ; chaque famille se
   décide indépendamment (les quatre combinaisons sont testées). Les lignes de total et les lignes calculées
   portent deux cellules vides, comme toutes les lignes d'un rapport dont seule une paire de net nul a changé quelque
   chose. `Sources` passe par `txt()` (`csv_sanitize`, comme les autres cellules texte) ; ses
   montants s'écrivent par `format_amount_iso`, **la fonction de la colonne `Solde`** (la règle de la 48-11 s'y
   applique si elle est fusionnée). ⚠️ Un numéro de compte n'est borné qu'en longueur (≤ 10, `routes/accounts.rs:358`) :
   un `:` ou un `|` dans un numéro rend la cellule ambiguë — limite écrite à `api-external.md`, non corrigée ici.
5. **Libellés** : les textes du PDF (titre de la note, « comprend », « déduit », « solde propre », « solde », mention
   et sa phrase) sont de **nouveaux champs de `SectionLabels`**, valeurs FR-CH dans `fr_ch_defaults()` — c'est la seule
   voie réelle : `load_pdf_context` (`routes/reports.rs:928`) ne résout aucun libellé par i18n, il appelle
   `PdfContext::fr_ch_default` (« i18n PDF complète déférée », L4/L11). **Même décision que #1327** (Beat,
   2026-11-18) : les clés `reports-pdf-…` correspondantes sont **posées dans les quatre catalogues** (parité
   vérifiée) mais **non lues** par les PDF, pas de mot allemand isolé dans un PDF français ; l'issue de traduction des
   PDF les reprendra. Le CSV garde ses en-têtes FR-CH en dur (L1/L4/L11, D3 de la 9-2a), valeurs `miroir` /
   `compense` sans accent.
6. **Montants** : formatés comme le reste du rapport (`format_swiss_amount` au PDF, `format_amount_iso` au CSV ;
   règle de la 48-11 si fusionnée, sinon celle en vigueur). **Le rendu ne filtre aucune ligne** : il écrit toutes
   celles que porte le rapport, comme en brut (le brut montre déjà un « 0.00 » pour un solde de 0.004) ; qu'une ligne
   présentée n'existe pas quand son net est exactement nul est la règle de la fonction (47-2a AC 3), non du rendu —
   un filtre ici ferait diverger le PDF, le CSV, l'écran (47-3b) et le JSON.
7. **Groupes (si la 46-3 est fusionnée)** : la marque et la note s'appliquent aux lignes de compte des rendus groupés ;
   un sous-total de groupe n'est jamais marqué (il agrège des lignes déjà présentées) ; la ligne `propre` d'un compte
   de groupe (46-3 AC-3) l'est si son compte porte des `sources`.
8. **Documentation (exigence de Guy, 2026-11-19)** : `docs/api-external.md` — à côté du paragraphe de la 47-2a sur
   `presentation` (et du paragraphe *Les rapports en devise*, `:1080`, dont la règle « colonnes seulement si le rapport
   en contient » est le patron) : les colonnes `Presentation` et `Sources`, leur condition, leur ordre avec les
   colonnes en devise, leur lecture (AC 4), la limite des numéros de compte, la marque et la note du PDF, la
   mention « Présentation compensée » ; manuel d'administration (`docs/manual/fr/admin-manual.tex` : il n'a pas de
   section des rapports — la puce *Rapports comptables* de l'introduction, `:84`, puis `sec:api-keys` `:3262` et
   `sec:mcp-outils` `:3521`, où la 47-2a pose `presentation=brute`) — exports
   présentés et bruts, y compris que **deux exports du même rapport portent le même nom de fichier** : seule la
   mention et les marques distinguent un PDF présenté d'un brut. Le manuel utilisateur est celui de la 47-3b (AC 10 ;
   `user-manual.tex:3937`, « cette présentation se retrouve à l'identique dans les exports », vise les fonds propres
   par rôle de la 14-3c, non la présentation de l'Epic 47 ; `:4176` liste les formats d'export). `.tex` seulement ; **le prompt de chaque passe de revue nomme ces documents** et exige le contrôle du PDF compilé
   localement, ainsi que celui d'un PDF de rapport réel (`pdftotext … | tr '\n' ' ' | tr -s ' '`).

## Tâches / sous-tâches

- [ ] **T0** : reprise de `main` (#1327 et #1346 fusionnés — sinon la story attend) ; lire la mise en page de #1327
  (`write_right`, `AMOUNT_RIGHT_MM`, `draw_currency_columns_header`, `largeurs.rs`) avant de placer la marque ;
  relever la 48-11 ; recompter les numéros de ligne cités.
- [ ] **T1 — PDF** (AC 1–3, 5) : `wrap_text`, marque, note, mention ; champs de `SectionLabels` et clés des quatre
  catalogues.
- [ ] **T2 — CSV** (AC 4, 6) : généraliser `write_fx_record` / `balance_row_extra` (colonnes optionnelles par
  famille, ordre de l'AC 4).
- [ ] **T3 — groupes** (AC 7), si la 46-3 est là.
- [ ] **T4 — tests** : unitaires de rendu (texte du PDF, octets du CSV) en présenté, en brut et sans lien ;
  `rapports_mono_devise.rs` rejoué **sans réenregistrement** ; rendu d'un rapport présenté aux liens tous sans effet
  (miroir à sources de sens normal, paire sans effet) identique au brut, PDF et CSV ; note d'une paire (« solde
  propre », « déduit ») ; CSV d'un rapport dont seule une paire de net nul a changé quelque chose (colonnes présentes,
  cellules vides) ; `wrap_text` (aucune ligne au-delà de 180 mm, mot
  trop long, note sur deux pages) ; mention absente pour un miroir seul, présente pour une paire de net nul et pour un
  rapport vidé ; quatre combinaisons de colonnes CSV ; `reports_e2e.rs` (export de l'exemple chiffré, les deux
  présentations).
- [ ] **T5 — documentation** (AC 8). **T6 — gate complet**, E2E complet au dernier commit de code.

## Dev Notes

### Code lu (à `origin/main` `fd04099f`)

- `crates/kesh-report/src/pdf.rs:66` (`PdfContext`), `:84` (`SectionLabels`), `:636` (`render_balance_sheet_pdf`),
  `:766` (`render_income_statement_pdf`) — **réécrit par #1327 (+1117 lignes)** : les numéros de ligne seront à
  relire après sa fusion.
- `crates/kesh-report/src/csv.rs:65` (`write_fx_record`), `:142` (`render_balance_sheet_csv`, colonnes de devise
  conditionnelles), `:271` (`render_income_statement_csv`) ; `crates/kesh-report/tests/rapports_mono_devise.rs`
  (garde de non-régression et ses fixtures).
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
identique. Mutations : mention affichée pour un miroir seul ; colonnes CSV présentes sans ligne présentée ; marque écrite après
le montant ; signe écrit « − » (U+2212) ; note sans césure ; fixture de `rapports_mono_devise` réenregistrée.

### Risques

Dépendance à deux correctifs non fusionnés (et à la 48-11) qui réécrivent les mêmes fonctions ; taille de la note sur
un rapport à nombreuses sources (saut de page, césure : AC 2) ; texte hors WinAnsi retiré en silence par `lopdf`
(`−`, `→`, `↳`, `⚠`) : tout caractère neuf du PDF se contrôle contre `WINANSI` de `largeurs.rs` (`*` 0x2A et `‡`
0x87 y sont) ; une paire de net nul ne laisse aucune trace au PDF hors la mention (limite assumée, AC 2).

### References

- Fiche 47-2a ; release 0.20.0 § *Correctifs de Beat* (#1327 avant la 48-11) ; `CLAUDE.md` § *Le prompt d'une passe
  doit NOMMER le manuel*.

## Change Log

- 2026-11-19 — validation P1 (Sonnet 5.5) : 0 C / 3 H / 6 M / 3 L, remédiés. H : exemple de note au « − » U+2212, hors
  WinAnsi, signe perdu à l'impression (AC 1 réécrit, `format_swiss_amount`) ; AC 5 supposait des libellés i18n que
  `load_pdf_context` ne résout pas (champs de `SectionLabels`, clés non lues comme #1327) ; aucune césure dans
  `PdfBuilder`, phrase de la mention et note hors page (`wrap_text`). M : place de la marque inexistante face aux
  colonnes de #1327 ; ordre des colonnes CSV avec `Devise;SoldeDevise` et la 46-3, format de `Sources`, sections
  scrutées ; AC 6 (filtre au rendu) contredisait la 47-2a et le brut ; paire de net nul (mention sans marque,
  rapport vidé) ; garde `rapports_mono_devise.rs` non nommée, marque `‡` = U+0087 aux helpers ; 48-11 absente des
  dépendances. L : numéros de ligne de `csv.rs`, nom de fichier identique brut/présenté, numéro de compte avec `:`
  ou `|`. Rapport : `kesh-gate-logs/epic-47-validation/47-2b-presentation-pdf-csv-P1.md`.
- 2026-11-19 — validation P2 (Opus 5.5) : 0 C / 0 H / 3 M / 6 L, remédiés. M : exemples de la note et du CSV aux
  numéros de l'ancien jeu (1179, « 2270 Dettes sociales »), que la 47-3b P1 a remplacés dans la 47-2a AC 11 par des
  comptes des plans livrés — G4-ter lit `api-external.md`, le manuel d'administration et les doc-comments (AC 1, 4
  réécrits sur 1040, 2270 AVS/AI/APG, 6950, 6900) ; note d'une **paire** non spécifiée et trompeuse (« comprend » pour
  un membre soustrait : « solde propre » / « déduit », lecture de `Sources` écrite) ; CSV d'un rapport dont seule une
  paire de net nul a changé quelque chose : ni colonne ni trace, et « identique à aujourd'hui » faux face au brut
  (colonnes ajoutées aussi sur `compensated`). L : mention seule sur sa ligne (une graisse par `wrap_text`, ligne
  cherchée par l'E2E de la 47-3b) ; largeur de césure moins le retrait ; garde `rapports_mono_devise.rs` limitée au
  rendu brut, test « présenté sans effet » ajouté, paires sans effet de la 47-2a reprises à l'AC 3 ; sections du
  manuel d'administration (pas de section des rapports : `:84` est une puce, `sec:api-keys`, `sec:mcp-outils`) et
  `user-manual.tex:3937` mal décrit ; ligne `propre` des groupes de la 46-3 ; 48-11 absente du tableau du découpage
  du fichier d'epic. Rapport : `kesh-gate-logs/epic-47-validation/47-2b-presentation-pdf-csv-P2.md`.
- 2026-11-19 — validation P3 ciblée (Sonnet 5.5) : 0 C / 0 H / 0 M / 1 L, remédié. Vérifié à `origin/main` : 1040 Avoirs à
  court terme (plan `pme` seul), 2270 AVS/AI/APG, 6950 Produits financiers, 6900 Charges financières existent sous ces
  noms (G4-bis/G4-ter) ; paire 1 020.00 − 450.00 = 570.00 et miroir 0 − (−1 200.00) = 1 200.00 concordent avec la
  47-2a AC 11 ; `compensated` ⇒ colonnes et mention cohérents avec la 47-2a AC 3, 6 et la 47-3b AC 5, 8. L : exemples de
  `Sources`/de note à « 1040:300.00 » (solde propre absent de l'exemple chiffré) rapprochés de l'exemple, valeur de la
  47-3b (`2270:-1200.00`). Rapport : `kesh-gate-logs/epic-47-validation/47-2b-presentation-pdf-csv-P3.md`.
- Registre de la dette du fichier d'epic (§ *Registre de la dette de l'epic*) : à la clôture de la boucle de revue,
  tout finding non remédié y est consigné, une ligne par problème ; la 47-5 en part (« on trace toujours »).

- 2026-11-19 — **Refonte du 2026-11-19 (miroir seul, KISS)** : fiche `superseded` (voir l'avertissement en tête) ; clé du registre de sprint passée à `superseded`.

## Notes de version

Les exports PDF et CSV du bilan et du compte de résultat suivent la présentation : les lignes présentées sont
marquées et leurs sources notées ; un rapport où une paire s'applique porte la mention **« Présentation compensée »**.
L'export brut reste identique à celui d'avant.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
