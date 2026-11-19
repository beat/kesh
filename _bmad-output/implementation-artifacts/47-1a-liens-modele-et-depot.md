# Story 47-1a : liens de présentation — modèle et dépôt (migration)

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md` — D1, D2, D3, D8, D10 ; Q3, Q10 ; FR-47-1 (dépôt),
FR-47-2, FR-47-7 ; NFR-47-1, NFR-47-3. Née du découpage de l'esquisse 47-1 (2026-11-19, splitting préventif : sept
modules) ; la suite — API, MCP, export, documentation — est la **47-1b**.

## Story

En tant que comptable,
je veux désigner sur un compte son **compte miroir** ou son **partenaire de paire**, enregistrés et contrôlés par
Kesh,
afin que les rapports puissent présenter un solde de sens inverse ou un net sans écriture de reclassement.

## Critères d'acceptation

1. **Migration non breaking** `crates/kesh-db/migrations/<horodatage>_account_presentation_links.sql`, horodatage
   **strictement postérieur à `20261118000002`** (Epic 49, 0.20.0) et à toute migration présente sur `main` à la
   reprise qui précède le développement (proposé : `20261119000001`, contrôlé à ce moment) : sur `accounts`,
   `mirror_account_id BIGINT NULL` et `offset_account_id BIGINT NULL` (commentaires de colonne, **sans clé
   étrangère**, comme `fx_gain_account_id`), index `(company_id, mirror_account_id)` et `(company_id,
   offset_account_id)`, et les `CHECK` de **D10** (« pas soi-même, pas les deux ») : `chk_accounts_presentation_single`
   (`mirror_account_id IS NULL OR offset_account_id IS NULL`), `chk_accounts_presentation_not_self`
   (`(mirror_account_id IS NULL OR mirror_account_id <> id) AND (offset_account_id IS NULL OR offset_account_id <>
   id)`) — en **une seule instruction** `ALTER TABLE accounts` (une instruction DDL MariaDB est atomique ; deux
   laisseraient une demi-migration après une coupure). Si MariaDB 10.11 refuse un `CHECK` qui lit `id`
   (`AUTO_INCREMENT`), **l'écart à D10 ne se tranche pas seul** : arrêt et question à Beat et Guy, constat au Change
   Log (T0) ; le contrôle `self` du dépôt (AC-5) existe de toute façon. **DDL seul**, aucune ligne réécrite ;
   `kesh_version_min_required` reste à 0.17.6 ; en-tête qui dit P1/P3 (non breaking), P7 (rien à trier), la raison
   de l'absence de FK (celle de `20261114000001` : cycle, remise à zéro, import sans ordre) et **le SQL qui défait la
   migration** (P1 : `DROP` des deux contraintes, des deux index, des deux colonnes, puis `DELETE FROM
   _sqlx_migrations WHERE version = …`), comme l'en-tête `(c)` de `20261118000002_invoice_line_types.sql`.
2. **Garde-fous de migration** : P5 — ligne à `docs/migrations-idempotence-audit.md` (verdict `tracked-by-sqlx`),
   les **deux** sites du total et les trois compteurs de partition **recomptés depuis la source** ; P6 — `grep -rn
   "migrations.len()\|apply_migrations_up_to" crates/` inspecté, chaque site positionnel résolu par version ou gardé ;
   P7 — `every_data_backfill_migration_is_triaged` vert sans entrée (DDL seul) ; squash
   `crates/kesh-db/test-schema/0001_schema_squash.sql` **régénéré** (`scripts/regen-test-schema.sh`, fichier
   « GÉNÉRÉ, NE PAS ÉDITER » ; son en-tête « Équivalent des 95 migrations », `:3`, suit), `test_schema_guard` vert ;
   `migrations_upgrade_path.rs` étendu (données d'avant intactes, colonnes neuves à `NULL`) et son compte en dur
   (`assert_eq!(total, 95, …)`, `:137-141` à `fd04099f`, garde de la fenêtre positionnelle « sauf les 61 dernières »)
   recompté. **Semis par le dépôt sur une base d'avant la migration** : `columns!()` lira les deux colonnes, si bien
   qu'un test qui monte une base partielle (`apply_migrations_up_to`) puis passe, **avant** `MIGRATOR.run()`, par une
   fonction du dépôt qui lit `COLUMNS` / `FIND_BY_ID_SQL` d'`accounts.rs` (`create`, `find_by_id`, `list_by_company`,
   `update`, `archive*`, `bulk_create*`, `check_fx_accounts_in_tx`, `conversion_line_accounts_in_tx`…) rougirait en
   `Unknown column` — mode d'échec payé à la 37-1. ⚠️ **À `origin/main` (`fd04099f`), aucun site n'est dans ce cas**
   (vérifié à la validation P3) : les semis des fenêtres partielles passent par du SQL brut
   (`test_fixtures::seed_accounting_company`, `monter` de `grouped_settlements.rs:198`) ou par des dépôts qui lisent
   des listes de colonnes explicites d'`accounts` (`invoices.rs:608`, `journal_entries.rs:277`,
   `refuse_foreign_currency_accounts_in_tx`) ; c'est vrai des appelants de `common::colonnes_multi_devises`
   (`migrations_upgrade_path.rs:1397, 2197, 2423, 2529`, `lettering_documents_backfill.rs:246` : `companies::create`,
   `supplier_invoices::*`, `payment_batches::*`, `journal_entries::reverse`), de `grouped_settlements.rs:1301`
   (`the_upgrade_keeps_unit_settlements_and_allows_a_group`, base d'avant la 43-1a ; la 49-1a y pose
   `common::colonnes_types_de_ligne`, non pour `accounts`) et des fenêtres en SQL brut
   (`*_backfill.rs`, `journal_entry_number_sequences_bootstrap.rs`, tests `n` / `n + 1` de
   `migrations_upgrade_path.rs`, `kesh-api/src/pre_upgrade_backup/tests.rs`, `kesh-api/tests/sauvegarde_avant_migration.rs`).
   **Aucune pose ni aucun retrait n'est donc prescrit.** Le développement refait cet inventaire après la reprise de
   `main` (T0 ; liste au Change Log) ; s'il trouve un site qui lit l'`Account` du dépôt avant la montée, il pose les
   deux colonnes le temps du semis et les retire avant `MIGRATOR.run()` (pas d'`IF NOT EXISTS`) : par
   `colonnes_multi_devises` sur une base d'avant la 37-1 (constante chaînée), par une fonction séparée sur une base
   postérieure à la 37-1 (un `ADD COLUMN` redoublé des colonnes de la 37-1 échouerait). **Fenêtre positionnelle**
   (couplage assumé, P6 : frontière historique figée à **34**, gardée par le compte en dur) : `total` et le
   soustracteur `N` bougent du même pas, `N = total − 34` — `total - 61` / 95 deviennent `total - 62` / 96 si cette
   migration est la seule ajoutée depuis `fd04099f`, et `total - 63` / 97 si la 49-1a (qui pose déjà `total - 62` /
   96 sur `origin/epic/49-types-de-lignes-facture`) est fusionnée avant ; une ligne de plus à la généalogie du
   commentaire. Laisser `N` en arrière déplacerait la frontière sans rien faire rougir.
3. **Entité** : `Account` porte `mirror_account_id: Option<i64>` et `offset_account_id: Option<i64>` ; toutes les
   lectures `SELECT` d'`Account` (`FIND_BY_ID_SQL`, `FIND_BY_ID_FOR_UPDATE_SQL`, listes) les décodent ;
   `account_snapshot_json` les écrit (`mirrorAccountId`, `offsetAccountId`) : l'audit avant/après les montre.
4. **Changement à trois états** : `AccountUpdate` gagne `presentation_link: PresentationLinkChange` —
   `Unchanged` (défaut : un client qui ne connaît pas le champ n'efface rien), `ClearMirror`, `ClearOffset`,
   `ClearAll` (retirer — **la nature est portée par la variante** : « retirer le miroir d'un compte qui tient une
   paire » est un no-op, et ne peut se juger qu'**ici, sous verrou**, non dans la route sur une lecture préalable),
   `Mirror(id)`, `Offset(id)` (poser une nature **remplace** le lien de l'autre nature que le compte tient, paire
   comprise : le partenaire est libéré, AC-6) — résolu **dans le dépôt**, sous le `FOR UPDATE` du compte, comme `CurrencyChange` et
   `OwnFxAccountsChange` ; `is_no_op_change` le prend en compte. Traduction depuis le corps de l'API : 47-1b AC-1.
5. **Règles de validité** (jugées sur l'état résultant, une seule fonction pure `check_presentation_link` testée sans
   base, puis le dépôt pour les lectures) ; refus **`DbError::PresentationLinkInvalid { field, reason,
   linked_account_number }`** (HTTP 400 `PRESENTATION_LINK_INVALID`, mappé ici, AC-15), raisons d'une **liste close**, dans
   cet ordre :
   - `self` — la cible est le compte lui-même ;
   - `unknown` — cible introuvable ou d'une autre société (non distingués, anti-IDOR) ;
   - `archived` — cible archivée ;
   - `equity` — le compte ou la cible porte un rôle de fonds propres (`EquityCapital`, `EquityOther`,
     `RetainedEarnings`, `CurrentYearResult`) (D8) ;
   - `accountType` — miroir : la cible n'est pas de l'autre côté du même rapport (actif ↔ passif, produit ↔
     charge) ; paire : les deux comptes ne sont pas de types opposés du même rapport (D2, D3) ;
   - `chain` — le compte ou la cible joue déjà un autre rôle : **un compte joue au plus un rôle** — source d'un
     miroir, cible d'un ou de plusieurs miroirs, ou membre d'une paire (D2 : « pas de chaîne », Guy). Couvre la cible
     qui a son propre lien, la cible déjà en paire avec un tiers, le compte, cible d'un miroir, qui demande un
     lien ; `linked_account_number` nomme le compte qui occupe le rôle. ⚠️ **Reposer le même lien est un no-op, jamais un
     `chain`** : la cible d'un miroir de ce compte-ci, ou le partenaire actuel de sa paire, ne « joue pas un autre
     rôle » — un client qui renvoie à `PUT` le corps lu par `GET` n'a ni refus ni changement de version (test).
   Deux raisons de plus appartiennent à la même énumération (`PresentationLinkReason`, `as_str`, `ALL`), **posées par
   l'API** (47-1b) et **jamais produites par le dépôt** (test sur toutes les entrées de ce dernier) : `bothLinks`
   (deux liens entiers dans une même requête) et `onCreate` (un lien à la création) ; elles suivent les six
   ci-dessus dans `ALL`. Le détail `field` de `PresentationLinkField` dit `mirrorAccountId` ou `offsetAccountId`.
   Une cible **non imputable** est admise (compte de présentation) ; une cible **en devise étrangère** aussi, de
   devise quelconque (Q3).
6. **Paire écrite des deux côtés, dans une transaction** : poser `Offset(B)` sur A écrit `A.offset = B` et
   `B.offset = A` ; déplacer (A passe de D à B) remet `D.offset` à `NULL` ; quitter la paire autrement — retrait
   (`ClearOffset`, `ClearAll`) ou passage à un miroir (`Mirror(T)` sur A, membre d'une paire avec D) — remet les deux
   côtés à `NULL` (`A.offset` et `D.offset`), le miroir posé dans la même instruction que le retrait sur A (le `CHECK`
   « pas les deux » se juge à la ligne) ; chaque compte modifié voit sa `version` incrémentée et reçoit **son** entrée d'audit `account.updated`
   (avant/après), dans la même transaction. Un miroir ne s'écrit que sur la source.
7. **Garde des gestes existants sur un compte lié** (Q10 étendu au retypage) — refus **`DbError::AccountPresentationLinked
   { reason, linked }`** (HTTP 409 `ACCOUNT_PRESENTATION_LINKED`, mappé ici, AC-15), `linked` = comptes liés (id, numéro,
   nom, actif), triés par numéro :
   - `archive` — archiver un compte **cible** d'un miroir ou **membre** d'une paire est refusé, **sans recours**
     (`confirm_designated` ne le lève pas) ; le message nomme le compte lié (Q10). Archiver une **source** de miroir
     reste admis (le lien demeure ; un rapport le présente encore). **Une source archivée compte** parmi les comptes
     liés — comme une désignation de change « archivé compris » : la cible ne s'archive qu'une fois le lien retiré,
     ce qui suppose de réactiver la source (`reactivate`, `accounts.rs:1430`), `update` refusant un compte archivé
     (`:983-988`) ; le message le dit (47-1b). `reactivate` n'a pas de garde neuve : tant que la source est archivée,
     sa cible ne peut être ni archivée, ni retypée, ni passée en fonds propres, si bien que le lien reste valide ;
   - `accountType` — retyper un compte qui **reçoit** un lien (cible d'un miroir, ou partenaire d'une paire que le
     même geste ne quitte pas) est refusé tant que le lien existe ;
   - `equityRole` — poser un rôle de fonds propres sur un compte qui **reçoit** un lien est refusé.
   **Partage avec l'AC-5, jugé sur l'état résultant du même `PUT`** (aucune situation n'a deux codes) : le **propre**
   lien du compte (source d'un miroir, ou son côté de la paire) se juge par l'AC-5 dès qu'il subsiste après le geste
   — posé, déplacé **ou inchangé** (`Unchanged`) : retyper ou passer en fonds propres un compte qui garde son miroir
   rend donc `PRESENTATION_LINK_INVALID` (`accountType` ou `equity`) ; le retirer dans le même `PUT` lève ce refus.
   L'AC-7 ne juge que ce que le compte **reçoit** des autres, que ce geste ne peut retirer. Dans une paire A↔B, retyper
   A en gardant la paire relève de l'AC-5 (côté propre de A) ; tests des deux branches.
   Ordre dans `archive_guarded` : sous-comptes actifs → différence de change → **lien de présentation** →
   désignation de facturation → version. Dans `update` : après les refus de devise (37-1) et avant le rôle singleton.
8. **Lien hors règle lu comme absent** : l'import d'une sauvegarde (`check_schema_compat` contrôle les colonnes,
   jamais les valeurs ; pas de FK) ou un SQL peut écrire ce que l'AC-5 refuse. Le lecteur (AC-9) écarte, avec
   `tracing::warn!` nommant les comptes, la **liste close** suivante — et la 47-2a ne reçoit que des liens valides
   (son test de propriété les suppose) : (a) cible absente (**orphelin**) ; (b) cible d'une autre société — **même cas que (a) au lecteur** : la requête ne lit
   que les comptes de la société, une cible d'ailleurs y est absente (anti-IDOR, comme `unknown` de l'AC-5) ; un
   seul `case` pour les deux, deux tests (cible supprimée, cible d'une autre société) ; (c) lien
   vers soi-même ; (d) paire **non réciproque** (`A.offset = B` sans `B.offset = A`), les deux côtés écartés ;
   (e) **chaîne** : miroir dont la cible porte elle-même un lien, ou vise un membre de paire — le miroir est écarté ; (d) et
   (e) se jugent sur les liens **tels que lus**, avant tout écartement (résultat indépendant de l'ordre : un miroir vers
   un compte dont la paire est écartée reste écarté comme chaîne) ;
   (f) lien d'un compte ou vers un compte de rôle de fonds propres ; (g) types qui ne sont plus de l'autre côté du
   même rapport. Un compte archivé (source ou cible) n'est **pas** écarté. Aucune panique, aucune boucle (la règle (e)
   suffit à rendre la relation acyclique). Un test par cas, posé par SQL direct.
   **Forme rendue (AC-9)** : un lien écarté n'est **jamais** rendu ; le compte qui le portait **reste dans le vecteur**,
   ses colonnes de lien concernées à `None` (un compte qui ne fait que recevoir un lien écarté y reste aussi, sans lien
   entrant), et chaque écart est tracé par un `tracing::warn!` nommant les comptes et le cas (a)–(g). Le tri se fait
   dans une fonction **pure** `retain_valid_links(rows) -> (Vec<PresentationLinkAccount>, Vec<DiscardedLink { account_id,
   field, case }>)`, testée sans base (un test par cas, sur des lignes construites) et par SQL direct via
   `presentation_links` (le vecteur n'a aucun lien invalide ; le cas se lit au `DiscardedLink`, que la fonction
   publique n'expose pas). Conséquence pour la 47-2a et la 47-4 : « le vecteur ne contient que des liens valides » est
   vrai par construction ; un compte sans lien dans le vecteur est un compte dont le lien a été écarté ou qui n'en
   a plus d'entrant : ils l'ignorent.
9. **Lecteur des liens pour les rapports** : `accounts::presentation_links(executor, company_id) ->
   Vec<PresentationLinkAccount>` rend, en **une requête**, chaque compte de la société qui a ou reçoit un lien (reçu d'un compte **de la même société** : un lien posé
   par SQL depuis une autre société vers celle-ci ne fait pas entrer la cible dans le vecteur) —
   `id`, `number`, `name`, `account_type`, `role`, `currency`, `active`, `mirror_account_id`, `offset_account_id` ;
   consommé par la 47-2a et la 47-4 ; ne rend que des liens valides (forme de l'AC-8). Test : une société sans lien → vecteur vide ; une autre société n'y paraît jamais.
10. **Concurrence** : deux poses croisées (A→B et B→A en paire ; liaison de T pendant l'archivage de T) ne laissent
    jamais un état qui viole l'AC-5 ou l'AC-7 : l'un passe, l'autre est refusé (409 de version ou refus nommé) ;
    test de dépôt à deux transactions.
11. **Sauvegarde** : une sauvegarde **0.19.0** (sans les deux colonnes) s'importe, liens vides (règle générique de
    `check_schema_compat`, colonnes facultatives ; `import.rs:597-640`) — de même une sauvegarde 0.20.0 ; test sur le
    patron de `full_import_of_a_0_17_6_backup_takes_the_multi_currency_defaults`
    (`crates/kesh-api/tests/admin_full_import_e2e.rs:5946`) : sauvegarde **privée des deux colonnes** ; l'aller-retour
    d'une sauvegarde neuve conserve les liens (paire des deux côtés, miroir) ; une base migrée est refusée par l'image
    de la **release précédente** (0.20.x : constat, non test, `sqlx` refuse une migration appliquée inconnue), d'où
    la sauvegarde d'avant la mise à jour (Notes de version). Les colonnes n'ont pas à figurer à
    `ABSENCES_ADMISES_EPIC_37` (`kesh-db/src/backup.rs:156-245`), liste close des seules migrations de l'Epic 37.
12. **Inventaire des colonnes de compte** : `crates/kesh-db/tests/account_columns_inventory.rs` classe les deux
    colonnes neuves (elles finissent par `_account_id`) avec leur preuve : une variante neuve de `Preuve`
    (« présentation seule : aucune écriture ne s'y impute, devise admise, Q3 ») adossée aux tests de l'AC-5 et à un
    test, pur, qui établit qu'aucun chemin d'écriture ne lit ces colonnes : il parcourt `crates/*/src/**/*.rs` et
    exige que l'ensemble des fichiers qui nomment `mirror_account_id` ou `offset_account_id` **égale**, dans les deux
    sens, une liste close écrite en dur — à cette story `kesh-db/src/entities/account.rs`,
    `kesh-db/src/repositories/accounts.rs`, `kesh-api/src/exports/csv_tables.rs` (AC-14) et `kesh-api/src/errors.rs`
    si l'AC-15 y nomme un champ ; chaque story suivante (47-1b, 47-2a, 47-4) y ajoute ses fichiers, aucun d'un chemin
    d'écriture comptable (`journal_entries`, `invoices`, `fx_revaluation`…).
13. **Aucune écriture comptable** (NFR-47-1) : poser, déplacer, retirer un lien ne crée ni ne modifie aucune ligne
    de `journal_entries` / `journal_entry_lines` (assertion de comptage dans les tests de l'AC-6).
14. **Export CSV des tables** (déplacé de la 47-1b à la validation P1 de celle-ci) : `serialize_accounts_csv` ajoute
    `mirror_account_id` et `offset_account_id` **à la fin de la ligne, après `updated_at`** (en-tête, cellules
    `fmt_opt_i64`, en-tête figé de `serialize_accounts_csv_includes_role_and_postable` mis à jour, un test d'une ligne
    liée). **Pourquoi ici** : `csv_tables::tests::chaque_colonne_du_schema_est_exportee_ou_ecartee` confronte chaque
    en-tête au squash `test-schema/0001_schema_squash.sql` que l'AC-2 met à jour ; sans ces deux colonnes, le gate
    complet de cette story rougit. Le seul site qui construit un `Account` littéral à `origin/main` est `sample_account`
    (`csv_tables.rs:1871`) ; il gagne les deux champs à `None` (compilation ; `journal_entry_reversal_e2e.rs` construit
    un `AccountUpdate`, compté à l'AC-15).
15. **Erreurs mappées à l'API dans cette story** (déplacé de la 47-1b à la validation P1 de celle-ci) :
    `crates/kesh-api/src/errors.rs` tient un **sous-match exhaustif** sur `DbError`, sans `_ =>` (commentaire
    `:4229-4231`, match `:4380`, à `fd04099f`) : ajouter `PresentationLinkInvalid` et `AccountPresentationLinked`
    sans les y mapper ne compile pas. Cette story pose donc le mapping que décrivait la 47-1b AC-2 —
    `PresentationLinkInvalid` → 400 `PRESENTATION_LINK_INVALID`, `details: { field, reason[, linkedAccountNumber] }` ;
    `AccountPresentationLinked` → 409 `ACCOUNT_PRESENTATION_LINKED`, `details: { reason, linkedAccounts: [{ id,
    number, name, active }] }` — avec ses messages dans les quatre locales (`crates/kesh-i18n/locales/*/messages.ftl`),
    qui **nomment** le compte lié (Q10) et, s'il est archivé, disent de le réactiver d'abord (AC-7) ; le contrat
    exact (détails, texte d'exemple) est celui de la 47-1b AC-2, que cette story réalise, et un test unitaire de réponse par variante (patron `fx_account_invalid_response`,
    `errors.rs:1881`). ⚠️ **Toutes les clés i18n des raisons naissent ici**, dans les quatre locales : chaque raison de
    `PresentationLinkReason::ALL` — les six du dépôt **et `bothLinks` et `onCreate`**, que seule l'API (47-1b) produit —
    et chaque raison d'`AccountPresentationLinked` (`archive`, `accountType`, `equityRole`) ; la fonction de réponse
    traite chaque raison par un `match` **sans cas par défaut** (comme `fx_account_invalid_response`), si bien qu'une
    raison sans clé ne compile pas ; un test parcourt `ALL` et exige une clé présente dans les quatre catalogues. Le
    Dev Agent Record **liste les noms de clés livrés** (la 47-1b le constate à son T0). Seule la clé de
    l'avertissement de paire, `accounts-presentation-offset-warning`, reste à la 47-1b. Les sites qui construisent un `AccountUpdate` (32 à `fd04099f`, dont la route
    `routes/accounts.rs` et 22 dans `repositories/accounts.rs`) reçoivent `presentation_link: Unchanged` : la route
    n'expose les champs qu'en 47-1b.

## Tâches / sous-tâches

- [ ] **T0 — reprise et contrôle** (AC 1, 2) : `git fetch origin`, reprise de `main` dans `epic/47-comptes-miroirs`
  (49-1a et ses migrations comprises si fusionnées) ; lister `crates/kesh-db/migrations/` et choisir l'horodatage ;
  vérifier sur MariaDB de dev que les deux `CHECK` de l'AC-1 (dont « pas soi-même », qui lit `id`) sont acceptés et
  n'interdisent rien d'attendu ; refus → arrêt et question (AC 1, C-47-1a-4).
- [ ] **T1 — migration** (AC 1, 2) : fichier SQL, ligne d'audit et compteurs recomptés, squash régénéré, P6, upgrade
  path et son compte en dur (`N = total − 34`), inventaire des semis par le dépôt sur base partielle (AC-2 : aucun à
  `fd04099f` ; pose et retrait seulement si la reprise de `main` en fait apparaître un).
- [ ] **T2 — entité et lectures** (AC 3, 4, 14) : champs, SQL des lectures, snapshot d'audit, `PresentationLinkChange`
  (variantes `Clear*`), `is_no_op_change`, colonnes de `accounts.csv` et sites de construction
  d'`Account`.
- [ ] **T3 — validation** (AC 5) : `check_presentation_link` pure (table de vérité : 4 types × 2 liens × rôles) ;
  `DbError::PresentationLinkInvalid`, `PresentationLinkField`, `PresentationLinkReason` (`as_str`, `ALL`), à l'image
  de `FxAccountInvalid` ; `error_code()` ; mapping HTTP et messages des quatre locales (AC 15).
- [ ] **T4 — écriture et verrous** (AC 6, 10, 13) : dans `accounts::update`, résolution sous verrou, écriture des deux
  côtés, audit par compte modifié ; ordre des verrous ci-dessous.
- [ ] **T5 — gestes gardés** (AC 7) : `archive_guarded`, retypage et rôle dans `update` ;
  `DbError::AccountPresentationLinked`.
- [ ] **T6 — lecteur** (AC 8, 9) : `presentation_links`.
- [ ] **T7 — tests** : dépôt (`crates/kesh-db/tests/account_presentation_links.rs`, neuf), inventaire des colonnes
  (AC 12), sauvegarde (`crates/kesh-api/tests/admin_full_import_e2e.rs`, patron `:5946` : import d'une sauvegarde
  privée des deux colonnes et aller-retour), réponses d'erreur (AC-15), mutations.
- [ ] **T8 — gate complet** (exception `kesh-db` : jamais ciblé, `CLAUDE.md` § *Pendant une boucle de revue*).

## Dev Notes

### Code lu (à `origin/main` `fd04099f`)

- Entité : `crates/kesh-db/src/entities/account.rs:222-256` (`Account`, `fx_gain_account_id` sans FK),
  `:270-320` (`CurrencyChange`, `OwnFxAccountsChange` : patron des changements à trois états), `:425-440`
  (`AccountUpdate` : « tous les champs requis, sauf les trois états »).
- Dépôt : `crates/kesh-db/src/repositories/accounts.rs:42` (colonnes des lectures), `:63`
  (`FIND_BY_ID_FOR_UPDATE_SQL`), `:71` (`account_snapshot_json`), `:713` (`is_no_op_change`), `:745-835`
  (`guard_currency_update`, ordre des refus), `:944-1196` (`update` : `company_of_account` sur le pool avant
  `begin()`, `lock_company_currency_in_tx`, `lock_invoice_settings_share_in_tx`, compte `FOR UPDATE`, garde de
  version, no-op, gardes, `UPDATE`, audit `{before, after}` et `account.retyped`), `:1198-1330` (`archive`,
  `archive_guarded` : ordre des refus et des verrous, `refuse_designated`), `:2601` (`check_own_fx_accounts_in_tx` :
  patron de validation d'un pointeur sans FK).
- Erreurs : `crates/kesh-db/src/errors.rs:1-90` (`FxAccountField`, `FxAccountReason`, liste close et `ALL`),
  `:1140` (`FxAccountInvalid`), `:1244` (`AccountIsFxDifferenceAccount`), `:2453` (`error_code`).
- Migration de référence : `crates/kesh-db/migrations/20261114000001_multi_currency_settings.sql:30-73` (colonnes
  sans FK, `CHECK`, index `(company_id, …)`, en-tête P1/P3/P7) ; la dernière de `main` à la rédaction est
  `20261118000001_grouped_settlements.sql` ; `20261118000002_invoice_line_types.sql` vit sur
  `origin/epic/49-types-de-lignes-facture` (0.20.0).
- Audit des migrations : `docs/migrations-idempotence-audit.md:17` (en-tête « 95 migrations ») et `:121` (ligne
  `Total`) — 96 après la 49-1a, 97 avec celle-ci si rien d'autre n'entre ; **recompter** (`ls … | wc -l`, `grep -c
  '^| \`20' …`).
- Sauvegarde : `crates/kesh-api/src/admin_backup/import.rs:597-640` (`check_schema_compat` : colonne inconnue → refus,
  colonne requise absente → refus ; une colonne `NULL` sans défaut n'est pas requise) ; une **table** neuve aurait
  été refusée (raison de D1).
- Inventaire : `crates/kesh-db/tests/account_columns_inventory.rs:27-50` (`Preuve`, liste close `INVENTAIRE`).
- Correctif concurrent : `fix/api-parent-compte` (non fusionné) ajoute `ParentChange` à `AccountUpdate` et touche
  `update`, `errors.rs` des deux crates : **conflit attendu**, à résoudre en gardant les deux changements à trois
  états (le second arrivé résout).

### Ordre des verrous (AC 6, 7, 10)

`update` garde son préfixe : `company_id` lu sur le pool ; `companies` en partage (en `FOR UPDATE` si comptes propres
changent) ; réglages de facturation en partage. Puis, **au lieu du seul `FOR UPDATE` du compte**, quand
`presentation_link` n'est pas `Unchanged` : lire **hors verrou** — sur le pool, avant `begin()`, comme
`company_of_account` (une lecture non verrouillante dans la transaction figerait son instantané) — l'ancien
partenaire de paire du compte, former
l'ensemble {compte, ancien partenaire, nouvelle cible}, et le verrouiller **en une instruction ordonnée par id**
(`SELECT … WHERE id IN (…) ORDER BY id FOR UPDATE`) ; relu sous verrou, si l'ancien partenaire diffère de la lecture
hors verrou, la version du compte a changé : `OptimisticLockConflict` (409). Les lectures de « qui pointe vers ce
compte » (règle `chain`, AC-7) se font **en lecture verrouillante par index** (`… WHERE company_id = ? AND
mirror_account_id = ? LOCK IN SHARE MODE`), après les `FOR UPDATE`. `archive_guarded` : inchangé jusqu'au compte
`FOR UPDATE`, puis la même lecture verrouillante des référents. Ainsi « lier T » et « archiver T » se sérialisent
sur la ligne de T.

### Choix proposés (à confirmer à la validation)

- **C-47-1a-1** : une seule paire de codes (`PRESENTATION_LINK_INVALID` 400 à raisons closes ;
  `ACCOUNT_PRESENTATION_LINKED` 409 à raison `archive|accountType|equityRole`), à l'image de `FX_ACCOUNT_INVALID` et
  `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`.
- **C-47-1a-2** : « un compte joue au plus un rôle » est la forme décidable de « pas de chaîne » (D2) : elle couvre
  aussi miroir vers un compte en paire et paire avec une cible de miroir, que la présentation ne saurait ordonner.
- **C-47-1a-3** : plusieurs sources peuvent viser la même cible (D2 : « le solde inversé de ses sources »).
- **C-47-1a-4** *(révisé à la validation P1 : la rédaction d'origine écartait ce `CHECK`, contre D10)* : le `CHECK`
  « pas soi-même » de D10 est posé (AC-1), **en plus** du contrôle `self` du dépôt. MySQL refuse un `CHECK` qui lit
  une colonne `AUTO_INCREMENT` ; MariaDB 10.11 l'admet selon toute vraisemblance — vérifié en T0 sur la MariaDB de
  dev ; s'il est refusé, arrêt et question à Beat et Guy (écart à une décision), jamais un retrait silencieux.

### Stratégie de tests

- **Unitaires** (sans base) : `check_presentation_link`, table de vérité complète ; `PresentationLinkReason::ALL`
  dans l'ordre de jugement (`bothLinks` et `onCreate` en dernier, jamais produits par le dépôt) ;
  `PresentationLinkChange::resolve` ; lien reposé = no-op.
- **Intégration dépôt** (`#[sqlx::test]` sur le squash) : chaque refus de l'AC-5 et de l'AC-7 avec son détail ; pose,
  déplacement, retrait d'une paire (deux côtés, versions, deux entrées d'audit) ; source archivée admise ; lien
  orphelin ; lecteur ; concurrence (AC-10) ; comptage des écritures inchangé (AC-13).
- **Migration** : `test_schema_guard`, `migrations_upgrade_path`, `migrations_fresh_install`.
- **Sauvegarde** : import d'une sauvegarde privée des deux colonnes (0.19.0 ou 0.20.0), aller-retour neuf.
- **Lecteur** : un test par cas de la liste close de l'AC-8, posé par SQL direct.
- **Partage AC-5 / AC-7** : retyper ou passer en fonds propres un compte qui garde son lien (`PRESENTATION_LINK_INVALID`),
  qui le retire dans le même `PUT` (admis), qui reçoit un lien (`ACCOUNT_PRESENTATION_LINKED`) ; cible dont la seule
  source est archivée (archivage refusé).
- **Mutations** (à consigner) : retirer l'écriture du second côté de la paire ; inverser l'ordre de deux raisons ;
  supprimer la garde d'archivage ; supprimer la lecture verrouillante des référents — chacune doit rougir un test.

### Risques

- Conflit de texte et de règle avec `fix/api-parent-compte` (même fonction `update`, mêmes erreurs). **Ordre des
  verrous** : ce correctif prend le nouveau parent `FOR UPDATE` **après** le compte, puis ses ancêtres en lecture
  verrouillante (`guard_parent_change_in_tx`) ; cette story verrouille un ensemble ordonné par id. Un `PUT` qui
  change à la fois parent et lien : le parent se prend après l'ensemble ordonné (jamais avant), et le cas se teste à
  la résolution du conflit.
- Recette de montée (`scripts/recette-montee.sh`) : l'empreinte de `accounts` change (deux colonnes) ; l'écart
  attendu s'écrit au journal de la release, comme `bank_accounts` à la 0.17.6.
- Ordre des migrations : si une autre migration entre sur `main` avant la fusion, renommer l'horodatage **avant toute
  publication** (P8 : jamais après).
- Interaction muette avec un test positionnel (P6) ou un compteur de l'audit (P5) : gate complet obligatoire.

### Project Structure Notes

Aucun fichier hors des chemins habituels ; migration nommée par horodatage ; pas de PDF de manuel commité ; les messages des deux
erreurs (AC-15) sont ici — **toutes** les clés de leurs raisons, `bothLinks` et `onCreate` comprises (réponse sans cas
par défaut, patron `fx_account_invalid_response`, `kesh-api/src/errors.rs:1881` ; noms livrés au Dev Agent Record) ;
à la 47-1b ne reste que la clé `accounts-presentation-offset-warning` et la documentation.

### References

- `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md` (D1, D2, D3, D8, D10, Q3, Q10, FR-47-1, FR-47-2,
  FR-47-7, NFR-47-1, NFR-47-3).
- `CLAUDE.md` § *Migration breaking policy* (P1–P8), § *Pendant une boucle de revue* (exception `kesh-db`).
- `_bmad-output/planning-artifacts/release-0.20.0.md` (migration de la 49-1a, ordre).
- Registre de la dette du fichier d'epic (§ *Registre de la dette de l'epic*) : à la clôture de la boucle de revue,
  tout finding non remédié y est consigné, une ligne par problème ; la 47-5 en part (« on trace toujours »).

## Notes de version

⚠️ **En tête des notes de la release : sauvegarde complète avant la mise à jour** — base, `.keshbackup` et dossier
des documents ; cette release porte une migration (`account_presentation_links`, non breaking : deux colonnes
facultatives sur les comptes) et le retour à la version précédente passe par cette sauvegarde. Une sauvegarde 0.19.0
ou 0.20.0 s'importe ; la version précédente refuse la base migrée et les sauvegardes neuves.

## Change Log

- 2026-11-19 — validation P1 de la **47-1b** (Sonnet 5.5), effets sur cette fiche : AC-4 (variantes `Clear*`,
  la nature du retrait se juge sous verrou), AC-5 (lien reposé = no-op ; raisons `bothLinks` et
  `onCreate` portées par l'énumération, posées par l'API), AC-14 neuve (colonnes de `accounts.csv`, déplacées de la
  47-1b : la garde d'exhaustivité des colonnes lit le squash), T2 et stratégie de tests. Rapport :
  `kesh-gate-logs/epic-47-validation/47-1b-liens-api-mcp-export-P1.md`. Cette fiche n'a pas subi sa propre passe
  de validation ici.
- 2026-11-19 — validation P1 (Opus 5.5) : **0 C / 1 H / 5 M / 6 L, remédiés**. H1 : le sous-match exhaustif de
  `DbError` (`kesh-api/src/errors.rs:4229-4231`, `:4380`) rendait la story non compilable avec le mapping en 47-1b →
  AC-15 neuve (mapping et messages des deux refus ici ; 47-1b AC-2 et T2, fichier d'epic ajustés). M1 : semis par le
  dépôt sur base partielle (`common::colonnes_multi_devises`) → AC-2. M2 : contradiction avec D10 (`CHECK` « pas
  soi-même » écarté) → AC-1, C-47-1a-4 révisé, T0 (arrêt et question si MariaDB refuse). M3 : chevauchement AC-5 /
  AC-7 (deux codes pour un même cas, retypage et retrait dans un même `PUT`) → partage écrit à l'AC-7. M4 : source
  archivée parmi les référents, non dit → AC-7 (comptée ; `reactivate` sans garde neuve, motif écrit). M5 : lecteur
  limité aux orphelins alors que l'import et le SQL peuvent écrire tout état refusé → AC-8, liste close (a)–(g). L :
  AC-6 (sortie de paire par `Mirror`, `None` périmé), AC-11 (image de la release précédente, patron
  `admin_full_import_e2e.rs:5946`), AC-12 (liste close des fichiers sources, testable), AC-1 (une instruction,
  noms des contraintes, SQL qui défait la migration), AC-2 (squash régénéré, compte en dur de
  `migrations_upgrade_path.rs`), ordre des verrous (lecture hors verrou sur le pool ; parent de `fix/api-parent-compte`)
  et recette de montée aux Risques. Rapport : `kesh-gate-logs/epic-47-validation/47-1a-liens-modele-et-depot-P1.md`.

- 2026-11-19 — validation P2 (Sonnet 5.5) : **0 C / 0 H / 2 M / 2 L, remédiés**. M1 : seconde famille de bases
  partielles (entre la 37-1 et la migration), `grouped_settlements.rs:1301` rouge en `Unknown column`, et
  `colonnes_multi_devises` ne peut y servir (ADD COLUMN redoublé) → AC-2, fonction séparée. M2 : forme du vecteur du
  lecteur pour un lien écarté non fixée (relevé de la P3 de la 47-4, K10) → AC-8 « Forme rendue », AC-9, fonction pure
  `retain_valid_links` et ses tests. L1 : fenêtre `total - 61` → 62 écrite (AC-2) ; (d)/(e) jugés sur les liens
  tels que lus (AC-8) ; L2 : un seul littéral `Account` (AC-15). Axes exercés : code de `origin/main` (migrations 95,
  epic 49 : 96, horodatage libre après `20261118000002`), sites positionnels, `Account` littéraux, sous-match
  d'`errors.rs`, écrivains de `accounts`, D1–D10. Non exercés : acceptation du `CHECK` sur `id` par MariaDB (T0, pas de
  base), manuels. Rapport : `kesh-gate-logs/epic-47-validation/47-1a-liens-modele-et-depot-P2.md`.

- 2026-11-19 — validation P3 ciblée sur la remédiation P2 (`c58de40a`), Opus 5.5 : **0 C / 0 H / 4 M / 1 L, remédiés**.
  M1 : la prémisse du M1 de la P2 est fausse à `origin/main` — `monter` (`grouped_settlements.rs:198`) sème
  `accounts` en SQL brut, `invoices.rs:608` et `journal_entries.rs:277` lisent des colonnes explicites ; aucune fenêtre
  partielle ne lit `columns!()` d'`Account` avant la montée (appelants de `colonnes_multi_devises` compris) → AC-2 et
  T1 : aucune pose prescrite, inventaire refait à la reprise, pose conditionnelle. M2 : `total - 62` écrit sans
  condition alors que la 49-1a pose déjà `total - 62` / 96 (`origin/epic/49-types-de-lignes-facture`) : avec elle,
  97 exige `total - 63`, sinon la frontière glisse à 35 sans rien faire rougir → AC-2 `N = total − 34`. M3 : le cas
  (b) n'est pas distinguable de (a) par `retain_valid_links` sur les seuls comptes de la société (« un test par
  cas » intestable tel qu'écrit) → AC-8 : un seul cas, deux tests. M4 (relevé par la P2 de la 47-1b, transmis par
  l'orchestrateur) : toutes les clés i18n des raisons, `bothLinks` et `onCreate` comprises, naissent ici, réponse
  sans cas par défaut, noms au Dev Agent Record ; seule `accounts-presentation-offset-warning` reste à la 47-1b →
  AC-15, *Project Structure Notes*. L1 : « reçoit un lien » limité aux sources de la même société (AC-9). Forme du
  vecteur (compte gardé, lien à `None`) conforme à ce qu'attendent la 47-2a (algorithme § 1, « reste à sa place,
  brut ») et la 47-4 (T2, « le compte reste relevé ») : rien à signaler. Rapport :
  `kesh-gate-logs/epic-47-validation/47-1a-liens-modele-et-depot-P3.md`.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
