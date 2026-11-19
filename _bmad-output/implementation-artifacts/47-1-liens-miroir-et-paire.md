# Story 47-1 : liens de présentation — miroir et paire (modèle)

Status: draft

Epic : `epic-47-comptes-miroirs.md` (D1, D2, D3, D8, D10 ; revue de Guy du 2026-11-19 : Q9, Q10, D2, D10).

## Intention

En tant que comptable, je désigne sur un compte son **compte miroir** ou son **partenaire de paire**, pour que les
rapports le présentent sans écriture de reclassement.

## Critères d'acceptation

1. Migration non breaking, horodatée **après `20261118000002`** (Epic 49, 0.20.0 ; Guy, D10), `<horodatage>_account_presentation_links.sql` : `accounts.mirror_account_id`,
   `accounts.offset_account_id` (`BIGINT NULL`, sans FK, indexées), `CHECK` : différent de `id`, pas les deux. DDL
   seul ; `kesh_version_min_required` inchangé. P5 (audit, compteurs recomptés), P6 (sites positionnels), P7 (rien à
   trier), squash `test-schema` à jour.
2. `Account` et le dépôt lisent et écrivent les deux champs ; la paire s'écrit des deux côtés dans une transaction
   (poser, déplacer, retirer) ; un lien orphelin (compte cible supprimé) est lu comme absent.
3. Refus nommés (400) : cible d'une autre société, même côté (miroir : actif↔passif, produit↔charge ; paire : types
   opposés du même rapport), rôle de fonds propres d'un côté ou de l'autre, soi-même, cible archivée, et **chaîne** —
   la cible a elle-même un lien — par un **code nommé** (Guy, D2).
3 bis. Création d'une paire : la réponse porte un **avertissement** (CO 958c al. 1 ch. 7 : la paire compense une charge
   et un produit ; présentation brute pour les comptes annuels officiels) ; l'écran le montre avant d'enregistrer (Q9).
4. API `PUT /api/v1/accounts/{id}` : `mirrorAccountId`, `offsetAccountId` (facultatifs, `null` retire) ;
   `AccountResponse` les rend ; `version` contrôlée ; audit avant/après.
5. MCP `kesh_plan_comptable` et export CSV des tables portent les deux champs.
6. Sauvegarde : une sauvegarde 0.19.0 s'importe (liens vides) ; aller-retour d'une sauvegarde neuve garde les liens.
7. Archivage d'un compte vers lequel pointe un miroir ou une paire : **refusé**, message qui **nomme le compte lié**
   (Guy, Q10).
8. Documentation (§ ci-dessous) : `docs/api-external.md` (`mirrorAccountId`, `offsetAccountId`, refus et
   avertissement) ; manuel d'administration (migration, sauvegarde complète avant mise à jour, API et export CSV).

## Tâches

- [ ] Migration, audit, squash, `migrations_upgrade_path.rs`.
- [ ] Entité, dépôt, validation (une fonction), erreurs et i18n (4 locales).
- [ ] Route, audit, MCP, export ; tests de dépôt et d'API ; test d'import de sauvegarde.
- [ ] `docs/api-external.md`, manuel d'administration.

## Documentation (exigence de Guy, 2026-11-19)

- `docs/api-external.md` : champs `mirrorAccountId`, `offsetAccountId`, codes de refus, avertissement de la paire.
- Manuel d'administration (`docs/manual/fr/admin-manual.tex`) : la migration, la sauvegarde complète avant mise à jour, l'API et l'export.

Manuels FR (`docs/manual/fr/`, `.tex` seulement, PDF régénérés à la release) ; s'ils sont traduits d'ici la livraison, le mécanisme y figure aussi. **Le prompt de chaque passe de revue de cette story nomme ces manuels** et exige le contrôle du PDF (`CLAUDE.md` § *Le prompt d'une passe doit NOMMER le manuel*).

## Fichiers

`crates/kesh-db/migrations/`, `docs/migrations-idempotence-audit.md`, `crates/kesh-db/test-schema/`,
`kesh-db/src/entities/account.rs`, `repositories/accounts.rs`, `errors.rs`, `kesh-api/src/routes/accounts.rs`,
`kesh-api/src/errors.rs`, `exports/csv_tables.rs`, `docs/api-external.md`, `docs/manual/fr/admin-manual.tex`, MCP, `kesh-i18n/locales/*/messages.ftl`. Conflit attendu avec
`fix/api-parent-compte`.

## Note de version

⚠️ En tête : **sauvegarde complète avant la mise à jour** (une migration). Un compte peut désigner un compte miroir ou un
partenaire de paire (présentation seulement) ; la paire s'accompagne d'un avertissement (CO 958c). Migration non breaking.
