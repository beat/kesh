# Story 47-1b : liens de présentation — API, MCP, export et documentation

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md` — D1, D9 ; Q9, Q10 ; FR-47-1 (API), FR-47-2,
FR-47-6 (avertissement à la création), FR-47-7, FR-47-8 (`api-external.md`, manuel d'administration). Née du
découpage de l'esquisse 47-1 (2026-11-19). Dépend de la **47-1a** (modèle, dépôt, erreurs de dépôt, **colonnes de
l'export CSV** — déplacées en 47-1a à la validation P1 : la garde d'exhaustivité des colonnes lit le squash).

## Story

En tant que comptable ou intégrateur,
je veux poser, déplacer et retirer un miroir ou une paire par l'API des comptes, et les retrouver au plan comptable du
MCP et dans l'export CSV,
afin de gérer les liens comme tout autre attribut d'un compte, avec des refus qui disent ce qui manque.

## Critères d'acceptation

1. **`PUT /api/v1/accounts/{id}`** accepte `mirrorAccountId` et `offsetAccountId`, chacun à trois états (absent =
   inchangé, `null` = retiré, entier = posé ; désérialisation `present_or_null`, sur le patron de
   `fxGainAccountId`). **Table de traduction vers `PresentationLinkChange`** (47-1a AC-4), jugée **dans la route**,
   sans lecture préalable de l'état du compte (le « sans effet » se résout dans le dépôt, sous verrou) :

   | `mirrorAccountId` | `offsetAccountId` | changement |
   |---|---|---|
   | absent | absent | `Unchanged` |
   | `null` | absent | `ClearMirror` (sans effet si le compte n'a pas de miroir) |
   | absent | `null` | `ClearOffset` (sans effet si le compte n'a pas de paire) |
   | `null` | `null` | `ClearAll` |
   | entier M | absent ou `null` | `Mirror(M)` |
   | absent ou `null` | entier P | `Offset(P)` |
   | entier | entier | **400 `PRESENTATION_LINK_INVALID`**, `reason: "bothLinks"`, `field: "offsetAccountId"` (jugé dans la route, avant le dépôt) |

   **Un compte n'a qu'un lien** : poser une nature sur un compte qui tient l'autre **remplace** le lien (l'écran de la
   47-3a n'envoie que le nouveau champ) ; si c'était une paire, le partenaire est libéré et reçoit sa version et son
   entrée d'audit (47-1a AC-6). Reposer **le même lien** que le compte porte déjà
   est un no-op (47-1a AC-5) : un client qui renvoie à `PUT` le corps lu par `GET`, champs de lien compris, n'a ni
   refus, ni avertissement, ni changement de version. La `version` **du compte modifié** est contrôlée (409
   `OPTIMISTIC_LOCK_CONFLICT`) ; **poser, déplacer ou retirer une paire incrémente aussi la `version` du ou des
   partenaires touchés** (47-1a AC-6) : un client qui tient une copie du partenaire la **relit** avant de la modifier
   (dit à `api-external.md`, AC-9) ; test : `PUT` d'un partenaire avec sa version d'avant la pose → 409.
2. **Refus du dépôt mappés** (`crates/kesh-api/src/errors.rs`) : `PresentationLinkInvalid` → 400
   `PRESENTATION_LINK_INVALID`, `details: { field, reason[, linkedAccountNumber] }` ; `AccountPresentationLinked` →
   409 `ACCOUNT_PRESENTATION_LINKED`, `details: { reason, linkedAccounts: [{ id, number, name }] }`. Message traduit
   (quatre locales) qui **nomme** le compte lié — p. ex. fr-CH, archivage d'un compte cible : « Le compte 1040
   Avoirs à court terme est le compte miroir de 2270 AVS/AI/APG : retirez d'abord ce lien. » (Q10). ⚠️ **Les numéros
   et noms d'exemple écrits dans la documentation et les catalogues sont ceux d'un plan livré** (`kesh-core/assets/
   charts/*.json` : 2270 « AVS/AI/APG », 1040 « Avoirs à court terme ») : la garde G4-bis/G4-ter
   de `textes_coherents.rs` rougit sur un numéro inconnu ou un nom faux (l'exemple « 1179 Créances sociales » /
   « 2270 Dettes sociales » de la rédaction d'origine aurait rougi). Les messages des catalogues portent des
   **variables** (`{ $number }`, `{ $name }`), non des exemples.
   **Messages** (une clé par ligne, quatre locales, repli Rust **égal à la valeur fr-CH** — garde G9) : un message par
   raison de `PRESENTATION_LINK_INVALID` (`self`, `unknown`, `archived`, `equity`, `accountType` — variante miroir et
   variante paire —, `chain` — avec `{ $number }` du compte qui occupe le rôle —, `bothLinks`, `onCreate`) et un par
   raison de `ACCOUNT_PRESENTATION_LINKED` (`archive` — variantes cible de miroir et membre de paire —,
   `accountType`, `equityRole`), plus l'avertissement de l'AC-4 ; la liste des clés est écrite au Dev Agent Record
   et un test lit chaque raison de `PresentationLinkReason::ALL` + `bothLinks` + `onCreate` et exige un message non
   vide par locale.
3. **`AccountResponse`** rend `mirrorAccountId` et `offsetAccountId` (absents quand `null`, comme
   `fxGainAccountId` — le JSON d'un plan sans lien reste identique) ; `GET /api/v1/accounts` aussi.
4. **Avertissement de la paire (Q9)** : la réponse d'un `PUT` qui **crée** une paire (le compte n'en avait pas, ou
   changeait de partenaire) porte `warnings: [{ code: "PRESENTATION_OFFSET_COMPENSATES", message }]` — message :
   la paire compense une charge et un produit (ou un actif et un passif), ce que CO 958c al. 1 ch. 7 interdit dans les
   comptes annuels ; la présentation brute reste disponible pour les comptes officiels. Absent (`skip`) sinon.
   **Mécanisme** (aucun précédent `warnings` dans l'API des comptes) : `AccountResponse` gagne
   `warnings: Vec<AccountWarning { code, message }>` avec `skip_serializing_if = "Vec::is_empty"` ; `new(…)` le
   laisse vide, un constructeur `with_warnings` n'est appelé que par `update_account` ; les quatre autres routes et la
   liste ne le rendent donc jamais. La condition « crée une paire » compare le `offsetAccountId` du compte lu par
   `find_by_id_in_company` **avant** la mise à jour (déjà lu pour le contrôle IDOR) à celui du compte rendu :
   `après.is_some() && après != avant` ; lecture hors verrou, **assumée** (au pire un avertissement manquant ou en
   trop sur une concurrence : il ne bloque rien). Reposer la même paire, poser un miroir, retirer : aucun
   avertissement. Le message vient de `t(…)` (locale d'installation, comme tout message de l'API).
5. **Audit** : chaque compte modifié a son entrée `account.updated` avant/après (47-1a AC-6) ; `details.before` et
   `details.after` portent `mirrorAccountId` et `offsetAccountId` — **test par `GET /api/v1/audit-log` en session**
   (la consultation du journal est fermée aux clés d'API) : une pose de paire donne deux entrées, une par compte ;
   aucun libellé d'action neuf (`account.updated` est déjà à `audit_labels.rs:77`). L'attribution à la clé d'API
   reste celle des comptes aujourd'hui (`actor_type = 'user'`, #431 : dit au manuel d'administration, § Clés API) —
   **non traité ici**, constat écrit.
6. **MCP `kesh_plan_comptable`** : chaque compte porte `compte_miroir_id` et `partenaire_id` (clés toujours
   présentes, `null` sans lien, comme `devise` ; le nom d'une clé est sa forme de sortie, `snake_case`, comme
   `parent_id`) ; `mcp_contrat`, `mcp_gardes_lexicales` verts. ⚠️ **Le budget est dépassé à coup sûr, et la révision
   des plafonds est un changement de contrat MCP, non un changement additif** : l'élément mesure 334 octets
   (`devise` comprise, 37-1) pour une page par défaut de 38 comptes à ≈ 13 061 o (plafond de l'assertion : 13 312) et
   une page maximale de 68 à ≈ 23 081 o (plafond : 23 552) ; les deux clés ajoutent
   `,"compte_miroir_id":null,"partenaire_id":null` = 45 octets, soit **≈ 379 o par compte** : 38 comptes donneraient
   ≈ 14 770 o, 68 comptes ≈ 26 100 o. Nouveaux plafonds **estimés 34 / 61** (à mesurer, C-32-9-44 : écrits dans la
   fiche après la mesure, jamais devinés), mesurés sur une fixture où au moins un compte porte un lien (identifiant
   non `null`, plus long que `null` de quelques octets). **Sites à réviser, tous** : `LIMITE_DEFAUT` et
   `LIMITE_MAX_MCP` (`plan_comptable.rs`) ; le littéral du schéma d'entrée (`maximum`, `default` de `schema_entree`) ;
   la table `mesurees` de `bornes_attendues` (`mcp_outils_e2e.rs:183`, `(68, 38)`) ; l'en-tête de module de
   `plan_comptable.rs` (mesure citée) ; la description de l'outil (ajouter « compte miroir, partenaire de paire ») ;
   les tests de pagination qui comptent des pages (`kesh_plan_comptable_rend_la_devise`, `:2217`) ; le manuel
   d'administration (la phrase « `kesh_plan_comptable` 38 comptes par défaut et 68 au plus » de la section « Passer à
   la 0.19.0 » est un relevé daté, **laissé tel quel** ; la section de cette release la corrige). **Contrat** : un
   client MCP qui envoie `limit` entre le nouveau et l'ancien maximum reçoit désormais `arguments invalides` (un
   `limit` au-delà du maximum est refusé, `arguments_inconnus_refuses`) : à écrire aux Notes de version et au manuel
   (même précédent que la 37-1).
7. **Export CSV des tables** : les colonnes `mirror_account_id` et `offset_account_id` de `accounts.csv` sont posées
   **par la 47-1a** (AC-14 de la 47-1a : la garde `chaque_colonne_du_schema_est_exportee_ou_ecartee` de
   `csv_tables.rs`, qui lit le squash, rougirait sinon au gate complet de la 47-1a), **à la fin de la ligne, après
   `updated_at`** — ⚠️ **ce n'est pas la place des `fx_*`**, insérées avant `version` (api-external § multi-devises,
   « décalent les suivantes ») : en fin de ligne, un lecteur par position n'est pas décalé. **Ici** : un test
   d'intégration de l'**export global** (`GET /api/v1/exports/global.zip`, `accounts.csv` du ZIP) où un compte lié
   porte son identifiant et un compte sans lien deux cellules vides ; la documentation (AC-9) dit la place des colonnes.
8. **Création** (tranché à la validation P1) : `POST /api/v1/accounts` ne prend pas de lien (un compte se crée, puis
   se lie). `CreateAccountRequest` **ignore aujourd'hui tout champ inconnu** (pas de `deny_unknown_fields`) :
   l'activer rendrait un refus à tout client qui envoie un champ qu'il croit utile — rejeté. Un lien posé à la
   création serait sinon **ignoré en silence**, et l'intégrateur le croirait posé : la requête déclare donc
   `mirrorAccountId` et `offsetAccountId` (`present_or_null`, sur le patron de `UpdateAccountRequest`), et un
   **entier** → **400 `PRESENTATION_LINK_INVALID`**, `reason: "onCreate"`, `field` du champ en cause, avant la
   transaction ; `null` ou absent : accepté, sans effet. Test : `POST` avec un lien → 400, aucun compte créé.
9. **Documentation (exigence de Guy, 2026-11-19)** :
   - `docs/api-external.md` : nouvelle sous-section après « Comptabilité multi-devises » (`:1060`), sur son patron :
     champs `mirrorAccountId`, `offsetAccountId` (trois états, **table de l'AC-1**, no-op d'un lien reposé,
     **version des partenaires incrémentée : relire**), règles de validité, codes `PRESENTATION_LINK_INVALID` (liste
     close des raisons : celles de la 47-1a **plus `bothLinks` et `onCreate`**, et le `field`) et
     `ACCOUNT_PRESENTATION_LINKED`, `warnings` de la paire (code, message, quand), effet sur les rapports renvoyé à la
     47-2a, **colonnes de `accounts.csv` en fin de ligne**, `kesh_plan_comptable` (deux clés, **nouveaux plafonds**) ;
     le sens inverse (« qui pointe vers ce compte ») n'est **pas** rendu : il se déduit de `GET /accounts` (sources
     archivées comprises : `includeArchived=true`, les sources archivées étant admises) ; ligne et note de bas de
     tableau du plan comptable (`:226`) ;
   - manuel d'administration `docs/manual/fr/admin-manual.tex`. ⚠️ **Les emplacements de la rédaction d'origine
     étaient faux** : `:2230` est la section de la sauvegarde `.keshbackup`, non de l'export CSV, et `:3262` (*Clés
     API*) ne décrit aucun champ de compte. Les foyers réels, sur le patron de « Passer à la 0.19.0 » (`:2550`, ouverte
     par la story qui porte la migration, `% Story 43-1a, AC13`) : une **nouvelle sous-section « Passer à la
     <release de l'Epic 47> »** dans *Mise à jour de Kesh* (`:2497`) — titre au numéro de release connu **au tag**
     (on parle d'epics, `CLAUDE.md` § *Travailler à plusieurs développeurs* : titre provisoire « Epic 47 », renommé à
     l'intégration), qui dit **en tête la sauvegarde complète** (base, `.keshbackup`, dossier des documents), la
     migration (**total recompté** depuis `ls … | wc -l`), le **retour à la release précédente** (refus de la base
     migrée, retour par la sauvegarde d'avant la mise à jour), puis, dans « Pour les intégrations », les champs de
     l'API, les **colonnes de `accounts.csv` en fin de ligne** et la **révision des plafonds du MCP** ; et, dans
     *Compatibilité* de la sauvegarde (`:2302`), une phrase : une sauvegarde de la release précédente s'importe,
     liens vides. Le texte est écrit **en respectant `textes_coherents.rs`** (G4-bis/ter : numéros et noms de comptes
     d'un plan livré ; G19 : URL au tag ; G25 : jamais « projet » pour l'axe analytique) — exécuter ce fichier de
     tests. `.tex` seulement, PDF régénéré à la release ; manuels traduits : le mécanisme y figure aussi s'ils
     existent à la livraison (aujourd'hui `docs/manual/{de,en,it}/` ne portent qu'un `README.md`).
   - **Le prompt de chaque passe de revue nomme ces deux documents** et exige le contrôle du PDF compilé localement
     (`pdftotext … | tr '\n' ' '`) (`CLAUDE.md` § *Le prompt d'une passe doit NOMMER le manuel*).
10. **i18n** : les messages de l'AC-2 et l'avertissement de l'AC-4 dans les quatre locales
    (`crates/kesh-i18n/locales/*/messages.ftl`), repli Rust égal à la valeur fr-CH (G9), mêmes clés dans les quatre
    catalogues ; `sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts` **inchangé** (aucun site frontend en
    47-1b : constat écrit, recompté par `grep` à la clôture).

## Tâches / sous-tâches

- [ ] **T0** : relire `CreateAccountRequest` / `UpdateAccountRequest` après reprise de `main` (`fix/api-parent-compte`
  ajoute `parentId` au même corps) ; trancher AC-8.
- [ ] **T1 — route** (AC 1, 3, 4, 8) : champs, `presentation_link_for_update` (comme `own_fx_for_update` ; table de
  l'AC-1, `bothLinks` jugé avant le dépôt), `presentation_link_for_create` (`onCreate`), réponse, `warnings`
  (`with_warnings`).
- [ ] **T2 — erreurs** (AC 2, 10) : mapping, messages, clés i18n.
- [ ] **T3 — MCP** (AC 6) : `Compte`, mesure des tailles.
- [ ] **T4 — export** (AC 7) : test du ZIP global ; les colonnes elles-mêmes sont de la 47-1a.
- [ ] **T5 — tests** : `crates/kesh-api/tests/accounts_e2e.rs` (chaque refus, trois états, `warnings`, audit, RBAC
  comme `update_account`), `accounts_designated_archive_e2e.rs` (archivage refusé nommant le compte lié),
  `mcp_outils_e2e.rs` (plafonds révisés, fixture avec lien), test de l'export global, `textes_coherents.rs`.
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

- **Contrat additif, et ses deux exceptions déclarées** : un client qui ignore les champs n'efface rien (absent =
  inchangé) ; un plan sans lien rend le même JSON qu'aujourd'hui (champs omis) ; `accounts.csv` gagne deux colonnes
  **en fin de ligne** (aucun décalage). **Exceptions** : (1) les plafonds du MCP baissent (AC-6) — un `limit` entre
  l'ancien et le nouveau maximum est refusé ; (2) un `PUT` qui renvoie le corps lu par `GET` reste sans effet (no-op
  d'un lien reposé). `POST` ignore toujours les champs inconnus, **sauf** un lien entier (400 `onCreate`).
- Les raisons `bothLinks` et `onCreate` sont **posées par la route**, mais **portées par la même énumération** que
  celles du dépôt (`PresentationLinkReason`, 47-1a AC-5) — seul moyen de construire la réponse par
  `AppError::Database(DbError::PresentationLinkInvalid { … })`, comme `own_fx_missing` pour `FX_ACCOUNT_INVALID` ; le
  dépôt, lui, ne les produit jamais (test). La liste close documentée est donc celle de l'énumération entière.
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

## Change Log

- 2026-11-19 — validation P1 (Sonnet 5.5) : 0 C / 2 H / 9 M / 4 L, remédiés (remplacement d'un lien par l'autre nature : aligné sur la 47-3a, qui n'envoie que le nouveau champ). H : colonnes CSV — la garde
  `chaque_colonne_du_schema_est_exportee_ou_ecartee` lit le squash que la 47-1a met à jour, donc le gate complet de la
  47-1a rougissait avant la 47-1b (colonnes déplacées en 47-1a AC-14) ; budget MCP dépassé à coup sûr (334 → ≈ 379
  o par compte, plafonds estimés 34 / 61) et révision des plafonds = changement de contrat MCP (`limit` refusé au-delà
  du nouveau maximum), sites à réviser listés. M : exemple « 1179 Créances sociales / 2270 Dettes sociales »
  inexistant dans les plans livrés (G4-bis/ter) ; `null` « sans effet » sans nature dans `PresentationLinkChange`
  (relecture hors verrou) — table de traduction et variantes `Clear*` ; `bothLinks` et `onCreate` hors de
  l'énumération close (`own_fx_missing` pour patron) et `field` indéfini ; AC-8 laissé à T0, `deny_unknown_fields`
  rejeté, `onCreate` ; mécanisme de `warnings` et détection de « crée une paire » ; corps de `GET` renvoyé en `PUT`
  (no-op) ; version des partenaires incrémentée, non documentée ; emplacements du manuel d'administration faux
  (`:2230` = `.keshbackup`, `:3262` sans champs de compte) ; messages i18n non énumérés (G9). L : « comme `fx_*` »
  faux (insérés avant `version`) ; audit « lisible » non testable et attribution à la clé (#431) ; table de
  `api-external.md` `:226` et sens inverse déduit de la liste ; gardes `textes_coherents.rs` nommées. Non exercés :
  #1346 et #1351 (aucune issue lisible sans `gh`). Rapport :
  `kesh-gate-logs/epic-47-validation/47-1b-liens-api-mcp-export-P1.md`.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
