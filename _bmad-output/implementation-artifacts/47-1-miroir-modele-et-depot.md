# Story 47-1 : compte miroir — modèle et dépôt (migration)

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md` — § *Décision du 2026-11-19 : miroir seul*,
règles R1 à R4 ; D4, D7, D8, D10 (adaptée) ; Q1, Q10. **Refonte du 2026-11-19 (miroir seul, KISS)** de l'ancienne
47-1a (liens miroir et paire) : la paire, ses deux côtés écrits, ses transitions et ses raisons propres sont retirées.
La suite — API et écran du plan comptable — est la **47-2** ; la présentation dans les rapports, la **47-3**.

## Story

En tant que comptable,
je veux désigner sur un compte un **compte miroir**, compte de présentation sans écriture, enregistré et contrôlé par
Kesh,
afin que les rapports puissent présenter son solde de sens inverse de l'autre côté, sans écriture de reclassement.

## Modules touchés (règle de splitting préventif)

Cinq : `kesh-db/migrations` (avec le squash de test et l'audit des migrations), `kesh-db` comptes (entité et dépôt),
`kesh-db` écritures (une garde au point de passage des lignes), `kesh-api/errors` (avec les quatre catalogues, que le
sous-match exhaustif impose), `kesh-api/exports` (une colonne, que la garde d'exhaustivité impose). Aucune route, aucun
écran.

## Critères d'acceptation

1. **Migration non breaking** `crates/kesh-db/migrations/<horodatage>_account_mirror.sql`, horodatage **strictement
   postérieur à `20261118000002`** (Epic 49, 0.20.0) et à toute migration présente sur `main` à la reprise qui précède
   le développement (proposé : `20261119000001`, contrôlé à ce moment). En **une seule instruction** `ALTER TABLE
   accounts` (atomique sous MariaDB) : `mirror_account_id BIGINT NULL` (commentaire de colonne, **sans clé
   étrangère**, comme `fx_gain_account_id`), index `(company_id, mirror_account_id)`, contrainte
   `chk_accounts_mirror_not_self` (`mirror_account_id IS NULL OR mirror_account_id <> id`). Si MariaDB 10.11 refuse un
   `CHECK` qui lit `id` (`AUTO_INCREMENT`), l'écart à D10 ne se tranche pas seul : arrêt et question à Beat et Guy,
   constat au Change Log (T0) ; le contrôle `self` du dépôt (AC-4) existe de toute façon. **DDL seul**, aucune ligne
   réécrite ; `kesh_version_min_required` reste à 0.17.6 ; en-tête qui dit P1/P3 (non breaking), P7 (rien à trier),
   la raison de l'absence de FK (celle de `20261114000001` : cycle, remise à zéro, import sans ordre) et **le SQL qui
   défait la migration** (`DROP` de la contrainte, de l'index, de la colonne, puis `DELETE FROM _sqlx_migrations WHERE
   version = …`), comme l'en-tête `(c)` de `20261118000002_invoice_line_types.sql`.
2. **Garde-fous de migration** (repris de l'ancienne 47-1a, AC-2, validé) : P5 — ligne à
   `docs/migrations-idempotence-audit.md` (verdict `tracked-by-sqlx`), les **deux** sites du total et les trois
   compteurs de partition **recomptés depuis la source** ; P6 — `grep -rn "migrations.len()\|apply_migrations_up_to"
   crates/` inspecté, chaque site positionnel résolu par version ou gardé ; P7 —
   `every_data_backfill_migration_is_triaged` vert sans entrée ; squash `crates/kesh-db/test-schema/0001_schema_squash.sql`
   **régénéré** (`scripts/regen-test-schema.sh` ; son en-tête « Équivalent des N migrations » suit),
   `test_schema_guard` vert ; `migrations_upgrade_path.rs` étendu (données d'avant intactes, colonne neuve à `NULL`) et
   son compte en dur recompté. **Fenêtre positionnelle** (couplage assumé, P6 : frontière historique figée à **34**,
   gardée par le compte en dur) : `total` et le soustracteur `N` bougent du même pas, **`N = total − 34`** —
   `total - 61` / 95 deviennent `total - 62` / 96 si cette migration est la seule ajoutée depuis `fd04099f`, `total -
   63` / 97 si la 49-1a est fusionnée avant ; une ligne de plus à la généalogie du commentaire. **Bases partielles** :
   `columns!()` lira la colonne neuve, si bien qu'un test qui monte une base partielle (`apply_migrations_up_to`) puis
   passe, **avant** `MIGRATOR.run()`, par une fonction du dépôt qui lit `COLUMNS` / `FIND_BY_ID_SQL` d'`accounts.rs`
   rougirait en `Unknown column` (mode d'échec payé à la 37-1). À `origin/main` (`fd04099f`), **aucun site n'est dans
   ce cas** (inventaire de l'ancienne 47-1a, validations P3 et P4 : semis en SQL brut ou par listes de colonnes
   explicites) ; le développement **refait l'inventaire** après la reprise de `main` (T0, liste au Change Log) et, s'il
   trouve un site, pose la colonne le temps du semis et la retire avant `MIGRATOR.run()`.
3. **Entité** : `Account` porte `mirror_account_id: Option<i64>` ; toutes les lectures `SELECT` d'`Account`
   (`FIND_BY_ID_SQL`, `FIND_BY_ID_FOR_UPDATE_SQL`, listes) la décodent ; `account_snapshot_json` l'écrit
   (`mirrorAccountId`) : l'audit avant/après la montre. `NewAccount` gagne `mirror_account_id: Option<i64>` (un miroir
   peut se poser à la création, AC-4) ; `AccountUpdate` gagne `mirror: MirrorChange` à trois états — `Unchanged`
   (défaut : un client qui ne connaît pas le champ n'efface rien), `Clear`, `Set(id)` — sur le patron de
   `CurrencyChange` ; `is_no_op_change` le prend en compte (reposer le miroir déjà posé est un no-op, sans changement de
   version). Les sites qui construisent un `AccountUpdate` (32 littéraux à `fd04099f`, `git grep -c "AccountUpdate {"`,
   dont la route ; le type ne dérive pas `Default`) ou un `NewAccount` (54 littéraux, même commande) reçoivent
   `Unchanged` / `None` : la route n'expose le champ qu'en 47-2.
4. **Règles de validité** (R2, R3), jugées sur l'état résultant par une **fonction pure** `check_mirror` testée sans
   base, puis par le dépôt pour les lectures ; refus **`DbError::MirrorAccountInvalid { reason, linked_account_number
   }`** (HTTP 400 `MIRROR_ACCOUNT_INVALID`, mappé ici, AC-9), raisons d'une **liste close**, dans cet ordre :
   - `self` — la cible est le compte lui-même ;
   - `unknown` — cible introuvable ou d'une autre société (non distingués, anti-IDOR) ;
   - `archived` — cible archivée ;
   - `accountType` — la cible n'est pas de l'autre côté du même rapport (actif ↔ passif, produit ↔ charge) ;
   - `equity` — le compte source porte un rôle de fonds propres (`EquityCapital`, `EquityOther`, `RetainedEarnings`,
     `CurrentYearResult`) (D8) ;
   - `notPresentationAccount` — la cible n'est pas un compte de présentation : elle est imputable, **ou** porte un
     rôle, **ou** porte au moins une ligne d'écriture (lecture de `journal_entry_lines` **après** le verrou de la
     cible, AC-7) ;
   - `chain` — la cible a elle-même un miroir, **ou** le compte source est déjà le miroir d'un autre compte ;
     `linked_account_number` nomme le compte en cause.
   **Plusieurs sources** peuvent viser le même miroir (R3). Une cible **en devise étrangère** est admise (la ligne du
   miroir se présente en devise principale, R8). Le contrôle s'applique à la **création** (`NewAccount` portant un
   miroir) comme à la **modification** ; à la modification d'un compte qui **garde** son miroir (`Unchanged`), il se
   rejoue sur l'état résultant — retyper la source de sorte que sa cible ne soit plus de l'autre côté, ou lui poser un
   rôle de fonds propres, rend `MIRROR_ACCOUNT_INVALID` (`accountType`, `equity`) ; retirer le miroir dans le même
   `PUT` lève ce refus. Sur `Unchanged`, seules ces deux règles du côté de la source se rejouent : la cible, figée par
   l'AC-5, n'est ni relue ni verrouillée — un état forgé de la cible (lignes, rôle) n'empêche pas de renommer la source.
5. **Garde du compte miroir** (R4) — tant qu'au moins un compte de la société le désigne comme miroir (**source
   archivée comprise**), refus **`DbError::AccountIsMirror { reason, sources }`** (HTTP 409 `ACCOUNT_IS_MIRROR`, mappé
   ici, AC-9), `sources` = comptes qui le désignent (id, numéro, nom, actif), triés par numéro, raisons d'une liste
   close : `archive` (archiver ; `confirm_designated` ne le lève pas), `postable` (le rendre imputable), `accountType`
   (le retyper), `role` (lui poser un rôle). Lui poser son propre miroir relève de l'AC-4 (`chain`). Une **source
   archivée** compte : le miroir ne s'archive qu'une fois le lien retiré, ce qui suppose de réactiver la source
   (`update` refuse un compte archivé) ; le message le dit. Archiver une **source** reste admis (le lien demeure ; les
   rapports la présentent encore). Ordre dans `archive_guarded` : sous-comptes actifs → différence de change →
   **compte miroir** → désignation de facturation → version. Dans `update` : après les refus de devise (37-1) et avant
   le rôle singleton.
6. **Aucune écriture sur un compte miroir** (R4, NFR-47-1) : `validate_lines_accounts_in_tx`
   (`crates/kesh-db/src/repositories/journal_entries.rs:263`), point de passage des deux seuls chemins qui insèrent
   des lignes (`create_in_tx_inner`, `update_in_tx` ; doc-comment de `insert_lines`, `:153`), refuse **sans condition**
   — y compris quand `enforce_postable` est faux, c'est-à-dire pour les flux automatiques (factures, avoirs, factures
   fournisseur, rapprochement, réévaluation de change) — toute ligne sur un compte désigné comme miroir par un compte
   de la même société : même erreur que la saisie manuelle sur un compte non imputable,
   **`DbError::AccountsNotPostable`** (fil : `400 ACCOUNT_NOT_POSTABLE`, `account_not_postable_response`,
   `kesh-api/src/errors.rs:2069`), qui nomme le compte (un compte miroir est non imputable par l'AC-4 ; aucun code
   neuf, aucun message neuf). **Lecture fraîche, verrouillante** : après la lecture `LOCK IN SHARE MODE` des comptes
   (`:276-283`), une **seconde instruction verrouillante**, `SELECT DISTINCT mirror_account_id FROM accounts WHERE
   company_id = ? AND mirror_account_id IN (…) LOCK IN SHARE MODE` (par l'index de l'AC-1) — **et non** un `EXISTS`
   corrélé dans la première : sous REPEATABLE READ, une sous-requête non verrouillante lit l'instantané de la
   transaction, déjà ouvert par la lecture non verrouillante de `books_locked_through` (`create_in_tx_inner`, `:531`)
   et par celles des appelants ; une pose commitée pendant l'attente du verrou sur la cible n'y serait pas vue, et la
   ligne passerait. La lecture verrouillante lit la dernière version commitée et pose ses verrous d'intervalle sur
   l'index, qui font aussi attendre une pose concurrente. L'ordre des refus devient : absent ou archivé → non
   imputable (si `enforce_postable`) **ou miroir** (toujours). Tests : saisie manuelle et un flux automatique
   (`create_in_tx` avec `enforce_postable = false`) refusés sur un compte miroir, nommé ; acceptés une fois le lien
   retiré (et le compte rendu imputable pour la saisie manuelle).
7. **Concurrence** : à la modification, A est **déjà** sous `FOR UPDATE` (`FIND_BY_ID_FOR_UPDATE_SQL`, première
   lecture d'`update`, `accounts.rs:968`) ; poser un miroir A → T verrouille ensuite **T seul** (`SELECT … WHERE id = ?
   AND company_id = ? FOR UPDATE`, patron du parent dans `create`, `:211`), après les gardes de devise ; à la création,
   seul T se verrouille (A n'existe pas). Puis lecture **verrouillante** (`LOCK IN SHARE MODE`, jamais une lecture
   d'instantané : même motif qu'à l'AC-6) des lignes d'écriture de T (`has_journal_line_locked`) et de « qui désigne A
   comme miroir » (index de l'AC-1). L'ancienne cible, si le miroir change, n'est pas verrouillée (retirer un lien ne
   viole aucune règle). Côté cible, les gardes de l'AC-5 (`update` et `archive_guarded` de T, qui tiennent T en `FOR
   UPDATE`) lisent « qui désigne T » de même, en lecture verrouillante. Ainsi « lier T » se sérialise sur la ligne de T
   avec « passer une écriture sur T » (`validate_lines_accounts_in_tx` lit T en `LOCK IN SHARE MODE`), « archiver T »,
   « rendre T imputable », et avec « désigner A comme miroir d'un tiers » (qui verrouille A). Un **interblocage**
   (`1213`) reste possible quand une écriture porte à la fois sur A et sur T (elle prend A et T en partage dans l'ordre
   de l'index, la pose tient A puis attend T) : InnoDB annule l'une, ce qui est un refus, non un état faux. Test de
   dépôt à deux transactions pour deux de ces couples (pose contre écriture, l'écriture ayant fait une lecture non
   verrouillante **avant** d'attendre — c'est ce qui rend la mutation « `EXISTS` d'instantané » rouge ; pose contre
   archivage) : l'un passe, l'autre est refusé (409 de version, refus nommé ou interblocage), jamais un état qui viole
   l'AC-4, l'AC-5 ou l'AC-6.
8. **Lecteur des liens pour les rapports** : `accounts::mirror_links(executor, company_id) -> Vec<MirrorLink>` rend,
   en **une requête**, chaque lien valide de la société — `source_id` et, pour la cible, `id`, `number`, `name`,
   `account_type`, `active` (la cible n'a aucune écriture : les rapports n'en ont pas la ligne et la construisent de là,
   `AccountBalance` exigeant `active`). **Lien hors règle lu comme absent** : l'import d'une sauvegarde
   (`check_schema_compat` contrôle les colonnes, jamais les valeurs) ou un SQL peut écrire ce que l'AC-4 refuse ; une
   fonction **pure** `retain_valid_links(rows) -> Vec<MirrorLink>` écarte, avec un `tracing::warn!` par lien écarté
   nommant les comptes et le cas, la **liste close** de ce qui fausserait la présentation : (a) cible absente ou d'une
   autre société (la requête ne lit que la société : un seul cas, deux tests) ; (b) types qui ne sont pas de l'autre
   côté du même rapport (le lien vers soi-même y tombe : même type) ; (c) chaîne — la cible a elle-même un miroir (les
   liens se jugent **tels que lus**, avant tout écartement). Ne sont **pas** écartés, faute d'effet sur les invariants de
   la 47-3 : une source de rôle de fonds propres (la présentation ne lit jamais la section `equity`), une cible à rôle,
   imputable ou porteuse d'écritures (état forgé : la présentation ajoute son solde propre, 47-3), un compte archivé. Un
   test par cas, sans base (lignes construites) et par SQL direct ; une société sans lien → vecteur vide ; une autre
   société n'y paraît jamais.
9. **Erreurs mappées à l'API dans cette story** (le sous-match de `crates/kesh-api/src/errors.rs` sur `DbError` est
   exhaustif, sans `_ =>` : ajouter les variantes sans les mapper ne compile pas) — `MirrorAccountInvalid` → 400
   `MIRROR_ACCOUNT_INVALID`, `details: { reason[, linkedAccountNumber] }` ; `AccountIsMirror` → 409
   `ACCOUNT_IS_MIRROR`, `details: { reason, sources: [{ id, number, name, active }] }` — avec leurs messages dans les
   quatre locales (`crates/kesh-i18n/locales/*/messages.ftl`), qui **nomment** le compte en cause et, si une source est
   archivée, disent de la réactiver d'abord. Une clé par raison, la fonction de réponse traitant chaque raison par un
   `match` **sans cas par défaut** (patron `fx_account_invalid_response`, `errors.rs:1881`), et un test qui parcourt
   la liste close des raisons et exige une clé dans les quatre catalogues. Le Dev Agent Record **liste les noms de clés
   livrés** (la 47-2 les reprend). Aucun numéro de compte en dur dans un catalogue (G4-bis) : variables Fluent.
10. **Sauvegarde** : une sauvegarde **0.19.0** ou **0.20.0** (sans la colonne) s'importe, miroirs vides (règle
    générique de `check_schema_compat`, colonne facultative ; `import.rs:597-640`) ; test sur le patron de
    `full_import_of_a_0_17_6_backup_takes_the_multi_currency_defaults` (`crates/kesh-api/tests/admin_full_import_e2e.rs:5946`) :
    sauvegarde privée de la colonne ; l'aller-retour d'une sauvegarde neuve conserve les miroirs. Une base migrée est
    refusée par l'image de la release précédente (constat, non test : `sqlx` refuse une migration appliquée inconnue),
    d'où la sauvegarde d'avant la mise à jour (Notes de version). La colonne n'a pas à figurer à
    `ABSENCES_ADMISES_EPIC_37`.
11. **Inventaire des colonnes de compte** : `crates/kesh-db/tests/account_columns_inventory.rs` classe la colonne
    neuve (elle finit par `_account_id`) avec une variante neuve de `Preuve` (« présentation seule : aucune écriture ne
    s'y impute, refusée au point de passage des lignes ; une cible en devise est admise, R8 ») adossée aux tests des
    AC-4 et AC-6 ; l'en-tête du fichier, qui dit l'inventaire limité aux colonnes « qui ne peuvent pas désigner un
    compte en devise », est élargi d'une phrase. *(Pas de liste close des fichiers qui nomment `mirror_account_id` :
    la garde de l'AC-6, au seul point de passage des lignes, refuse toute écriture sur un miroir quel que soit le code
    qui lit la colonne ; une telle liste ne garderait rien de plus et coûterait une mise à jour à chaque story.)*
12. **Aucune écriture comptable produite** (NFR-47-1) : poser, changer, retirer un miroir ne crée ni ne modifie aucune
    ligne de `journal_entries` / `journal_entry_lines` (assertion de comptage) ; seul le compte source est écrit (une
    entrée d'audit `account.updated`, sa `version` incrémentée ; la cible n'est pas modifiée).
13. **Export CSV des tables** : `serialize_accounts_csv` ajoute `mirror_account_id` **à la fin de la ligne, après
    `updated_at`** (en-tête, cellule `fmt_opt_i64`, en-tête figé de `serialize_accounts_csv_includes_role_and_postable`
    mis à jour, un test d'une ligne liée) — sans quoi `chaque_colonne_du_schema_est_exportee_ou_ecartee`, qui confronte
    les en-têtes au squash, rougit. `sample_account` (`csv_tables.rs:1871`) gagne le champ à `None`.

## Tâches / sous-tâches

- [ ] **T0 — reprise et contrôle** (AC 1, 2) : `git fetch origin`, reprise de `main` dans `epic/47-comptes-miroirs`
  (49-1a et ses migrations comprises si fusionnées) ; horodatage choisi ; `CHECK` sur `id` vérifié sur la MariaDB de
  dev (refus → arrêt et question) ; inventaire des bases partielles.
- [ ] **T1 — migration** (AC 1, 2) : SQL, audit et compteurs, squash, P6, upgrade path (`N = total − 34`).
- [ ] **T2 — entité** (AC 3, 13) : champ, lectures, snapshot, `MirrorChange`, `NewAccount`, `is_no_op_change`, CSV.
- [ ] **T3 — règles** (AC 4, 5) : `check_mirror` pure (table de vérité : 4 types × rôles × imputable × écritures ×
  chaîne), `DbError::MirrorAccountInvalid`, `MirrorReason` (`as_str`, `ALL`), `DbError::AccountIsMirror`,
  `AccountIsMirrorReason` ; `error_code()`.
- [ ] **T4 — écriture et verrous** (AC 4, 5, 7, 12) : `create`, `update`, `archive_guarded`.
- [ ] **T5 — garde des lignes** (AC 6) : `validate_lines_accounts_in_tx`.
- [ ] **T6 — lecteur** (AC 8) : `mirror_links`, `retain_valid_links` (trois cas).
- [ ] **T7 — erreurs et i18n** (AC 9).
- [ ] **T8 — tests** : dépôt (`crates/kesh-db/tests/account_mirror.rs`, neuf), inventaire (AC 11), sauvegarde (AC 10),
  réponses d'erreur (AC 9), mutations (retirer la garde des lignes en flux automatique ; inverser deux raisons ;
  supprimer la garde d'archivage ; supprimer le verrou de la cible ; remplacer la lecture verrouillante de l'AC-6 par
  un `EXISTS` corrélé d'instantané — chacune doit rougir un test).
- [ ] **T9 — gate complet** (exception `kesh-db` : jamais ciblé).

## Dev Notes

### Code lu (à `origin/main` `fd04099f`, relevé de l'ancienne 47-1a)

- Entité : `crates/kesh-db/src/entities/account.rs:222-256` (`Account`, `fx_gain_account_id` sans FK), `:270-320`
  (`CurrencyChange` : patron du changement à trois états), `:425-440` (`AccountUpdate`).
- Dépôt : `crates/kesh-db/src/repositories/accounts.rs:42` (colonnes), `:63` (`FIND_BY_ID_FOR_UPDATE_SQL`), `:71`
  (`account_snapshot_json`), `:156` (`effective_postable` : un compte à sous-comptes ou de rôle résultat est non
  imputable), `:713` (`is_no_op_change`), `:745-835` (`guard_currency_update`), `:944-1196` (`update`), `:1198-1330`
  (`archive_guarded`), `:2601` (`check_own_fx_accounts_in_tx` : patron d'un pointeur sans FK) ; lignes d'écriture d'un
  compte : `has_journal_line_locked`.
- Écritures : `crates/kesh-db/src/repositories/journal_entries.rs:153` (`insert_lines`, deux seuls `INSERT` de
  production), `:263-320` (`validate_lines_accounts_in_tx` : absent/archivé puis non imputable si `enforce_postable`),
  `:368-376` (doc d'`enforce_postable` : les flux automatiques passent `false`).
- Erreurs : `crates/kesh-db/src/errors.rs:1-90` (`FxAccountReason` : liste close, `ALL`), `:1140`, `:2453`.
- Les désignations de change refusent déjà un compte non imputable (`FxAccountReason::NotPostable`) : un compte miroir
  ne peut pas devenir compte de gains ou de pertes de change.
- Correctif concurrent : `fix/api-parent-compte` (#1351, non fusionné) ajoute `ParentChange` à `AccountUpdate` et
  touche `update`, `errors.rs` : conflit attendu, le second arrivé résout en gardant les deux. S'il est fusionné :
  créer un sous-compte sous un compte miroir le laisse non imputable (`effective_postable`) ; rien à garder de plus
  (un miroir peut avoir des sous-comptes : il se présente comme un compte de regroupement à solde propre, ce que les
  groupes de l'Epic 46 savent rendre).

### Choix KISS (refonte du 2026-11-19)

- **C-47-1-1** : une colonne, un lien posé sur la source seulement ; la cible n'est jamais écrite (une entrée d'audit,
  une version). *Écarté* : la paire réciproque écrite des deux côtés (retirée de l'epic).
- **C-47-1-2** : « compte de présentation » = non imputable, sans rôle, sans écriture — trois conditions, une raison
  (`notPresentationAccount`). *Écarté* : une raison par condition (trois clés i18n de plus pour un même remède).
- **C-47-1-3** : plusieurs sources vers un même miroir **admises** : l'interdire coûterait une règle et un refus de
  plus, l'admettre une somme dans la présentation ; le cas des dettes sociales le demande (plusieurs passifs débiteurs,
  un seul compte d'actif de présentation).
- **C-47-1-4** : aucune écriture sur un miroir, gardée **au seul point de passage** des lignes, par l'erreur existante
  `DbError::AccountsNotPostable` (`400 ACCOUNT_NOT_POSTABLE`). *Écarté* : compter sur `postable` seul (les flux automatiques passent `enforce_postable =
  false` et posteraient sur un miroir) ; un code d'erreur neuf.
- **C-47-1-5** : un miroir se pose aussi à la création (même contrôle) : un seul `<select>` dans le dialogue, création
  et modification. *Écarté* : le refus à la création (raison `onCreate`) de l'ancienne 47-1b.
- **C-47-1-6** : le `CHECK` « pas soi-même » de D10 est gardé ; le `CHECK` « pas les deux » disparaît avec la paire.

### Stratégie de tests

Unitaires (`check_mirror`, `retain_valid_links`, `MirrorReason::ALL`) ; intégration dépôt (`#[sqlx::test]` sur le
squash : chaque refus des AC-4 et AC-5 avec son détail, pose, changement, retrait, source archivée admise, garde des
lignes manuelle et automatique, lecteur, concurrence, comptage des écritures) ; migration (`test_schema_guard`,
`migrations_upgrade_path`, `migrations_fresh_install`) ; sauvegarde ; réponses d'erreur.

### Risques

- Conflit avec `fix/api-parent-compte` (même `update`, mêmes erreurs). Un `PUT` qui change parent et miroir : le parent
  se verrouille après la cible du miroir (A, puis T, puis le parent).
- Un réglage qui désigne un compte jamais mouvementé puis rendu non imputable (compte de produit d'un article, compte
  d'un relevé bancaire…) peut être choisi comme miroir : le flux automatique qui l'emploierait est alors refusé
  (`ACCOUNT_NOT_POSTABLE`, nommé) — bruyant, jamais faux. Constat au manuel (47-2), non garde de plus.
- Recette de montée : l'empreinte de `accounts` change (une colonne) ; l'écart attendu s'écrit au journal de la
  release.
- Ordre des migrations : renommer l'horodatage **avant toute publication** si une autre migration entre sur `main`
  (P8). Gate complet obligatoire (P5, P6).

### References

- Fichier d'epic : § *Décision du 2026-11-19 : miroir seul*, D10, § *Exemple chiffré*.
- `CLAUDE.md` § *Migration breaking policy* (P1–P8), § *Pendant une boucle de revue* (exception `kesh-db`).
- Registre de la dette du fichier d'epic : à la clôture de la boucle de revue, tout finding non remédié y est consigné,
  une ligne par problème ; la 47-5 en part (« on trace toujours »).

## Notes de version

⚠️ **En tête des notes de la release : sauvegarde complète avant la mise à jour** — base, `.keshbackup` et dossier
des documents ; cette release porte une migration (`account_mirror`, non breaking : une colonne facultative sur les
comptes) et le retour à la version précédente passe par cette sauvegarde. Une sauvegarde 0.19.0 ou 0.20.0 s'importe ;
la version précédente refuse la base migrée et les sauvegardes neuves. Un compte peut désigner un **compte miroir**,
compte de présentation sur lequel aucune écriture ne se passe.

## Change Log

- 2026-11-19 — **Refonte du 2026-11-19 (miroir seul, KISS)** : fiche renommée de `47-1a-liens-modele-et-depot`,
  réécrite sur la décision de Beat et Guy (fichier d'epic, § *Décision du 2026-11-19*). Retirés : `offset_account_id`,
  paire écrite des deux côtés, transitions paire ↔ miroir, raisons `bothLinks`, `onCreate`, `CHECK` « pas les deux »,
  cas (d) « paire non réciproque » du lecteur. Ajoutés : la cible est un compte de présentation (non imputable, sans
  rôle, sans écriture), la garde des lignes au point de passage (AC-6), le miroir à la création, le mapping et les
  messages restent ici. Gardés de la fiche validée : horodatage après `20261118000002`, `N = total − 34`, inventaire
  des bases partielles, lecteur tolérant (`retain_valid_links`, `tracing::warn!`), sauvegarde, inventaire des colonnes,
  export CSV, sous-match exhaustif. Treize critères (quinze avant). **Statut remis à `drafted`** en attendant la
  revalidation (boucle complète, P1 par Sonnet ou Opus).

- 2026-11-19 — validation P1 de la fiche refondue (Opus 5.5, avec la 47-3) : **0 C / 0 H / 5 M / 4 L, remédiés**.
  M1 : code du fil `ACCOUNT_NOT_POSTABLE` (non `ACCOUNTS_NOT_POSTABLE`, `kesh-api/src/errors.rs:2080`) → AC-6, C-47-1-4,
  Risques ; propagé au fichier d'epic (R4) et à la 47-2 (AC-11). M2 : l'`EXISTS` corrélé de l'AC-6 lisait
  l'instantané de la transaction, ouvert avant par `books_locked_through` (`journal_entries.rs:531`) : une pose
  commitée pendant l'attente du verrou de la cible n'était pas vue → seconde instruction `LOCK IN SHARE MODE` ; même
  exigence pour « qui désigne T » aux gardes de l'AC-5 (AC-7) ; mutation ajoutée (T8). M3 : « {A, T} en une instruction
  ordonnée par id » contredisait `update`, qui tient A en `FOR UPDATE` dès `:968` → A puis T seul, interblocage admis
  comme refus (AC-7, Risques). M4 (KISS) : `DiscardedLink`, vestige du contrôle K10 retiré, sans consommateur →
  `retain_valid_links` rend `Vec<MirrorLink>`, trois cas (cible absente, types, chaîne ; soi-même tombe sous les types ;
  fonds propres et rôle de cible sans effet sur la présentation) (AC-8). M5 (KISS) : la liste close des fichiers qui
  nomment `mirror_account_id` ne gardait rien que la garde de l'AC-6 ne garde déjà, et coûtait une mise à jour à
  chaque story → retirée (AC-11 ; 47-2 AC-12 et 47-3 AC-14 alignés). L : `MirrorLink` porte `active` (exigé par
  `AccountBalance`, 47-3) ; décomptes 32 `AccountUpdate` / 54 `NewAccount` ; sur `Unchanged`, seules les règles de la
  source se rejouent (AC-4) ; ordre des verrous du parent (Risques). Vérifié sans écart à `origin/main` `fd04099f` :
  95 migrations, dernière `20261118000002` sur `origin/epic/49-types-de-lignes-facture` (96, `total - 62`) ;
  `validate_lines_accounts_in_tx` (`:263`, `LOCK IN SHARE MODE`, `enforce_postable`), deux seuls `INSERT` de lignes
  (`insert_lines`) ; `check_schema_compat` (`import.rs:597-640`) ; `serialize_accounts_csv` (`csv_tables.rs:287`) ;
  `fx_account_invalid_response` (`errors.rs:1881`) ; `full_import_of_a_0_17_6_…` (`:5946`) ; `scripts/regen-test-schema.sh`.
  Rapport : `kesh-gate-logs/epic-47-validation/47-1-miroir-modele-et-depot-P1.md`.

### Historique — Change Log de la 47-1a (avant la refonte, laissé tel quel)

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
  `kesh-gate-logs/epic-47-validation/47-1a-liens-modele-et-depot-P3.md`. Complément de la même passe (relevé par la
  P4 de la 47-3a, transmis par l'orchestrateur ; M5) : transitions paire avec D → miroir vers D et source unique d'un
  miroir vers T → paire avec T, refusées en `chain` si l'état d'avant était jugé → AC-5 (jugement sur l'état après le
  `PUT`, sondes nommées) et AC-6 (écritures, deux tests d'intégration et deux sondes). Total P3 : 0 C / 0 H / 5 M / 1 L.

- 2026-11-19 — validation P4 ciblée sur la remédiation P3 (`aaf7b310`, `1cd1849d`), Opus 5.5 : **0 C / 0 H / 1 M / 2 L,
  remédiés**. Vérifié sans écart à `origin/main` `fd04099f` et `origin/epic/49-types-de-lignes-facture` : compte en dur
  95 / `total - 61` (`migrations_upgrade_path.rs:137-140, 239`), 96 / `total - 62` à la 49-1a, d'où 96 / 62 ou 97 / 63
  (`N = total − 34`) ; entête du squash « 95 migrations » (`:3`) ; sites de `apply_migrations_up_to` et de
  `migrations_before` résolus par version ou relatifs (P6) ; « aucune base partielle ne lit `columns!()` d'`Account` » :
  hors `accounts.rs`, les lecteurs sont `companies.rs:1076` (`check_fx_accounts_in_tx`), `journal_entry_conversions`
  (`:287, :318, :455`), des routes et `kesh-seed`, aucun dans une fenêtre partielle (`invoices.rs:608`,
  `journal_entries.rs:277`, `refuse_foreign_currency_accounts_in_tx` lisent des colonnes explicites ; `journal_entries.rs:3629,
  5545` sont dans `mod tests`) ; la 49-1a ne touche pas `accounts`. `retain_valid_links` : (a)/(b) un cas, deux tests,
  conforme. **M** (aussi relevé par la P5 de la 47-3a) : la sonde (1) de l'AC-5 refusait en `chain` un D libéré encore cible
  du miroir d'un tiers, contre D2 et l'AC-5 (« cible d'un ou de plusieurs miroirs ») → sonde (1) retirée, sonde (2)
  gardée, état forgé renvoyé à l'AC-8 (d)/(e), un seul test de refus (AC-6). Les deux transitions sont exactement celles
  de la 47-3a AC-2 (lue à HEAD, non modifiée). **L** : AC-6, le côté de D n'est remis à `NULL` que s'il désigne A.
  **L** : AC-8, « un test par cas » précisé pour (a)/(b). Rapport :
  `kesh-gate-logs/epic-47-validation/47-1a-liens-modele-et-depot-P4.md`.

- 2026-11-19 — validation P5 ciblée sur la remédiation P4 (`d5eff9e0`), Sonnet 5.5, une lentille : concordance de
  l'AC-5, de l'AC-6 et de l'AC-8 avec la 47-3a AC-2 : **0 C / 0 H / 0 M / 0 L**, aucune fiche modifiée. Sonde (1)
  absente partout (citée aux seuls Change Log), sonde (2) identique dans les deux fiches, candidats conformes à l'AC-5,
  état forgé renvoyé à l'AC-8 (d)/(e). Non exercés : code de `origin/main`, manuel, i18n (hors lentille). Boucle close ;
  `Status: ready-for-dev`. Rapport : `kesh-gate-logs/epic-47-validation/47-1a-liens-modele-et-depot-P5-ciblee.md`.


## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
