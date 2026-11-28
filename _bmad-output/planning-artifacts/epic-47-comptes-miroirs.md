# Epic 47 — Comptes miroirs

*(Titre jusqu'au 2026-11-19 : « Comptes miroirs et paires compensées » ; les paires sont retirées par la décision
ci-dessous.)*

## Décision du 2026-11-19 : miroir seul

**Le responsable de l'epic et Guy, après étude de l'art. 959a CO** (et de l'interdiction de compenser, CO 958c al. 1 ch. 7) : l'epic ne
garde **que la fonction miroir**. Un compte peut avoir un **compte miroir**, qui **ne porte aucune écriture** et ne sert
qu'à la présentation — bilan, compte de résultat, total de l'actif et du passif, résultat. Le solde du compte source,
s'il est de sens inverse de sa nature, se présente sous le miroir, de l'autre côté : c'est la présentation que veut le
CO, **sans aucune compensation**. Consigne du responsable de l'epic : implémentation et interface **aussi simples que possible (KISS)**.

Cas d'usage : « Gains de change » seul porte les écritures de change, « Pertes de change » est son miroir (un gain net
négatif paraît en charge) ; une dette sociale à solde débiteur paraît à l'actif ; une banque à découvert, au passif.

**Approbation du responsable de l'epic (2026-11-20)**, page de revue « miroir seul » : principe et écrans **approuvés** ; retrait du
contrôle K10 **d'accord** ; cinq stories **approuvées**. Développement autorisé, 47-1 en premier.

### Règles retenues

- **R1 — Modèle** : une seule colonne nullable `accounts.mirror_account_id`, sans clé étrangère (comme
  `fx_gain_account_id`), un index `(company_id, mirror_account_id)` ; le « pas soi-même » est refusé par le code (`check_mirror`, raison `self`)
  — écart constaté au T0 le 2026-11-20 (MariaDB 10.11 : ERROR 1901, AUTO_INCREMENT interdit dans un CHECK) ; contrainte retirée, refus tenu par le code. Le lien s'écrit sur la source seulement.
- **R2 — Le miroir est un compte de présentation** : même société, non archivé, **non imputable, sans rôle, sans
  écriture** ; de l'autre côté du même rapport (actif ↔ passif, produit ↔ charge). La source ne porte pas de rôle de
  fonds propres (D8).
- **R3 — Pas de chaîne** : un compte miroir n'a pas lui-même de miroir ; un compte source n'est le miroir de personne.
  **Plusieurs sources peuvent partager un miroir** (l'admettre coûte une somme, l'interdire une règle et un refus de
  plus ; le cas des dettes sociales le demande).
- **R4 — Garde** : tant qu'un compte est le miroir d'un autre (source archivée comprise), le rendre imputable, le
  retyper, lui poser un rôle ou l'archiver est refusé (409, nommant les sources) ; **aucune écriture** ne s'y passe,
  saisie manuelle **et** flux automatiques — refus `ACCOUNT_NOT_POSTABLE` (`DbError::AccountsNotPostable`) au seul point de passage des lignes
  (`validate_lines_accounts_in_tx`), les flux automatiques ne contrôlant pas `postable` (`enforce_postable = false`).
- **R5 — Présentation** : bilan et compte de résultat, à toute date d'arrêté (D4) ; une source dont le solde est de sens
  inverse quitte sa section ; la ligne du miroir, de l'autre côté, vaut la somme des soldes inversés de ses sources ;
  une source de sens normal reste à sa place ; rien n'est compensé. Fonds propres et lignes calculées inchangés (D8) ;
  équation du bilan et résultat net identiques au brut. Une fonction pure, avant les groupes de l'Epic 46 (D5, Q5).
- **R6 — Pas de présentation brute** : ni paramètre d'API, ni bascule, ni argument MCP. La balance de vérification et le
  grand livre montrent les soldes réels (D7) ; un rapport sans miroir est identique à celui d'aujourd'hui ; l'accueil
  reste sur les soldes réels (Q11).
- **R7 — Traçabilité minimale** : la ligne du miroir porte à l'API `mirrorOf` (comptes sources et soldes bruts) ;
  l'écran écrit « Solde de 2270 AVS/AI/APG » sous la ligne ; au PDF, au CSV et au MCP, le miroir paraît comme un
  compte ordinaire, avec son solde présenté, sans colonne, note ni champ neufs.
- **R8 — Devise** : la ligne du miroir se présente en devise principale seulement.

### Retiré de l'epic et des fonctions

Les paires compensées (`offset_account_id`) et tout ce qui s'y attache : l'avertissement CO 958c, la mention
« Présentation compensée », la marque ‡, le drapeau `compensated`, la « paire appliquée », les transitions paire ↔
miroir, les colonnes et notes de paire des CSV et PDF, les cas de lecture (paire non réciproque) et de K10 propres aux
paires. Par la consigne KISS, en plus : la présentation brute (D6), la règle de devise Q3, les `sources` à genres, les
marques et notes de présentation des PDF et CSV (ancienne 47-2b), les pastilles du plan comptable, la clé MCP du plan
comptable et la révision de ses plafonds, le contrôle de clôture K10 (ancienne 47-4, Q6).

### Choix KISS et alternatives écartées

| Choix retenu | Alternative écartée | Pourquoi |
|---|---|---|
| un `<select>` « Compte miroir » au dialogue, création et modification | section à trois boutons radio et quatre états ; refus du lien à la création (`onCreate`) | un seul champ, un seul contrôle |
| miroir non imputable, sans rôle ni écriture, une raison de refus `notPresentationAccount` | miroir qui porte ses propres écritures (D2) ; une raison par condition | rien à additionner ni à expliquer ; un remède |
| garde des lignes au point de passage, par l'erreur existante | se fier à `postable` seul ; code d'erreur neuf | les flux automatiques ignorent `postable` |
| plusieurs sources par miroir | une source par miroir | une somme plutôt qu'un refus de plus |
| présentation toujours appliquée | `presentation=brute` partout (D6) | conforme au CO sans compensation ; la balance donne les soldes réels ; évite route, service, audit, MCP et bascule |
| `mirrorOf` sur la ligne, mention à l'écran | `sources` à genres, pastille, dépliage, note du PDF, colonnes CSV | le minimum qui dit d'où vient le solde |
| PDF, CSV, MCP inchangés | marque, note, champs neufs, tailles MCP remesurées | le miroir est un compte ordinaire pour le lecteur |
| pas de K10 | contrôle de clôture non bloquant (Q6) | le solde négatif se voit déjà au rapport ; à rouvrir en issue si voulu |
| test de propriété maison | dépendance `proptest` | aucune dépendance neuve |

### Exigences (remplacent celles du kickoff, § *Exigences* ci-dessous)

- **FR-47-1** Désigner, changer, retirer le compte miroir d'un compte (API, écran ; création comprise), audité.
- **FR-47-2** Refuser un miroir invalide (R2, R3) par `MIRROR_ACCOUNT_INVALID` à raison nommée, et les gestes interdits
  sur un compte miroir (R4) par `ACCOUNT_IS_MIRROR` nommant les sources ; refuser toute écriture sur un compte miroir.
- **FR-47-3** Bilan et compte de résultat présentés selon R5, à l'écran, à l'API, au PDF, au CSV et au MCP.
- **FR-47-4** `mirrorOf` à l'API et mention à l'écran (R7).
- **FR-47-5** Documentation (exigence de Guy) : manuel utilisateur, manuel d'administration, `docs/api-external.md`.
- **NFR-47-1** Aucune écriture comptable produite ; balance et grand livre identiques avant/après.
- **NFR-47-2** Équation du bilan et résultat net identiques en brut et en présenté (test de propriété).
- **NFR-47-3** Sauvegarde 0.19.0 et 0.20.0 importable ; aucun relèvement de version minimale.
- **NFR-47-4** Exemple chiffré ci-dessous rejoué par un test d'intégration et tenu égal au manuel par une garde (G26).

### Exemple chiffré (source unique ; le test de la 47-3, l'E2E et le manuel de la 47-4 le reprennent)

Société au plan **PME** livré (`crates/kesh-core/assets/charts/pme.json`), un exercice. Le plan livré n'a aucun compte
de change : les comptes de présentation et de change **se créent**, en sous-comptes **sans compte parent** (6900 et
6950 restent imputables ; forme `NNNN.N` admise par G4-bis, qui ne contrôle que le numéro de base).

| Compte | Création | Type | Imputable | Miroir |
|---|---|---|---|---|
| 6950.1 « Gains de change » | à créer | produit | oui | 6900.1 |
| 6900.1 « Pertes de change » | à créer | charge | **non** | — |
| 1040.1 « Créances AVS/AI/APG » | à créer | actif | **non** | — |
| 2100.1 « Découvert bancaire » | à créer | passif | **non** | — |
| 1020 « Banque » | livré | actif | oui | 2100.1 |
| 2270 « AVS/AI/APG » | livré | passif | oui | 1040.1 |

Écritures, chacune contre 2800 « Capital social » (rôle capital) : 1000 « Caisse » débit 500.00 ; 1010 « Poste » débit
12 000.00 ; 1020 crédit 800.00 ; 2000 « Créanciers » crédit 3 000.00 ; 2270 débit 1 200.00 ; 3200 « Prestations de
services » crédit 2 000.00 ; 6900 « Charges financières » débit 50.00 ; 6950.1 débit 250.00. Solde de 2800 : crédit
8 200.00 (14 000.00 − 5 800.00).

| | Brut (aujourd'hui) | Présenté |
|---|---|---|
| Actif | 1000 : 500.00 ; 1010 : 12 000.00 ; 1020 : **−800.00** | 1000 : 500.00 ; 1010 : 12 000.00 ; **1040.1 : 1 200.00** (solde de 2270) |
| Total de l'actif | 11 700.00 | **13 700.00** |
| Passif (hors fonds propres) | 2000 : 3 000.00 ; 2270 : **−1 200.00** | 2000 : 3 000.00 ; **2100.1 : 800.00** (solde de 1020) |
| Total du passif | 1 800.00 | **3 800.00** |
| Fonds propres | 2800 : 8 200.00 ; résultat 1 700.00 ; total 9 900.00 | identiques |
| Équation | 11 700.00 = 1 800.00 + 9 900.00 | 13 700.00 = 3 800.00 + 9 900.00 |
| Produits | 3200 : 2 000.00 ; 6950.1 : **−250.00** ; total 1 750.00 | 3200 : 2 000.00 ; total **2 000.00** |
| Charges | 6900 : 50.00 ; total 50.00 | 6900 : 50.00 ; **6900.1 : 250.00** (solde de 6950.1) ; total **300.00** |
| Résultat de l'exercice | 1 700.00 | 1 700.00 |

Les deux côtés du bilan augmentent de 2 000.00 ; produits et charges du compte de résultat de 250.00 chacun ; le
résultat ne bouge pas. La balance de vérification montre toujours 1020 à −800.00, 2270 à −1 200.00 et 6950.1 à
−250.00. `mirrorOf` : 1040.1 ← 2270 (−1 200.00), 2100.1 ← 1020 (−800.00), 6900.1 ← 6950.1 (−250.00). Liste des
montants que portent le test (`EXEMPLE_PRESENTATION`) et le manuel : 13700.00, 11700.00, 3800.00, 1800.00, 9900.00,
8200.00, 1200.00, 800.00, 2000.00, 1750.00, 300.00, 250.00, 50.00, 1700.00. *(Recalculé le 2026-11-19 ; remplace
l'exemple de l'ancienne 47-2a AC-11, qui posait une paire 6900/6950 et des miroirs imputables 1040 et 2100.)*

En multi-devises (Epic 37), 6950.1 se désigne compte des gains **et** des pertes de change (le même compte est admis ;
un compte non imputable ne peut pas l'être, `FxAccountReason::NotPostable`) : la réévaluation de fin d'exercice n'écrit
que sur 6950.1, et 6900.1 en présente le solde débiteur.

### Ce qui suit de la décision

Les décisions D1 à D10 et les questions Q1 à Q11 ci-dessous restent comme historique, **non réécrites** ; celles que la
décision rend caduques portent la mention « remplacé par la décision du 2026-11-19 ». Les fiches sont refondues (§
*Stories*), **remises à `drafted`** et à revalider en boucle (`CLAUDE.md` § *Review Iteration Rule*) ; la validation du
2026-11-19 (§ *Validation des fiches*) ne vaut plus pour elles.

### Décision du responsable de l'epic du 2026-11-21 (point e) : miroir sous un groupe d'un autre type

Un compte miroir rangé sous un groupe d'**un autre type** que le sien donne un **avertissement non bloquant**, non un
refus : cet état est **admis**. Effet attendu à la fusion avec l'Epic 46 : la ligne du miroir, rangée dans la section du
type du miroir, remonte sous un groupe de l'autre section ; ce groupe a alors des descendants dans deux sections et est
dissous avec ses ancêtres (46-1 AC-4) ; les totaux restent justes (NFR-46-2, recalculés sur les sections présentées).
**Qui porte l'avertissement : personne à ce jour** — la 47-4 (contrôle K10) est `superseded` ; à rouvrir en issue
`enhancement` (« on trace toujours »), porteur à désigner par le Project Lead. Un cas « miroir sous un groupe d'un autre
type » est à ajouter au rejeu de l'exemple par niveaux, à la fusion.


## Kickoff du 2026-11-18 et orientation du 2026-11-19 (historique)

**Kickoff : 2026-11-18** (esquisse) ; **orientation approuvée par Guy le 2026-11-19** (§ *Revue de Guy*) **et par le responsable de l'epic le même jour, telle quelle** (§ *Revue du responsable de l'epic*) : **kickoff accompli le 2026-11-19**. Numéro réservé au responsable de l'epic (contributeur) par Guy ; responsable : le responsable de l'epic.
Branche `epic/47-comptes-miroirs`, partie de `main` = `773df80d` (v0.19.0). **Passe après l'Epic 46** (rapports par
groupes de comptes), préparé en parallèle. Release : un epic = une release, **ultérieure à la 0.20.0** (Guy, 2026-11-19), numéro au tag par le Project Lead. *(Annoté le 2026-11-28, à la reprise de `main` après la release du 28 novembre : release visée, **celle du 4 décembre** ; son numéro se donne au tag.)*

Règles des fichiers communs (`CLAUDE.md` § *Travailler à plusieurs développeurs*) : rien au `CHANGELOG.md` (section
*Note de version* par fiche) ; aucun PDF de manuel commité ; i18n et `sprint-status.yaml` en bloc propre ; compteurs
recomptés après chaque reprise de `main`. Pas d'issue ni de PR à ce stade (le responsable de l'epic : lecture seule sur le dépôt de Guy).

## Intention

*(La « paire compensée » ci-dessous est **remplacée par la décision du 2026-11-19** : retirée de l'epic. Le compte miroir, lui, ne porte plus d'écriture.)*

**Aucun montant négatif dans le bilan ni dans le compte de résultat**, sans écriture de reclassement.

- **Compte miroir** : le solde **de sens inverse** d'un compte se présente sur un autre compte, de l'autre côté
  (dette sociale à solde débiteur → actif ; banque à découvert → passif). Présentation seulement : aucune écriture.
- **Paire compensée** : deux comptes de types opposés dont seul le **net** paraît, sur le compte du sens du net
  (pertes de change 6960 et gains de change 6961 de l'Epic 37 : une ligne).

Besoin réel : à la recette de l'exercice 2024 du responsable de l'epic, il a fallu au 31.12 des écritures de reclassement (quatre
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

## Décisions — décidées (Guy et le responsable de l'epic, 2026-11-19)

- **D1 — Modèle : deux colonnes sur `accounts`, pas de table.** `mirror_account_id BIGINT NULL` (compte miroir) et
  `offset_account_id BIGINT NULL` (partenaire de paire, posé **des deux côtés**, réciproque, écrit dans une
  transaction). Pointeurs sans clé étrangère, comme `fx_gain_account_id`. Raison : une table neuve rend toute
  sauvegarde 0.19.0 inimportable (ou exige une liste d'absences admises) ; deux colonnes facultatives passent la règle
  générique. Un compte n'a pas les deux (`CHECK`). Pas de chaîne : un compte miroir n'a lui-même ni miroir ni paire.
  *Guy, 2026-11-19 :* d'accord, deux colonnes facultatives, compatibles avec l'import des sauvegardes 0.19.0 ; une table l'empêcherait. → **décidé (Guy et le responsable de l'epic, 2026-11-19).**
  ⇒ **remplacé par la décision du 2026-11-19** en partie : une seule colonne, `mirror_account_id` (R1) ; `offset_account_id` et le `CHECK` « pas les deux » disparaissent avec la paire. Le choix des colonnes plutôt qu'une table (sauvegardes) tient.
- **D2 — Miroir** : de l'autre côté du même rapport (actif ↔ passif hors fonds propres ; produit ↔ charge), même
  société. Le compte miroir peut être non imputable (compte de présentation créé pour l'usage) ou porter ses propres
  écritures ; sa ligne = son solde + le solde inversé de ses sources. Un compte dont le solde est dans son sens normal
  reste à sa place.
  *Guy, 2026-11-19 :* d'accord ; pas de chaîne : un miroir ne pointe jamais vers un compte qui a lui-même un miroir — **refus nommé** (47-1 AC3). → **décidé (Guy et le responsable de l'epic, 2026-11-19).**
  ⇒ **remplacé par la décision du 2026-11-19** : le miroir ne porte **aucune** écriture (non imputable, sans rôle, R2, R4) ; sa ligne vaut la somme des soldes inversés de ses sources (R5). « Pas de chaîne » et « plusieurs sources » tiennent (R3).
- **D3 — Paire** : deux comptes de **types opposés** du même rapport (produit/charge ; actif/passif). Le net paraît
  sur le compte du sens du net, sous son numéro et son libellé ; net nul → aucune ligne. Deux comptes de même type ne
  forment pas de paire : leur somme est un groupe (Epic 46).
  *Guy, 2026-11-19 :* approuvé, sous réserve de Q9 : la paire compense une charge et un produit (CO 958c al. 1 ch. 7) → avertissement à sa création, mention « présentation compensée » au PDF. → **décidé (Guy et le responsable de l'epic, 2026-11-19).**
  ⇒ **remplacé par la décision du 2026-11-19** : paires retirées de l'epic.
- **D4 — Bascule à la date d'arrêté de tout rapport** (bilan à toute date, compte de résultat sur toute période), pas
  seulement à la clôture : Kesh calcule les soldes en direct et ne passe aucune écriture de clôture.
  *Guy, 2026-11-19 :* d'accord, un rapport intermédiaire se présente comme celui de clôture. → **décidé (Guy et le responsable de l'epic, 2026-11-19).**
- **D5 — Une seule fonction de présentation** dans `kesh-report` (pure, testée seule), appliquée aux lignes avant
  les totaux et avant les groupes de l'Epic 46. Bilan et compte de résultat seulement ; écran, PDF, CSV, API, MCP.
  L'équation du bilan tient (les deux côtés bougent du même montant) ; le résultat net ne change pas.
  *Guy, 2026-11-19 :* approuvé ; présentation d'abord, puis regroupement (Q5). → **décidé (Guy et le responsable de l'epic, 2026-11-19).**
  ⇒ tient (R5) ; « écran, PDF, CSV, API, MCP » : présentation appliquée partout, sans rendu propre au PDF, au CSV ni au MCP (R7).
- **D6 — Présentation brute toujours disponible** : paramètre `presentation=brute` à l'API (et bascule à l'écran)
  rend le rapport d'aujourd'hui, compte par compte, signé. Défaut : présentée. L'accueil reste sur les totaux bruts.
  *Guy, 2026-11-19 :* essentiel : la présentation brute reste disponible **partout** (écran, PDF, CSV, API), notamment pour les comptes annuels officiels si une paire est utilisée (Q9). → **décidé (Guy et le responsable de l'epic, 2026-11-19).**
  ⇒ **remplacé par la décision du 2026-11-19** : pas de présentation brute (R6) ; le motif (comptes officiels avec une paire) tombe avec la paire, et la balance donne les soldes réels.
- **D7 — Balance de vérification, grand livre, journal, TVA, projets : inchangés.** Le détail par compte y reste.
  *Guy, 2026-11-19 :* confirmé (Q4) : balance, grand livre et journal montrent les soldes réels. → **décidé (Guy et le responsable de l'epic, 2026-11-19).**
- **D8 — Fonds propres et lignes calculées exclus** : un compte de rôle de fonds propres ne prend ni miroir ni paire ;
  une perte (reportée ou de l'exercice) reste en déduction des fonds propres (CO 959a).
  *Guy, 2026-11-19 :* d'accord ; fonds propres hors du mécanisme, une perte reste en déduction (CO 959a). → **décidé (Guy et le responsable de l'epic, 2026-11-19).**
- **D9 — Traçabilité** : chaque ligne présentée porte à l'API ses `sources` (compte, solde brut) ; l'écran les montre
  au survol ou au dépliage ; le PDF et le CSV, une marque et une note.
  *Guy, 2026-11-19 :* d'accord ; la marque du PDF dit en clair **« présentation compensée »** quand une paire est appliquée (Q9). → **décidé (Guy et le responsable de l'epic, 2026-11-19).**
  ⇒ **remplacé par la décision du 2026-11-19** : `mirrorOf` à l'API, mention à l'écran ; ni marque ni note au PDF et au CSV, ni mention « présentation compensée » (R7).
- **D10 — Migration de la 47-1, non breaking** : `2026112?000001_account_presentation_links.sql` (horodatage au
  développement), DDL seul : deux `ADD COLUMN … BIGINT NULL`, deux index, `CHECK` (pas soi-même, pas les deux). Ne
  relève pas `kesh_version_min_required` (0.17.6). P5 : ligne à l'audit, compteurs recomptés depuis la source à la reprise de `main` (95 à `773df80d`, plus la 47-1 et les migrations fusionnées avant elle, dont `20261118000002` de l'Epic 49) ; P6 : grep des
  sites positionnels ; P7 : rien à trier (aucune écriture de données) ; squash `test-schema` mis à jour. Une sauvegarde
  0.19.0 s'importe (colonnes facultatives) ; la version précédente refuse la base migrée et les sauvegardes neuves :
  retour par la sauvegarde d'avant la mise à jour. **Annoncée au projet au kickoff** (tableau des fichiers communs).
  *Guy, 2026-11-19 :* d'accord sur le principe. **Ordre : cette migration passe après celle de l'Epic 49 (`20261118000002`, release 0.20.0 du 2026-11-27)**, dans une release ultérieure à la 0.20.0 ; l'Epic 47 dépend de l'Epic 46. Annoncée au projet GitHub au kickoff, avec **la sauvegarde complète avant mise à jour en tête des notes**. → **décidé (Guy et le responsable de l'epic, 2026-11-19).**
  ⇒ adapté par la décision du 2026-11-19 : `<horodatage>_account_mirror.sql`, une colonne, un index ; le `CHECK` « pas soi-même » : écart constaté au T0 le 2026-11-20 (MariaDB 10.11 : ERROR 1901, AUTO_INCREMENT interdit dans un CHECK) ; contrainte retirée, refus tenu par le code ; tout le reste (non breaking, après `20261118000002`, P5–P8, squash, sauvegarde) tient (fiche 47-1).

## Exigences

*(Remplacées par celles du § *Décision du 2026-11-19 : miroir seul* ; liste du kickoff laissée telle quelle.)*

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

*(Refondues le 2026-11-19, décision « miroir seul, KISS ». Fiches remises à `drafted`, à revalider.)*

| Story | Objet | Modules | Critères | Migration | Dépend de |
|---|---|---|---|---|---|
| 47-0 | kickoff — **done** (2026-11-19) | — | 4 | non | — |
| 47-1 `47-1-miroir-modele-et-depot` | migration (une colonne), entité, règles R2–R4, garde des lignes au point de passage, lecteur tolérant, sauvegarde, colonne de `accounts.csv`, erreurs mappées et messages (sous-match exhaustif de `DbError`) | 5 | 13 | **oui**, non breaking | — |
| 47-2 `47-2-miroir-api-et-ecran-du-plan` | API des comptes (`mirrorAccountId`), un `<select>` au dialogue, refus à l'écran, archivage refusé, manuel utilisateur (plan comptable), manuel d'administration (mise à jour), `api-external.md` (comptes), E2E | 5 | 12 | non | 47-1 |
| 47-3 `47-3-miroir-presentation-rapports` | fonction pure de présentation, service toujours présenté, `mirrorOf`, MCP (descriptions), accueil brut, exemple rejoué, `api-external.md` (rapports) | 5 | 14 | non | 47-1 ; **avant** le regroupement de l'Epic 46 (recâblage à sa fusion) |
| 47-4 `47-4-miroir-ecrans-rapports-manuel-e2e` | mention « Solde de … » aux écrans des rapports, E2E du parcours, manuel utilisateur (rapports, exemple), garde G26 | 5 | 10 | non | 47-2, 47-3 |
| 47-5 `47-5-solde-dette-epic` | solde de la dette de l'epic (**plafond : 2 issues**, fixé au kickoff ; annoté le 2026-11-22 : le second temps, décidé par le responsable de l'epic, a corrigé en plus les LOW reportés des 47-1 et 47-2, hors plafond d'issues) | selon les issues | 6 | non | 47-1 à 47-4 closes (fin d'epic, avant le tag) |

**Dans la borne de six** (`CLAUDE.md` § *Numérotation des versions* : au plus six stories, solde comprise) : cinq
stories hors kickoff. La dérogation dite au kickoff (huit stories) **tombe** ; elle n'a plus à être signalée à Guy.
Chaque story compte au plus cinq modules (règle de splitting préventif), déclarés en tête de sa fiche.

*Historique — stories du kickoff (2026-11-19), remplacées par la décision du 2026-11-19* : 47-1a (modèle) → 47-1 ;
47-1b (API, avertissement de la paire, MCP du plan) → API reprise par 47-2, le reste retiré (`superseded`) ; 47-2a
(présentation, `presentation=brute`, `sources`) → 47-3 ; 47-2b (PDF et CSV présentés) → retirée (`superseded`) ;
47-3a (écran du plan) → 47-2 ; 47-3b (écrans des rapports, bascule brute, E2E, manuel) → 47-4 ; 47-4 (K10) → retirée
(`superseded`) ; 47-5 inchangée. Les clés `47-1`, `47-2`, `47-3` d'origine (`split`) sont passées à `superseded`.

## Ordre de développement

1. **47-1** d'abord : elle porte la migration et **ne sort jamais** (une story sortie emporte sa migration) ; son
   horodatage se contrôle à la reprise de `main` (D10) ; `fix/api-parent-compte` (#1351 : même `update`, mêmes erreurs)
   repris s'il est fusionné, sinon le second arrivé résout.
2. **En parallèle, après la 47-1** : 47-2 (API et écran du plan) et 47-3 (présentation). Fichiers partagés entre elles :
   catalogues i18n, `docs/api-external.md`, manuel d'administration ; chacune y écrit son bloc, la seconde fusionnée
   dans la branche d'epic résout en gardant les deux.
3. **Reprise de `main`** (Epic 46, #1327, #1346, 48-11 si fusionnés) avant le recâblage de la 47-3 et avant la 47-4.
4. **47-4** après 47-2 et 47-3 (E2E du parcours complet, manuel utilisateur, garde G26).
5. **47-5** en dernier, une fois 47-1 à 47-4 closes.

**Si le temps manque** (coupure du jeudi soir : n'entrent que les stories closes ; `CLAUDE.md` § *Cadence*) : epic de
contributeur, l'orchestrateur n'y touche pas ; la coupure se décide par **le responsable de l'epic**, et **le Project Lead garde le dernier
mot**. La **47-5** sort la première (ses lignes du registre restent ouvertes en issues, et l'absence de solde s'écrit à
la PR de release et à la rétrospective) ; la 47-1 ne sort jamais ; l'epic ne se publie pas amputé de sa présentation
(47-3) ni de son écran de pose (47-2) : sans elles, le miroir n'a aucun effet visible, et **l'epic entier attend une
release suivante** (la date de la release ne bouge pas). La 47-4 peut sortir seule si la 47-3 et la 47-2 sont closes
(la présentation se voit alors sans mention ; le manuel et l'E2E suivent à la release d'après), décision du Project
Lead.

Hors epic : la story de correction de bugs de la release (`CLAUDE.md` § *Cadence*) n'est pas comptée ici.

## Registre de la dette de l'epic

Les revues de chaque story consignent ici, à la clôture de leur boucle (chaque fiche 47-1 à 47-4 le rappelle en § *References*), une ligne par finding **non remédié** qui devrait s'ouvrir en issue (« on trace toujours ») : story, passe, sévérité, objet, issue (numéro reporté quand le Project Lead l'ouvre), sort (ouverte, retenue en 47-5, corrigée en 47-5, reportée). L'agent du responsable de l'epic n'ouvre pas d'issue (lecture seule sur le dépôt de Guy) : le Project Lead ouvre les issues d'après ce registre, et la 47-5 en part. Une ligne par problème, sans regroupement ; l'étiquette `bug` seulement pour un comportement faux visible (`CLAUDE.md` § *Priorités des défauts*).

| Story | Passe | Sévérité | Objet | Issue | Sort |
|---|---|---|---|---|---|
| 47-1 | P1 (revue de code) | LOW | phrase « réactivez-le d'abord » (`error-account-is-mirror-reactivate-first`, quatre locales) au singulier quand plusieurs sources archivées sont nommées (L5) — `bug`, P4 | non ouverte | corrigée en 47-5 (clé `-many`, décision du responsable de l'epic du 22.11) |
| 47-1 | P1 (revue de code) | LOW | liste des sources nommées dans le refus `ACCOUNT_IS_MIRROR` non bornée : message long quand beaucoup de comptes partagent un miroir (L5, second point) — `enhancement`, P4 | à ouvrir | reportée : choisir une borne et la forme du reste (« et N autres ») est une décision d'interface, non un correctif |
| 47-1 | P1 (revue de code) | LOW ×7 | course « pose contre archivage » (L1) ; doc-comments d'ordre des refus (L2) ; `Set(miroir actuel)` et lien forgé (L3) ; verrous d'intervalle de la garde des lignes (L4) ; chemin `update_in_tx` sans test (L6) ; audit des migrations qui affirme la 0.20.0 (L8) ; table G9 (L9) ; et L7 (rédaction du Dev Agent Record) | non ouverte | corrigée en 47-5 (décision du responsable de l'epic du 22.11 ; L4 documentée, non mesurée, comme le recommandait la revue) |
| 47-2 | P2 (revue de code, ciblée) | LOW ×3 | commentaire du rejeu de l'archivage (`routes/accounts.rs:598-606`) qui allègue un mécanisme faux (P2-L1) ; doc publique de `create` sans le refus du miroir ni le verrou de sa cible (P2-L2) ; ordre des refus à la création non fixé par un test (P2-L3) — `technical-debt`, P4 | non ouverte | corrigée en 47-5 (décision du responsable de l'epic du 22.11) |
| 47-3 | P2 (revue de code, ciblée) | LOW ×3 | `presentation.rs` : repli `ecart_conserve` exercé par aucun test (LOW-1) ; message de repli qui parle de l'équation quand seul l'écart diffère, écart non journalisé (LOW-2) ; filtre `mirror_account_type != section_type` retirable sans test rouge (LOW-3) — une issue `technical-debt`, P4 | non ouverte | corrigée en 47-5 |
| 47-4 | P1 (revue de code) | LOW ×4 | manuel et E2E : phrase de l'accueil (« solde de sens inverse ») non revérifiée au PDF (L1, L5 b) ; « la présentation que l'art. 959a CO attend » trop forte (L5 a) ; figure `mi-balance.png` avec des comptes du jeu d'essai au lieu du plan PME (L4) ; clés `reports-mirror-of-*` à ranger avec les clés des rapports (L6) — une issue `documentation`, P4 | non ouverte | corrigée en 47-5 (avec la reprise de `tjDe` dans `invoice-qr-caracteres.spec.ts`, L6) |
| 47-4 | gate E2E | — | `bank-import.spec.ts:130` compte les lignes avant le chargement de la liste (course connue, spec antérieure à l'epic) — `known-failure`, P3 | #1283 (le gate de la 47-4 a aussi tracé #1282, `backup-restore-new-install.spec.ts`) | reportée : non retenue en 47-5 (spec antérieure à l'epic, hors de son périmètre) |
| 47-3 (L-5) | décision du responsable de l'epic du 2026-11-21, point e | — | avertissement non bloquant pour un compte miroir rangé sous un groupe d'un autre type (puis contrôle sur tous les comptes) — `enhancement`, P3 | à ouvrir (versée à la liste des issues à ouvrir à la revue d'intégration P1, L-4) | reportée : la fiche 47-5 ne la prévoit pas ; l'effet (dissolution du groupe et de ses ancêtres) est testé et dit au manuel depuis la remédiation de cette revue (M-2) |

## Dépendance à l'Epic 46

Les groupes de l'Epic 46 additionnent des lignes de compte en sous-totaux. Règle (Q5) : **la présentation précède le
regroupement** — la ligne d'un miroir entre dans le groupe du **miroir**, jamais dans celui de la source. Il faut donc,
dans l'Epic 46, un point d'entrée unique où les lignes passent avant d'être groupées (fiche 47-3, § *Recâblage*).
*(Rédaction du kickoff : « le miroir, ou le compte du sens du net » — le second cas est remplacé par la décision du
2026-11-19.)*

## Fichiers communs

*(Mis à jour le 2026-11-19, refonte « miroir seul, KISS ».)*

| Fichier | 47 | Epic 46 | `fix/pdf-montants-alignes` (#1327) | `fix/bilan-resultat-reporte` (#1346) | `fix/api-parent-compte` (#1351) |
|---|---|---|---|---|---|
| `kesh-report/src/balance_sheet.rs`, `income_statement.rs` (champ `mirror_of`) ; `presentation.rs` (neuf) | 47-3 | probable | — | oui (`balance_sheet.rs`) | — |
| `kesh-report/src/pdf.rs`, `csv.rs`, `benches/export.rs` | 47-3 (**littéraux des tests seulement**) | probable | **oui (+1100 lignes)** | oui | — |
| `frontend/…/reports/` (deux vues, `reports.types.ts`, composant de la mention) | 47-4 | probable | — | oui | — |
| `kesh-db/src/entities/account.rs`, `repositories/accounts.rs` | 47-1 | possible | — | — | oui |
| `kesh-db/src/repositories/journal_entries.rs` (`validate_lines_accounts_in_tx`) | 47-1 | — | — | — | — |
| `kesh-db/src/errors.rs`, `kesh-api/src/errors.rs` | 47-1 | possible | — | — | oui |
| `kesh-api/src/routes/accounts.rs` | 47-2 | — | — | — | oui |
| `kesh-api/src/services/rapports.rs`, `mcp/outils/rapports.rs` (descriptions) | 47-3 | probable | — | — | — |
| `frontend/…/accounts/`, `routes/(app)/accounts/+page.svelte` | 47-2 | — | — | — | oui |
| catalogues i18n (4 locales), `sitesTotal` | 47-1 (refus), 47-2 (dialogue), 47-4 (mention) | oui | oui | — | oui |
| `docs/manual/fr/user-manual.tex` | 47-2 (plan comptable), 47-4 (rapports) | oui | oui | oui | oui |
| `docs/manual/fr/admin-manual.tex`, `docs/api-external.md` | 47-2 (comptes, mise à jour), 47-3 (rapports, MCP) | oui | oui | oui | oui |
| `kesh-db/migrations/`, audit, `test-schema/`, `migrations_upgrade_path.rs`, `account_columns_inventory.rs` | 47-1 | si migration | — | — | — |
| `kesh-api/src/exports/csv_tables.rs` | 47-1 (colonne) | — | — | — | — |
| `kesh-api/tests/textes_coherents.rs` (G26) | 47-4 | — | — | — | — |
| `kesh-api/tests/homepage_summary_e2e.rs`, `reports_e2e.rs` | 47-3 | possible | — | — | — |

**Plus touchés par l'epic depuis la refonte** : `kesh-api/tests/services_motif.rs` (aucune clé d'audit neuve),
`kesh-db/src/repositories/closing_checks.rs` et `FiscalYearChecks.svelte` (K10 retiré), le MCP `kesh_plan_comptable`
(inchangé), le code de rendu de `pdf.rs` et `csv.rs`.

**Hors tableau (0.20.0)** : la story **48-11** (arrondi au centime des rapports CSV et PDF, après #1327) touche les
tests de `pdf.rs` et `csv.rs`, comme les littéraux de la 47-3 : le second arrivé résout. Les stories de l'Epic 49
changent les catalogues i18n et la migration (horodatage de la 47-1 postérieur à `20261118000002`).

Geste : reprendre `main` (avec l'Epic 46 et les trois correctifs) dans la branche avant le recâblage de la 47-3 et
avant la 47-4 ; ordre des migrations contrôlé à cette reprise si l'Epic 46 en porte une.

## Décomptes d'issues

**Relevés par le Project Lead au kickoff de la 0.20.0** (`release-0.20.0.md` § *Décompte au kickoff et objectif*, sur
`origin/main` à `fd04099f`) ; non remesurés ici (`gh` hors de portée de l'agent du responsable de l'epic) :

| | Tag v0.19.0 | Kickoff 0.20.0 (2026-11-18) | Le soir du 2026-11-18 |
|---|---|---|---|
| Issues ouvertes | 496 | 496 | 507 |
| Bugs (`bug` ou `known-failure`) | 247 | 247 | 257 |
| P1 ouvertes | 14 | 14 | 14 |

Hors les 56 issues masquées du compte de l'agent du responsable de l'epic (#1266–#1271, #1277–#1326), non comptées (décision de Guy).
**Plafond de la story de solde (47-5)** : **2 issues**, fixé ici au kickoff (`CLAUDE.md` § *Priorités des défauts*) ; annoté le 2026-11-22 : le plafond vaut pour les issues, les LOW des 47-1 et 47-2 corrigés au second temps (décision du 22.11) n'en comptent pas.
**Objectif de l'epic** : moins d'issues ouvertes au tag de cette release que **au tag précédent, v0.20.0** (CLAUDE.md § *Priorités des défauts* : objectif, non barrière) ; ce compte n'existe pas encore (la 0.20.0 n'est pas taguée) et se relève au tag par le Project Lead, comme le compte du tag de l'epic à la PR de release et à la rétrospective, avec l'écart et sa cause. Le tableau ci-dessus est la base du kickoff, non la référence de l'objectif.

**Objectif de l'epic** (règle de release, non barrière) : moins d'issues ouvertes au tag de sa release qu'au tag de la
release précédente (la 0.20.0, compte **à relever au tag v0.20.0**) ; le compte au tag de l'Epic 47 s'écrit à sa PR
de release et à sa rétrospective, avec l'écart et sa cause. *(Annoté le 2026-11-28 : une release est sortie entre-temps, celle du 28 novembre (`287deea33`) ; le tag précédent de la release du 4 décembre est désormais le sien.)*

## Questions — répondues (Guy, 2026-11-19), réponses adoptées (Guy et le responsable de l'epic, 2026-11-19)

- **Q1** — Modèle : deux colonnes sur le compte (D1) ou table de paires/miroirs ? (D1 recommandé : sauvegardes.)
  *Guy, 2026-11-19 :* colonnes sur le compte (D1). → **décidé (Guy et le responsable de l'epic, 2026-11-19).**
- **Q2** — Bascule à toute date d'arrêté (D4) ou à la date de clôture seulement ?
  *Guy, 2026-11-19 :* à toute date (D4). → **décidé (Guy et le responsable de l'epic, 2026-11-19).**
- **Q3** — Comptes en devise : lien admis entre devises différentes ? Proposé : admis ; le solde en devise de la
  ligne présentée n'est montré que si toutes ses sources ont la même devise.
  *Guy, 2026-11-19 :* oui, avec la règle proposée (solde en devise seulement si les sources ont la même devise). → **décidé (Guy et le responsable de l'epic, 2026-11-19).**
  ⇒ **remplacé par la décision du 2026-11-19** : la ligne du miroir est en devise principale seulement (R8) ; un miroir en devise est admis.
- **Q4** — Balance et grand livre hors périmètre (D7) : confirmé ?
  *Guy, 2026-11-19 :* confirmé : balance, grand livre et journal montrent les soldes réels, sans présentation. → **décidé (Guy et le responsable de l'epic, 2026-11-19).**
- **Q5** — Epic 46 : présentation avant regroupement (§ *Dépendance*) ; point d'entrée à convenir avec son auteur.
  *Guy, 2026-11-19 :* oui : présentation d'abord, puis regroupement ; la ligne va au groupe du compte qui la porte. → **décidé (Guy et le responsable de l'epic, 2026-11-19).**
- **Q6** — Contrôle de clôture (47-4) : avertissement non bloquant, ou rien ?
  *Guy, 2026-11-19 :* **avertissement non bloquant**, qui signale les comptes à solde inverse sans miroir ni paire, sans empêcher de clore. → **décidé (Guy et le responsable de l'epic, 2026-11-19).**
  ⇒ **remplacé par la décision du 2026-11-19** : K10 retiré de l'epic (le solde négatif se voit au rapport) ; à rouvrir en issue si Guy ou le responsable de l'epic le veulent.
- **Q7** — Libellé de la ligne présentée : celui du compte qui la porte (proposé), ou un libellé de présentation
  propre au lien (colonne de plus) ?
  *Guy, 2026-11-19 :* le libellé du compte qui porte la ligne. → **décidé (Guy et le responsable de l'epic, 2026-11-19).**
- **Q8** — Vocabulaire : « compte miroir » sert déjà, dans le code, aux paires de contre-passation
  (`homepage_summary.rs`). Garder « miroir » à l'écran, ou « compte de présentation inverse » ?
  *Guy, 2026-11-19 :* « compte miroir » convient : le mot n'apparaît que dans des commentaires internes du code, jamais à l'écran ni au manuel. → **décidé (Guy et le responsable de l'epic, 2026-11-19).**
- **Q9** — Compensation et CO 958c al. 1 ch. 7 (interdiction de compenser) : la paire est un choix de l'utilisateur ;
  Kesh l'admet sans restriction, ou avec un avertissement au plan comptable ?
  *Guy, 2026-11-19 :* **avec un avertissement.** Le miroir ne compense rien (un découvert au passif est la présentation correcte) ; la paire compense une charge et un produit, ce que CO 958c al. 1 ch. 7 interdit dans les comptes annuels. Donc : **avertissement à la création de la paire**, mention **« présentation compensée »** au PDF du rapport, **présentation brute toujours disponible** (D6) pour les comptes officiels. → **décidé (Guy et le responsable de l'epic, 2026-11-19).**
  ⇒ **remplacé par la décision du 2026-11-19** : sans paire, rien n'est compensé ; ni avertissement ni mention.
- **Q10** — Archiver un compte cible d'un miroir ou partenaire d'une paire : refuser, ou retirer le lien ?
  *Guy, 2026-11-19 :* **refuser l'archivage** tant qu'un compte pointe vers la cible, avec un message qui **nomme le compte lié**. → **décidé (Guy et le responsable de l'epic, 2026-11-19).**
  ⇒ tient pour le miroir (R4 : archivage du miroir refusé, sources nommées) ; « partenaire d'une paire » caduc.
- **Q11** — Accueil (D6) : reste brut ? (Ses totaux ne recoupent alors que le compte de résultat brut.)
  *Guy, 2026-11-19 :* oui, l'accueil reste sur les totaux bruts. → **décidé (Guy et le responsable de l'epic, 2026-11-19).**
  ⇒ tient (R6) ; la phrase de `api-external.md` sur l'égalité des totaux de l'accueil et du compte de résultat est corrigée par la 47-3.

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

## Revue du responsable de l'epic (2026-11-19) : approuvé tel quel

Le responsable de l'epic a relu la page de revue complétée des réponses de Guy et **l'approuve telle quelle, sans autre commentaire** :
toutes les décisions D1–D10 et toutes les réponses de Guy aux questions Q1–Q11 sont **adoptées** ; chaque D et
chaque Q porte ci-dessus la mention « décidé (Guy et le responsable de l'epic, 2026-11-19) ». Le kickoff (47-0) est accompli ;
`sprint-status.yaml` : `epic-47: in-progress`, `47-0-kickoff: done`.

## Validation des fiches (2026-11-19)

*(Annoté le 2026-11-19 : **ne vaut plus** depuis la refonte « miroir seul, KISS » — les fiches refondues 47-1 à 47-5 sont `drafted` et à revalider ; les trois fiches retirées sont `superseded`. Relevé laissé tel quel. De « Reste à faire au développement », tiennent, adaptés : le contrôle du `CHECK` sur `id` et de l'horodatage (47-1 T0) ; la reprise de l'Epic 46, de #1327, #1346 et #1351 ; l'annonce de la migration par le Project Lead. Ne tiennent plus : la mesure des plafonds MCP (aucun champ MCP neuf) et l'attente de #1327 / #1346 pour un rendu PDF/CSV propre (seuls des littéraux de tests changent).)*

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
  `AUTO_INCREMENT`) est accepté ; arrêt et question au responsable de l'epic s'il est refusé (D10, C-47-1a-4). *(Fait à la 47-1 : écart constaté au T0 le 2026-11-20 (MariaDB 10.11 : ERROR 1901, AUTO_INCREMENT interdit dans un CHECK) ; contrainte retirée, refus tenu par le code.)* Horodatage de la migration
  postérieur à `20261118000002` **et** à la migration de l'Epic 49 (compte en dur et fenêtre `N = total − 34` de
  `migrations_upgrade_path.rs`), contrôlé à la reprise de `main`.
- **Tailles MCP** : 47-1b, plafonds de `kesh_plan_comptable` **mesurés** (estimés 34 / 61, jamais devinés ; changement
  de contrat MCP écrit aux Notes de version) ; 47-2a, mesure du bilan et du compte de résultat rejouée avec les champs
  `sources`.
- **Dépendances** : 47-2b et le recâblage de la 47-2a attendent l'**Epic 46** (point d'entrée du regroupement, Q5) ;
  47-2b attend **#1327** (`fix/pdf-montants-alignes`) et **#1346** (`fix/bilan-resultat-reporte`) fusionnés, et la 48-11
  si elle passe avant ; 47-1b et 47-3a résolvent avec **#1351** (`fix/api-parent-compte`) selon l'ordre de fusion ;
  migration de l'**Epic 49** avant la nôtre.
- **Annonce de la migration** au projet GitHub « Kesh » : par le **Project Lead** (l'agent du responsable de l'epic n'écrit pas sur le
  dépôt de Guy).

## Avant le développement

*(Annoté le 2026-11-19, refonte « miroir seul, KISS » : reste à faire — revalidation en boucle des fiches 47-1 à 47-5 (P1 par Sonnet ou Opus), `Status: ready-for-dev` à la clôture de chaque boucle ; la décision du 2026-11-19 est à reporter au projet GitHub par le Project Lead (titre de l'epic, annonce de la migration à une colonne).)*

Fait le 2026-11-19 : réponses du responsable de l'epic (approuvé tel quel), décomptes repris du kickoff de la 0.20.0, fiches complètes
47-1a à 47-5 (`Status: drafted`), fichier de l'Epic 46 lu pour Q5 (`epic/46-rapports-groupes`, `4d23b3f9`).
Reste : approbation des fiches et maquettes par le responsable de l'epic (formulaire hors dépôt), puis validation en boucle de chaque
fiche (`CLAUDE.md` § *Review Iteration Rule*), `Status: ready-for-dev` à sa clôture ; point d'entrée du regroupement
convenu dans la fiche 46-1 (Q5). *(Annoté le 2026-11-19 : validation des huit fiches faite, § *Validation des fiches*
ci-dessus ; `Status: ready-for-dev`.)*

## Avant la PR (revue du code d'intégration P1, 2026-11-22)

- **Reprise de `main` après l'Epic 49** (M4) : la PR ne s'ouvre qu'après l'entrée de l'Epic 49 dans `main` (D10), puis reprise de `main` dans la branche. À recompter depuis la source après la fusion, sans les fusionner : audit des migrations (96 devient 97 : en-tête, ligne `Total`, partitions, ligne de la migration de l'Epic 49), `crates/kesh-db/tests/migrations_upgrade_path.rs` (`assert_eq!(total, 97)`, `total - 63`, frontière 34), schéma de test squash (`crates/kesh-db/test-schema/`), `migrations.sha384`, `sitesTotal` de `i18n-keys.test.ts` (relevé par la garde), et le « 96 au total » du manuel d'administration. Ordre des sections du manuel à résoudre en gardant les deux côtés ; conflits relevés par `git merge-tree --write-tree origin/epic/49-types-de-lignes-facture HEAD` (2026-11-22) : `crates/kesh-api/tests/admin_full_import_e2e.rs` et `crates/kesh-db/tests/grouped_settlements.rs` (garder les deux appels `colonne_miroir(false)` et `colonnes_types_de_ligne(false)`), `migrations.sha384`, `migrations_upgrade_path.rs`, l'audit des migrations et `admin-manual.tex` ; `epic-49-…md` s'auto-fusionne. Rejouer tous les gates, l'E2E complet et la recette de montée depuis la dernière image publiée.
- **Classement du changement de présentation des rapports** (M3), **à confirmer par le Project Lead** : proposé comme **non-rupture** au sens de la règle *Versions de l'API* (aucun champ retiré, renommé ni retypé ; la présentation ne change que si l'utilisateur pose un miroir, fonction neuve ; un rapport sans miroir rend le même JSON). Le manuel d'administration l'écrit ainsi et le dit à l'intégrateur. Si le Project Lead le tient pour une rupture, la mise en œuvre passe par la CR de la règle ; aucune `/api/v2/` n'est ouverte ici.
- **Pour la release** (LOW de la revue) : titre provisoire « Passer à la release de l'Epic 47 » et phrase « se pose sur la 0.19.1 » à renommer et à rectifier au tag ; captures du manuel (`mi-bilan.png` et les deux autres) au pied de page « Kesh v0.19.1 », à reprendre sur le montage de la release ; `sitesTotal` : valeur 3513 posée à la reprise de la 0.19.1, sans relevé cité (juste, confirmée par la garde à la 47-4) ; le commit « règle 8 » a réécrit des artefacts des Epics 48 et 49 (dont `epic-49-…md`, aussi modifié sur `epic/49`) : à signaler dans la PR, conflit attendu.
