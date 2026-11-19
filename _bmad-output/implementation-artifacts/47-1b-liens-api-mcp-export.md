# Story 47-1b : liens de présentation — API, MCP, export et documentation

Status: ready-for-dev

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
2. **Refus du dépôt mappés** (`crates/kesh-api/src/errors.rs`) — **réalisé par la 47-1a AC-15** (validation P1 de
   la 47-1a : le sous-match exhaustif de `DbError` y impose le mapping dès que les variantes existent) ; cette story
   l'exerce de bout en bout par les routes (T5), contrat inchangé : `PresentationLinkInvalid` → 400
   `PRESENTATION_LINK_INVALID`, `details: { field, reason[, linkedAccountNumber] }` ; `AccountPresentationLinked` →
   409 `ACCOUNT_PRESENTATION_LINKED`, `details: { reason, linkedAccounts: [{ id, number, name, active }] }` (`active` : une source archivée garde son
   lien et bloque l'archivage de sa cible, 47-1a AC-7 ; l'écran de la 47-3a AC-5 en tire le remède « réactivez-le, retirez le
   lien »). Message traduit
   (quatre locales) qui **nomme** le compte lié — et, s'il est archivé, dit de le réactiver d'abord — p. ex. fr-CH, archivage d'un compte cible : « Le compte 1040
   Avoirs à court terme est le compte miroir de 2270 AVS/AI/APG : retirez d'abord ce lien. » (Q10). ⚠️ **Les numéros
   et noms d'exemple écrits dans la documentation et les catalogues sont ceux d'un plan livré** (`kesh-core/assets/
   charts/*.json` : 2270 « AVS/AI/APG », 1040 « Avoirs à court terme ») : la garde G4-bis/G4-ter
   de `textes_coherents.rs` rougit sur un numéro inconnu ou un nom faux (l'exemple « 1179 Créances sociales » /
   « 2270 Dettes sociales » de la rédaction d'origine aurait rougi). Les messages des catalogues portent des
   **variables** (`{ $number }`, `{ $name }`), non des exemples.
   **Messages — contrat que réalise la 47-1a** (AC-15 de la 47-1a ; une clé par ligne, quatre locales, repli Rust
   **égal à la valeur fr-CH** — garde G9) : un message par raison de `PRESENTATION_LINK_INVALID` (`self`, `unknown`,
   `archived`, `equity`, `accountType` — variante miroir et variante paire —, `chain` — avec `{ $number }` du compte
   qui occupe le rôle —, **`bothLinks` et `onCreate` compris** : ils appartiennent à `PresentationLinkReason` (47-1a
   AC-5) et la fonction de réponse de la 47-1a, sur le patron de `fx_account_invalid_response` (`errors.rs:1881`),
   fait un `match` exhaustif sur la raison — leurs bras, donc leurs clés, naissent avec elle) et un par raison de
   `ACCOUNT_PRESENTATION_LINKED` (`archive` — variantes cible de miroir et membre de paire —, `accountType`,
   `equityRole`) ; la liste des clés est écrite au Dev Agent Record **de la 47-1a**, avec le test qui lit chaque
   raison de `PresentationLinkReason::ALL` (`bothLinks` et `onCreate` y sont déjà : les ajouter en plus les
   compterait deux fois) et exige un message non vide par locale. **Ici** : seule la clé de l'avertissement de l'AC-4
   s'ajoute ; T0 constate que la 47-1a a bien posé les clés de `bothLinks` et `onCreate` (sinon, elles s'écrivent ici,
   constat au Change Log), et T5 exerce chaque message par les routes.
3. **`AccountResponse`** rend `mirrorAccountId` et `offsetAccountId` (absents quand `null`, comme
   `fxGainAccountId` — le JSON d'un plan sans lien reste identique) ; `GET /api/v1/accounts` aussi.
4. **Avertissement de la paire (Q9)** : la réponse d'un `PUT` qui **crée** une paire (le compte n'en avait pas, ou
   changeait de partenaire) porte `warnings: [{ code: "PRESENTATION_OFFSET_COMPENSATES", message }]` — message :
   la paire compense une charge et un produit (ou un actif et un passif), ce que CO 958c al. 1 ch. 7 interdit dans les
   comptes annuels ; la présentation brute reste disponible pour les comptes officiels. Absent (`skip`) sinon.
   **Mécanisme** (aucun précédent `warnings` dans l'API des comptes) : `AccountResponse` gagne
   `warnings: Vec<AccountWarning { code, message }>` avec `skip_serializing_if = "Vec::is_empty"` ; `new(…)` le
   laisse vide, un constructeur `with_warnings` n'est appelé que par `update_account` ; les trois autres routes qui
   rendent un compte (création, archivage, réactivation) et la liste ne le rendent donc jamais. La condition « crée une paire » compare le `offsetAccountId` du compte lu par
   `find_by_id_in_company` **avant** la mise à jour (déjà lu pour le contrôle IDOR) à celui du compte rendu :
   `après.is_some() && après != avant` ; lecture hors verrou, **assumée** (au pire un avertissement manquant ou en
   trop sur une concurrence : il ne bloque rien). Reposer la même paire, poser un miroir, retirer : aucun
   avertissement. Le message vient de `t(…)` (locale d'installation, comme tout message de l'API).
   **Clé i18n unique** : `accounts-presentation-offset-warning`, **sans variable**, un seul texte pour la paire
   charge/produit et la paire actif/passif (D3), quatre locales, repli Rust égal à la valeur fr-CH (G9). Texte fr-CH
   proposé : « Une paire présente le net de deux comptes — une charge et un produit, ou un actif et un passif —, ce
   que l'art. 958c al. 1 ch. 7 CO interdit dans les comptes annuels. Kesh présentera le net ; la présentation brute
   reste disponible pour les comptes officiels. » **Le nom de la clé est un contrat** : l'écran de la 47-3a (AC-3)
   affiche **cette clé même**, lue dans le catalogue servi par `GET /api/v1/i18n/messages` (`lib.rs:1282`), avant
   d'enregistrer — il ne la lit pas dans la réponse `warnings`, et n'a pas de texte à lui ; le Dev Agent Record écrit
   le nom livré, que la 47-3a relit à son T0. Préfixe `accounts-` comme `accounts-retype-warning`, l'autre
   avertissement du plan comptable.
5. **Audit** : chaque compte modifié a son entrée `account.updated` avant/après (47-1a AC-6) ; `details.before` et
   `details.after` portent `mirrorAccountId` et `offsetAccountId` — **test par `GET /api/v1/audit-log` en session**
   (la consultation du journal est fermée aux clés d'API) : une pose de paire donne deux entrées, une par compte ;
   un déplacement de paire (A passe de D à B) en donne trois (A, B, D) ; aucun libellé d'action neuf
   (`account.updated` est déjà à `audit_labels.rs:77`). L'attribution à la clé d'API
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
   le témoin de refus au maximum `plafond_au_maximum` (fixture large : la page maximale doit encore dépasser, sinon
   l'élargir) ; le manuel
   d'administration (la phrase « `kesh_plan_comptable` 38 comptes par défaut et 68 au plus » de la section « Passer à
   la 0.19.0 » est un relevé daté, **laissé tel quel** ; la section de cette release la corrige). **Contrat** : un
   client MCP qui envoie `limit` entre le nouveau et l'ancien maximum reçoit désormais `arguments invalides` (un
   `limit` au-delà du maximum est refusé, `arguments_inconnus_refuses`) : à écrire aux Notes de version et au manuel
   (même précédent que la 37-1). **Valeurs** : un test sur le patron de `kesh_plan_comptable_rend_la_devise`
   (`mcp_outils_e2e.rs:2217`, pages parcourues par `LIMITE_MAX_MCP`, constante qui suit d'elle-même) pose un miroir
   et une paire, et exige `compte_miroir_id` = id de la cible sur la source (`null` sur la cible), `partenaire_id` =
   l'id de l'autre membre **sur chacun des deux**, `null` ailleurs, clés présentes sur chaque compte — sans lui,
   inverser les deux clés ne rougirait pas (`mcp_contrat` ne contrôle que la forme). ⚠️ `fixture_tailles` est
   partagée avec la 47-2a (qui mesure `kesh_bilan` avec un miroir à plusieurs sources) : le second arrivé
   **re-mesure** les outils touchés après la fusion. **Documentation du MCP** : au manuel d'administration — tableau
   des outils (`admin-manual.tex:3533`, « le plan comptable, par tranches, avec la devise de chaque compte » : y
   ajouter le compte miroir et le partenaire) et « Pour les intégrations » de la section de la release (AC-9) ;
   `docs/api-external.md` ne documente pas le MCP (aucune occurrence de `kesh_plan_comptable`).
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
   **entier** → **400 `PRESENTATION_LINK_INVALID`**, `reason: "onCreate"`, `field` du champ en cause
   (`mirrorAccountId` si les deux sont entiers), avant la transaction ; `null` ou absent : accepté, sans effet. Test : `POST` avec un lien → 400, aucun compte créé.
9. **Documentation (exigence de Guy, 2026-11-19)** :
   - `docs/api-external.md` : nouvelle sous-section après « Comptabilité multi-devises » (`:1060`), sur son patron :
     champs `mirrorAccountId`, `offsetAccountId` (trois états, **table de l'AC-1**, no-op d'un lien reposé,
     **version des partenaires incrémentée : relire**), règles de validité, codes `PRESENTATION_LINK_INVALID` (liste
     close des raisons : celles de la 47-1a **plus `bothLinks` et `onCreate`**, et le `field`) et
     `ACCOUNT_PRESENTATION_LINKED`, `warnings` de la paire (code, message, quand), effet sur les rapports renvoyé à la
     47-2a, **colonnes de `accounts.csv` en fin de ligne** ; un lien **hors règle** (posé par l'import d'une
     sauvegarde ou par SQL, 47-1a AC-8) est **rendu tel quel** par `GET /accounts` et l'export (colonnes brutes), mais
     **ignoré par les rapports** (lecteur de la 47-1a) : le retirer par `PUT` (`null`) le corrige ; le MCP n'est pas
     dans ce guide (AC-6) ; le sens inverse (« qui pointe vers ce compte ») n'est **pas** rendu : il se déduit de `GET /accounts` (sources
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
     l'API, les **colonnes de `accounts.csv` en fin de ligne** et la **révision des plafonds du MCP** (deux clés,
     nouveaux plafonds, `limit` au-delà refusé) ; le **tableau des outils MCP** (`:3533`, AC-6) ; et, dans
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
- [ ] **T2 — erreurs et i18n** (AC 2, 4, 10) : mapping et messages des deux refus posés par la 47-1a (AC-15), clés
  de `bothLinks` et `onCreate` comprises (constat de T0) ; ici, la seule clé
  `accounts-presentation-offset-warning` (quatre locales, repli égal à fr-CH), nom écrit au Dev Agent Record.
- [ ] **T3 — MCP** (AC 6) : `Compte`, mesure des tailles, test des valeurs des deux clés, `plafond_au_maximum`.
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
- Erreurs : `crates/kesh-api/src/errors.rs:1881` (`fx_account_invalid_response`, `match` exhaustif sur la raison),
  `:4428` (`AccountDesignatedInSettings` → 409, `details`), `:4882`
  (`FxAccountInvalid` → `fx_account_invalid_response`, patron à reprendre).
- MCP : `crates/kesh-api/src/mcp/outils/plan_comptable.rs:1-58` (budget de tailles, `LIMITE_DEFAUT` 38,
  `LIMITE_MAX_MCP` 68, `Compte`).
- Export : `crates/kesh-api/src/exports/csv_tables.rs:287-330` (`serialize_accounts_csv`, en-tête `:306`).
- Documentation : `docs/api-external.md:1060` (section multi-devises, patron d'un attribut de compte à trois états) ;
  `docs/manual/fr/admin-manual.tex:2302` (*Compatibilité*), `:2497` (*Mise à jour de Kesh*), `:2550` (« Passer à la
  0.19.0 », patron), `:2569` (« Pour les intégrations », relevé daté des plafonds), `:3533` (tableau des outils MCP).
  (`:2230` et `:3262`, cités par la rédaction d'origine, ne sont pas des foyers : AC-9.)

### Points d'attention

- **Contrat additif, et ses deux exceptions déclarées** : un client qui ignore les champs n'efface rien (absent =
  inchangé) ; un plan sans lien rend le même JSON qu'aujourd'hui (champs omis) ; `accounts.csv` gagne deux colonnes
  **en fin de ligne** (aucun décalage) ; un `PUT` qui renvoie le corps lu par `GET` reste sans effet (no-op d'un lien
  reposé). **Exceptions** : (1) les plafonds du MCP baissent (AC-6) — un `limit` entre le nouveau et l'ancien maximum
  est refusé ; (2) `POST` ignore toujours les champs inconnus, **sauf** un lien entier (400 `onCreate`, là où il était
  ignoré) ; (3) poser ou retirer une paire incrémente la `version` du partenaire : une copie détenue rend 409.
- Les raisons `bothLinks` et `onCreate` sont **posées par la route**, mais **portées par la même énumération** que
  celles du dépôt (`PresentationLinkReason`, 47-1a AC-5) — seul moyen de construire la réponse par
  `AppError::Database(DbError::PresentationLinkInvalid { … })`, comme `own_fx_missing` pour `FX_ACCOUNT_INVALID` ; le
  dépôt, lui, ne les produit jamais (test). La liste close documentée est donc celle de l'énumération entière.
- L'écran (47-3a AC-3) montre l'avertissement **avant** d'enregistrer, par **la clé même** du message `warnings`
  (`accounts-presentation-offset-warning`, AC-4), lue dans le catalogue — il n'a pas de texte à lui et n'affiche pas
  la réponse `warnings`, qui sert les intégrations et le contrôle E2E.

### Stratégie de tests

API de bout en bout (Rust, `reqwest`), MCP (`mcp_outils_e2e`, `mcp_contrat`, tailles), export ; mutations : retirer
`warnings`, inverser `null`/absent, omettre une colonne CSV — chacune doit rougir.

### Risques

Conflit avec `fix/api-parent-compte` (même corps de requête, mêmes mappings) ; dépassement du budget MCP ;
`fixture_tailles` et `bornes_des_schemas` partagés avec la 47-2a (re-mesure par le second arrivé, AC-6).

### References

- Fiche 47-1a ; `CLAUDE.md` § *Le prompt d'une passe doit NOMMER le manuel* ; `docs/api-external.md`.
- Registre de la dette du fichier d'epic (§ *Registre de la dette de l'epic*) : à la clôture de la boucle de revue,
  tout finding non remédié y est consigné, une ligne par problème ; la 47-5 en part (« on trace toujours »).

## Notes de version

Un compte peut désigner un **compte miroir** ou un **partenaire de paire** (présentation seulement, aucune écriture) :
API des comptes (`mirrorAccountId`, `offsetAccountId`), plan comptable du MCP, export CSV. La création d'une paire
s'accompagne d'un avertissement (CO 958c al. 1 ch. 7). Archiver un compte cible d'un miroir ou membre d'une paire est
refusé, le compte lié nommé. **Pour les intégrations** : `accounts.csv` gagne deux colonnes **en fin de ligne** ;
poser ou retirer une paire incrémente la `version` du partenaire ; `POST /accounts` refuse un lien (400
`PRESENTATION_LINK_INVALID`, `onCreate`) au lieu de l'ignorer ; ⚠️ le **serveur MCP réduit ses pages** —
`kesh_plan_comptable` <défaut> comptes par défaut et <maximum> au plus (valeurs mesurées au développement, estimées
34 / 61), un `limit` au-delà est refusé.

## Change Log

- 2026-11-19 — validation P3 ciblée (Sonnet 5.5) sur la remédiation de la P2 : 0 C / 0 H / 0 M / 1 L, remédié. Sites
  relus à `origin/main` `fd04099f` sans écart : `errors.rs:1881` (`fx_account_invalid_response`), `admin-manual.tex:3533`
  (tableau des outils) et `:2302`/`:2497`/`:2550`/`:2569`, `lib.rs:1282` et `routes/i18n.rs` (`all_messages` sert tout
  le catalogue, la clé de l'avertissement y sera), `mcp_outils_e2e.rs:183` (`(68, 38)`), `:1439` (`plafond_au_maximum`),
  `:2217`, `fixture_tailles :792`, `audit_labels.rs:77`, `api-external.md:226` et `:1060` ; arithmétique des 45 / 379
  octets et des plafonds 34 / 61 refaite (13 255 et 23 488 o, sous 13 312 et 23 552). Cohérence : 47-1a AC-15 (clés de
  `bothLinks` et `onCreate`, test sur `ALL`, noms au Dev Agent Record, seule clé d'avertissement ici) et 47-3a AC-3 (clé
  lue au catalogue, `isNewOffsetPair` = définition de l'AC-4) alignées. L1 : la 47-3a AC-3 disait encore que la 47-1b
  contredisait la clé unique → phrase corrigée dans la 47-3a (une phrase, propagation). Rapport :
  `kesh-gate-logs/epic-47-validation/47-1b-liens-api-mcp-export-P3.md`.

- 2026-11-19 — validation P2 (Opus 5.5) : 0 C / 0 H / 4 M / 4 L, remédiés. M1 : doublon et trou avec la 47-1a sur
  les messages — l'AC-2 listait comme travail de cette story les messages que la 47-1a AC-15 pose, T2 gardait ici
  `bothLinks` et `onCreate` alors que la fonction de réponse exhaustive de la 47-1a les exige, et le test comptait
  `bothLinks` et `onCreate` deux fois (déjà dans `ALL`) → AC-2 « contrat que réalise la 47-1a », T2, constat en T0.
  M2 : clé de l'avertissement de paire non nommée alors que la 47-3a affiche cette clé, et Notes qui disaient
  l'écran « depuis ses propres textes » (relevé de la P2 de la 47-3a, transmis par l'orchestrateur) → AC-4 (clé
  `accounts-presentation-offset-warning`, texte générique charge/produit et actif/passif, sans variable), Points
  d'attention. M3 : MCP documenté dans `api-external.md`, qui ne le décrit pas, et tableau des outils du manuel
  d'administration (`:3533`) omis → AC-6, AC-9, Dev Notes. M4 : aucun test des **valeurs** de `compte_miroir_id` /
  `partenaire_id` (deux clés inversées passaient), et Notes de version sans le changement de contrat que l'AC-6 dit
  d'y écrire (CHANGELOG assemblé depuis elles) → AC-6, T3, Notes. L : « quatre autres routes » (trois : création,
  archivage, réactivation ; pas de `GET` unitaire) ; `kesh_plan_comptable_rend_la_devise` donné pour un site à
  réviser alors qu'il lit la constante, `plafond_au_maximum` omis ; lien hors règle rendu brut par l'API et l'export
  mais ignoré des rapports, non dit ; « exception (2) » qui n'en était pas une, version du partenaire non déclarée,
  audit d'un déplacement (trois entrées), `field` de `onCreate` à deux entiers, `fixture_tailles` partagée avec la
  47-2a. Signalé, non modifié (autre agent) : la 47-1a AC-15 et ses *Project Structure Notes* devraient dire en
  toutes lettres que les clés de `bothLinks` et `onCreate` y naissent et que seule la clé de l'avertissement reste à
  la 47-1b. Rapport : `kesh-gate-logs/epic-47-validation/47-1b-liens-api-mcp-export-P2.md`.

- 2026-11-19 — validation P1 de la **47-3a** (Opus 5.5), effet sur cette fiche : AC-2, `linkedAccounts[].active` et
  message d'une source archivée (remède de l'archivage refusé, 47-3a AC-5).
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
