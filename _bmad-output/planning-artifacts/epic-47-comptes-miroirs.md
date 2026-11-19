# Epic 47 — Comptes miroirs et paires compensées

**Kickoff : 2026-11-18** (esquisse) ; **orientation approuvée par Guy le 2026-11-19** (§ *Revue de Guy*), réponses de Beat attendues. Numéro réservé à Beat (contributeur) par Guy ; responsable : Beat.
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

## Décisions proposées (à valider par Beat, puis Guy)

- **D1 — Modèle : deux colonnes sur `accounts`, pas de table.** `mirror_account_id BIGINT NULL` (compte miroir) et
  `offset_account_id BIGINT NULL` (partenaire de paire, posé **des deux côtés**, réciproque, écrit dans une
  transaction). Pointeurs sans clé étrangère, comme `fx_gain_account_id`. Raison : une table neuve rend toute
  sauvegarde 0.19.0 inimportable (ou exige une liste d'absences admises) ; deux colonnes facultatives passent la règle
  générique. Un compte n'a pas les deux (`CHECK`). Pas de chaîne : un compte miroir n'a lui-même ni miroir ni paire.
  *Guy, 2026-11-19 :* d'accord, deux colonnes facultatives, compatibles avec l'import des sauvegardes 0.19.0 ; une table l'empêcherait.
- **D2 — Miroir** : de l'autre côté du même rapport (actif ↔ passif hors fonds propres ; produit ↔ charge), même
  société. Le compte miroir peut être non imputable (compte de présentation créé pour l'usage) ou porter ses propres
  écritures ; sa ligne = son solde + le solde inversé de ses sources. Un compte dont le solde est dans son sens normal
  reste à sa place.
  *Guy, 2026-11-19 :* d'accord ; pas de chaîne : un miroir ne pointe jamais vers un compte qui a lui-même un miroir — **refus nommé** (47-1 AC3).
- **D3 — Paire** : deux comptes de **types opposés** du même rapport (produit/charge ; actif/passif). Le net paraît
  sur le compte du sens du net, sous son numéro et son libellé ; net nul → aucune ligne. Deux comptes de même type ne
  forment pas de paire : leur somme est un groupe (Epic 46).
  *Guy, 2026-11-19 :* approuvé, sous réserve de Q9 : la paire compense une charge et un produit (CO 958c al. 1 ch. 7) → avertissement à sa création, mention « présentation compensée » au PDF.
- **D4 — Bascule à la date d'arrêté de tout rapport** (bilan à toute date, compte de résultat sur toute période), pas
  seulement à la clôture : Kesh calcule les soldes en direct et ne passe aucune écriture de clôture.
  *Guy, 2026-11-19 :* d'accord, un rapport intermédiaire se présente comme celui de clôture.
- **D5 — Une seule fonction de présentation** dans `kesh-report` (pure, testée seule), appliquée aux lignes avant
  les totaux et avant les groupes de l'Epic 46. Bilan et compte de résultat seulement ; écran, PDF, CSV, API, MCP.
  L'équation du bilan tient (les deux côtés bougent du même montant) ; le résultat net ne change pas.
  *Guy, 2026-11-19 :* approuvé ; présentation d'abord, puis regroupement (Q5).
- **D6 — Présentation brute toujours disponible** : paramètre `presentation=brute` à l'API (et bascule à l'écran)
  rend le rapport d'aujourd'hui, compte par compte, signé. Défaut : présentée. L'accueil reste sur les totaux bruts.
  *Guy, 2026-11-19 :* essentiel : la présentation brute reste disponible **partout** (écran, PDF, CSV, API), notamment pour les comptes annuels officiels si une paire est utilisée (Q9).
- **D7 — Balance de vérification, grand livre, journal, TVA, projets : inchangés.** Le détail par compte y reste.
  *Guy, 2026-11-19 :* confirmé (Q4) : balance, grand livre et journal montrent les soldes réels.
- **D8 — Fonds propres et lignes calculées exclus** : un compte de rôle de fonds propres ne prend ni miroir ni paire ;
  une perte (reportée ou de l'exercice) reste en déduction des fonds propres (CO 959a).
  *Guy, 2026-11-19 :* d'accord ; fonds propres hors du mécanisme, une perte reste en déduction (CO 959a).
- **D9 — Traçabilité** : chaque ligne présentée porte à l'API ses `sources` (compte, solde brut) ; l'écran les montre
  au survol ou au dépliage ; le PDF et le CSV, une marque et une note.
  *Guy, 2026-11-19 :* d'accord ; la marque du PDF dit en clair **« présentation compensée »** quand une paire est appliquée (Q9).
- **D10 — Migration de la 47-1, non breaking** : `2026112?000001_account_presentation_links.sql` (horodatage au
  développement), DDL seul : deux `ADD COLUMN … BIGINT NULL`, deux index, `CHECK` (pas soi-même, pas les deux). Ne
  relève pas `kesh_version_min_required` (0.17.6). P5 : ligne à l'audit, compteurs recomptés depuis la source à la reprise de `main` (95 à `773df80d`, plus la 47-1 et les migrations fusionnées avant elle, dont `20261118000002` de l'Epic 49) ; P6 : grep des
  sites positionnels ; P7 : rien à trier (aucune écriture de données) ; squash `test-schema` mis à jour. Une sauvegarde
  0.19.0 s'importe (colonnes facultatives) ; la version précédente refuse la base migrée et les sauvegardes neuves :
  retour par la sauvegarde d'avant la mise à jour. **Annoncée au projet au kickoff** (tableau des fichiers communs).
  *Guy, 2026-11-19 :* d'accord sur le principe. **Ordre : cette migration passe après celle de l'Epic 49 (`20261118000002`, release 0.20.0 du 2026-11-27)**, dans une release ultérieure à la 0.20.0 ; l'Epic 47 dépend de l'Epic 46. Annoncée au projet GitHub au kickoff, avec **la sauvegarde complète avant mise à jour en tête des notes**.

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
| 47-0 | kickoff : décomptes, validation du fichier, annonce de la migration | non | — |
| 47-1 | modèle : colonnes, dépôt, validation, API comptes, audit, export, sauvegarde | **oui**, non breaking | — |
| 47-2 | présentation au bilan et au compte de résultat : fonction, API, PDF, CSV, MCP | non | 47-1, **Epic 46 fusionné** |
| 47-3 | écrans : plan comptable (lien), rapports (marque, sources, bascule brute), E2E, manuel | non | 47-2 |
| 47-4 | contrôle de clôture « sens inverse sans miroir » | non | 47-1 |
| 47-5 | solde de la dette de l'epic (plafond : 2 issues) | non | fin d'epic |

Six stories, solde comprise (borne de `CLAUDE.md`). 47-1 peut avancer pendant l'Epic 46 ; 47-2 attend sa fusion.

## Dépendance à l'Epic 46

Les groupes de l'Epic 46 additionnent des lignes de compte en sous-totaux. Règle proposée : **la présentation
précède le regroupement** — une ligne présentée entre dans le groupe du compte qui la porte (le miroir, ou le compte
du sens du net), jamais dans celui du compte d'origine. Il faut donc, dans l'Epic 46, un point d'entrée unique où
les lignes passent avant d'être groupées (à demander à son auteur ; Q5).

## Fichiers communs

| Fichier | 47 | Epic 46 | `fix/pdf-montants-alignes` | `fix/bilan-resultat-reporte` | `fix/api-parent-compte` |
|---|---|---|---|---|---|
| `kesh-report/src/balance_sheet.rs` | 47-2 | probable | — | oui | — |
| `kesh-report/src/income_statement.rs` | 47-2 | probable | — | — | — |
| `kesh-report/src/pdf.rs` | 47-2 | probable | **oui (+1100 lignes)** | oui | — |
| `kesh-report/src/csv.rs` | 47-2 | probable | — | oui | — |
| `frontend/…/reports/` (vues, `reports.types.ts`, `reports.api.ts`) | 47-3 | probable | — | oui | — |
| `kesh-db/src/entities/account.rs`, `repositories/accounts.rs` | 47-1 | possible | — | — | oui |
| `kesh-api/src/routes/accounts.rs`, `errors.rs` (db, api) | 47-1 | possible | — | — | oui |
| `frontend/…/accounts/`, `routes/(app)/accounts/+page.svelte` | 47-3 | — | — | — | oui |
| catalogues i18n (4 locales), `sitesTotal` | 47-1 à 47-3 | oui | oui | — | oui |
| `docs/manual/fr/user-manual.tex`, `docs/api-external.md` | 47-1 à 47-3 | oui | oui | oui | oui |
| `kesh-db/migrations/`, audit, `test-schema/`, `migrations_upgrade_path.rs` | 47-1 | si migration | — | — | — |
| `kesh-db/src/repositories/closing_checks.rs` | 47-4 | — | — | — | — |
| MCP (`kesh_plan_comptable`, `kesh_bilan`, `kesh_compte_de_resultat`) | 47-1, 47-2 | probable | — | — | — |
| `kesh-api/src/exports/csv_tables.rs` | 47-1 | — | — | — | — |

Geste : reprendre `main` (avec l'Epic 46 et les trois correctifs) dans la branche **avant** 47-2 ; ordre des
migrations contrôlé à cette reprise si l'Epic 46 en porte une.

## Décomptes d'issues

À mesurer au kickoff (47-0) : issues ouvertes et bugs (commande de `CLAUDE.md` § *Priorités des défauts*), compte
au tag v0.19.0 et objectif. Non mesurés ici.

## Questions ouvertes

- **Q1** — Modèle : deux colonnes sur le compte (D1) ou table de paires/miroirs ? (D1 recommandé : sauvegardes.)
  *Guy, 2026-11-19 :* colonnes sur le compte (D1).
- **Q2** — Bascule à toute date d'arrêté (D4) ou à la date de clôture seulement ?
  *Guy, 2026-11-19 :* à toute date (D4).
- **Q3** — Comptes en devise : lien admis entre devises différentes ? Proposé : admis ; le solde en devise de la
  ligne présentée n'est montré que si toutes ses sources ont la même devise.
  *Guy, 2026-11-19 :* oui, avec la règle proposée (solde en devise seulement si les sources ont la même devise).
- **Q4** — Balance et grand livre hors périmètre (D7) : confirmé ?
  *Guy, 2026-11-19 :* confirmé : balance, grand livre et journal montrent les soldes réels, sans présentation.
- **Q5** — Epic 46 : présentation avant regroupement (§ *Dépendance*) ; point d'entrée à convenir avec son auteur.
  *Guy, 2026-11-19 :* oui : présentation d'abord, puis regroupement ; la ligne va au groupe du compte qui la porte.
- **Q6** — Contrôle de clôture (47-4) : avertissement non bloquant, ou rien ?
  *Guy, 2026-11-19 :* **avertissement non bloquant**, qui signale les comptes à solde inverse sans miroir ni paire, sans empêcher de clore.
- **Q7** — Libellé de la ligne présentée : celui du compte qui la porte (proposé), ou un libellé de présentation
  propre au lien (colonne de plus) ?
  *Guy, 2026-11-19 :* le libellé du compte qui porte la ligne.
- **Q8** — Vocabulaire : « compte miroir » sert déjà, dans le code, aux paires de contre-passation
  (`homepage_summary.rs`). Garder « miroir » à l'écran, ou « compte de présentation inverse » ?
  *Guy, 2026-11-19 :* « compte miroir » convient : le mot n'apparaît que dans des commentaires internes du code, jamais à l'écran ni au manuel.
- **Q9** — Compensation et CO 958c al. 1 ch. 7 (interdiction de compenser) : la paire est un choix de l'utilisateur ;
  Kesh l'admet sans restriction, ou avec un avertissement au plan comptable ?
  *Guy, 2026-11-19 :* **avec un avertissement.** Le miroir ne compense rien (un découvert au passif est la présentation correcte) ; la paire compense une charge et un produit, ce que CO 958c al. 1 ch. 7 interdit dans les comptes annuels. Donc : **avertissement à la création de la paire**, mention **« présentation compensée »** au PDF du rapport, **présentation brute toujours disponible** (D6) pour les comptes officiels.
- **Q10** — Archiver un compte cible d'un miroir ou partenaire d'une paire : refuser, ou retirer le lien ?
  *Guy, 2026-11-19 :* **refuser l'archivage** tant qu'un compte pointe vers la cible, avec un message qui **nomme le compte lié**.
- **Q11** — Accueil (D6) : reste brut ? (Ses totaux ne recoupent alors que le compte de résultat brut.)
  *Guy, 2026-11-19 :* oui, l'accueil reste sur les totaux bruts.

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

**Réponses de Beat** à la même page : attendues (formulaire mis à jour avec les réponses de Guy en regard).

## Avant le développement

Réponses de Beat (formulaire ; accord de Guy obtenu le 2026-11-19), décomptes 47-0, fiches 47-1 à 47-5 complétées
et validées en boucle (`CLAUDE.md` § *Review Iteration Rule*), fichier de l'Epic 46 lu pour Q5.
