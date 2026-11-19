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
   offset_account_id)`, `CHECK` « pas les deux » (`mirror_account_id IS NULL OR offset_account_id IS NULL`) ; **DDL
   seul**, aucune ligne réécrite ; `kesh_version_min_required` reste à 0.17.6 ; en-tête qui dit P1/P3 (non breaking),
   P7 (rien à trier) et la raison de l'absence de FK.
2. **Garde-fous de migration** : P5 — ligne à `docs/migrations-idempotence-audit.md` (verdict `tracked-by-sqlx`),
   les **deux** sites du total et les trois compteurs de partition **recomptés depuis la source** ; P6 — `grep -rn
   "migrations.len()\|apply_migrations_up_to" crates/` inspecté, chaque site positionnel résolu par version ou gardé ;
   P7 — `every_data_backfill_migration_is_triaged` vert sans entrée (DDL seul) ; squash
   `crates/kesh-db/test-schema/0001_schema_squash.sql` mis à jour, `test_schema_guard` vert ;
   `migrations_upgrade_path.rs` étendu (données d'avant intactes, colonnes neuves à `NULL`).
3. **Entité** : `Account` porte `mirror_account_id: Option<i64>` et `offset_account_id: Option<i64>` ; toutes les
   lectures `SELECT` d'`Account` (`FIND_BY_ID_SQL`, `FIND_BY_ID_FOR_UPDATE_SQL`, listes) les décodent ;
   `account_snapshot_json` les écrit (`mirrorAccountId`, `offsetAccountId`) : l'audit avant/après les montre.
4. **Changement à trois états** : `AccountUpdate` gagne `presentation_link: PresentationLinkChange` —
   `Unchanged` (défaut : un client qui ne connaît pas le champ n'efface rien), `None` (retirer), `Mirror(id)`,
   `Offset(id)` — résolu **dans le dépôt**, sous le `FOR UPDATE` du compte, comme `CurrencyChange` et
   `OwnFxAccountsChange` ; `is_no_op_change` le prend en compte.
5. **Règles de validité** (jugées sur l'état résultant, une seule fonction pure `check_presentation_link` testée sans
   base, puis le dépôt pour les lectures) ; refus **`DbError::PresentationLinkInvalid { field, reason,
   linked_account_number }`** (HTTP 400 `PRESENTATION_LINK_INVALID` en 47-1b), raisons d'une **liste close**, dans
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
     qui a son propre lien, la cible déjà en paire avec un tiers, et le compte, cible d'un miroir, qui demande un
     lien ; `linked_account_number` nomme le compte qui occupe le rôle.
   Une cible **non imputable** est admise (compte de présentation) ; une cible **en devise étrangère** aussi, de
   devise quelconque (Q3).
6. **Paire écrite des deux côtés, dans une transaction** : poser `Offset(B)` sur A écrit `A.offset = B` et
   `B.offset = A` ; déplacer (A passe de D à B) remet `D.offset` à `NULL` ; retirer (`None`) remet les deux côtés à
   `NULL` ; chaque compte modifié voit sa `version` incrémentée et reçoit **son** entrée d'audit `account.updated`
   (avant/après), dans la même transaction. Un miroir ne s'écrit que sur la source.
7. **Garde des gestes existants sur un compte lié** (Q10 étendu au retypage) — refus **`DbError::AccountPresentationLinked
   { reason, linked }`** (HTTP 409 `ACCOUNT_PRESENTATION_LINKED` en 47-1b), `linked` = comptes liés (id, numéro,
   nom), triés par numéro :
   - `archive` — archiver un compte **cible** d'un miroir ou **membre** d'une paire est refusé, **sans recours**
     (`confirm_designated` ne le lève pas) ; le message nomme le compte lié (Q10). Archiver une **source** de miroir
     reste admis (le lien demeure ; un rapport le présente encore) ;
   - `accountType` — retyper un compte qui a ou reçoit un lien est refusé tant que le lien existe ;
   - `equityRole` — poser un rôle de fonds propres sur un compte qui a ou reçoit un lien est refusé.
   Ordre dans `archive_guarded` : sous-comptes actifs → différence de change → **lien de présentation** →
   désignation de facturation → version. Dans `update` : après les refus de devise (37-1) et avant le rôle singleton.
8. **Lien orphelin** (cible supprimée hors de l'API — SQL, ou restauration partielle) lu comme absent : le lecteur
   des liens (AC-9) l'écarte avec `tracing::warn!` ; aucune panique, aucune boucle.
9. **Lecteur des liens pour les rapports** : `accounts::presentation_links(executor, company_id) ->
   Vec<PresentationLinkAccount>` rend, en **une requête**, chaque compte de la société qui a ou reçoit un lien —
   `id`, `number`, `name`, `account_type`, `role`, `currency`, `active`, `mirror_account_id`, `offset_account_id` ;
   consommé par la 47-2a. Test : une société sans lien → vecteur vide ; une autre société n'y paraît jamais.
10. **Concurrence** : deux poses croisées (A→B et B→A en paire ; liaison de T pendant l'archivage de T) ne laissent
    jamais un état qui viole l'AC-5 ou l'AC-7 : l'un passe, l'autre est refusé (409 de version ou refus nommé) ;
    test de dépôt à deux transactions.
11. **Sauvegarde** : une sauvegarde **0.19.0** (sans les deux colonnes) s'importe, liens vides (règle générique de
    `check_schema_compat`, colonnes facultatives) ; l'aller-retour d'une sauvegarde neuve conserve les liens ; une
    base migrée est refusée par l'image 0.19.0 (constat, non test : `sqlx`), d'où la sauvegarde d'avant la mise à
    jour (Notes de version).
12. **Inventaire des colonnes de compte** : `crates/kesh-db/tests/account_columns_inventory.rs` classe les deux
    colonnes neuves (elles finissent par `_account_id`) avec leur preuve : une variante neuve de `Preuve`
    (« présentation seule : aucune écriture ne s'y impute, devise admise, Q3 ») adossée aux tests de l'AC-5 et à un
    test qui établit qu'aucun chemin d'écriture ne lit ces colonnes (grep des requêtes d'écriture, inventaire clos).
13. **Aucune écriture comptable** (NFR-47-1) : poser, déplacer, retirer un lien ne crée ni ne modifie aucune ligne
    de `journal_entries` / `journal_entry_lines` (assertion de comptage dans les tests de l'AC-6).

## Tâches / sous-tâches

- [ ] **T0 — reprise et contrôle** (AC 1, 2) : `git fetch origin`, reprise de `main` dans `epic/47-comptes-miroirs`
  (49-1a et ses migrations comprises si fusionnées) ; lister `crates/kesh-db/migrations/` et choisir l'horodatage ;
  vérifier sur MariaDB de dev qu'un `CHECK` n'interdit rien d'attendu (AC 1).
- [ ] **T1 — migration** (AC 1, 2) : fichier SQL, ligne d'audit et compteurs recomptés, squash, P6, upgrade path.
- [ ] **T2 — entité et lectures** (AC 3, 4) : champs, SQL des lectures, snapshot d'audit, `PresentationLinkChange`,
  `is_no_op_change`.
- [ ] **T3 — validation** (AC 5) : `check_presentation_link` pure (table de vérité : 4 types × 2 liens × rôles) ;
  `DbError::PresentationLinkInvalid`, `PresentationLinkField`, `PresentationLinkReason` (`as_str`, `ALL`), à l'image
  de `FxAccountInvalid` ; `error_code()`.
- [ ] **T4 — écriture et verrous** (AC 6, 10, 13) : dans `accounts::update`, résolution sous verrou, écriture des deux
  côtés, audit par compte modifié ; ordre des verrous ci-dessous.
- [ ] **T5 — gestes gardés** (AC 7) : `archive_guarded`, retypage et rôle dans `update` ;
  `DbError::AccountPresentationLinked`.
- [ ] **T6 — lecteur** (AC 8, 9) : `presentation_links`.
- [ ] **T7 — tests** : dépôt (`crates/kesh-db/tests/account_presentation_links.rs`, neuf), inventaire des colonnes
  (AC 12), sauvegarde (`crates/kesh-api/tests/admin_backup_e2e.rs` ou fichier voisin : import d'une sauvegarde au
  schéma 0.19.0 et aller-retour), mutations.
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
`presentation_link` n'est pas `Unchanged` : lire **hors verrou** l'ancien partenaire de paire du compte, former
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
- **C-47-1a-4** : pas de `CHECK (mirror_account_id <> id)` : le contrôle `self` est au dépôt. MySQL refuse un `CHECK`
  qui lit une colonne `AUTO_INCREMENT` ; MariaDB l'admet peut-être, sans gain (le lien ne se pose que par `UPDATE`) —
  à confirmer en T0, sinon écrit tel quel au Change Log.

### Stratégie de tests

- **Unitaires** (sans base) : `check_presentation_link`, table de vérité complète ; `PresentationLinkReason::ALL`
  dans l'ordre de jugement ; `PresentationLinkChange::resolve`.
- **Intégration dépôt** (`#[sqlx::test]` sur le squash) : chaque refus de l'AC-5 et de l'AC-7 avec son détail ; pose,
  déplacement, retrait d'une paire (deux côtés, versions, deux entrées d'audit) ; source archivée admise ; lien
  orphelin ; lecteur ; concurrence (AC-10) ; comptage des écritures inchangé (AC-13).
- **Migration** : `test_schema_guard`, `migrations_upgrade_path`, `migrations_fresh_install`.
- **Sauvegarde** : import d'une sauvegarde au schéma 0.19.0, aller-retour neuf.
- **Mutations** (à consigner) : retirer l'écriture du second côté de la paire ; inverser l'ordre de deux raisons ;
  supprimer la garde d'archivage ; supprimer la lecture verrouillante des référents — chacune doit rougir un test.

### Risques

- Conflit de texte et de règle avec `fix/api-parent-compte` (même fonction `update`, mêmes erreurs).
- Ordre des migrations : si une autre migration entre sur `main` avant la fusion, renommer l'horodatage **avant toute
  publication** (P8 : jamais après).
- Interaction muette avec un test positionnel (P6) ou un compteur de l'audit (P5) : gate complet obligatoire.

### Project Structure Notes

Aucun fichier hors des chemins habituels ; migration nommée par horodatage ; pas de PDF de manuel commité ; i18n et
documentation en 47-1b.

### References

- `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md` (D1, D2, D3, D8, D10, Q3, Q10, FR-47-1, FR-47-2,
  FR-47-7, NFR-47-1, NFR-47-3).
- `CLAUDE.md` § *Migration breaking policy* (P1–P8), § *Pendant une boucle de revue* (exception `kesh-db`).
- `_bmad-output/planning-artifacts/release-0.20.0.md` (migration de la 49-1a, ordre).

## Notes de version

⚠️ **En tête des notes de la release : sauvegarde complète avant la mise à jour** — base, `.keshbackup` et dossier
des documents ; cette release porte une migration (`account_presentation_links`, non breaking : deux colonnes
facultatives sur les comptes) et le retour à la version précédente passe par cette sauvegarde. Une sauvegarde 0.19.0
ou 0.20.0 s'importe ; la version précédente refuse la base migrée et les sauvegardes neuves.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
