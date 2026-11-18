# Story 47-1 : liens de présentation — miroir et paire (modèle)

Status: draft

Epic : `epic-47-comptes-miroirs.md` (D1, D2, D3, D8, D10).

## Intention

En tant que comptable, je désigne sur un compte son **compte miroir** ou son **partenaire de paire**, pour que les
rapports le présentent sans écriture de reclassement.

## Critères d'acceptation

1. Migration non breaking `2026112?000001_account_presentation_links.sql` : `accounts.mirror_account_id`,
   `accounts.offset_account_id` (`BIGINT NULL`, sans FK, indexées), `CHECK` : différent de `id`, pas les deux. DDL
   seul ; `kesh_version_min_required` inchangé. P5 (audit, compteurs recomptés), P6 (sites positionnels), P7 (rien à
   trier), squash `test-schema` à jour.
2. `Account` et le dépôt lisent et écrivent les deux champs ; la paire s'écrit des deux côtés dans une transaction
   (poser, déplacer, retirer) ; un lien orphelin (compte cible supprimé) est lu comme absent.
3. Refus nommés (400) : cible d'une autre société, même côté (miroir : actif↔passif, produit↔charge ; paire : types
   opposés du même rapport), rôle de fonds propres d'un côté ou de l'autre, chaîne (la cible a un lien), soi-même,
   cible archivée.
4. API `PUT /api/v1/accounts/{id}` : `mirrorAccountId`, `offsetAccountId` (facultatifs, `null` retire) ;
   `AccountResponse` les rend ; `version` contrôlée ; audit avant/après.
5. MCP `kesh_plan_comptable` et export CSV des tables portent les deux champs.
6. Sauvegarde : une sauvegarde 0.19.0 s'importe (liens vides) ; aller-retour d'une sauvegarde neuve garde les liens.
7. Archivage d'un compte cible : selon Q10.

## Tâches

- [ ] Migration, audit, squash, `migrations_upgrade_path.rs`.
- [ ] Entité, dépôt, validation (une fonction), erreurs et i18n (4 locales).
- [ ] Route, audit, MCP, export ; tests de dépôt et d'API ; test d'import de sauvegarde.
- [ ] `docs/api-external.md`.

## Fichiers

`crates/kesh-db/migrations/`, `docs/migrations-idempotence-audit.md`, `crates/kesh-db/test-schema/`,
`kesh-db/src/entities/account.rs`, `repositories/accounts.rs`, `errors.rs`, `kesh-api/src/routes/accounts.rs`,
`kesh-api/src/errors.rs`, `exports/csv_tables.rs`, MCP, `kesh-i18n/locales/*/messages.ftl`. Conflit attendu avec
`fix/api-parent-compte`.

## Note de version

Un compte peut désigner un compte miroir ou un partenaire de paire (présentation seulement). Une migration, non
breaking.
