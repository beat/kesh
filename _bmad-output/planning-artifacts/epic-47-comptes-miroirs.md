# Epic 47 — Comptes miroirs et paires compensées

**Kickoff : 2026-11-18** (esquisse) ; **orientation approuvée par Guy le 2026-11-19** (§ *Revue de Guy*) **et par Beat le même jour, telle quelle** (§ *Revue de Beat*) : **kickoff accompli le 2026-11-19**. Numéro réservé à Beat (contributeur) par Guy ; responsable : Beat.
Branche `epic/47-comptes-miroirs`, partie de `main` = `773df80d` (v0.19.0). **Passe après l'Epic 46** (rapports par
groupes de comptes), préparé en parallèle. Release : un epic = une release, **ultérieure à la 0.20.0** (Guy, 2026-11-19), numéro au tag par le Project Lead.

Règles des fichiers communs (`CLAUDE.md` § *Travailler à plusieurs développeurs*) : rien au `CHANGELOG.md` (section
*Note de version* par fiche) ; aucun PDF de manuel commité ; i18n et `sprint-status.yaml` en bloc propre ; compteurs
recomptés après chaque reprise de `main`. Pas d'issue ni de PR à ce stade (Beat : lecture seule sur le dépôt de Guy).

## Intention

**Aucun montant négatif dans le bilan ni dans le compte de résultat**, sans écriture de reclassement.

- **Compte miroir** : le solde **de sens inverse** d'un compte se présente sur un autre compte, de l'autre côté
  (dette sociale à solde débiteur → actif ; banque à découvert → passif). Présentation seulement : aucune écriture.
- **Paire compensée** : deux comptes de types opposés dont seul le **net** paraît, sur le compte du sens du net
  (pertes de change 6960 et gains de change 6961 de l'Epic 37 : une ligne).

Besoin réel : à la recette de l'exercice 2024 de Beat, il a fallu au 31.12 des écritures de reclassement (quatre
passifs débiteurs vers des comptes d'actif créés pour cela, gains et pertes de change virés) pour retrouver bilan et
compte de résultat de l'ancien logiciel ligne à ligne. L'epic rend ces écritures inutiles.

## Code lu (à `773df80d`)

| Domaine | Constat | Où |
|---|---|---|
| Comptes | `Account` : `account_type`, `parent_id` (FK), `role`, `postable`, `currency`, `fx_gain/loss_account_id` (pointeurs **sans FK**, revalidés par le dépôt) | `kesh-db/src/entities/account.rs:222-256` ; `migrations/20261114000001_multi_currency_settings.sql:30-73` |
| Bilan | sections par `account_type` (Asset, Liability) ; fonds propres partitionnés **par rôle** ; soldes signés selon la nature (`sign_exprs`), un compte de sens inverse sort **négatif** ; lignes calculées `retained_earnings`, `equity_result` ; `equation_holds` | `kesh-report/src/balance_sheet.rs:60-260` |
| Compte de résultat | Revenue, Expense, `HAVING balance != 0` ; même `AccountBalance` | `kesh-report/src/income_statement.rs` |
| Rendus | PDF (`render_balance_sheet_pdf`, `_income_statement_pdf`), CSV ; écrans `BalanceSheetView`, `IncomeStatementView` ; MCP `kesh_bilan`, `kesh_compte_de_resultat` | `kesh-report/src/{pdf,csv}.rs` ; `frontend/src/lib/features/reports/` ; `kesh-api/src/mcp*` |
| Accueil | invariant : ses totaux égalent ceux du compte de résultat | `kesh-report/src/homepage_summary.rs:6-11` |
| Clôture | `closing_checks` (codes, statuts, détails) | `kesh-db/src/repositories/closing_checks.rs:131-150, 612` |
| Sauvegarde | une **colonne** facultative manquante est tolérée par la règle générique ; une **table** manquante est refusée hors liste close | `kesh-api/src/admin_backup/import.rs:600-640` ; `kesh-db/src/backup.rs:156-245` |
| Migrations | 95 ; `kesh_version_min_required` = 0.17.6 | `docs/migrations-idempotence-audit.md:17, 121` |

## Décisions — décidées (Guy et Beat, 2026-11-19)

- **D1 — Modèle : deux colonnes sur `accounts`, pas de table.** `mirror_account_id BIGINT NULL` (compte miroir) et
  `offset_account_id BIGINT NULL` (partenaire de paire, posé **des deux côtés**, réciproque, écrit dans une
  transaction). Pointeurs sans clé étrangère, comme `fx_gain_account_id`. Raison : une table neuve rend toute
  sauvegarde 0.19.0 inimportable (ou exige une liste d'absences admises) ; deux colonnes facultatives passent la règle
  générique. Un compte n'a pas les deux (`CHECK`). Pas de chaîne : un compte miroir n'a lui-même ni miroir ni paire.
  *Guy, 2026-11-19 :* d'accord, deux colonnes facultatives, compatibles avec l'import des sauvegardes 0.19.0 ; une table l'empêcherait. → **décidé (Guy et Beat, 2026-11-19).**
- **D2 — Miroir** : de l'autre côté du même rapport (actif ↔ passif hors fonds propres ; produit ↔ charge), même
  société. Le compte miroir peut être non imputable (compte de présentation créé pour l'usage) ou porter ses propres
  écritures ; sa ligne = son solde + le solde inversé de ses sources. Un compte dont le solde est dans son sens normal
  reste à sa place.
  *Guy, 2026-11-19 :* d'accord ; pas de chaîne : un miroir ne pointe jamais vers un compte qui a lui-même un miroir — **refus nommé** (47-1 AC3). → **décidé (Guy et Beat, 2026-11-19).**
- **D3 — Paire** : deux comptes de **types opposés** du même rapport (produit/charge ; actif/passif). Le net paraît
  sur le compte du sens du net, sous son numéro et son libellé ; net nul → aucune ligne. Deux comptes de même type ne
  forment pas de paire : leur somme est un groupe (Epic 46).
  *Guy, 2026-11-19 :* approuvé, sous réserve de Q9 : la paire compense une charge et un produit (CO 958c al. 1 ch. 7) → avertissement à sa création, mention « présentation compensée » au PDF. → **décidé (Guy et Beat, 2026-11-19).**
- **D4 — Bascule à la date d'arrêté de tout rapport** (bilan à toute date, compte de résultat sur toute période), pas
  seulement à la clôture : Kesh calcule les soldes en direct et ne passe aucune écriture de clôture.
  *Guy, 2026-11-19 :* d'accord, un rapport intermédiaire se présente comme celui de clôture. → **décidé (Guy et Beat, 2026-11-19).**
- **D5 — Une seule fonction de présentation** dans `kesh-report` (pure, testée seule), appliquée aux lignes avant
  les totaux et avant les groupes de l'Epic 46. Bilan et compte de résultat seulement ; écran, PDF, CSV, API, MCP.
  L'équation du bilan tient (les deux côtés bougent du même montant) ; le résultat net ne change pas.
  *Guy, 2026-11-19 :* approuvé ; présentation d'abord, puis regroupement (Q5). → **décidé (Guy et Beat, 2026-11-19).**
- **D6 — Présentation brute toujours disponible** : paramètre `presentation=brute` à l'API (et bascule à l'écran)
  rend le rapport d'aujourd'hui, compte par compte, signé. Défaut : présentée. L'accueil reste sur les totaux bruts.
  *Guy, 2026-11-19 :* essentiel : la présentation brute reste disponible **partout** (écran, PDF, CSV, API), notamment pour les comptes annuels officiels si une paire est utilisée (Q9). → **décidé (Guy et Beat, 2026-11-19).**
- **D7 — Balance de vérification, grand livre, journal, TVA, projets : inchangés.** Le détail par compte y reste.
  *Guy, 2026-11-19 :* confirmé (Q4) : balance, grand livre et journal montrent les soldes réels. → **décidé (Guy et Beat, 2026-11-19).**
- **D8 — Fonds propres et lignes calculées exclus** : un compte de rôle de fonds propres ne prend ni miroir ni paire ;
  une perte (reportée ou de l'exercice) reste en déduction des fonds propres (CO 959a).
  *Guy, 2026-11-19 :* d'accord ; fonds propres hors du mécanisme, une perte reste en déduction (CO 959a). → **décidé (Guy et Beat, 2026-11-19).**
- **D9 — Traçabilité** : chaque ligne présentée porte à l'API ses `sources` (compte, solde brut) ; l'écran les montre
  au survol ou au dépliage ; le PDF et le CSV, une marque et une note.
  *Guy, 2026-11-19 :* d'accord ; la marque du PDF dit en clair **« présentation compensée »** quand une paire est appliquée (Q9). → **décidé (Guy et Beat, 2026-11-19).**
- **D10 — Migration de la 47-1, non breaking** : `2026112?000001_account_presentation_links.sql` (horodatage au
  développement), DDL seul : deux `ADD COLUMN … BIGINT NULL`, deux index, `CHECK` (pas soi-même, pas les deux). Ne
  relève pas `kesh_version_min_required` (0.17.6). P5 : ligne à l'audit, compteurs recomptés depuis la source à la reprise de `main` (95 à `773df80d`, plus la 47-1 et les migrations fusionnées avant elle, dont `20261118000002` de l'Epic 49) ; P6 : grep des
  sites positionnels ; P7 : rien à trier (aucune écriture de données) ; squash `test-schema` mis à jour. Une sauvegarde
  0.19.0 s'importe (colonnes facultatives) ; la version précédente refuse la base migrée et les sauvegardes neuves :
  retour par la sauvegarde d'avant la mise à jour. **Annoncée au projet au kickoff** (tableau des fichiers communs).
  *Guy, 2026-11-19 :* d'accord sur le principe. **Ordre : cette migration passe après celle de l'Epic 49 (`20261118000002`, release 0.20.0 du 2026-11-27)**, dans une release ultérieure à la 0.20.0 ; l'Epic 47 dépend de l'Epic 46. Annoncée au projet GitHub au kickoff, avec **la sauvegarde complète avant mise à jour en tête des notes**. → **décidé (Guy et Beat, 2026-11-19).**

## Exigences

- **FR-47-1** Désigner, retirer, modifier un miroir ou une paire sur un compte (API, écran), audité.
- **FR-47-2** Refuser un lien invalide (autre société, même côté, fonds propres, chaîne, soi-même, compte archivé
  comme cible) par un code d'erreur nommé.
- **FR-47-3** Bilan et compte de résultat présentés selon D2–D5 ; brut sur demande (D6).
- **FR-47-4** Sources de chaque ligne présentée (D9) ; MCP et CSV au même contrat que l'écran.
- **FR-47-5** Contrôle de clôture non bloquant : comptes à solde de sens inverse sans miroir ni paire (47-4).
- **FR-47-6** Avertissement (CO 958c al. 1 ch. 7) à la création d'une paire ; mention « présentation compensée » au PDF d'un rapport où une paire s'applique (Q9).
- **FR-47-7** Archivage refusé d'un compte cible d'un miroir ou partenaire d'une paire, message nommant le compte lié (Q10).
- **FR-47-8** Documentation du mécanisme (exigence de Guy) : manuel utilisateur, manuel d'administration, `docs/api-external.md` ; manuels FR, et les traductions qui existeront à la livraison.
- **NFR-47-1** Aucune écriture comptable produite ; balance et grand livre identiques avant/après.
- **NFR-47-2** Équation du bilan et résultat net identiques en brut et en présenté (test de propriété).
- **NFR-47-3** Sauvegarde 0.19.0 importable ; aucun relèvement de version minimale.
- **NFR-47-4** Exemple chiffré du manuel rejoué par un test d'intégration.

## Stories

| Story | Objet | Migration | Dépend de |
|---|---|---|---|
| 47-0 | kickoff : décomptes, validation du fichier, annonce de la migration — **done** (2026-11-19) | non | — |
| 47-1a | modèle : migration, entité, dépôt, validation, archivage, retypage, lecteur des liens, sauvegarde, colonnes de l'export CSV, mapping HTTP des deux refus (sous-match exhaustif de `DbError`) | **oui**, non breaking | — |
| 47-1b | API des comptes, avertissement de la paire, MCP `kesh_plan_comptable` (plafonds révisés), test de l'export global, `api-external.md`, manuel d'administration (colonnes de l'export CSV : en 47-1a) | non | 47-1a |
| 47-2a | fonction de présentation, rapports JSON (`presentation=brute`, `sources`), MCP `kesh_bilan` / `kesh_compte_de_resultat` | non | 47-1a ; **avant** le regroupement de l'Epic 46 (recâblage à sa fusion) |
| 47-2b | PDF et CSV présentés : marque, note, « présentation compensée », brut | non | 47-2a ; #1327 et `fix/bilan-resultat-reporte` fusionnés ; 48-11 et 46-3 si fusionnées |
| 47-3a | écran du plan comptable : section *Présentation*, avertissement CO 958c, refus nommés, marque | non | 47-1b |
| 47-3b | écrans des rapports : marque, sources, bascule brute, exports, E2E du parcours, manuel utilisateur | non | 47-2a, 47-2b, 47-3a |
| 47-4 | contrôle de clôture K10 « sens inverse sans miroir ni paire », non bloquant | non | 47-1a |
| 47-5 | solde de la dette de l'epic (**plafond : 2 issues**, fixé au kickoff) | non | 47-1a à 47-4 closes (fin d'epic, avant le tag) |

**Dérogation à la borne de six stories** — **décision de Beat (2026-11-19)** : l'Epic 47 reste **un seul epic** de huit stories (47-1a, 47-1b, 47-2a, 47-2b, 47-3a, 47-3b, 47-4, 47-5) ; la 47-4 n'en sort pas. Dérogation assumée pour un epic de **fonctionnalité nouvelle**, dite au kickoff (`CLAUDE.md` § *Numérotation des versions*), aucune story n'étant trop longue ; **à signaler à Guy** (Beat pense qu'il sera d'accord).

**Découpage du 2026-11-19 (splitting préventif, `CLAUDE.md` § *Règle de splitting préventif*)** : à l'écriture des
fiches complètes, 47-1, 47-2 et 47-3 touchaient chacune plus de cinq modules au grain retenu par la release 0.20.0
(49-1 « neuf modules ») ; chacune est découpée en deux (clés `47-1`, `47-2`, `47-3` passées à `split`). L'epic compte
donc **huit stories hors kickoff**, solde comprise : il dépasse la borne de six (`CLAUDE.md` § *Numérotation des
versions*), ce qu'admet un epic de **fonctionnalité nouvelle** à condition de le dire au kickoff — **dit ici** (décision de Beat du 2026-11-19, à signaler à Guy ; 47-4 reste dans l'epic). 47-1a, 47-1b et 47-4 peuvent avancer pendant
l'Epic 46 ; 47-2a se développe sans lui (fonction pure appliquée avant tout regroupement) et se recâble à sa fusion ;
47-2b attend #1327 et `fix/bilan-resultat-reporte`.

## Ordre de développement

1. **47-1a** d'abord : elle porte la migration, **ne sort jamais** (une story sortie emporte sa migration ; sans elle, rien ne se présente), et son horodatage se contrôle à la reprise de `main` (D10) ; `fix/api-parent-compte` (même `update`, mêmes erreurs : fiche 47-1a) repris s'il est fusionné, sinon le second arrivé résout.
2. **En parallèle, après la 47-1a** : 47-1b (API, MCP, export), 47-2a (fonction de présentation, JSON, MCP) et 47-4 (K10). Fichiers partagés entre elles : catalogues i18n, manuels, `docs/api-external.md` (les trois) et `crates/kesh-api/tests/mcp_outils_e2e.rs` (47-1b, 47-2a) ; chacune y écrit son bloc, la seconde fusionnée dans la branche d'epic résout en gardant les deux.
3. **47-3a** après la 47-1b.
4. **Reprise de `main`** (Epic 46, #1327, #1346, 48-11 de la 0.20.0 si fusionnés) **avant** la 47-2b et avant le recâblage de la 47-2a ; puis **47-2b** (après 47-2a et les correctifs de `pdf.rs` / `csv.rs`).
5. **47-3b** après 47-2a, 47-2b et 47-3a (E2E du parcours complet, manuel utilisateur).
6. **47-5** en dernier, une fois 47-1a à 47-4 closes : elle ne prend que la dette de ces stories.

**Si le temps manque** (coupure du jeudi soir : n'entrent que les stories closes ; `CLAUDE.md` § *Cadence*) : epic de contributeur, **l'orchestrateur n'y touche pas** (`CLAUDE.md` § *Cadence*) ; la coupure se décide par **Beat**, responsable de l'epic, et **le Project Lead garde le dernier mot**. Ordre : la **47-5** sort la première (ses lignes du registre restent ouvertes en issues, au milestone que décide le Project Lead, et l'absence de solde s'écrit à la PR de release et à la rétrospective), puis, si la coupure l'impose (la 47-4 reste dans l'epic par décision de Beat), la **47-4** ; la 47-3b ne sort pas sans la 47-2b ; la 47-1a ne sort jamais. L'epic ne se publie pas amputé de sa présentation (47-2a à 47-3b) : sans elle, les liens n'ont aucun effet visible ; si elle n'est pas close, **l'epic entier attend une release suivante** (la date de la release ne bouge pas : elle part sans lui) ; écart assumé avec « un epic qui n'aboutit pas se réduit » (`CLAUDE.md` § *Cadence*) : réduit à ses liens seuls, l'epic n'aurait aucun effet visible, et le Project Lead a le dernier mot).

Hors epic : la story de correction de bugs que chaque release planifiée porte (`CLAUDE.md` § *Cadence*) est celle de la **release**, non de l'epic ; elle compte dans la borne de six de l'epic qui la porte (un epic interne du Project Lead, comme l'Epic 48 à la 0.20.0), non dans les huit stories ici.

## Registre de la dette de l'epic

Les revues de chaque story consignent ici, à la clôture de leur boucle (chaque fiche 47-1a à 47-4 le rappelle en § *References*), une ligne par finding **non remédié** qui devrait s'ouvrir en issue (« on trace toujours ») : story, passe, sévérité, objet, issue (numéro reporté quand le Project Lead l'ouvre), sort (ouverte, retenue en 47-5, corrigée en 47-5, reportée). L'agent de Beat n'ouvre pas d'issue (lecture seule sur le dépôt de Guy) : le Project Lead ouvre les issues d'après ce registre, et la 47-5 en part. Une ligne par problème, sans regroupement ; l'étiquette `bug` seulement pour un comportement faux visible (`CLAUDE.md` § *Priorités des défauts*).

| Story | Passe | Sévérité | Objet | Issue | Sort |
|---|---|---|---|---|---|
| — | — | — | (vide à ce jour) | — | — |

## Dépendance à l'Epic 46

Les groupes de l'Epic 46 additionnent des lignes de compte en sous-totaux. Règle proposée : **la présentation
précède le regroupement** — une ligne présentée entre dans le groupe du compte qui la porte (le miroir, ou le compte
du sens du net), jamais dans celui du compte d'origine. Il faut donc, dans l'Epic 46, un point d'entrée unique où
les lignes passent avant d'être groupées (à demander à son auteur ; Q5).

## Fichiers communs

| Fichier | 47 | Epic 46 | `fix/pdf-montants-alignes` | `fix/bilan-resultat-reporte` | `fix/api-parent-compte` |
|---|---|---|---|---|---|
| `kesh-report/src/balance_sheet.rs` | 47-2a | probable | — | oui | — |
| `kesh-report/src/income_statement.rs` | 47-2a | probable | — | — | — |
| `kesh-report/src/pdf.rs` | 47-2b ; 47-2a (littéraux des tests : champs `sources`, `compensated`) | probable | **oui (+1100 lignes)** | oui | — |
| `kesh-report/src/csv.rs` | 47-2b ; 47-2a (littéraux des tests) | probable | — | oui | — |
| `frontend/…/reports/` (vues, `reports.types.ts`, `reports.api.ts`) | 47-3b | probable | — | oui | — |
| `kesh-db/src/entities/account.rs`, `repositories/accounts.rs` | 47-1a | possible | — | — | oui |
| `kesh-api/src/routes/accounts.rs`, `errors.rs` (db, api) | 47-1a, 47-1b | possible | — | — | oui |
| `frontend/…/accounts/`, `routes/(app)/accounts/+page.svelte` | 47-3a | — | — | — | oui |
| catalogues i18n (4 locales), `sitesTotal` | 47-1a (messages des deux refus) à 47-4 | oui | oui | — | oui |
| `docs/manual/fr/user-manual.tex`, `docs/api-external.md` | 47-1b à 47-4 | oui | oui | oui | oui |
| `kesh-db/migrations/`, audit, `test-schema/`, `migrations_upgrade_path.rs`, `account_columns_inventory.rs` | 47-1a | si migration | — | — | — |
| `kesh-db/src/repositories/closing_checks.rs`, `FiscalYearChecks.svelte` | 47-4 | — | — | — | — |
| MCP (`kesh_plan_comptable`, `kesh_bilan`, `kesh_compte_de_resultat`) | 47-1b, 47-2a | probable | — | — | — |
| `kesh-api/tests/services_motif.rs` (détails d'audit en dur), `tests/homepage_summary_e2e.rs` | 47-2a | possible | — | — | — |
| `kesh-api/src/exports/csv_tables.rs` | 47-1a (colonnes), 47-1b (test du ZIP) | — | — | — | — |

**Hors tableau (0.20.0)** : la story **48-11** (arrondi au centime des rapports CSV et PDF, après #1327) touche `kesh-report/src/pdf.rs` et `csv.rs`, comme 47-2b ; elle part avant : le second arrivé (47-2b) reprend et résout. Les stories de l'Epic 49 changent les catalogues i18n et la migration (horodatage de la 47-1a postérieur).

Geste : reprendre `main` (avec l'Epic 46 et les trois correctifs) dans la branche **avant** 47-2b et avant le recâblage de 47-2a ; ordre des
migrations contrôlé à cette reprise si l'Epic 46 en porte une.

## Décomptes d'issues

**Relevés par le Project Lead au kickoff de la 0.20.0** (`release-0.20.0.md` § *Décompte au kickoff et objectif*, sur
`origin/main` à `fd04099f`) ; non remesurés ici (`gh` hors de portée de l'agent de Beat) :

| | Tag v0.19.0 | Kickoff 0.20.0 (2026-11-18) | Le soir du 2026-11-18 |
|---|---|---|---|
| Issues ouvertes | 496 | 496 | 507 |
| Bugs (`bug` ou `known-failure`) | 247 | 247 | 257 |
| P1 ouvertes | 14 | 14 | 14 |

Hors les 56 issues masquées du compte de l'agent de Beat (#1266–#1271, #1277–#1326), non comptées (décision de Guy).
**Plafond de la story de solde (47-5)** : **2 issues**, fixé ici au kickoff (`CLAUDE.md` § *Priorités des défauts*).
**Objectif de l'epic** : moins d'issues ouvertes au tag de cette release que **au tag précédent, v0.20.0** (CLAUDE.md § *Priorités des défauts* : objectif, non barrière) ; ce compte n'existe pas encore (la 0.20.0 n'est pas taguée) et se relève au tag par le Project Lead, comme le compte du tag de l'epic à la PR de release et à la rétrospective, avec l'écart et sa cause. Le tableau ci-dessus est la base du kickoff, non la référence de l'objectif.

**Objectif de l'epic** (règle de release, non barrière) : moins d'issues ouvertes au tag de sa release qu'au tag de la
release précédente (la 0.20.0, compte **à relever au tag v0.20.0**) ; le compte au tag de l'Epic 47 s'écrit à sa PR
de release et à sa rétrospective, avec l'écart et sa cause.

## Questions — répondues (Guy, 2026-11-19), réponses adoptées (Guy et Beat, 2026-11-19)

- **Q1** — Modèle : deux colonnes sur le compte (D1) ou table de paires/miroirs ? (D1 recommandé : sauvegardes.)
  *Guy, 2026-11-19 :* colonnes sur le compte (D1). → **décidé (Guy et Beat, 2026-11-19).**
- **Q2** — Bascule à toute date d'arrêté (D4) ou à la date de clôture seulement ?
  *Guy, 2026-11-19 :* à toute date (D4). → **décidé (Guy et Beat, 2026-11-19).**
- **Q3** — Comptes en devise : lien admis entre devises différentes ? Proposé : admis ; le solde en devise de la
  ligne présentée n'est montré que si toutes ses sources ont la même devise.
  *Guy, 2026-11-19 :* oui, avec la règle proposée (solde en devise seulement si les sources ont la même devise). → **décidé (Guy et Beat, 2026-11-19).**
- **Q4** — Balance et grand livre hors périmètre (D7) : confirmé ?
  *Guy, 2026-11-19 :* confirmé : balance, grand livre et journal montrent les soldes réels, sans présentation. → **décidé (Guy et Beat, 2026-11-19).**
- **Q5** — Epic 46 : présentation avant regroupement (§ *Dépendance*) ; point d'entrée à convenir avec son auteur.
  *Guy, 2026-11-19 :* oui : présentation d'abord, puis regroupement ; la ligne va au groupe du compte qui la porte. → **décidé (Guy et Beat, 2026-11-19).**
- **Q6** — Contrôle de clôture (47-4) : avertissement non bloquant, ou rien ?
  *Guy, 2026-11-19 :* **avertissement non bloquant**, qui signale les comptes à solde inverse sans miroir ni paire, sans empêcher de clore. → **décidé (Guy et Beat, 2026-11-19).**
- **Q7** — Libellé de la ligne présentée : celui du compte qui la porte (proposé), ou un libellé de présentation
  propre au lien (colonne de plus) ?
  *Guy, 2026-11-19 :* le libellé du compte qui porte la ligne. → **décidé (Guy et Beat, 2026-11-19).**
- **Q8** — Vocabulaire : « compte miroir » sert déjà, dans le code, aux paires de contre-passation
  (`homepage_summary.rs`). Garder « miroir » à l'écran, ou « compte de présentation inverse » ?
  *Guy, 2026-11-19 :* « compte miroir » convient : le mot n'apparaît que dans des commentaires internes du code, jamais à l'écran ni au manuel. → **décidé (Guy et Beat, 2026-11-19).**
- **Q9** — Compensation et CO 958c al. 1 ch. 7 (interdiction de compenser) : la paire est un choix de l'utilisateur ;
  Kesh l'admet sans restriction, ou avec un avertissement au plan comptable ?
  *Guy, 2026-11-19 :* **avec un avertissement.** Le miroir ne compense rien (un découvert au passif est la présentation correcte) ; la paire compense une charge et un produit, ce que CO 958c al. 1 ch. 7 interdit dans les comptes annuels. Donc : **avertissement à la création de la paire**, mention **« présentation compensée »** au PDF du rapport, **présentation brute toujours disponible** (D6) pour les comptes officiels. → **décidé (Guy et Beat, 2026-11-19).**
- **Q10** — Archiver un compte cible d'un miroir ou partenaire d'une paire : refuser, ou retirer le lien ?
  *Guy, 2026-11-19 :* **refuser l'archivage** tant qu'un compte pointe vers la cible, avec un message qui **nomme le compte lié**. → **décidé (Guy et Beat, 2026-11-19).**
- **Q11** — Accueil (D6) : reste brut ? (Ses totaux ne recoupent alors que le compte de résultat brut.)
  *Guy, 2026-11-19 :* oui, l'accueil reste sur les totaux bruts. → **décidé (Guy et Beat, 2026-11-19).**

## Revue de Guy (2026-11-19)

Page de revue relue par Guy : **toutes les sections approuvées** (intention, exemple, D1–D10, Q1–Q11, stories 47-0 à
47-5). Ses réponses sont reportées sous chaque D et Q (« *Guy, 2026-11-19* »). Points de fond :

- **Q9 — paire compensée avec avertissement** : avertissement à sa création (CO 958c al. 1 ch. 7), mention
  « présentation compensée » au PDF, présentation brute toujours disponible pour les comptes officiels. Le miroir,
  lui, ne compense rien.
- **Q10** : archivage d'une cible refusé, message nommant le compte lié. **Q6** : contrôle de clôture non bloquant.
  **D2** : chaîne refusée par un code nommé.
- **D10 — ordre des migrations** : la migration de la 47-1 passe **après** `20261118000002` (Epic 49, 0.20.0 du
  2026-11-27), dans une release **ultérieure à la 0.20.0** ; horodatage postérieur, contrôlé à la reprise de `main`.
  Sauvegarde complète avant mise à jour **en tête des notes** ; annonce au projet GitHub au kickoff.
- **Exigence de Guy — documentation** : le mécanisme (miroir, paire, brut/présenté, avertissement CO 958c) est
  documenté dans le **manuel utilisateur** (poser un miroir ou une paire, ce que montrent bilan et compte de
  résultat, présentation brute, exemple chiffré), le **manuel d'administration** (migration, sauvegarde avant mise à
  jour, API et export) et **`docs/api-external.md`** (`mirrorAccountId`, `offsetAccountId`, `presentation=brute`).
  Manuels FR aujourd'hui (`docs/manual/fr/`) ; s'ils sont traduits d'ici la livraison, le mécanisme y figure aussi.
  Inscrit aux critères de 47-1, 47-2, 47-3 ; **leurs prompts de revue nomment les manuels** et exigent le contrôle du
  PDF (`CLAUDE.md` § *Le prompt d'une passe doit NOMMER le manuel*).

## Revue de Beat (2026-11-19) : approuvé tel quel

Beat a relu la page de revue complétée des réponses de Guy et **l'approuve telle quelle, sans autre commentaire** :
toutes les décisions D1–D10 et toutes les réponses de Guy aux questions Q1–Q11 sont **adoptées** ; chaque D et
chaque Q porte ci-dessus la mention « décidé (Guy et Beat, 2026-11-19) ». Le kickoff (47-0) est accompli ;
`sprint-status.yaml` : `epic-47: in-progress`, `47-0-kickoff: done`.

## Validation des fiches (2026-11-19)

Les huit fiches ont été validées en boucle (`CLAUDE.md` § *Review Iteration Rule*), en contexte frais, P1 par Sonnet
ou Opus, rotation Sonnet ↔ Opus, passes ciblées sur la dernière remédiation en fin de boucle ; chaque boucle se clôt
sur une passe sans finding au-dessus de LOW. `Status: ready-for-dev` aux huit fiches et au registre. Décomptes recomptés
depuis les entrées de Change Log des fiches et les rapports de `kesh-gate-logs/epic-47-validation/` (29 rapports) ;
sévérités en C / H / M / L, les LOW tous remédiés.

| Fiche | Passes | Modèles | Trend (C/H/M/L) | Dernier commit de validation |
|---|---|---|---|---|
| 47-1a | 5 (P3 à P5 ciblées) | Opus, Sonnet, Opus, Opus, Sonnet | 0/1/5/6 → 0/0/2/2 → 0/0/5/1 → 0/0/1/2 → 0/0/0/0 | `d5eff9e0` (P4 ; la P5 n'a rien modifié) |
| 47-1b | 3 (P3 ciblée) | Sonnet, Opus, Sonnet | 0/2/9/4 → 0/0/4/4 → 0/0/0/1 | `c384655a` |
| 47-2a | 3 | Opus, Sonnet, Opus | 0/0/6/9 → 0/0/1/1 → 0/0/0/7 | `2367e233` |
| 47-2b | 3 (P3 ciblée) | Sonnet, Opus, Sonnet | 0/3/6/3 → 0/0/3/6 → 0/0/0/1 | `93b1669b` |
| 47-3a | 5 (P3 à P5 ciblées) + clôture par la P5 ciblée de la 47-1a | Opus, Sonnet, Opus, Sonnet, Opus ; clôture Sonnet | 0/1/9/5 → 0/0/5/5 → 0/0/1/3 → 0/0/1/2 → 0/0/1/3 → 0/0/0/0 | `a1870a45` (P5) |
| 47-3b | 4 (P3, P4 ciblées) | Sonnet, Opus, Sonnet, Opus | 0/2/6/4 → 0/0/3/6 → 0/0/1/1 → 0/0/0/3 | `004cf24b` |
| 47-4 | 3 (P3 ciblée) | Opus, Sonnet, Opus | 0/0/7/7 → 0/0/1/2 → 0/0/0/3 | `44fbd414` |
| 47-5 | 3 (P3 ciblée) | Sonnet, Opus, Sonnet | 0/0/4/3 → 0/0/4/4 → 0/0/0/3 | `d35dec24` |

Total : 29 passes (la P5 ciblée de la 47-1a clôt aussi la 47-3a, dont elle exerce l'AC-2) ; 0 CRITICAL, 9 HIGH,
toutes en P1. Deux passes sans baisse de sévérité, signalées ici (`CLAUDE.md` § *Règle de splitting préventif*) :
47-1a P3 (2 → 5 M : trois nés de la remédiation P2, deux transmis par la P2 de la 47-1b et la P4 de la 47-3a) et 47-5
P2 (4 → 4 M : trois nés de la remédiation P1) ; toutes deux closes par des passes ciblées qui ne rendent plus de
MEDIUM, sans nouveau découpage (les fiches sont déjà issues du découpage de 47-1, 47-2, 47-3). Haiku n’a mené
aucune passe. Registre de la dette : aucun
finding non remédié (les écarts « signalés, non remédiés » des P2 de la 47-1b et P3 de la 47-4 ont été repris par les
fiches visées : 47-1a AC-15 et AC-9, 47-2a AC-1 et AC-2).

**Reste à faire au développement** :

- **47-1a, T0** : vérifier sur la MariaDB de dev que le `CHECK` « pas soi-même » (sur `id`, colonne
  `AUTO_INCREMENT`) est accepté ; arrêt et question à Beat s'il est refusé (D10, C-47-1a-4). Horodatage de la migration
  postérieur à `20261118000002` **et** à la migration de l'Epic 49 (compte en dur et fenêtre `N = total − 34` de
  `migrations_upgrade_path.rs`), contrôlé à la reprise de `main`.
- **Tailles MCP** : 47-1b, plafonds de `kesh_plan_comptable` **mesurés** (estimés 34 / 61, jamais devinés ; changement
  de contrat MCP écrit aux Notes de version) ; 47-2a, mesure du bilan et du compte de résultat rejouée avec les champs
  `sources`.
- **Dépendances** : 47-2b et le recâblage de la 47-2a attendent l'**Epic 46** (point d'entrée du regroupement, Q5) ;
  47-2b attend **#1327** (`fix/pdf-montants-alignes`) et **#1346** (`fix/bilan-resultat-reporte`) fusionnés, et la 48-11
  si elle passe avant ; 47-1b et 47-3a résolvent avec **#1351** (`fix/api-parent-compte`) selon l'ordre de fusion ;
  migration de l'**Epic 49** avant la nôtre.
- **Annonce de la migration** au projet GitHub « Kesh » : par le **Project Lead** (l'agent de Beat n'écrit pas sur le
  dépôt de Guy).

## Avant le développement

Fait le 2026-11-19 : réponses de Beat (approuvé tel quel), décomptes repris du kickoff de la 0.20.0, fiches complètes
47-1a à 47-5 (`Status: drafted`), fichier de l'Epic 46 lu pour Q5 (`epic/46-rapports-groupes`, `4d23b3f9`).
Reste : approbation des fiches et maquettes par Beat (formulaire hors dépôt), puis validation en boucle de chaque
fiche (`CLAUDE.md` § *Review Iteration Rule*), `Status: ready-for-dev` à sa clôture ; point d'entrée du regroupement
convenu dans la fiche 46-1 (Q5). *(Annoté le 2026-11-19 : validation des huit fiches faite, § *Validation des fiches*
ci-dessus ; `Status: ready-for-dev`.)*
