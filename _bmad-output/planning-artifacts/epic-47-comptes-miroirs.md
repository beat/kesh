# Epic 47 — Comptes miroirs et paires compensées

**Kickoff : 2026-11-18** (esquisse, à valider). Numéro réservé à Beat (contributeur) par Guy ; responsable : Beat.
Branche `epic/47-comptes-miroirs`, partie de `main` = `773df80d` (v0.19.0). **Passe après l'Epic 46** (rapports par
groupes de comptes), préparé en parallèle. Release : un epic = une release, numéro au tag par le Project Lead.

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
- **D2 — Miroir** : de l'autre côté du même rapport (actif ↔ passif hors fonds propres ; produit ↔ charge), même
  société. Le compte miroir peut être non imputable (compte de présentation créé pour l'usage) ou porter ses propres
  écritures ; sa ligne = son solde + le solde inversé de ses sources. Un compte dont le solde est dans son sens normal
  reste à sa place.
- **D3 — Paire** : deux comptes de **types opposés** du même rapport (produit/charge ; actif/passif). Le net paraît
  sur le compte du sens du net, sous son numéro et son libellé ; net nul → aucune ligne. Deux comptes de même type ne
  forment pas de paire : leur somme est un groupe (Epic 46).
- **D4 — Bascule à la date d'arrêté de tout rapport** (bilan à toute date, compte de résultat sur toute période), pas
  seulement à la clôture : Kesh calcule les soldes en direct et ne passe aucune écriture de clôture.
- **D5 — Une seule fonction de présentation** dans `kesh-report` (pure, testée seule), appliquée aux lignes avant
  les totaux et avant les groupes de l'Epic 46. Bilan et compte de résultat seulement ; écran, PDF, CSV, API, MCP.
  L'équation du bilan tient (les deux côtés bougent du même montant) ; le résultat net ne change pas.
- **D6 — Présentation brute toujours disponible** : paramètre `presentation=brute` à l'API (et bascule à l'écran)
  rend le rapport d'aujourd'hui, compte par compte, signé. Défaut : présentée. L'accueil reste sur les totaux bruts.
- **D7 — Balance de vérification, grand livre, journal, TVA, projets : inchangés.** Le détail par compte y reste.
- **D8 — Fonds propres et lignes calculées exclus** : un compte de rôle de fonds propres ne prend ni miroir ni paire ;
  une perte (reportée ou de l'exercice) reste en déduction des fonds propres (CO 959a).
- **D9 — Traçabilité** : chaque ligne présentée porte à l'API ses `sources` (compte, solde brut) ; l'écran les montre
  au survol ou au dépliage ; le PDF et le CSV, une marque et une note.
- **D10 — Migration de la 47-1, non breaking** : `2026112?000001_account_presentation_links.sql` (horodatage au
  développement), DDL seul : deux `ADD COLUMN … BIGINT NULL`, deux index, `CHECK` (pas soi-même, pas les deux). Ne
  relève pas `kesh_version_min_required` (0.17.6). P5 : ligne à l'audit, compteurs recomptés (96) ; P6 : grep des
  sites positionnels ; P7 : rien à trier (aucune écriture de données) ; squash `test-schema` mis à jour. Une sauvegarde
  0.19.0 s'importe (colonnes facultatives) ; la version précédente refuse la base migrée et les sauvegardes neuves :
  retour par la sauvegarde d'avant la mise à jour. **Annoncée au projet au kickoff** (tableau des fichiers communs).

## Exigences

- **FR-47-1** Désigner, retirer, modifier un miroir ou une paire sur un compte (API, écran), audité.
- **FR-47-2** Refuser un lien invalide (autre société, même côté, fonds propres, chaîne, soi-même, compte archivé
  comme cible) par un code d'erreur nommé.
- **FR-47-3** Bilan et compte de résultat présentés selon D2–D5 ; brut sur demande (D6).
- **FR-47-4** Sources de chaque ligne présentée (D9) ; MCP et CSV au même contrat que l'écran.
- **FR-47-5** Contrôle de clôture non bloquant : comptes à solde de sens inverse sans miroir ni paire (47-4).
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
- **Q2** — Bascule à toute date d'arrêté (D4) ou à la date de clôture seulement ?
- **Q3** — Comptes en devise : lien admis entre devises différentes ? Proposé : admis ; le solde en devise de la
  ligne présentée n'est montré que si toutes ses sources ont la même devise.
- **Q4** — Balance et grand livre hors périmètre (D7) : confirmé ?
- **Q5** — Epic 46 : présentation avant regroupement (§ *Dépendance*) ; point d'entrée à convenir avec son auteur.
- **Q6** — Contrôle de clôture (47-4) : avertissement non bloquant, ou rien ?
- **Q7** — Libellé de la ligne présentée : celui du compte qui la porte (proposé), ou un libellé de présentation
  propre au lien (colonne de plus) ?
- **Q8** — Vocabulaire : « compte miroir » sert déjà, dans le code, aux paires de contre-passation
  (`homepage_summary.rs`). Garder « miroir » à l'écran, ou « compte de présentation inverse » ?
- **Q9** — Compensation et CO 958c al. 1 ch. 7 (interdiction de compenser) : la paire est un choix de l'utilisateur ;
  Kesh l'admet sans restriction, ou avec un avertissement au plan comptable ?
- **Q10** — Archiver un compte cible d'un miroir ou partenaire d'une paire : refuser, ou retirer le lien ?
- **Q11** — Accueil (D6) : reste brut ? (Ses totaux ne recoupent alors que le compte de résultat brut.)

## Avant le développement

Réponses de Beat (formulaire), accord de Guy sur D1 et la migration, décomptes 47-0, fiches 47-1 à 47-5 complétées
et validées en boucle (`CLAUDE.md` § *Review Iteration Rule*), fichier de l'Epic 46 lu pour Q5.
