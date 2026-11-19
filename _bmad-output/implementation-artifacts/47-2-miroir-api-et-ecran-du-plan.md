# Story 47-2 : compte miroir — API des comptes et écran du plan comptable

Status: drafted

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
   inexistante), création avec miroir, RBAC (la boucle de `consultation_role_cannot_write` couvre un corps portant
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
   ne corrige pas — le serveur reste juge, 47-1 AC-4 et AC-5) : comptes de la société, actifs (non archivés), autres
   que le compte lui-même, de l'autre côté du type **choisi dans le formulaire** (actif ↔ passif, produit ↔ charge),
   **non postables** (la case *Postable* décochée), sans rôle, qui n'ont pas eux-mêmes de miroir ; libellés « numéro
   nom », dans l'ordre du plan. Le miroir enregistré figure toujours dans la liste, même s'il ne passe plus le filtre.
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
   code) s'affiche **sous le `<select>`** (`role="alert"`, `aria-invalid`), en création et en modification ;
   `ACCOUNT_IS_MIRROR` à la modification (rendre postable, retyper, poser un rôle) prend le chemin existant
   (`editError`, au pied du formulaire : aucun code neuf) ; au dialogue *Archiver le compte*, `ACCOUNT_IS_MIRROR`
   (`archive`) s'affiche **dans le dialogue** (état `archiveMirrorMessage`, `role="alert"`,
   `data-testid="account-archive-mirror"`, patron de `archiveDesignated`), **sans** « Archiver quand même » **et sans**
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
8. **E2E** (fichier neuf `frontend/tests/e2e/accounts-mirror.spec.ts`) : comptes **créés par l'API** à numéros uniques
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
   CO, sans compensation ni écriture ; **sans dire que le bilan de Kesh y est « conforme »** : l'avertissement du bilan
   (`:3913`, « pas encore structuré selon l'art. 959a CO ») reste vrai) ; créer le compte de présentation (*Postable*
   décoché, sans rôle) puis le choisir ; plusieurs comptes peuvent
   partager un miroir ; ce qui est refusé tant qu'un compte est miroir (écriture, case *Postable*, retypage, rôle,
   archivage) ; **ne pas choisir comme miroir un compte que les réglages de facturation, un article ou une règle de
   rapprochement désignent** : le flux qui l'emploierait serait refusé, nommé (constat promis par la 47-1, *Risques*) ; les exemples de l'**exemple chiffré du fichier d'epic** (comptes **à créer** 6900.1 « Pertes de
   change », 1040.1 « Créances AVS/AI/APG », 2100.1 « Découvert bancaire », comptes livrés 6950, 1020, 2270 sous leur
   nom), et, pour les comptes en devise, la désignation de 6950.1 comme compte des gains **et** des pertes de change
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
      miroir, et le sens inverse (sa sauvegarde dans la release précédente) tel que la recette de montée l'a constaté. Gardes G4-bis/ter, G19, G25 respectées.
    - `.tex` seulement (aucun PDF commité) ; **le prompt de chaque passe de revue nomme ces documents** et exige le
      contrôle du PDF compilé localement (`pdftotext … | tr '\n' ' ' | tr -s ' '`).
11. **i18n** : clés `accounts-…` (préfixe du dossier, `lint-i18n-ownership`) dans les quatre locales ; `sitesTotal` de
    `frontend/src/lib/shared/i18n-keys.test.ts` **recompté** depuis la garde après la reprise de `main` ;
    `i18n-libelle-en-dur` et `fx-account-filter-inventory.test.ts` verts ; aucun numéro de compte en dur dans un
    catalogue ou un repli (variables Fluent).
12. **Inventaire** (47-1 AC-11) : rien à ajouter — la 47-1 ne tient plus de liste close des fichiers qui nomment
    `mirror_account_id` (validation P1 de la 47-1, 2026-11-19 : la garde au point de passage des lignes suffit).

## Tâches / sous-tâches

- [ ] **T0** : reprise de `main` ; relire au code livré par la 47-1 les codes d'erreur et les raisons (les messages sont
  ceux du serveur : aucune clé de refus à reprendre) ; `fix/api-parent-compte`
  (#1351) — s'il est fusionné, le champ *Compte parent* précède le `<select>` ; sinon le second arrivé résout.
- [ ] **T1 — route** (AC 1, 12) : champs, réponse, rejeu de l'archivage sur interblocage ; tests `accounts_e2e.rs`.
- [ ] **T2 — front** (AC 2–5) : types, `mirror.ts`, `<select>`, phrase, refus, archivage.
- [ ] **T3** : Vitest, E2E, captures (AC 6–8).
- [ ] **T4** : manuels, `api-external.md`, i18n (AC 9–11).
- [ ] **T5** : gates backend, frontend et E2E complets au dernier commit de code.

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

### Debug Log References

### Completion Notes List

### File List
