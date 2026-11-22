# Story 47-2 : compte miroir — API des comptes et écran du plan comptable

Status: done

Epic : `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md` — § *Décision du 2026-11-19 : miroir seul*,
règles R2 à R4, R7 ; Q10 ; FR-47-1, FR-47-2, FR-47-5 (manuels). **Refonte du 2026-11-19 (miroir seul, KISS)** : fusion
de l'ancienne 47-3a (écran du plan comptable, fiche renommée) et de la partie « API des comptes » de l'ancienne 47-1b
(désormais `superseded`). Retirés : la section à trois boutons radio, la paire et son avertissement CO 958c, les
pastilles du plan, le MCP `kesh_plan_comptable`. **Dépend de la 47-1** (champ, règles, erreurs et messages).

## Story

En tant que comptable,
je veux choisir, dans le dialogue d'un compte, son **compte miroir** dans une seule liste,
afin que son solde de sens inverse se présente de l'autre côté du bilan ou du compte de résultat.

## Modules touchés (règle de splitting préventif)

Cinq : `kesh-api/routes/accounts`, `frontend/src/lib/features/accounts` (avec la page du plan comptable), les quatre
catalogues i18n, la documentation (`docs/api-external.md`, manuels utilisateur et d'administration), l'E2E
(`frontend/tests/e2e/`).

## Critères d'acceptation

1. **API des comptes** : `POST /api/v1/accounts` et `PUT /api/v1/accounts/{id}` acceptent `mirrorAccountId` ; au `PUT`
   à **trois états** (absent = inchangé, `null` = retiré, entier = posé ; désérialisation `present_or_null`, patron de
   `fxGainAccountId`) traduits en `MirrorChange` (47-1 AC-3) dans la route, sans lecture préalable ; au `POST`, entier
   = posé, `null` ou absent = aucun miroir. `AccountResponse` rend `mirrorAccountId` (absent quand `null`, comme les
   autres champs facultatifs). Le sens inverse (« qui me désigne ») **n'est pas** rendu : il se déduit de `GET
   /accounts` (sources archivées comprises avec `includeArchived=true`). Reposer le miroir déjà posé : ni refus ni
   changement de version. Refus : ceux de la 47-1 (`MIRROR_ACCOUNT_INVALID` 400, `ACCOUNT_IS_MIRROR` 409), et
   `OPTIMISTIC_LOCK_CONFLICT` 409 sur la version du compte modifié. Audit : l'entrée `account.created` /
   `account.updated` du compte source porte `mirrorAccountId` (47-1 AC-3). Tests : `crates/kesh-api/tests/accounts_e2e.rs`
   — trois états, chaque refus avec son détail (une cible **d'une autre société** rend la même `unknown` qu'une cible
   inexistante), création avec miroir, RBAC (la boucle de `consultation_role_cannot_write`, qui ne parcourt à
   `fd04099f` que `POST /accounts` et `PUT /accounts/{id}/reactivate`, gagne `PUT /accounts/{id}`, avec un corps portant
   `mirrorAccountId`), aucune entrée d'audit ni version touchée sur la cible. **Interblocage** : à la pose
   (47-1 AC-7), A est déjà sous `FOR UPDATE` puis T se verrouille ; `archive_guarded` de T prend sa ligne puis lit
   « qui désigne T » sous verrou — deux gestes concurrents (poser A → T, archiver T) ferment un cycle (A tenue et T
   attendue d'un côté, T tenue et A attendue de l'autre), que la 47-1 range parmi les refus acceptables ; `create_account`
   et `update_account` sont déjà rejouées par `retry_app_on_deadlock`, **`archive_account` ne l'est pas** : elle le
   devient (`"accounts::archive"`, patron de `update_account`), sans quoi le cycle se lit « 500 » à l'archivage.
2. **Dialogue du compte** (`routes/(app)/accounts/+page.svelte`, création **et** modification) : **un seul `<select>`
   « Compte miroir »** (natif, comme `AccountCurrencyFields`), en dernier des champs, avec l'option *Aucun* (défaut) et
   les **candidats** calculés par une fonction pure `mirrorCandidates(account, form, accounts)` (fichier neuf
   `frontend/src/lib/features/accounts/mirror.ts`, testée en Vitest) depuis **le plan que la page a chargé**
   (`fetchAccounts(showArchived)` : les archivés n'y sont que si *Afficher les archivés* est cochée, ce que la fonction
   ne corrige pas — le serveur reste juge, 47-1 AC-4 et AC-5 ; ce plan est déjà celui de la société) : comptes actifs (non archivés), autres
   que le compte lui-même, de l'autre côté du type **choisi dans le formulaire** (actif ↔ passif, produit ↔ charge),
   **non postables** (la case *Postable* décochée), sans rôle, qui n'ont pas eux-mêmes de miroir ; libellés « numéro
   nom », dans l'ordre du plan. Le miroir enregistré figure toujours dans la liste, même s'il ne passe plus le filtre :
   par `withCurrentAccount` (`account-options.ts`, en tête de liste), non par une seconde règle. Les comptes de
   regroupement (parents d'un sous-compte, donc non postables) y figurent : la 47-1 les admet, l'écran ne les écarte pas.
   **Aucun cas particulier de rôle** : un rôle de fonds propres, un type changé dans le formulaire, un compte qui porte
   des écritures sont jugés par le serveur (`equity`, `accountType`, `notPresentationAccount`), dont le refus s'affiche
   sous le champ (AC-3) — le `<select>` n'a donc jamais à disparaître ni à décider seul (un `<select>` caché laisserait
   le miroir en place et le refus `equity` sans champ où s'afficher). Aide sous le champ (par `aria-describedby`) :
   « Le compte miroir est un compte de présentation : non postable, sans rôle ni écriture. Le solde de ce compte, s'il
   est de sens inverse, y paraît dans le bilan ou le compte de résultat. » (le mot de la case, *Postable*, non
   « imputable »). Quand le compte **est lui-même le miroir** d'autres comptes (une source au moins **du plan chargé**
   le désigne), le `<select>` est remplacé par une phrase : « Ce compte est le compte miroir de 2270. » (numéros en
   variable Fluent, une seule clé, valable pour un ou plusieurs numéros), et le champ n'est pas envoyé ; si les seules
   sources sont archivées et non affichées, le `<select>` paraît et c'est le refus `chain` du serveur qui parle.
   **Sélecteurs d'E2E** : `account-create-dialog-mirror`, `account-edit-dialog-mirror` (le `<select>`),
   `account-edit-dialog-mirror-of` (la phrase). Champ envoyé seulement s'il a changé (le `PUT` d'un client qui ne
   connaît pas le champ n'efface rien, 47-1 AC-3) ; il rejoint `putAccount`, si bien que le rejeu après confirmation
   du retypage (`confirmRetype`) le porte aussi. Composant natif, **jamais** `AccountAutocomplete` (qui écarte les
   comptes en devise étrangère, admis comme miroirs) ; la page porte déjà l'exemption `fx-filtre-comptes`
   (`+page.svelte:84`) ; s'il reçoit le plan en prop (`accounts: AccountResponse[]` déclaré sur sa propre ligne
   terminée par `;`, `PROP_RE` de `fx-account-filter-inventory.test.ts`), il porte l'exemption écrite
   `fx-filtre-comptes : exempté — un compte miroir admet un compte de toute devise`.
3. **Refus de l'API à l'écran** : l'écran n'a **aucun texte de refus à lui** — le serveur rend `err.message` traduit,
   qui nomme le compte en cause (47-1 AC-9), comme pour `FX_ACCOUNT_INVALID`. `MIRROR_ACCOUNT_INVALID` (reconnu par son
   code) s'affiche **sous le `<select>`** (`role="alert"`, `aria-invalid`), en création et en modification — y compris
   au rejeu de `confirmRetype`, où tombe la raison `accountType` d'un retypage : un seul assistant
   `mirrorFieldErrorOf(err)` (patron de `fxFieldErrorOf`, `+page.svelte:150`) aux trois `catch` (`submitCreate`,
   `submitEdit`, `confirmRetype`) ;
   `ACCOUNT_IS_MIRROR` à la modification (rendre postable, retyper, poser un rôle) prend le chemin existant
   (`editError`, au pied du formulaire : aucun code neuf) ; au dialogue *Archiver le compte*, `ACCOUNT_IS_MIRROR`
   (`archive`) s'affiche **dans le dialogue** (état `archiveMirrorMessage`, `role="alert"`,
   `data-testid="account-archive-mirror"`, patron de `archiveDesignated`, remis à vide par `openArchive` comme lui), **sans** « Archiver quand même » **et sans**
   le bouton *Archiver* (rien n'est à confirmer : seul *Annuler* reste), et sans toast ; le remède d'une source
   archivée (« réactivez-la, retirez le lien, puis archivez ») est dans le message du serveur, que l'écran n'a pas à
   reconstruire.
4. **Aucune autre marque au plan comptable** : ni colonne, ni pastille (choix KISS ; le dialogue suffit à voir et
   changer un miroir). Les dialogues *Nouveau compte* et *Modifier*, plus hauts, **défilent**
   (`Dialog.Content` à `max-h-[90vh] overflow-y-auto`, patron de `contacts/+page.svelte:828` ; le composant n'a que
   `sm:max-w-sm`, sans hauteur bornée).
5. **Types et API front** : `accounts.types.ts` — `AccountResponse.mirrorAccountId?: number`, champ facultatif des
   requêtes de création et de modification (absent = inchangé, `null` = retiré) ; rien d'autre (pas de lecteur de
   `details` : l'écran n'emploie que `err.code` et `err.message`).
6. **Vitest** : `mirrorCandidates` (table : quatre types, rôle, postable, archivé, soi-même, compte qui a déjà un
   miroir, miroir enregistré gardé, changement de type dans le formulaire) ; le dialogue (select présent, remplacé par
   la phrase pour un compte miroir, champ non envoyé s'il n'a pas changé, refus sous le champ). Mutations à faire rougir : candidat postable proposé ; champ envoyé inchangé ; phrase « compte miroir de »
   absente ; `<select>` qui disparaît pour un rôle de fonds propres (le refus serveur s'affiche sous le champ).
7. **Captures d'écran** (thème clair, 1280 × 720) : dialogue sans miroir, miroir choisi, compte miroir (phrase),
   refus sous le champ, archivage refusé (source active, source archivée) ; **une** capture du dialogue le plus haut à
   390 × 844 — jointes au Dev Agent Record et au prompt de revue (`kesh-gate-logs/`).
8. **E2E** (fichier neuf `frontend/tests/e2e/accounts-mirror.spec.ts`) : comptes **créés par l'API**, sans parent, à numéros uniques
   (patron `D${Date.now()…}` d'`accounts.spec.ts`), jamais de miroir sur un compte du seed ; liens retirés en `finally`
   (le `finally` réactive une source archivée avant de retirer son lien). Sélecteurs : ceux de l'AC-2 et de l'AC-3, plus
   les existants (`account-row-{numéro}-edit-button`, `-archive-button`, `-reactivate-button`,
   `account-show-archived-toggle`, `account-archive-dialog-confirm`). Parcours : créer un compte non postable de l'autre
   côté, le choisir comme miroir d'un compte à l'écran, enregistrer, rouvrir (le `<select>` le montre) ; ouvrir le
   miroir (la phrase « compte miroir de … ») ; tenter d'archiver le miroir (refus nommant la source, ni « Archiver quand
   même » ni *Archiver*) ; retirer le miroir (*Aucun*), puis archiver (accepté) ; source archivée : archivage du miroir
   refusé avec le remède, puis réactivation, retrait, archivage accepté.
9. **Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, section *Plan comptable* `:421`) : nouvelle sous-section
   « Compte miroir » (`\label{sec:compte-miroir}`), après *Modification d'un compte* (`:465`) : à quoi sert un compte
   miroir (un solde de sens inverse présenté de l'autre côté, comme le prévoit la présentation du bilan de l'art. 959a
   CO, sans compensation ni écriture — l'article n'est cité que pour le bilan ; **sans dire que le bilan de Kesh y est
   « conforme »** : les avertissements du bilan (`:3918`, « pas encore structuré selon l'art. 959a CO ») et du compte de
   résultat (`:3948`) restent vrais) ; créer le compte de présentation (*Postable* décoché, sans rôle, **sans compte
   parent** : un parent devient non imputable à son premier sous-compte — règle 14-3a, `accounts::create` —, si bien
   que 6900.1 créé sous 6900 rendrait 6900 non imputable ; l'exemple de *Ajout d'un compte*, `:446-449`, pose un parent,
   la sous-section dit l'inverse en toutes lettres, comme le fichier d'epic) puis le choisir ; plusieurs comptes peuvent
   partager un miroir ; ce qui est refusé tant qu'un compte est miroir (écriture, case *Postable*, retypage, rôle,
   archivage) ; **ne pas choisir comme miroir un compte que les réglages de facturation, un article ou une règle de
   rapprochement désignent** : le flux qui l'emploierait serait refusé, nommé (constat promis par la 47-1, *Risques*) ; les exemples de l'**exemple chiffré du fichier d'epic** (comptes **à créer** 6950.1 « Gains de
   change » (imputable), 6900.1 « Pertes de change », 1040.1 « Créances AVS/AI/APG », 2100.1 « Découvert bancaire »,
   comptes livrés 1020, 2270 sous leur nom), et, pour les comptes en devise, la désignation de 6950.1 comme compte des gains **et** des pertes de change
   (un compte non imputable ne peut pas l'être, `FxAccountReason::NotPostable`). À *Suppression d'un compte* (`:486`),
   le refus d'archivage d'un compte miroir et le cas d'une source archivée. Les deux listes de champs sont complétées
   du champ **Compte miroir** : *Ajout d'un compte* (`:449-451`, après *Postable*) et le paragraphe de *Modification*
   (`:468`, « Vous pouvez modifier le nom, le rôle et l'indicateur postable »). Le renvoi aux rapports (`\ref`) est écrit
   par la 47-4, qui pose le label. ⚠️ **G4-bis et G4-ter** (`crates/kesh-api/tests/textes_coherents.rs`) lisent ce
   manuel : un numéro à quatre chiffres doit être un compte d'un plan livré, sous son nom ; les comptes à créer
   s'écrivent en sous-compte (`6900.1`, forme admise : seul le parent est contrôlé). Exécuter ce fichier de tests.
10. **Documentation API et administration** :
    - `docs/api-external.md` : sous-section « Compte miroir » **avant** « Notes du tableau des ressources » (`:1139`), après
      les trois sections multi-devises (`:1060`, `:1092`, `:1113`, qu'elle ne sépare pas), sur le patron de la première : champ `mirrorAccountId` (trois états au `PUT`, posé au `POST`), règles (R2, R3), codes
      `MIRROR_ACCOUNT_INVALID` (liste close des raisons) et `ACCOUNT_IS_MIRROR`, refus `ACCOUNT_NOT_POSTABLE` d'une
      écriture sur un compte miroir, effet sur les rapports renvoyé à la 47-3, colonne `mirror_account_id` de
      `accounts.csv` en fin de ligne, **après `updated_at`** (les colonnes de l'Epic 37 sont, elles, au milieu) ; un lien hors règle (sauvegarde, SQL) est rendu tel quel par `GET /accounts` mais
      ignoré par les rapports, et se corrige par `PUT` (`null`) ; ligne du tableau du plan comptable (`:226`, renvoi à la sous-section par une note neuve du tableau — numéro libre
      à la reprise de `main`, la 47-3 ajoutant le sien : le second arrivé renumérote) ; **les deux codes neufs entrent au
      tableau des erreurs du § 10** (`:1257-1323`, une rangée chacun, comme `FX_ACCOUNT_INVALID` et
      `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`), et `ACCOUNT_NOT_POSTABLE` (`:1257`) gagne « ou un compte miroir » ; ⚠️ G4-bis,
      G4-ter et G25 lisent ce fichier : comptes de l'exemple seulement, jamais « projet » ;
    - manuel d'administration (`docs/manual/fr/admin-manual.tex`) : sous-section « Passer à la <release de l'Epic 47> »
      dans *Mise à jour de Kesh* (`:2497`, patron de « Passer à la 0.19.0 », `:2550` ; titre provisoire « Epic 47 »,
      renommé au tag) — **en tête la sauvegarde complète**, la migration (total recompté), le retour par la sauvegarde
      d'avant la mise à jour, puis, pour les intégrations, le champ de l'API et la colonne de `accounts.csv` ; dans
      *Compatibilité* de la sauvegarde (`:2302`), une phrase : une sauvegarde de la release précédente s'importe, sans
      miroir ; et, tel que la recette de montée le constate, si une sauvegarde de cette release s'importe dans la
      précédente (à la 0.17.5, une colonne inconnue faisait refuser l'import, `:2302`). Gardes G4-bis/ter, G19, G25 respectées.
    - `.tex` seulement (aucun PDF commité) ; **le prompt de chaque passe de revue nomme ces documents** et exige le
      contrôle du PDF compilé localement (`pdftotext … | tr '\n' ' ' | tr -s ' '`).
11. **i18n** : clés `accounts-…` (préfixe du dossier, `lint-i18n-ownership`) dans les quatre locales ; `sitesTotal` de
    `frontend/src/lib/shared/i18n-keys.test.ts` **recompté** depuis la garde après la reprise de `main` ;
    `i18n-libelle-en-dur` et `fx-account-filter-inventory.test.ts` verts ; aucun numéro de compte en dur dans un
    catalogue ou un repli (variables Fluent).
12. **Inventaire** (47-1 AC-11) : rien à ajouter — la 47-1 ne tient plus de liste close des fichiers qui nomment
    `mirror_account_id` (validation P1 de la 47-1, 2026-11-19 : la garde au point de passage des lignes suffit).

## Tâches / sous-tâches

- [x] **T0** : reprise de `main` ; relire au code livré par la 47-1 les codes d'erreur et les raisons (les messages sont
  ceux du serveur : aucune clé de refus à reprendre) ; `fix/api-parent-compte`
  (#1351) — s'il est fusionné, le champ *Compte parent* précède le `<select>` ; sinon le second arrivé résout.
- [x] **T1 — route** (AC 1, 12) : champs, réponse, rejeu de l'archivage sur interblocage ; tests `accounts_e2e.rs`.
- [x] **T2 — front** (AC 2–5) : types, `mirror.ts`, `<select>`, phrase, refus, archivage.
- [x] **T3** : Vitest, E2E, captures (AC 6–8).
- [x] **T4** : manuels, `api-external.md`, i18n (AC 9–11).
- [x] **T5** : gates backend, frontend et E2E complets au dernier commit de code.

## Dev Notes

### Code lu (à `origin/main` `fd04099f`, relevé des anciennes 47-1b et 47-3a)

- `crates/kesh-api/src/routes/accounts.rs` (`CreateAccountRequest`, `UpdateAccountRequest`, `AccountResponse`,
  `own_fx_for_update` : patron de traduction des trois états) ; `present_or_null`.
- `frontend/src/routes/(app)/accounts/+page.svelte` (951 lignes : préférer un petit composant si le dialogue grossit),
  `frontend/src/lib/features/accounts/AccountCurrencyFields.svelte` (`<select>` natif des comptes de change, prop
  `accounts` ligne 33), `fx-account-filter-inventory.test.ts` (`PROP_RE`), `accounts.types.ts`.
- `docs/manual/fr/user-manual.tex:421` (*Plan comptable*), `:449-451` (*Ajout d'un compte* : parent facultatif, *Postable*), `:465`, `:468`
  (champs modifiables), `:486` ; `docs/api-external.md:226`, `:1060` ; `docs/manual/fr/admin-manual.tex:2302`, `:2497`, `:2550`.

### Choix KISS (refonte du 2026-11-19)

- **C-47-2-1** : un seul `<select>` au dialogue, création et modification. *Écarté* : la section à trois boutons radio
  (Aucun, Miroir, Paire) et ses quatre états calculés.
- **C-47-2-2** : aucune marque au plan comptable. *Écarté* : pastilles « miroir » des deux côtés. À rouvrir en issue si
  l'usage le demande.
- **C-47-2-3** : le MCP `kesh_plan_comptable` est inchangé (aucune clé neuve, aucun plafond à remesurer). *Écarté* :
  `compte_miroir_id` dans la sortie MCP, qui imposait la révision des plafonds (34 / 61 estimés).
- **C-47-2-4** : les candidats se filtrent au client sur ce qu'il sait (type, postable, rôle, actif, chaîne) ; la
  condition « sans écriture » reste au serveur.
- **C-47-2-5** (validation P1) : l'écran ne traduit aucun refus et ne décide d'aucune règle — il affiche `err.message`
  et ne cache jamais le `<select>` (rôle de fonds propres, type changé : le serveur refuse, sous le champ). *Écarté* :
  `readMirrorError`, clés de refus côté écran, `<select>` absent pour les fonds propres.
- **C-47-2-6** (validation P1) : douze captures (six états × deux largeurs) ramenées à six plus une en 390 × 844.

### Risques

Conflit avec #1351 (même dialogue, mêmes requêtes) ; interblocage pose / archivage (rejeu de l'archivage, AC-1) ; `sitesTotal` à recompter après chaque reprise de `main` ;
G4-bis sur les exemples du manuel.

### References

- Fiche 47-1 (AC-3, AC-4, AC-5, AC-9) ; fichier d'epic (§ *Exemple chiffré*) ; `CLAUDE.md` § *Recette visuelle*,
  § *Le prompt d'une passe doit NOMMER le manuel*.
- Registre de la dette du fichier d'epic : à la clôture de la boucle de revue, tout finding non remédié y est consigné ;
  la 47-5 en part (« on trace toujours »).

## Notes de version

Le dialogue d'un compte propose un **compte miroir** : un compte de présentation, non postable, sur lequel le solde de
sens inverse du compte paraîtra dans le bilan ou le compte de résultat. API : champ `mirrorAccountId` des comptes ;
colonne `mirror_account_id` en fin de ligne de `accounts.csv`.

## Change Log

- 2026-11-21 — remédiation de la revue de code P1 (Opus 5.5 : 0 C / 0 H / 0 M / 8 L, tous corrigés dans la story, règle
  « un LOW qui touche la production se corrige dans la story, puis passe ciblée »). Un commit de code, `9f2ae083`. L3 :
  `AccountMirrorField.svelte` vide `value` quand il sort de `candidates` (un `$effect`) ; test neuf
  `AccountMirrorField.test.ts` avec son banc `AccountMirrorFieldHarness.svelte` (rouge sans l'effet : mutation
  `if (false && …)` vérifiée). L6 : `create` verrouille la cible du miroir **avant** le parent, comme `update`
  (commentaire dans `repositories/accounts.rs`). L5 : commentaire du `catch` de `confirmRetype` et intitulé du Vitest
  dits « cas de concurrence ». L4 : commentaire du rejeu de l'archivage récrit (rejeu défensif, aucun interblocage
  établi). L1 : phrase de *Compatibilité* du manuel d'administration écrite « d'après le contrôle de compatibilité du
  schéma … lecture du code, à confirmer à la recette de montée ». L2 : manuel utilisateur — quand la phrase « compte
  miroir de … » s'affiche (sources du plan *affiché*), refus du retypage d'une source et son remède (*Aucun* en même
  temps), paragraphe *Un compte qui a un compte miroir*. L7 : triage E2E reformulé (Completion Notes), deux specs
  rejouées seules (2 passés). L8 : capture 07 refaite sans toast. Passe ciblée P2 : `kesh-gate-logs/s47-2/revue-p2-prompt.md`
  (à lancer). ⚠️ Première tentative du commit : un remplacement de commentaire avait dupliqué `archive_account` (clippy
  rouge), corrigé par amendement avant tout gate rendu.

- 2026-11-20 — développement (Opus 5.5, `bmad-dev-story`) : reprise de `origin/main` (`aef10456`, #1351 fusionné : champ
  *Compte parent* au dialogue, `ParentChange` à côté de `MirrorChange`), route, écran, Vitest, E2E, captures, manuels,
  `api-external.md`, i18n ; gates complets au dernier commit de code `90c4b5aa`, sur la cible propre. Détail au Dev Agent
  Record. Statut `review`.

- 2026-11-19 — validation P3 (Sonnet 5.5, ciblée sur la remédiation P2) : **0 C / 0 H / 1 M / 0 L, remédié**. Relu au
  code d'`origin/main` `fd04099f` : règle 14-3a (`accounts::create` passe le parent à non imputable, `repositories/accounts.rs:308-317`),
  les trois `catch` de `+page.svelte` (`submitCreate` `:269`, `submitEdit` `:362`, `confirmRetype` `:398`, patron `fxFieldErrorOf` `:150`),
  `withCurrentAccount` (`account-options.ts:59`, en tête de liste), boucle de `consultation_role_cannot_write`
  (`accounts_e2e.rs:977`, `POST` et `PUT …/reactivate` seulement ; la boucle branche sur `POST` sinon `put`, `PUT /accounts/{id}`
  s'y ajoute sans autre changement), `archive_account` sans rejeu (`routes/accounts.rs:524`) contre `accounts::create` / `accounts::update`
  (`:415`, `:509`), lignes du manuel (`:421`, `:446-451`, `:465`, `:468`, `:486`, `:3918`, `:3948`) et de `api-external.md`
  (`:226`, `:1060`, `:1092`, `:1113`, `:1139`, `:1257`), `admin-manual.tex` `:2302`, `:2497`, `:2550`, `max-h-[90vh]`
  de `contacts/+page.svelte:828` — exacts. M1 : l'AC-9 listait « comptes livrés 6950 » et omettait 6950.1, contre le
  fichier d'epic (§ *Exemple chiffré* : 6950.1 « Gains de change », à créer, imputable) et la 47-4 AC-7 : aligné.
  Rapport : `kesh-gate-logs/epic-47-validation/47-2-miroir-api-et-ecran-du-plan-P3.md`.
- 2026-11-19 — validation P2 (Opus 5.5, complète, contexte frais) : **0 C / 0 H / 2 M / 6 L, remédiés**. Remédiation P1
  relue au code d'`origin/main` `fd04099f` : `fetchAccounts(showArchived)` (`+page.svelte:212`), `archiveDesignated`
  et ses boutons (`:117`, `:920-925`), `fxFieldErrorOf` (`:150`), `putAccount`/`confirmRetype` (`:324`, `:392`),
  `archive_account` sans rejeu (`routes/accounts.rs:524`, `archive_guarded` ouvre sa transaction à chaque appel :
  rejeu sûr), `max-h-[90vh]` (`contacts/+page.svelte:828`), `PROP_RE`, sélecteurs existants, `api-external.md`
  (`:226`, `:1060`-`:1139`, `:1257`, `:1307`, `:1323`), gardes G4-bis (lit le manuel utilisateur), G4-ter et G25 (lisent
  `api-external.md`), lignes du manuel `:421`, `:446-451`, `:465`, `:468`, `:486`, `:2302`, `:2497`, `:2550`. M1 :
  sous-comptes à créer **sans compte parent** non dit, alors que l'exemple d'*Ajout d'un compte* en pose un et qu'un
  parent devient non imputable à son premier sous-compte (`accounts::create`, règle 14-3a) — 6900.1 sous 6900 rendait
  6900 non imputable (AC-9, AC-8 ; propagé à la 47-4). M2 : le refus `MIRROR_ACCOUNT_INVALID` au rejeu de
  `confirmRetype` (raison `accountType`, celle d'un retypage) tombait au pied du formulaire, contre « sous le `<select>`
  en modification » → un assistant aux trois `catch` (AC-3). L : `withCurrentAccount` pour le miroir gardé (DRY) ;
  « de la société » redondant ; comptes de regroupement parmi les candidats, dit ; `archiveMirrorMessage` remis à vide à
  l'ouverture ; boucle RBAC sans `PUT /accounts/{id}` ; `:3913` → `:3918`, art. 959a cité pour le bilan seul, `:3948` ;
  phrase de compatibilité de la sauvegarde récrite. Concordance : 47-1 à HEAD (`c7ba5bb5`) — AC-9 ne dit plus que la
  47-2 reprend les clés, AC-7 décrit l'interblocage pose / archivage et le rejeu de la 47-2 : écart de la P1 levé ;
  47-3 (exports par le service, aucun rendu propre) et 47-4 (label `sec:compte-miroir`, renvoi) concordants. Rapport :
  `kesh-gate-logs/epic-47-validation/47-2-miroir-api-et-ecran-du-plan-P2.md`.
- 2026-11-19 — validation P1 (Sonnet 5.5) : 0 C / 0 H / 8 M / 8 L, remédiés. M : candidats « archivés compris » alors que la
  page ne charge les archivés que sur case cochée ; `<select>` caché pour les fonds propres (refus `equity` sans champ,
  miroir laissé en place) — cas particulier retiré, le serveur juge ; contradiction clés de refus / `err.message`
  (`readMirrorError` retiré) ; `archive_account` non rejouée sur interblocage alors que la pose et l'archivage ferment
  un cycle ; `api-external.md` : sous-section placée entre les sections multi-devises, rangées du § 10, note du
  tableau, gardes G4/G25 ; manuel utilisateur : listes de champs d'*Ajout* et de *Modification* omises, constat promis
  par la 47-1 (*Risques*) absent ; « conforme à l'art. 959a CO » contre l'avertissement du bilan ; sélecteurs d'E2E
  absents, bouton *Archiver* laissé après refus. L : défilement aussi à la création ; captures réduites ; « postable »
  (mot de la case) pour « imputable » ; `putAccount` porte le champ ; cible d'une autre société ; numéros de ligne ;
  sens inverse de la compatibilité de sauvegarde ; Notes de version. Concordance 47-1 (état 99bc3636) : écart signalé à
  l'orchestrateur — la 47-1 AC-9 dit que « la 47-2 reprend » les clés de refus, or l'écran affiche `err.message`.
  Rapport : `kesh-gate-logs/epic-47-validation/47-2-miroir-api-et-ecran-du-plan-P1.md`.
- 2026-11-19 — **Refonte du 2026-11-19 (miroir seul, KISS)** : fiche renommée de `47-3a-ecran-plan-comptable`, fusionnée
  avec la partie API de `47-1b-liens-api-mcp-export` (superseded). Retirés : section à trois boutons radio et ses quatre
  états, paire, avertissement CO 958c et son dialogue, pastilles du plan, MCP `kesh_plan_comptable` et ses plafonds,
  table de traduction à deux champs, raisons `bothLinks` / `onCreate`, test du ZIP global (la colonne et son test sont
  à la 47-1). Gardés : `<select>` natif et exemption de la garde d'inventaire, refus au champ, archivage refusé et cas de
  la source archivée, E2E en fichier neuf à numéros uniques, sous-section du manuel après *Modification d'un compte*,
  sous-section « Passer à la release » du manuel d'administration. Douze critères (douze à la 47-3a, dix à la 47-1b).
  **Statut remis à `drafted`** en attendant la revalidation.

### Historique — Change Log de la 47-3a (avant la refonte, laissé tel quel)

- 2026-11-19 — validation P1 (Opus 5.5) : 0 C / 1 H / 9 M / 5 L, remédiés. H : remède de l'archivage refusé
  impossible quand la source est archivée (elle garde son lien, un compte archivé ne se modifie pas) et « lien vers ce
  compte » sans adresse au plan (AC-5 réécrite ; `active` ajouté à `linkedAccounts` dans la 47-1b
  AC-2 ; le même champ dans `linked` de la 47-1a AC-7 reste à reporter). M : texte CO 958c réduit à charge/produit alors qu'une paire actif/passif existe (D3), et recopié au lieu de
  la clé du `warnings` de la 47-1b ; états de la section non définis (compte cible d'un miroir, rôle et type changés
  dans le formulaire) ; index des liens bâti sur la liste sans archivés ; champs du `PUT` non spécifiés (passage
  miroir → paire, « Aucun », champs absents) ; refus au champ « traduit selon la raison » en double du message
  serveur ; dialogue sans défilement (`sm:max-w-sm`, plus haut avec #1351) ; accessibilité non dite (`fieldset`,
  `aria-describedby`, flèches des pastilles) ; E2E sur la base partagée sans isolement des liens, dans le fichier que
  #1351 étend ; manuel sans les refus de retypage et de rôle, refus d'archivage hors de *Suppression d'un compte*,
  renvoi vers un label de la 47-3b inexistant. L : numéros de ligne du dialogue (décalés d'environ huit) ; #1351 non
  nommé, T0 muet s'il n'est pas fusionné ; `archiveDesignated` laissé après un refus de lien ; `data-testid` et
  pastille à nombreuses sources ; `warnings` typé sur la liste. Rapport : `kesh-gate-logs/epic-47-validation/
  47-3a-ecran-plan-comptable-P1.md`.

- 2026-11-19 — validation P2 (Sonnet 5.5) : 0 C / 0 H / 5 M / 5 L, remédiés. M : candidats d'un **miroir** excluant
  les cibles d'autres miroirs, contre D2 (plusieurs sources sur un même compte miroir) et la pastille « Miroir de 2270,
  2271 » — règle désormais par nature ; priorité des quatre états non fixée (compte cible au rôle de fonds propres) et
  état « index pas encore lu » ; garde d'inventaire `fx-account-filter-inventory.test.ts` (prop `AccountResponse[]`)
  sans exemption écrite, alors que le sélecteur admet toute devise (Q3) — section extraite en composant, exemption
  posée ; archivage refusé : source archivée invisible sans *Afficher les comptes archivés*, `details` illisible non
  traité ; clé de l'avertissement dite « sans phrase recopiée » alors que la 47-1b (Notes) dit que l'écran a ses
  propres textes — contradiction signalée, clé tenue par le Dev Agent Record de la 47-1b. L : ordre validation →
  avertissement → `PUT` ; `PRESENTATION_LINK_INVALID` sans sélecteur ; texte de l'état *cible* ; pluriel de la
  pastille ; E2E de la source archivée et `finally`. Cohérence vérifiée sans écart : 47-1a AC-7 (`linked` porte `actif`),
  47-1b AC-1 (table), AC-2 (`active`), #1351 (`sitesTotal` +4, `parentFieldForUpdate`, `sec:modification-compte`).
  Rapport : `kesh-gate-logs/epic-47-validation/47-3a-ecran-plan-comptable-P2.md`.

- 2026-11-19 — validation P3, ciblée sur la remédiation P2 (Opus 5.5) : 0 C / 0 H / 1 M / 3 L, remédiés. M : l'AC-12
  disait les clés du composant `account-…`/`accounts-…` ; `lint-i18n-ownership.js` (dossier `accounts` à un segment,
  `keyBelongsToFeature`) refuse `account-…` — clés `accounts-…` seulement, comme `AccountCurrencyFields`. L : sélecteur
  nommé (`<select>` natif, non `AccountAutocomplete`, qui écarte les comptes en devise contre Q3) et prop du composant
  précisée (`accounts: AccountResponse[]`, ce qui fonde l'exemption au regard de `PROP_RE`) ; candidats comptés hors le
  lien propre du compte édité (état résultant, 47-1a AC-5/AC-6) ; texte de l'avertissement non reformulé (« Kesh
  présentera le net » absent de la 47-1b AC-4). Vérifié sans écart à `origin/main` `fd04099f` : forme de l'exemption
  (`EXEMPT_RE`), précédents `AccountCurrencyFields.svelte:22` et page `:84`, 951 lignes ; candidats contre D2, D3,
  47-1a AC-5 (`chain`) et C-47-1a-3. Propagation toujours en attente : 47-1b, Notes (« depuis ses propres textes »)
  et nom de la clé de l'avertissement. Rapport : `kesh-gate-logs/epic-47-validation/47-3a-ecran-plan-comptable-P3.md`.

- 2026-11-19 — validation P4, ciblée sur la remédiation P3 (Sonnet 5.5) : 0 C / 0 H / 1 M / 2 L, remédiés. M : les deux
  transitions que la remédiation P3 a ajoutées à la table de vérité des candidats (paire avec D → miroir vers D ; miroir
  vers T, seule source → paire avec T) reposent sur « l'état résultant » (47-1a AC-5, AC-6) mais aucun test ni texte de
  la 47-1a ne les nomme (AC-5 `chain` dit « la cible qui a son propre lien » : D, encore en paire avec le compte, y
  répond avant libération) — contrôle ajouté en T0, retrait des lignes si le dépôt refuse ; **signalé à l'orchestrateur
  pour la 47-1a** (non modifiée) : deux tests d'intégration de ces transitions. L : `PROP_RE` écrite avec sa forme
  exacte (ligne propre terminée par `;`, `interface Props`) ; fonction de construction de l'index absente de l'AC-2
  (`buildPresentationIndex`). Vérifié sans écart à `origin/main` : `lint-i18n-ownership.js` (premier segment ;
  `KNOWN_VIOLATIONS` sans entrée `accounts`, `.ts` du dossier parcourus aussi, aucune clé `i18nMsg` hors `accounts-`
  prescrite), `EXEMPT_RE`, `isCandidate` (un `.ts` à paramètre `AccountResponse[]` hors garde). Rapport :
  `kesh-gate-logs/epic-47-validation/47-3a-ecran-plan-comptable-P4.md`.

- 2026-11-19 — validation P5, ciblée sur la remédiation P4 (Opus 5.5) : 0 C / 0 H / 1 M / 3 L, remédiés côté 47-3a. Concordance
  avec la 47-1a depuis sa P3 (`aaf7b310`, `1cd1849d`) : AC-5 admet les deux transitions, jugées sur l'état après le
  `PUT`, AC-6 les écrit avec deux tests d'intégration et leurs sondes ; la table de vérité des candidats concorde
  (miroir : cible admise, source et membre de paire exclus hors lien propre ; paire : cibles, sources archivées
  comprises, et membres d'une paire avec un tiers exclus). L : phrase « aucun test de la 47-1a » périmée → renvoi à
  la 47-1a AC-5/AC-6, T0 réduit à un contrôle (tests cités, arrêt s'ils manquent) ; « autres sources » de T précisé
  « archivée comprise » (sonde (2)) ; `buildPresentationIndex` ne portait pas le plan que `presentationLinkCandidates`
  doit parcourir → contenu de l'index écrit. Vérifié sans écart à `origin/main` `fd04099f` : `PROP_RE`
  (`fx-account-filter-inventory.test.ts:26`) recopiée exactement, `isCandidate` (`.svelte` seul pour la prop),
  `AccountCurrencyFields.svelte:33` (`accounts: AccountResponse[];` dans `interface Props`), page à 951 lignes, aucun
  `PresentationIndex` existant. M : concordance non exacte sur la sonde (1) de la 47-1a — **signalé à l'orchestrateur
  pour la 47-1a** (non modifiée), comportement de l'écran écrit à l'AC-2 : sa sonde (1) — D, libéré,
  « cible du miroir d'un tiers » refusé en `chain` — contredit D2 (une cible admet plusieurs sources) ; D en paire ne
  peut porter d'autre rôle que celui de cible (`CHECK` « pas les deux »), la sonde est donc à reformuler ou à retirer.
  Rapport : `kesh-gate-logs/epic-47-validation/47-3a-ecran-plan-comptable-P5.md`.

- 2026-11-19 — clôture : la passe P5 ciblée de la 47-1a (Sonnet 5.5, sur `d5eff9e0`, qui retire la sonde (1) signalée
  ci-dessus) a exercé l'AC-1, l'AC-2 et le T0 de cette fiche contre la 47-1a AC-5, AC-6 et AC-8 : **0 C / 0 H / 0 M /
  0 L**, aucune fiche modifiée ; le M de la P5 est levé des deux côtés. Boucle close ; `Status: ready-for-dev`.
  Rapport : `kesh-gate-logs/epic-47-validation/47-1a-liens-modele-et-depot-P5-ciblee.md`.

### Historique — Change Log de la partie API de la 47-1b (avant la refonte, laissé tel quel)

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

Opus 5.5 (Claude Code, `bmad-dev-story`), 2026-11-20.

### Debug Log References

Journaux hors dépôt, `kesh-gate-logs/s47-2/` : `pipeline-1.log` (première série, invalidée par la base pleine),
`pipeline2.log` (reprise), `gate-backend.log`, `front-*.log`, `e2e.log`, `gate-ci.log`, `mutations.log`,
`mutations-back-1.log`, `mutations-back.log`, `captures/`.

### Completion Notes List

- **T0 — reprise de `main`** (`aef10456`) : #1351 (`fix-api-parent-compte`) fusionné ; conflits résolus en gardant les
  deux intentions : `AccountUpdate` porte `mirror` **et** `parent` ; `accounts::update` juge la garde du miroir puis celle
  du parent, `UPDATE … mirror_account_id = ?, parent_id = ?` ; `entities::account` garde `MirrorChange`, `ParentChange`,
  `MirrorLink` ; initialisations de test complétées (dont `account_parent_change.rs`, `account_mirror.rs`, qui ne
  conflictaient pas mais ne compilaient plus). Aucune migration venue de `main` : 96, `docs/migrations-idempotence-audit.md`
  inchangé (96 lignes, recompté). Le champ *Compte parent* précède le `<select>` du miroir. Interaction relevée, non
  traitée (déjà possible à la création avant la reprise) : un compte miroir peut être pris comme parent.
- **T1 — route** : `CreateAccountRequest.mirror_account_id`, `UpdateAccountRequest.mirror_account_id`
  (`present_or_null`) traduit par `mirror_for_update` ; `AccountResponse.mirror_account_id` (absent si `None`) ;
  `archive_account` rejouée sur interblocage (`"accounts::archive"`). Quatre tests neufs dans `accounts_e2e.rs` (22 → 26
  `#[sqlx::test]`, de `aef10456` à `HEAD`) et la boucle RBAC étendue à `PUT /accounts/{id}`. **Aucun test ne force
  l'interblocage pose / archivage** (rejeu non exercé).
- **T2 — écran** : `mirror.ts` (`mirrorCandidates`, `mirrorSourceNumbers`, `mirrorFieldForCreate`, `mirrorFieldForUpdate`,
  `mirrorFieldErrorOf`), composant `AccountMirrorField.svelte` (`<select>` natif, exemption `fx-filtre-comptes`, phrase pour
  un compte miroir), page : champ en dernier, refus aux trois `catch`, `archiveMirrorMessage`, dialogues défilants ; types.
- **T3** : Vitest `mirror.test.ts` (13 sites `it`, 19 tests) et `accounts-mirror.test.ts` (11) ; E2E
  `accounts-mirror.spec.ts` (2 parcours) ; sept captures (`kesh-gate-logs/s47-2/captures/`, 1280 × 720 et une 390 × 844 ;
  le seed `with-company` n'a pas de 1020 : le script le crée).
- **T4** : quatre clés `accounts-mirror-*` × quatre locales ; `sitesTotal` 3499 → 3504 (relevé par la garde) ;
  `user-manual.tex` (sous-section `sec:compte-miroir`, listes d'*Ajout* et de *Modification*, *Archiver un compte miroir*),
  `admin-manual.tex` (`sec:maj-epic-47`, phrase de *Compatibilité*), `api-external.md` (sous-section, note ¹⁴, deux rangées
  du § 10, `ACCOUNT_NOT_POSTABLE`). PDF compilés dans une copie temporaire (`make user admin`, aucune référence indéfinie),
  contrôlés par `pdftotext` ; non commités. ⚠️ La phrase « une sauvegarde de cette release est refusée par la précédente »
  est **lue au code** (`check_schema_compat`, colonne inconnue), non constatée : à confirmer par la recette de montée
  (commentaire LaTeX en tête de la sous-section).
- **Sondes négatives** : Vitest M1–M4 (candidat postable, champ envoyé inchangé, phrase absente, `<select>` caché pour un
  rôle de fonds propres) — toutes rouges (`mutations.log`) ; backend B2–B4 (réponse sans champ, `POST` et `PUT` qui
  l'ignorent) rouges (`mutations-back-1.log`, ⚠️ lancées alors que B1 était resté appliqué par un essai interrompu par
  `timeout` — fichier restauré avant les commits, vérifié par `git diff`) ; B1 (`null` traduit en inchangé) rejouée seule,
  rouge (`mutations-back.log`).
- **T5 — gates au dernier commit de code `90c4b5aa`**, cible propre `<station>/kesh-targets/kesh-wt47`,
  `CARGO_INCREMENTAL=0`, par `heavy.sh` :
  - **incidents** : (1) la cible partagée `kesh-target` rendait le binaire `accounts_e2e` d'une autre worktree (tests
    absents de cette branche) — les nextest ciblés joués dessus sont **sans valeur** et ont été rejoués sur la cible
    propre (749 passés) ; (2) `kesh-mariadb-dev2` plein (errno 135) pendant la première série : backend et mutation B1
    invalidés, rejoués après sa recréation (17:50) ;
  - backend `scripts/test-fast.sh --ci` : **6193 exécutés, 6193 passés** (3 lents, 3 instables au premier essai), 14 ignorés
    (`gate-backend.log`) ;
  - frontend : `check`, `lint-i18n-ownership`, `test:unit`, `build` verts (`front-*.log`, première série, sans base) ;
  - E2E complet : **478 passés, 12 échecs, 18 ignorés** ; échecs jugés contre la liste de `docs/testing.md` (sept de la
    KF-029 #97 ; KF-065 #1282 : `backup-restore-new-install.spec.ts:262` et `:381` ; KF-067 #1284 :
    `opening-balances.spec.ts:156` — soit les dix attendus), plus **deux échecs de connexion**, `invoice-qr-caracteres:90`
    et `invoices:645` (`toHaveURL('/')` dépassé, la page reste sur `/login`, station chargée), hors de cette liste, sans
    lien avec le plan comptable ; les deux parcours `accounts-mirror` verts. *(Annoté le 2026-11-21, revue P1 L7 : ces
    deux-là ont été rejoués seuls par `heavy.sh` — backend E2E sur la base de la place — : 2 passés, `e2e-rejeu.log`.
    Le rapport de la P1 notait aussi que la liste de `docs/testing.md` de cette branche ne porte pas encore KF-065 et
    KF-067 : le rattachement des trois échecs vient de la liste de l'orchestrateur, non relu ici.)* ;
  - `scripts/gate-ci.sh` (copie `git archive`, base recréée) : **6212 passés, 0 échec, 18 ignorés** (`gate-ci.log`) ; sa
    cible supprimée.
- **Gates de la remédiation P1 — dernier commit de code `9f2ae083`**, cible propre `kesh-targets/kesh-wt47`,
  `CARGO_INCREMENTAL=0`, par `heavy.sh` (journaux `kesh-gate-logs/s47-2/`, `pipeline3.log`) ; les gates ci-dessus (à
  `90c4b5aa`) sont rejoués, non repris :
  - ciblé (`cible2-*.log`) : `fmt` vert, `clippy --workspace --all-targets -D warnings` vert, nextest `accounts_e2e`,
    `textes_coherents`, `account_mirror` : **67 passés** ; Vitest `src/lib/features/accounts` et `src/routes/(app)/accounts` +
    `i18n-keys` : 11 fichiers, **134 passés** ; `npm run check` 0 erreur (29 avertissements, hors de ces fichiers) ;
    `lint-i18n-ownership` vert ;
  - backend `scripts/test-fast.sh --ci` : **6193 exécutés, 6193 passés** (2 lents), 14 ignorés (`gate-backend-p3.log` ;
    inchangé à 6193 : aucun test Rust neuf). ⚠️ Le premier essai (dans `pipeline3`) est mort à l'instant, `mem-guard`
    n'atteignant pas le bus utilisateur de systemd (« Failed to connect to user scope bus ») ; rejoué avec
    `KESH_NO_MEMGUARD=1` (le plafond mémoire étant déjà posé par `heavy.sh`) ;
  - frontend : `check` 0 erreur, `lint-i18n-ownership`, `test:unit` **281 fichiers, 4658 passés**, `build` verts ;
  - E2E complet : **480 passés, 10 échecs, 18 ignorés** (`e2e-p3.log`) : les dix attendus de la liste — sept de la
    KF-029 (`mode-expert` ×2, `onboarding-path-b` ×2, `onboarding` ×3), `backup-restore-new-install.spec.ts:262` et `:381`,
    `opening-balances.spec.ts:156` — et **aucun échec de connexion** cette fois ; `invoice-qr-caracteres.spec.ts:90` et
    `invoices.spec.ts:645` ont aussi été rejoués seuls (`e2e-rejeu.log`) : **2 passés** ; les deux parcours
    `accounts-mirror` verts ;
  - `scripts/gate-ci.sh` : **6212 passés, 0 échec, 18 ignorés** (`gate-ci.log`). Deux essais précédents rendus invalides
    et conservés (`gate-ci-1-echec-place.log`, `gate-ci-2-collision-sqlx-test.log`) : le premier visait un serveur de
    test d'une autre place que celui de son URL (le script de gate ferme l'environnement et perd `KESH_DB_PORT` ; serveur
    et port fixés sur `kesh-mariadb-dev2` dans l'enveloppe) ; le second est tombé sur « Unknown database '_sqlx_test_… » (4
    tests de `users_e2e`, collision de harnais décrite aux consignes), rejoué en entier. Sa cible a été supprimée ;
  - capture 07 refaite sans toast (`captures-p3.log`, `captures/07-dialogue-390x844.png` : le dialogue du compte 2800,
    refus des fonds propres sous le champ, *Annuler* visible).

### File List

- `crates/kesh-api/src/routes/accounts.rs`
- `crates/kesh-api/tests/accounts_e2e.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `frontend/src/lib/features/accounts/AccountMirrorField.svelte` (neuf)
- `frontend/src/lib/features/accounts/mirror.ts` (neuf)
- `frontend/src/lib/features/accounts/mirror.test.ts` (neuf)
- `frontend/src/lib/features/accounts/accounts.types.ts`
- `frontend/src/routes/(app)/accounts/+page.svelte`
- `frontend/src/routes/(app)/accounts/accounts-mirror.test.ts` (neuf)
- `frontend/tests/e2e/accounts-mirror.spec.ts` (neuf)
- `frontend/src/lib/shared/i18n-keys.test.ts`
- `docs/api-external.md`, `docs/manual/fr/user-manual.tex`, `docs/manual/fr/admin-manual.tex`
- `_bmad-output/implementation-artifacts/47-2-miroir-api-et-ecran-du-plan.md`, `sprint-status.yaml`
- Reprise de `main` (`aef10456`) : `crates/kesh-db/src/entities/{account,mod}.rs`, `crates/kesh-db/src/repositories/accounts.rs`,
  `crates/kesh-api/src/routes/accounts.rs`, `crates/kesh-db/tests/{account_mirror,account_parent_change,account_currency_repository,credit_card_repository,invoice_write_off,invoices_validate_vat,letterings,supplier_invoices_repository}.rs`,
  `crates/kesh-api/tests/journal_entry_reversal_e2e.rs`


### Clôture (2026-11-21, orchestrateur)

Boucle de revue : P1 complète Opus (0/0/0/8 LOW, corrigés en `9f2ae083`) → P2 ciblée Haiku sur `9f2ae083` (0/0/0/3 LOW,
`kesh-gate-logs/s47-2/revue-p2-rapport.md`). Gates au dernier commit de code `9f2ae083` : backend complet 6193 passés /
14 ignorés, frontend 281 fichiers / 4658, E2E 480 passés / 10 échecs attendus / 18 ignorés, gate-ci 6212 / 0 / 18.
LOW de la P2 laissés en dette (aucun comportement faux) : P2-L1 commentaire de l'archivage (`routes/accounts.rs:598-606`,
la raison « sans cycle » est `guard_mirror_update`, non l'absence de verrou sur A) ; P2-L2 doc publique de `create`
(refus du miroir et verrou de la cible) ; P2-L3 ordre des refus à la création non fixé par un test. À reprendre à la
story de solde de l'Epic 47. ⚠️ Un gate de cette story a tourné sans plafond mémoire (`KESH_NO_MEMGUARD=1`) pendant la
panne du bus systemd de la nuit ; rejoué depuis sous plafond.
