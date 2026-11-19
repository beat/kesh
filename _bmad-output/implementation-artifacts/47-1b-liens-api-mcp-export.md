# Story 47-1b : liens de présentation — API, MCP, export et documentation

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md` — D1, D9 ; Q9, Q10 ; FR-47-1 (API), FR-47-2,
FR-47-6 (avertissement à la création), FR-47-7, FR-47-8 (`api-external.md`, manuel d'administration). Née du
découpage de l'esquisse 47-1 (2026-11-19). Dépend de la **47-1a** (modèle, dépôt, erreurs de dépôt).

## Story

En tant que comptable ou intégrateur,
je veux poser, déplacer et retirer un miroir ou une paire par l'API des comptes, et les retrouver au plan comptable du
MCP et dans l'export CSV,
afin de gérer les liens comme tout autre attribut d'un compte, avec des refus qui disent ce qui manque.

## Critères d'acceptation

1. **`PUT /api/v1/accounts/{id}`** accepte `mirrorAccountId` et `offsetAccountId`, chacun à trois états (absent =
   inchangé, `null` = retiré, entier = posé ; désérialisation `present_or_null`). Les deux à un entier dans la même
   requête → **400 `PRESENTATION_LINK_INVALID`**, `reason: "bothLinks"` (raison propre à l'API, jugée avant le
   dépôt). `null` sur le champ de l'autre nature que le lien existant est sans effet (pas un refus). La `version` est
   contrôlée (409 `OPTIMISTIC_LOCK_CONFLICT`).
2. **Refus du dépôt mappés** (`crates/kesh-api/src/errors.rs`) : `PresentationLinkInvalid` → 400
   `PRESENTATION_LINK_INVALID`, `details: { field, reason[, linkedAccountNumber] }` ; `AccountPresentationLinked` →
   409 `ACCOUNT_PRESENTATION_LINKED`, `details: { reason, linkedAccounts: [{ id, number, name }] }`. Message traduit
   (quatre locales) qui **nomme** le compte lié — p. ex. fr-CH : « Le compte 1179 Créances sociales est le compte
   miroir de 2270 Dettes sociales : retirez d'abord ce lien. » (Q10).
3. **`AccountResponse`** rend `mirrorAccountId` et `offsetAccountId` (absents quand `null`, comme
   `fxGainAccountId` — le JSON d'un plan sans lien reste identique) ; `GET /api/v1/accounts` aussi.
4. **Avertissement de la paire (Q9)** : la réponse d'un `PUT` qui **crée** une paire (le compte n'en avait pas, ou
   changeait de partenaire) porte `warnings: [{ code: "PRESENTATION_OFFSET_COMPENSATES", message }]` — message :
   la paire compense une charge et un produit (ou un actif et un passif), ce que CO 958c al. 1 ch. 7 interdit dans les
   comptes annuels ; la présentation brute reste disponible pour les comptes officiels. Absent (`skip`) sinon.
5. **Audit** : chaque compte modifié a son entrée `account.updated` avant/après (47-1a AC-6) ; le journal d'audit les
   affiche avec les deux champs lisibles ; aucun libellé d'action neuf (sinon inscrit à
   `crates/kesh-api/src/audit_labels.rs` et traduit).
6. **MCP `kesh_plan_comptable`** : chaque compte porte `compte_miroir_id` et `partenaire_id` (clés toujours
   présentes, `null` sans lien, comme `devise`) ; `mcp_contrat`, `mcp_gardes_lexicales` verts ; **tailles
   re-mesurées** et, si le budget est dépassé, `LIMITE_DEFAUT` / `LIMITE_MAX_MCP` révisés comme à la 37-1 (C-32-9-44).
7. **Export CSV des tables** : `serialize_accounts_csv` ajoute les colonnes `mirror_account_id` et
   `offset_account_id` (en fin de ligne d'en-tête, comme `fx_*`) ; test d'en-tête et d'une ligne liée.
8. **Création** : `POST /api/v1/accounts` ne prend pas de lien (un compte se crée, puis se lie) ; un corps qui en porte
   un est refusé en 400 (`deny_unknown_fields` ou contrôle explicite, selon le patron de la route) — à trancher en
   T0 par lecture de `CreateAccountRequest`.
9. **Documentation (exigence de Guy, 2026-11-19)** :
   - `docs/api-external.md` : champs `mirrorAccountId`, `offsetAccountId` (trois états), règles de validité, codes
     `PRESENTATION_LINK_INVALID` (liste close des raisons) et `ACCOUNT_PRESENTATION_LINKED`, `warnings` de la paire,
     effet sur les rapports renvoyé à la 47-2a ;
   - manuel d'administration `docs/manual/fr/admin-manual.tex` : à la section *Mise à jour de Kesh* (`:2497`), la
     migration de la release et la **sauvegarde complète avant la mise à jour** ; aux sections de l'API (*Clés API
     (PAT)*, `:3262`) et de l'export (`:2230`), les deux champs et les colonnes CSV. `.tex` seulement, PDF régénéré à
     la release ; manuels traduits : le mécanisme y figure aussi s'ils existent à la livraison.
   - **Le prompt de chaque passe de revue nomme ces deux documents** et exige le contrôle du PDF compilé localement
     (`pdftotext … | tr '\n' ' '`) (`CLAUDE.md` § *Le prompt d'une passe doit NOMMER le manuel*).
10. **i18n** : messages d'erreur et d'avertissement dans les quatre locales (`crates/kesh-i18n/locales/*/messages.ftl`),
    `sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts` **recompté** si des sites frontend s'y ajoutent
    (sinon inchangé, constat écrit).

## Tâches / sous-tâches

- [ ] **T0** : relire `CreateAccountRequest` / `UpdateAccountRequest` après reprise de `main` (`fix/api-parent-compte`
  ajoute `parentId` au même corps) ; trancher AC-8.
- [ ] **T1 — route** (AC 1, 3, 4, 8) : champs, `presentation_link_for_update` (comme `own_fx_for_update`), réponse,
  `warnings`.
- [ ] **T2 — erreurs** (AC 2, 10) : mapping, messages, clés i18n.
- [ ] **T3 — MCP** (AC 6) : `Compte`, mesure des tailles.
- [ ] **T4 — export** (AC 7).
- [ ] **T5 — tests** : `crates/kesh-api/tests/accounts_e2e.rs` (chaque refus, trois états, `warnings`, audit, RBAC
  comme `update_account`), `accounts_designated_archive_e2e.rs` (archivage refusé nommant le compte lié),
  `mcp_outils_e2e.rs`, test de l'export.
- [ ] **T6 — documentation** (AC 9) : `api-external.md`, manuel d'administration, compilation locale du PDF (non
  commité), contrôle par `pdftotext`.
- [ ] **T7 — gate complet** et E2E complet au dernier commit de code (aucun écran : smoke + suite locale).

## Dev Notes

### Code lu (à `origin/main` `fd04099f`)

- Route : `crates/kesh-api/src/routes/accounts.rs:147` (`present_or_null`), `:164-216` (`UpdateAccountRequest`,
  `fxGainAccountId` à trois états), `:244-305` (`AccountResponse`, `skip_serializing_if`), `:433-523`
  (`update_account`, `own_fx_for_update`), `:524` (`archive_account`) ; routage `crates/kesh-api/src/lib.rs:580-595`.
- Erreurs : `crates/kesh-api/src/errors.rs:4428` (`AccountDesignatedInSettings` → 409, `details`), `:4882`
  (`FxAccountInvalid` → `fx_account_invalid_response`, patron à reprendre).
- MCP : `crates/kesh-api/src/mcp/outils/plan_comptable.rs:1-58` (budget de tailles, `LIMITE_DEFAUT` 38,
  `LIMITE_MAX_MCP` 68, `Compte`).
- Export : `crates/kesh-api/src/exports/csv_tables.rs:287-330` (`serialize_accounts_csv`, en-tête `:306`).
- Documentation : `docs/api-external.md:1060` (section multi-devises, patron d'un attribut de compte à trois états) ;
  `docs/manual/fr/admin-manual.tex:2230`, `:2497`, `:3262`.

### Points d'attention

- **Contrat additif** : un client qui ignore les champs n'efface rien (absent = inchangé) ; un plan sans lien rend le
  même JSON qu'aujourd'hui (champs omis), la même sortie MCP aux deux clés `null` près (taille re-mesurée).
- La raison `bothLinks` n'appartient qu'à l'API ; la liste close documentée = celle du dépôt + `bothLinks`.
- L'écran (47-3a) montre l'avertissement **avant** d'enregistrer, depuis ses propres textes ; la réponse `warnings`
  sert les intégrations et le contrôle E2E.

### Stratégie de tests

API de bout en bout (Rust, `reqwest`), MCP (`mcp_outils_e2e`, `mcp_contrat`, tailles), export ; mutations : retirer
`warnings`, inverser `null`/absent, omettre une colonne CSV — chacune doit rougir.

### Risques

Conflit avec `fix/api-parent-compte` (même corps de requête, mêmes mappings) ; dépassement du budget MCP.

### References

- Fiche 47-1a ; `CLAUDE.md` § *Le prompt d'une passe doit NOMMER le manuel* ; `docs/api-external.md`.
- Registre de la dette du fichier d'epic (§ *Registre de la dette de l'epic*) : à la clôture de la boucle de revue,
  tout finding non remédié y est consigné, une ligne par problème ; la 47-5 en part (« on trace toujours »).

## Notes de version

Un compte peut désigner un **compte miroir** ou un **partenaire de paire** (présentation seulement, aucune écriture) :
API des comptes (`mirrorAccountId`, `offsetAccountId`), plan comptable du MCP, export CSV. La création d'une paire
s'accompagne d'un avertissement (CO 958c al. 1 ch. 7). Archiver un compte cible d'un miroir ou membre d'une paire est
refusé, le compte lié nommé.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
