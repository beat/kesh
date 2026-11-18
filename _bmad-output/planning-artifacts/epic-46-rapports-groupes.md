# Epic 46 — Rapports par groupes de comptes

**Kickoff : 2026-11-18** (branche `epic/46-rapports-groupes`, partie de `main` à `773df80d`, release v0.19.0).
Numéro **46** réservé par Guy ; responsable **Beat** (contributeur). État : **esquisse à valider par Beat** (page de
revue hors dépôt), puis validation en boucle des fiches (`CLAUDE.md` § *Review Iteration Rule*). Aucun code écrit.

**Cible** : une release mineure (un epic = une release) ; numéro, CHANGELOG daté, macros des manuels et tag par Guy.
**Aucune migration attendue** (D2) : l'epic peut entrer dans n'importe quelle release planifiée.

**Taille** : 6 stories hors kickoff (46-1 à 46-6), story de solde comprise — dans la borne de six. 46-5 est
**conditionnelle** (Q9) ; si elle sort, l'epic en compte cinq.

**Règles des fichiers communs** (`CLAUDE.md` § *Travailler à plusieurs développeurs*) : aucune ligne au `CHANGELOG.md`
en cours d'epic (chaque fiche a sa *Note de version*) ; aucun PDF de manuel commité (`.tex` seul) ; catalogues i18n et
`sprint-status.yaml` en bloc propre ; `sitesTotal` et autres compteurs recomptés après chaque reprise de `main`.

## Intention

Le bilan et le compte de résultat se présentent **par groupes de comptes**, avec des **sous-totaux tirés de la
hiérarchie du plan** (`accounts.parent_id`), et un **niveau de détail choisi au moment de l'affichage ou de
l'impression** : des seuls groupes de premier niveau jusqu'au détail de chaque compte. Écran, PDF, CSV, API et MCP.

Besoin réel de Beat : son ancien logiciel présentait le bilan par groupes (1 ACTIFS › 10 Actifs circulants ›
100 Trésorerie › comptes, avec le solde en devise d'un groupe). Il structure son plan ainsi : **10 numéros réservés
par banque** (néobanques comprises : Wise, PayPal…), un compte par devise, **un compte de groupe par banque = le numéro
sans son dernier chiffre**, puis un groupe de toutes les liquidités, plus court encore (**groupe 10**).

**Hors périmètre** (prévu plus tard) : des **modèles de sortie** (presets) qui enregistrent ces choix ; les rubriques
légales imposées (Q7) ; la balance et le grand livre (Q6, proposition).

## Inventaire du code (relevé le 2026-11-18 à `773df80d`)

| Domaine | Ce qui existe | Où |
|---|---|---|
| Hiérarchie | `accounts.parent_id` (FK `ON DELETE RESTRICT`), `postable` (un parent devient non imputable, 14-3a D2) ; **aucune contrainte de type** entre parent et enfant | `migrations/20260411000001_accounts.sql:12,18` ; `kesh-db/src/entities/account.rs:228-240` |
| | parent modifiable par `PUT /accounts/{id}` : **non fusionné** (`fix/api-parent-compte`, `review`) ; refuse un cycle et un parent qui porte des écritures | fiche `fix-api-parent-compte.md` AC-2, AC-3 |
| Plan PME livré | 86 comptes, trois niveaux : classe (`1` Actifs) › groupe (`10` Actifs circulants) › comptes (`1000`…) ; 9 classes | `kesh-core/assets/charts/pme.json` |
| Bilan | sections par **type** et **rôle** : `assets`, `liabilities` (dettes), `equity` (rôles de fonds propres, ordre CO 959a), lignes calculées `retained_earnings`, `equity_result` ; liste **plate** triée par numéro ; `HAVING balance != 0 OR balance_currency != 0` | `kesh-report/src/balance_sheet.rs:74-330` |
| Compte de résultat | `revenues`, `expenses`, plats, même `AccountBalance` | `kesh-report/src/income_statement.rs` |
| Devise | `AccountBalance.currency`, `balance_currency` (37-4) ; aucun total en devise (NFR-37-4-3) | `balance_sheet.rs:152-180` |
| Rendus | PDF `render_balance_sheet_pdf`, `render_income_statement_pdf` (sous-titres de rôle) ; CSV `render_balance_sheet_csv`, `render_income_statement_csv` | `pdf.rs:636,766` ; `csv.rs:142,271` |
| API | `GET /api/v1/reports/{balance-sheet,income-statement}[/export]`, `ReportQuery`, `ExportQuery` | `kesh-api/src/lib.rs:1381-1430` ; `routes/reports.rs:57-200` |
| MCP | `kesh_bilan`, `kesh_compte_de_resultat` (`ArgumentsPeriode`, `deny_unknown_fields`) | `kesh-api/src/mcp/outils/rapports.rs` |
| Écrans | `BalanceSheetView.svelte` (groupes de rôle), `IncomeStatementView.svelte`, `reports.api.ts`, `reports.types.ts` | `frontend/src/lib/features/reports/` |
| Lecture de `parent_id` | **aucun rapport** ne la lit (relevé de la fiche `fix-api-parent-compte`, AC-7) | — |

## Décisions proposées (à approuver par Beat ; D1, D4, D6, D7 dépendent de questions ouvertes)

- **D1 — Les groupes viennent de `parent_id` seul**, jamais du numéro (comme la règle « jamais par numéro » de 14-1).
  Un groupe = tout compte qui a au moins un sous-compte. Le préfixe de numéro ne sert qu'à l'outil d'aide (46-5). *(Q1)*
- **D2 — Aucune migration.** Les soldes par compte restent calculés par les requêtes actuelles ; une seule lecture
  de l'arbre des comptes de la société s'y ajoute ; l'agrégation est une **fonction pure** (`kesh-report/src/groupes.rs`),
  testée sans base.
- **D3 — Niveau de détail = profondeur dans l'arbre**, non longueur du numéro : niveau 1 = racines, niveau k =
  nœuds de profondeur ≤ k, « tous » = chaque compte. Un compte plus profond que le niveau est **replié** dans son
  ancêtre de ce niveau. Un compte sans parent (racine sans enfant) s'affiche à tout niveau. Le mode **« à plat »**
  (sans groupes) reste celui d'aujourd'hui.
- **D4 — Le squelette légal du bilan demeure** : Actifs, Dettes, Fonds propres, puis les deux lignes calculées (14-1,
  14-3c, CO 959a). Les groupes s'emboîtent **dans** chaque section. Un groupe dont les comptes tombent dans plusieurs
  sections (p. ex. `2 Passifs` = dettes + fonds propres ; `7 Résultats annexes` = produits + charges) **n'est pas
  affiché** : ses enfants remontent d'un niveau. Les totaux restent exacts. Les lignes calculées restent hors groupe,
  en fin de fonds propres. En mode groupé, l'ordre des fonds propres suit l'arbre (numéros) ; sous-titres de rôle
  seulement en mode à plat. *(Q4)*
- **D5 — Solde propre d'un groupe.** Un compte de groupe qui porte lui-même des écritures (admis à la création) donne
  une ligne « solde propre » parmi ses enfants ; son sous-total l'inclut.
- **D6 — Groupes vides masqués** : un groupe dont le sous-total CHF **et** le solde en devise sont nuls ne s'affiche
  pas (même règle que `HAVING` des comptes). *(Q2)*
- **D7 — Solde en devise d'un groupe** : donné seulement si **tous** ses comptes à solde non nul sont dans la **même
  devise étrangère** ; sinon absent. Les totaux de section restent en CHF (NFR-37-4-3). *(Q3)*
- **D8 — Contrat additif.** Sans paramètre, JSON, CSV et PDF restent **identiques** à aujourd'hui. Avec le paramètre
  `detailLevel` (`1`…`n` | `all`), chaque section gagne un champ `groupedRows` : lignes déjà ordonnées
  `{ kind: group|account|own, depth, accountId, number, name, balance, currency?, balanceCurrency? }`. Les trois rendus
  et le MCP lisent la même liste. *(Q5 pour le MCP)*
- **D9 — Choix du niveau à l'écran** : un sélecteur « À plat · Niveau 1 · … · Tous les comptes » sur le bilan et le
  compte de résultat ; le niveau passe à l'export (PDF, CSV). Retenu par visiteur (`localStorage`, convenance) ; aucun
  preset enregistré (hors périmètre).
- **D10 — Ordre des frères** : par numéro, comme aujourd'hui (comparaison de chaînes, `ORDER BY a.number`).
- **D11 — Balance et grand livre hors périmètre.** *(Q6)*
- **D12 — Structuration du plan par l'utilisateur** : avec `fix/api-parent-compte`, l'écran du plan suffit. L'outil
  d'aide (46-5) **propose**, l'utilisateur confirme ; rien n'est créé d'office (`CLAUDE.md` § *Un appariement
  automatique propose*). *(Q9)*

## Exigences

**Fonctionnelles** (détail par fiche)

- FR-46-1 — Arbre des groupes et sous-totaux (D1, D3–D7, D10), exposé par l'API (D8). → 46-1
- FR-46-2 — Écrans du bilan et du compte de résultat au niveau choisi (D9). → 46-2
- FR-46-3 — PDF et CSV au niveau choisi, groupes en gras, retrait par profondeur. → 46-3
- FR-46-4 — Outils MCP `kesh_bilan`, `kesh_compte_de_resultat` avec le niveau. → 46-4
- FR-46-5 — Aide à la création des groupes d'un plan existant. → 46-5 (conditionnelle)

**Non fonctionnelles**

- NFR-46-1 — Sans paramètre : JSON, CSV (octet pour octet) et PDF (mêmes lignes de texte) identiques à l'actuel.
- NFR-46-2 — Somme des sous-totaux de premier niveau = total de la section, à tout niveau ; équation du bilan inchangée.
- NFR-46-3 — Une seule requête de plus par rapport (l'arbre) ; agrégation en mémoire, O(n comptes).
- NFR-46-4 — Arbre défensif : un cycle ou un parent d'une autre société (impossible par l'API, possible par SQL) ne
  fait ni boucle ni panique : le compte est traité en racine, avec `tracing::warn!`.
- NFR-46-5 — Quatre locales ; `i18n-keys`, `i18n-libelle-en-dur` verts ; manuel utilisateur FR (`.tex`).
- NFR-46-6 — MCP : `mcp_contrat`, `mcp_gardes_lexicales` verts, tailles re-mesurées.

## Stories

| Clé | Titre | Dépend de | Taille |
|---|---|---|---|
| 46-0 | Kickoff (ce fichier ; pas de fiche) | — | — |
| 46-1 | Arbre des groupes : modèle et API | `fix/bilan-resultat-reporte` fusionné | M |
| 46-2 | Écrans du bilan et du compte de résultat par groupes | 46-1 | M |
| 46-3 | PDF et CSV par groupes | 46-1, `fix/pdf-montants-alignes` fusionné | M |
| 46-4 | Outils MCP par groupes | 46-1 | S |
| 46-5 | Aide à la création des groupes d'un plan (conditionnelle, Q9) | `fix/api-parent-compte` fusionné | M |
| 46-6 | Solde de la dette de l'epic (plafond : 2 issues, à confirmer au kickoff) | toutes | S |

Ordre proposé : 46-1 → (46-2, 46-3, 46-4 en parallèle) → 46-5 → 46-6.

## Dépendances

- **`fix/api-parent-compte`** (non fusionné, `review`) : nécessaire à Beat pour ranger un plan existant, et à 46-5.
  Non nécessaire à 46-1–46-4 (l'arbre existe déjà). ⚠️ Il refuse un parent qui porte des écritures : un compte déjà
  mouvementé (p. ex. `1020 Banque`) ne peut pas devenir le groupe d'une banque ; le groupe doit être un compte neuf.
- **`fix/bilan-resultat-reporte`** (non fusionné) : touche `balance_sheet.rs`, `csv.rs`, `pdf.rs`,
  `BalanceSheetView.svelte`, `reports.api.ts`. 46-1 part de `main` **après** sa fusion, ou le reprend.
- **`fix/pdf-montants-alignes`** (non fusionné) : réécrit `pdf.rs` (+1117 lignes). 46-3 ne commence qu'après sa fusion.
- **Epic 47 (comptes miroirs)** : s'appuiera sur ces générateurs. Proposé : 46 avant 47 (Q8).

## Fichiers communs

| Fichier ou module | Stories | Conflit possible avec |
|---|---|---|
| `crates/kesh-report/src/groupes.rs` (neuf), `lib.rs` | 46-1 | — |
| `crates/kesh-report/src/balance_sheet.rs`, `income_statement.rs` | 46-1 | `fix/bilan-resultat-reporte`, Epic 47 |
| `crates/kesh-report/src/pdf.rs` | 46-3 | `fix/pdf-montants-alignes` (fort), `fix/bilan-resultat-reporte`, Epic 47 |
| `crates/kesh-report/src/csv.rs` | 46-3 | `fix/bilan-resultat-reporte`, Epic 47 |
| `crates/kesh-api/src/routes/reports.rs`, `tests/reports_e2e.rs` | 46-1, 46-3 | `fix/bilan-resultat-reporte` (tests), Epic 47 |
| `crates/kesh-api/src/mcp/outils/rapports.rs`, tests MCP (`mcp_contrat`, `mcp_gardes_lexicales`, `mcp_matrice_manuel`) | 46-4 | Epic 47 |
| `crates/kesh-api/src/routes/accounts.rs`, `kesh-db/src/repositories/accounts.rs` | 46-5 | `fix/api-parent-compte` (fort) |
| `crates/kesh-i18n/locales/*/messages.ftl` | 46-2, 46-3, 46-5 | tous (bloc propre) |
| `frontend/src/lib/features/reports/{BalanceSheetView,IncomeStatementView}.svelte`, `reports.api.ts`, `reports.types.ts`, `routes/(app)/reports/+page.svelte` | 46-2 | `fix/bilan-resultat-reporte`, Epic 47 |
| `frontend/src/routes/(app)/accounts/+page.svelte`, `features/accounts/` | 46-5 | `fix/api-parent-compte` |
| `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal`) | 46-2, 46-5 | tous (recompté) |
| `docs/api-external.md`, `docs/manual/fr/user-manual.tex` | toutes | les trois correctifs, Epic 47 |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | toutes | tous (bloc propre) |

## Migrations

Aucune attendue (D2). Si une story en découvre le besoin, elle le dit à sa validation, et l'epic passe par la
sauvegarde d'avant mise à jour (§ *Migration breaking policy*).

## Décomptes

Issues ouvertes et bugs : **à mesurer au kickoff** (au tag v0.19.0 et ce jour, commandes de § *Priorités des
défauts*) ; objectif au tag de la release de l'epic : moins qu'au tag v0.19.0 (objectif, non barrière).

## Questions ouvertes

- **Q1 — Source des groupes** : `parent_id` seul (D1), ou aussi un regroupement par préfixe de numéro sans compte de
  groupe ? *(Beat)*
- **Q2 — Groupes vides** : masqués (D6), ou option « afficher les groupes vides » ? *(Beat)*
- **Q3 — Solde en devise d'un groupe** : une seule devise seulement (D7), ou une ligne par devise (USD x, EUR y) ? *(Beat)*
- **Q4 — Bilan** : squelette légal conservé et groupes à cheval dissous (D4), ou l'arbre remplace les sections
  (`1 ACTIFS`, `2 PASSIFS` comme titres) ? Où placer les lignes calculées ? *(Beat, puis Guy)*
- **Q5 — API et MCP** : nom et forme du paramètre (`detailLevel`), champ `groupedRows` ; le MCP en a-t-il besoin ? *(Beat, Guy)*
- **Q6 — Balance et grand livre** : par groupes aussi (sous-totaux de la balance) ? Proposé : non, plus tard. *(Beat)*
- **Q7 — Rubriques légales du bilan suisse (art. 959a CO)** : imposer ou contrôler la correspondance des groupes aux
  rubriques minimales ? Proposé : non (le plan PME y correspond déjà). *(Guy)*
- **Q8 — Ordre avec l'Epic 47** (comptes miroirs, qui s'appuiera sur ces générateurs) : 46 d'abord ? *(Guy, Beat)*
- **Q9 — Outil d'aide (46-5)** : utile, ou l'écran du plan suffit-il après `fix/api-parent-compte` ? Règle de
  proposition : « numéro sans son dernier chiffre » ? *(Beat)*
- **Q10 — Niveau par défaut à l'écran** : « Tous les comptes » groupé, ou « à plat » comme aujourd'hui ? *(Beat)*

## Avant le développement

1. Revue de Beat (page `data-not-committed/epic-46-revue.html`), réponses versées ici.
2. Décomptes d'issues mesurés (46-0) ; fiches complétées et validées en boucle.
3. Fusion de `fix/bilan-resultat-reporte` (46-1), `fix/pdf-montants-alignes` (46-3), `fix/api-parent-compte` (46-5).
