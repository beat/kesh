# Story 47-3a : écran du plan comptable — poser un miroir ou une paire

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md` — D2, D3 ; Q8, Q9, Q10 ; FR-47-1 (écran),
FR-47-6, FR-47-7, FR-47-8 (manuel utilisateur, partie « poser un lien »). Née du découpage de l'esquisse 47-3
(2026-11-19) ; les écrans des rapports, l'E2E du parcours complet et le reste du manuel : **47-3b**. Dépend de la
**47-1b** (API). Maquettes : formulaire d'approbation hors dépôt (`data-not-committed/epic-47-stories-revue.html`,
§ M1 à M3). Interagit avec le correctif **#1351** (`fix/api-parent-compte`, champ *Compte parent* du même dialogue,
non fusionné à `fd04099f`).

## Story

En tant que comptable,
je veux, dans le dialogue de modification d'un compte, choisir son compte miroir ou le compte avec lequel il forme une
paire, être averti avant de créer une paire, et comprendre pourquoi un archivage est refusé,
afin de poser ces liens sans passer par l'API.

## Critères d'acceptation

1. **Section « Présentation dans les rapports »** du dialogue *Modifier le compte* (`routes/(app)/accounts/+page.svelte`),
   **en dernier** des champs (sous `AccountCurrencyFields` en multi-devises, et sous le champ *Compte parent* de #1351
   s'il est fusionné). Absente du dialogue de création (47-1b AC-8). Elle a **quatre états**, calculés par une
   fonction pure (`presentationSectionState`, AC-2) depuis le compte enregistré **et** l'état du formulaire
   (`editType`, `editRole`), testés chacun en Vitest :
   - **choix** : un groupe de trois boutons radio — **Aucun** · **Compte miroir** · **Compenser avec (paire)** — puis,
     pour les deux derniers, un sélecteur de compte. À l'ouverture, le choix et le compte sont ceux du lien
     enregistré (le partenaire ou la cible actuels figurent toujours dans le sélecteur, même s'ils ne passeraient plus
     le filtre) ;
   - **fonds propres** : le rôle **choisi dans le formulaire** est de fonds propres (`EquityCapital`, `EquityOther`,
     `RetainedEarnings`, `CurrentYearResult`, liste de la 47-1a AC-5) et le compte n'a pas de lien : pas de choix,
     texte « Les comptes de fonds propres ne se présentent pas autrement. » ; s'il a un lien, l'état **choix** reste
     affiché (l'API refusera le rôle, AC-4, et le refus dit quoi défaire) ;
   - **cible** : le compte est la cible d'un ou de plusieurs miroirs (une source au moins, archivée comprise, pointe
     vers lui) : pas de choix (un compte joue au plus un rôle, 47-1a AC-5 `chain`), texte « Ce compte présente le
     solde de sens inverse de 2270, 2271. Pour le lier autrement, retirez d'abord ces liens. » ;
   - **lecture impossible** : l'index des liens (AC-6) n'a pas pu être lu : pas de choix, texte d'erreur ; aucun
     envoi des champs de lien.
   Quand le **type** choisi dans le formulaire change et que le compte choisi dans la section ne passe plus le filtre
   pour ce type, le choix revient à l'état enregistré (aucun lien inventé ni retiré en silence) ; l'API juge l'état
   résultant (47-1a AC-5, AC-7) et ses refus s'affichent (AC-4).
2. **Fonctions pures** dans un fichier neuf `frontend/src/lib/features/accounts/presentation-links.ts` (et non
   `account-options.ts`, que #1351 modifie), testées seules (`presentation-links.test.ts`) :
   - `presentationLinkCandidates(index, account, kind, formType)` : les comptes admis par la règle de la 47-1a AC-5 —
     même société (la liste l'est), **actifs**, autres que le compte lui-même, **autre côté** du même rapport pour un
     miroir (actif ↔ passif, produit ↔ charge), **type opposé** du même rapport pour une paire, hors rôle de fonds
     propres, hors comptes qui jouent déjà un autre rôle (source d'un miroir, cible d'un miroir — **sources archivées
     comprises** —, membre d'une paire avec un tiers) ; **admis** : compte non imputable (compte de présentation),
     compte en devise étrangère, de devise quelconque (Q3) ; le lien actuel toujours rendu. Table de vérité : quatre
     types × deux natures × rôles × états (actif, archivé, déjà lié). La règle reste celle de l'API, qui seule fait
     foi ;
   - `presentationSectionState(index, account, formType, formRole)` : l'état de l'AC-1 ;
   - `presentationLinkFieldsForUpdate(account, choice)` : les champs du `PUT`, **seulement ce qui change** (patron
     `currencyFieldsForUpdate` et `parentFieldForUpdate` de #1351) — rien si le lien choisi est le lien enregistré ;
     `{ mirrorAccountId: id }` ou `{ offsetAccountId: id }` pour poser ou déplacer ; passer d'un miroir à une paire (ou
     l'inverse) n'envoie **que** le nouveau champ (le dépôt remplace le lien, `CHECK` « pas les deux ») ; **Aucun**
     envoie `null` sur le seul champ du lien enregistré ; jamais les deux champs à un entier (`bothLinks`). Les champs
     de la réponse sont **absents** sans lien (47-1b AC-3) : toujours lus `?? null` ;
   - `isNewOffsetPair(account, choice)` : vrai si le choix est une paire et que le compte n'avait pas de partenaire
     ou en avait un autre (même définition que la 47-1b AC-4).
   Texte d'aide sous le sélecteur, relié par `aria-describedby` (miroir : « Quand le solde de ce compte est de sens
   inverse, il est présenté sur le compte choisi, de l'autre côté. » ; paire : « Seul le net des deux comptes paraît,
   sur le compte du sens du net. »).
3. **Avertissement de la paire (Q9, CO 958c al. 1 ch. 7)** : à l'enregistrement, si `isNewOffsetPair`, un dialogue
   de confirmation s'ouvre **avant** l'appel, au-dessus du dialogue de modification qui reste ouvert (même empilement
   que le dialogue de retypage, mais ouvert avant l'envoi et non sur un 409) : titre « Compenser 6960 et 6961 ? »
   (numéros en variables), texte selon les types — produit et charge : la compensation d'une charge et d'un produit
   n'est pas admise dans les comptes annuels ; actif et passif : celle d'un actif et d'un passif non plus (CO 958c
   al. 1 ch. 7 les interdit toutes deux) ; puis : Kesh présentera le net ; la présentation brute reste disponible pour
   les comptes officiels. Le texte est **celui du message `warnings` de la 47-1b AC-4** — mêmes clés du catalogue
   (`crates/kesh-i18n/locales/*/messages.ftl`, que lit aussi `i18nMsg`), aucune phrase recopiée. Boutons **Annuler**
   (focus à l'ouverture, `autofocus` comme le dialogue de retypage ; il ferme l'avertissement sans rien envoyer et
   laisse la saisie intacte) et **Compenser quand même**. **Aucun** avertissement pour un miroir, un retrait, ni une
   paire inchangée (enregistrer le nom d'un compte déjà en paire). Si l'envoi confirmé rencontre ensuite
   l'avertissement de retypage (`ACCOUNT_HAS_ENTRIES`), la paire n'est pas redemandée (`putAccount` porte les champs
   de lien dans les deux appels). La réponse `warnings` n'affiche rien de plus (la question a été posée).
4. **Refus de l'API** : `PRESENTATION_LINK_INVALID` s'affiche **sous le sélecteur** (`role="alert"`, sélecteur
   `aria-invalid`, comme `FX_ACCOUNT_INVALID` dans `AccountCurrencyFields`) avec **`err.message`**, déjà traduit par le
   serveur et qui nomme le compte qui occupe le rôle (47-1b AC-2) — aucun catalogue de raisons côté écran ;
   `ACCOUNT_PRESENTATION_LINKED` (`reason` `accountType` ou `equityRole`, au retypage ou à la pose d'un rôle de fonds
   propres, y compris après la confirmation du retypage) s'affiche au pied du dialogue (`editError`), le message
   nommant les comptes liés. Lecture des `details` validée, non supposée (`readPresentationLinked`, patron
   `readDesignatedSettings` / `readRetypeImpact`) ; `details` illisible : le seul message.
5. **Archivage refusé (Q10)** : le dialogue *Archiver le compte* affiche, sur `ACCOUNT_PRESENTATION_LINKED`
   (`reason: archive`), un avis `role="alert"` (`data-testid="account-archive-presentation-linked"`) qui nomme le ou
   les comptes liés (`details.linkedAccounts`) et dit le remède **selon le lien et l'état de chaque compte lié** :
   - cible d'un miroir : « Retirez d'abord le lien depuis le compte 2270. » (une ligne par source) ;
   - membre d'une paire : « Retirez d'abord la paire, depuis ce compte ou depuis le compte 6961. » ;
   - compte lié **archivé** (`linkedAccounts[].active` à `false`, 47-1b AC-2 ; une source archivée garde son lien,
     47-1a AC-7, et un compte archivé ne se modifie pas) : « Le compte 2270 est archivé : réactivez-le, retirez le
     lien, puis archivez-le de nouveau. »
   Pour chaque compte lié **actif** présent dans la liste, un bouton « Ouvrir le compte 2270 » ferme le dialogue
   d'archivage et ouvre son dialogue *Modifier* (`openEdit`) — le plan comptable n'a pas d'adresse par compte. **Aucun
   bouton « Archiver quand même »** : ce refus remet `archiveDesignated` à `null` (il peut suivre une première
   question de désignation si un lien a été posé entre-temps) et le pied ne garde que **Annuler**.
6. **Marque au plan comptable** : un compte lié porte une pastille discrète (patron des pastilles de devise et de
   « Non postable », `data-testid="account-row-{number}-presentation-badge"`) — « Miroir → 1179 », « Miroir de 2270,
   2271 » (au-delà de trois sources : « Miroir de 2270, 2271 et 2 autres »), « Paire ↔ 6961 » — avec `title`
   explicatif ; la flèche est `aria-hidden` et doublée d'un texte `sr-only` (« vers », « de », « compensé avec »), le
   `title` n'étant pas lu de façon fiable par les lecteurs d'écran. **Source des liens** : la page tient un **index des
   liens** construit sur le plan **archivés compris** (`fetchAccounts(true)`, ou la liste déjà chargée quand *Afficher
   les comptes archivés* est coché), indépendant du filtre d'affichage : la pastille d'une cible et le filtre de l'AC-2
   comptent les sources archivées (la liste affichée par défaut, `fetchAccounts(showArchived)`, ne les contient pas).
   Rechargé après chaque enregistrement, archivage et réactivation.
7. **Types et API front** : `accounts.types.ts` — `AccountResponse.mirrorAccountId?: number`,
   `offsetAccountId?: number` (absents = sans lien, comme `fxGainAccountId`) ; `UpdateAccountRequest` les deux champs
   à trois états (`?: number | null`) ; type de la réponse du `PUT` avec `warnings?: { code; message }[]` (réponse du
   `PUT` seulement, non de la liste) ; `readPresentationLinked(details)` (`reason`, `linkedAccounts[{ id, number,
   name, active }]`). Vitest : fonctions de l'AC-2, lecteur des `details`, états du dialogue (quatre états, avertissement
   ouvert ou non, refus au champ, refus d'archivage sans « Archiver quand même »).
8. **Accessibilité** : le choix est un `<fieldset>` à `<legend>` (« Présentation dans les rapports »), trois
   `<input type="radio">` étiquetés (patron du `fieldset` des comptes de change d'`AccountCurrencyFields`), navigables
   au clavier ; le sélecteur a son `<label>` ; aide par `aria-describedby` ; erreur au champ par `role="alert"` et
   `aria-invalid` ; dialogue d'avertissement à titre et description (`Dialog.Title`, `Dialog.Description`), focus sur
   **Annuler** ; le dialogue *Modifier*, plus haut avec la devise, le parent (#1351) et cette section, **défile**
   (`Dialog.Content` à `max-h-[90vh] overflow-y-auto`, patron de `contacts/+page.svelte`) : le bouton *Enregistrer*
   reste atteignable sur un écran de téléphone.
9. **Captures d'écran** (thème clair) de chaque état : section vide (choix *Aucun*), miroir choisi, paire choisie,
   état *cible*, état *fonds propres*, avertissement de la paire (produit/charge et actif/passif), refus au champ,
   archivage refusé (source active, source archivée), pastilles ; plus le dialogue *Modifier* en multi-devises à
   **390 × 844** (téléphone) et **1280 × 720** — jointes au Dev Agent Record et au prompt de revue
   (`kesh-gate-logs/`).
10. **E2E** dans un **fichier neuf** `frontend/tests/e2e/accounts-presentation-links.spec.ts` (non `accounts.spec.ts`,
    que #1351 étend) : comptes **créés par l'API** à numéros uniques (patron `D${Date.now()…}` d'`accounts.spec.ts`),
    jamais de lien sur un compte du seed — un lien laissé sur la base partagée `kesh_e2e` bloquerait l'archivage dans
    une autre spec ; liens retirés en `finally`. Parcours : poser un miroir (pastilles des deux côtés) ; poser une
    paire — *Annuler* n'envoie rien (aucun `PUT` observé), *Compenser quand même* enregistre ; état *cible* d'un compte
    miroir ; archivage refusé nommant le compte lié, sans « Archiver quand même » ; retrait du lien, puis archivage
    accepté.
11. **Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, section *Plan comptable* `:421`) :
    - nouvelle sous-section « Présenter un compte de l'autre côté : miroirs et paires » (`\label{sec:miroirs-paires}`),
      placée après *Modification d'un compte* (`:465` ; `\label{sec:modification-compte}` posé par #1351 s'il est
      fusionné) : poser, déplacer, retirer un miroir ou une paire ; quels comptes le sélecteur propose et pourquoi
      (autre côté, type opposé, fonds propres exclus, un compte joue un seul rôle, compte de présentation non imputable
      admis, devise quelconque) ; l'état *cible* ; l'avertissement CO 958c ; la pastille du plan ;
    - à *Modification d'un compte*, le refus du **retypage** et d'un **rôle de fonds propres** sur un compte lié (comme
      le paragraphe « Un compte lié à un compte bancaire »), sans confirmation qui le lève ;
    - à *Suppression d'un compte* (`:486`, où sont les autres refus d'archivage et de réactivation), le **refus
      d'archivage** d'une cible ou d'un membre de paire, sans « Archiver quand même », et le cas d'une source archivée
      (réactiver, retirer le lien, ré-archiver) ; archiver une source reste admis ;
    - le renvoi aux rapports présentés (`\ref` vers la sous-section de la 47-3b) est **écrit par la 47-3b**, qui pose
      le label : la 47-3a n'écrit aucun `\ref` vers un label qui n'existe pas encore (PDF sans « ?? »).
    Le mot « miroir » est celui de l'écran (Q8). `.tex` seulement ; **le prompt de chaque passe de revue nomme le
    manuel** et exige le contrôle du PDF compilé localement (`pdftotext … | tr '\n' ' ' | tr -s ' '`).
12. **i18n** : clés dans les quatre locales ; `sitesTotal` de `i18n-keys.test.ts` **recompté** depuis la garde après
    la reprise de `main` (#1351 le relève de 4) ; `lint-i18n-ownership`, `i18n-libelle-en-dur` verts. ⚠️ Aucun numéro de compte en dur dans un catalogue, un repli ou le manuel hors plans livrés : G4-bis et G4-ter (`textes_coherents.rs`) lisent les quatre catalogues et le manuel utilisateur — « Compenser 6960 et 6961 ? » se compose par variables Fluent, et les exemples du manuel citent des comptes des trois plans livrés (cf. 47-2a AC-11). *(Ajouté à la validation P1 de la 47-3b.)*

## Tâches / sous-tâches

- [ ] **T0** : reprise de `main`. Si #1351 y est, partir de son dialogue (champ parent, `parentFieldForUpdate`) ; sinon,
  ne pas fusionner sa branche : développer sur `main`, la section en dernier du formulaire et les fonctions dans un
  fichier neuf pour que les hunks soient distincts, et le second arrivé résout en gardant les deux champs du `PUT`
  (`CLAUDE.md` § *Travailler à plusieurs développeurs*). Relire le contrat livré par la 47-1b (`linkedAccounts[].active`,
  clés du message `warnings`).
- [ ] **T1** : types, `readPresentationLinked`, `presentation-links.ts` + tests (AC 2, 7).
- [ ] **T2** : index des liens, section du dialogue (quatre états), aide, refus au champ, défilement (AC 1, 2, 4, 6, 8).
- [ ] **T3** : dialogue d'avertissement (AC 3) ; archivage refusé (AC 5) ; pastilles (AC 6).
- [ ] **T4** : Vitest, E2E, captures (AC 7, 9, 10).
- [ ] **T5** : manuel, i18n (AC 11, 12). **T6** : gates frontend et E2E complets au dernier commit de code.

## Dev Notes

### Code lu (à `origin/main` `fd04099f`)

- `frontend/src/routes/(app)/accounts/+page.svelte` (951 lignes) : `:209` (`loadAccounts`, `fetchAccounts(showArchived)`
  — archivés exclus par défaut), `:290` (`openEdit`), `:324` (`putAccount`, `currencyFieldsForUpdate`), `:349`
  (`submitEdit`, refus `FX_ACCOUNT_INVALID` au champ, `ACCOUNT_HAS_ENTRIES`), `:392` (`confirmRetype`), `:448-480`
  (`submitArchive`, cas `ACCOUNT_DESIGNATED_IN_SETTINGS`, repli `toast.error`), `:569-645` (lignes et pastilles du
  plan ; boutons *Modifier* / *Archiver* sur les seuls comptes actifs), `:746-826` (dialogue *Modifier*, sans
  `max-h`/défilement, `AccountCurrencyFields`), `:832-875` (dialogue d'avertissement de retypage, `autofocus` sur
  *Annuler*), `:877-931` (dialogue d'archivage, « Archiver quand même » affiché dès que `archiveDesignated` n'est pas
  `null`).
- `frontend/src/lib/components/ui/dialog/dialog-content.svelte` (`sm:max-w-sm`, centré, sans défilement) ; défilement
  ailleurs : `contacts/+page.svelte:828` (`max-h-[90vh] … overflow-y-auto`).
- `frontend/src/lib/features/accounts/AccountCurrencyFields.svelte` (`fieldset`/`legend`, radios, erreur au champ
  `role="alert"` + `aria-invalid`), `fx-accounts.ts` (`fxDifferenceAccountOptions`, `currencyFieldsForUpdate` : patrons
  de l'AC-2), `account-options.ts`, `accounts.types.ts:45-87` (`AccountResponse`, champs facultatifs absents),
  `:187` (`readRetypeImpact`), `:218` (`readDesignatedSettings`), `accounts.api.ts` (`updateAccount` rend
  `AccountResponse`).
- `frontend/src/lib/shared/utils/api-client.ts:307` : `err.message` est celui du corps, traduit par le serveur.
- Mise à jour refusée sur un compte archivé : `crates/kesh-db/src/repositories/accounts.rs:899`
  (`IllegalStateTransition`) — d'où le remède de l'AC-5 pour une source archivée.
- Catalogue : `crates/kesh-i18n/locales/fr-CH/messages.ftl:240-265` (`accounts-edit-*`, `accounts-archive-*`,
  `accounts-retype-*`, `account-field-*`).
- E2E : `frontend/tests/e2e/accounts.spec.ts` (285 lignes ; `beforeAll` qui ré-sème, comptes créés par l'API à numéro
  unique, l. 38-72).
- Manuel : `docs/manual/fr/user-manual.tex:421` (*Plan comptable*), `:465` (*Modification d'un compte*, paragraphe
  « Un compte lié à un compte bancaire »), `:486` (*Suppression d'un compte*, refus de réactivation).
- #1351 (`origin/integration/fix-api-parent-compte`, `b8f9524c`) : champ *Compte parent* dans le dialogue *Modifier*
  (`editParentId`, `parentCandidates`, `parentFieldForUpdate` dans `account-options.ts`), E2E dans `accounts.spec.ts`,
  `sitesTotal` relevé de 4, `\label{sec:modification-compte}` au manuel.

### Stratégie de tests

Vitest (fonctions pures, lecteur des `details`, quatre états, avertissement, refus), E2E du parcours, captures ;
mutations (à consigner) : afficher l'avertissement pour un miroir ou une paire inchangée ; laisser « Archiver quand
même » après un refus `archive` qui suit une question de désignation ; construire l'index des liens sur la seule liste
affichée (la pastille d'une cible à source archivée doit rougir) ; envoyer les deux champs à un entier au passage
miroir → paire.

### Risques

Conflit avec #1351 (même dialogue, même `+page.svelte`, même `PUT`, `sitesTotal`) ; hauteur du dialogue (AC-8, à juger
aux captures de téléphone).

### References

- Fiches 47-1a (AC-5, AC-7), 47-1b (AC-2, AC-3, AC-4) ; maquettes M1 à M3 du formulaire d'approbation ; `CLAUDE.md`
  § *Recette visuelle*, § *Le prompt d'une passe doit NOMMER le manuel*.

## Notes de version

Le dialogue d'un compte permet de choisir son **compte miroir** ou de le **compenser avec** un autre compte (avec un
avertissement avant de créer une paire) ; le plan comptable marque les comptes liés ; un archivage refusé nomme le
compte lié et dit comment lever le refus.

## Change Log

- 2026-11-19 — validation P1 (Opus 5.5) : 0 C / 1 H / 9 M / 5 L, remédiés. H : remède de l'archivage refusé
  impossible quand la source est archivée (elle garde son lien, un compte archivé ne se modifie pas) et « lien vers ce
  compte » sans adresse au plan (AC-5 réécrite ; `active` ajouté à `linkedAccounts` dans les 47-1a AC-7 et 47-1b
  AC-2). M : texte CO 958c réduit à charge/produit alors qu'une paire actif/passif existe (D3), et recopié au lieu de
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

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
