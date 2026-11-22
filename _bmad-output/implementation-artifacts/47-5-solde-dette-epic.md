# Story 47-5 : solde de la dette de l'Epic 47

Status: done

Epic : `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md` — règle « une story de solde à la fin de chaque
epic » (`CLAUDE.md` § *Priorités des défauts*, puce « Une story de solde à la fin de chaque epic »). Fin d'epic, avant
le tag. **Dépend de** la clôture des stories 47-1 à 47-4 ; part du **registre de la dette** du fichier d'epic
(§ *Registre de la dette de l'epic*). Dans l'ordre des coupures, elle sort la première.

## Modules touchés (règle de splitting préventif)

Selon les issues retenues (deux au plus) ; au plus cinq modules, sinon la ligne n'est pas retenue. *(Annoté le 2026-11-22, revue de code P1, L6 : la décision du responsable de l'epic du 22.11 — corriger aussi les LOW reportés des 47-1 et 47-2 — a fait dépasser les cinq modules au second temps (kesh-api, kesh-db, kesh-i18n, kesh-report, manuel, E2E, audit des migrations) ; le plafond de **deux issues** tient, lui, pour les issues ouvertes ou à ouvrir.)*

## Story

En tant que mainteneur de Kesh,
je veux solder, avant la release, la dette ouverte par les revues des stories de l'Epic 47, dans la limite fixée au
kickoff,
afin que l'epic ne fasse pas monter le compte d'issues plus que nécessaire.

## Critères d'acceptation

0. **Entrée** : le registre de la dette du fichier d'epic porte, pour chaque story 47-1 à 47-4, les findings non
   remédiés de ses revues (chaque fiche sœur l'alimente à la clôture de sa boucle, § *References*), et le Project
   Lead ouvre les issues d'après lui (l'agent du responsable de l'epic n'en ouvre pas) ; la liste des lignes (story d'origine, passe,
   sévérité, objet, numéro d'issue s'il existe, priorité, étiquettes) est copiée au Change Log. **Le registre, non
   la liste des issues, fait foi** : une ligne dont l'issue n'est pas encore ouverte reste candidate (AC 1). Le
   registre **vide** à la date de la story, la 47-5 est close sans code, le constat écrit (AC 3 et 5 sans objet ;
   le décompte de l'AC 4 s'écrit quand même).
1. **Plafond : deux issues** *(annoté le 2026-11-22 : le second temps, décidé par le responsable de l'epic le 22.11, a corrigé les LOW reportés des 47-1 et 47-2 en plus des deux lignes retenues ; le plafond vaut pour les issues, non pour ces corrections)*, fixé au kickoff (fichier d'epic, § *Décomptes d'issues*) : au plus deux lignes du
   registre (issues ouvertes ou à ouvrir) sont traitées. **Critère de choix**, dans l'ordre : priorité (P1 avant
   P2…), puis effet visible pour l'utilisateur, puis le plus ancien. Une ligne qui exigerait une **migration** ou
   sortirait du périmètre de l'epic n'est pas retenue (elle va à un epic ultérieur, et le dit). La story **propose**
   le choix et son motif au Change Log, avec la liste de **toutes** les lignes du registre ; **le Project Lead
   décide** et rattache les issues retenues au milestone de la release (`CLAUDE.md` § *Planification* : rattacher
   une issue à une release est sa décision, les contributeurs proposent ; précédent : les quatre issues de la 48-14,
   choisies par Guy). Une ligne corrigée avant l'ouverture de son issue porte au registre le sort « corrigée en
   47-5 » : le Project Lead ne l'ouvre pas.
2. Les issues non retenues restent ouvertes et tracées (« on trace toujours ») ; aucune n'est fermée sans correctif,
   et aucune n'est fermée pour tenir le chiffre.
3. Chaque correctif vient avec son test (**rouge avant, vert après**, constat écrit) et suit la boucle de revue
   (`CLAUDE.md` § *Review Iteration Rule*), première passe non ciblée. Un correctif visible met à jour, dans la même
   story, sa ligne des *Notes de version* et, **le cas échéant**, les quatre catalogues i18n (texte changé ;
   `sitesTotal` recompté) et le `.tex` du manuel qui le décrit (aucun PDF commité ; il se contrôle au gate de
   release).
4. **Gates complets** (backend, frontend) et **E2E complet** au dernier commit de code (échecs attendus jugés contre
   `docs/testing.md`), `crates/kesh-db/` comprise le cas échéant (gate complet même en boucle) ; le compte d'issues
   et de bugs au moment de la story est écrit (relevé par le Project Lead si `gh` n'est pas accessible à l'agent),
   à côté du compte du tag v0.20.0 (tag précédent, relevé par le Project Lead) et de l'objectif de l'epic (fichier d'epic, § *Décomptes d'issues*).
5. Les issues traitées se ferment par le corps de la PR de release (`closes #N`), jamais par un message de commit
   (grep des mots-clés sur `git log` avant la PR) ; les lignes `closes #N` sont préparées par la story, la PR est
   ouverte et l'état des issues vérifié après la fusion par le Project Lead (l'agent du responsable de l'epic n'a ni PR ni `gh`).

## Tâches / sous-tâches

- [x] **T0** (AC 0) : relire le registre de la dette et les numéros d'issues fournis par le Project Lead ; reporter au Change Log.
- [x] **T1** (AC 1, 2) : proposer au plus deux lignes selon le critère, choix et motif écrits ; décision du Project Lead reportée.
- [x] **T2** (AC 3) : corriger, tester (rouge puis vert), i18n, manuel `.tex`, note de version ; boucle de revue.
- [x] **T3** (AC 4, 5) : gates, E2E, décompte ; `closes #N` préparés pour le corps de la PR de release.

## Dev Notes

Fichiers selon les issues retenues. Une issue qui exigerait une migration sort du solde (elle irait à un epic
ultérieur). Un LOW de revue qui touche la production peut aussi se corriger dans sa story d'origine, sur passe ciblée,
plutôt qu'attendre ce solde.

### References

- `CLAUDE.md` § *Priorités des défauts* ; § *Issue Tracking Rule* ; § *Cadence* (coupure : n'entrent que les stories closes).
- Précédent d'une story de solde : la 48-14 de la release 0.20.0 (plafond de quatre issues), `release-0.20.0.md`.
- Fichier d'epic : § *Décomptes d'issues*, § *Registre de la dette de l'epic*, § *Ordre de développement*.

## Notes de version

- **Changed** — Manuel utilisateur, comptes miroirs : la phrase de l'accueil précise que l'écart entre les barres et
  le compte de résultat ne naît que d'un solde **de sens inverse** ; la figure de la balance montre les noms du plan
  PME. (Aucun changement d'écran ni d'API ; le journal du serveur nomme l'invariant rompu si la présentation miroir
  devait se replier sur le brut.)
- **Changed** — Comptes miroirs : le refus d'archiver ou de modifier un compte miroir désigné par plusieurs comptes
  s'écrit au pluriel (« réactivez-les d'abord », quatre langues) ; l'epic n'étant pas publié, aucune release antérieure
  n'avait la phrase au singulier pour plusieurs sources.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (développement, 2026-11-21).

### Debug Log References

Journaux hors dépôt, dossier `s47-5/` des journaux de gate du contributeur : `chaine.log` (ordre et codes de retour),
`cible-*.log`, `mut-*.log` (sondes par mutation), `manuel-make.log` et `manuel-plat.txt` (PDF compilé dans une copie),
`gate-backend.log`, `front-*.log`, `e2e.log`, `e2e-backend.log`, `gate-ci.log`, `gate-ci-commit.txt`, `captures/`.

### Completion Notes List

**T0 — registre (AC 0).** Le registre du fichier d'epic était vide : les revues des 47-1 à 47-4 avaient consigné leur
dette dans leurs fiches (47-1, 47-2) ou dans la liste des issues à ouvrir tenue hors dépôt (47-3, 47-4), non au
registre. Reporté au registre (§ *Registre de la dette de l'epic*), huit lignes, regroupées comme les issues à ouvrir
(le Project Lead peut les scinder, une issue par problème) :

| Story | Passe | Sévérité | Objet | Issue | Sort |
|---|---|---|---|---|---|
| 47-1 | P1 | LOW | phrase « réactivez-le d'abord » au singulier pour plusieurs sources (L5), visible | non ouverte (`bug`, P4) | corrigée en 47-5 (second temps) |
| 47-1 | P1 | LOW | liste des sources du refus non bornée (L5, second point) | à ouvrir (`enhancement`, P4) | reportée (décision d'interface) |
| 47-1 | P1 | LOW ×7 | L1 course, L2 doc-comments, L3 commentaire `Set`, L4 verrous, L6 test `update_in_tx`, L7 rédaction, L8 sauvegarde 0.20.0, L9 table G9 | non ouverte (`technical-debt`, P4) | corrigée en 47-5 (second temps) |
| 47-2 | P2 | LOW ×3 | commentaire du rejeu de l'archivage, doc de `create`, ordre des refus non testé | non ouverte (`technical-debt`, P4) | corrigée en 47-5 (second temps) |
| 47-3 | P2 | LOW ×3 | `presentation.rs` : `ecart_conserve` non testé ; message de repli ; filtre de type non tué | non ouverte (`technical-debt`, P4) | corrigée en 47-5 |
| 47-4 | P1 | LOW ×4 | manuel : phrase de l'accueil, « 959a CO attend », figure `mi-balance.png`, rangement des clés `.ftl` | non ouverte (`documentation`, P4) | corrigée en 47-5 |
| 47-4 | gate E2E | — | `bank-import.spec.ts:130`, course (compte avant chargement) | #1283 (`known-failure`, P3) ; `backup-restore-new-install.spec.ts` : #1282 | reportée |
| 47-3 (L-5) | décision du point e | — | avertissement non bloquant, miroir sous un groupe d'un autre type | à ouvrir (`enhancement`, P3) | reportée |

(L7 de la 47-1 — deux échecs E2E hors liste — est depuis identifié : #1282 ; seule la rédaction est reprise.)

**T1 — choix (AC 1, 2).** Plafond de deux issues : les lignes 47-3 P2 et 47-4 P1, une issue chacune telle que la liste des
issues à ouvrir les groupe, désignées par la consigne de l'orchestrateur ; aucune n'exige de migration. **Écart au
critère à signaler au Project Lead** : à priorité égale (P4), le critère place d'abord l'effet visible, puis le plus
ancien ; la ligne 47-1 L5 (phrase d'erreur au singulier pour plusieurs sources, visible) passerait avant la ligne
47-3 P2 (aucun effet visible). Elle reste en issue, hors plafond, comme les deux autres lignes des 47-1 et 47-2, qui
n'étaient pas connues de la consigne. La course de `bank-import.spec.ts:130` est de priorité plus haute (P3) mais
vient d'une spec antérieure à l'epic, hors de son périmètre ; la décision du point e n'est pas prévue par la fiche et dépend des groupes de l'Epic 46 (entrée
d'issue complétée dans la liste des issues à ouvrir). Choix donné par la consigne de l'orchestrateur, à confirmer par
le Project Lead ; les deux lignes corrigées ne sont pas à ouvrir (marquées « corrigée en 47-5 » dans la liste).

**T2 — correctifs (AC 3).**
- `crates/kesh-report/src/presentation.rs` : le contrôle des totaux présentés passe dans `controler_bilan`, qui rend
  l'équation présentée et l'invariant rompu (`DefautBilan::Ecart`, contrôlé le premier, ou `DefautBilan::Equation`) ;
  le repli sur le brut se fait dans les mêmes cas qu'avant ; le journal porte un message propre à l'invariant et les
  champs `ecart_brut` / `ecart_presente`. Tests neufs : `controle_du_bilan_nomme_l_invariant_rompu` (quatre cas et les
  deux phrases) et `miroir_du_meme_type_que_la_source_sans_effet` (`reclasser` rend `None`, rapport égal au brut).
  **Rouge sans le code** : quatre sondes par mutation dans une copie `git archive` hors dépôt, chacune rouge (1 échec
  sur 10 tests de `presentation`) : contrôle d'écart retiré, contrôle d'équation retiré, messages égaux, filtre
  `mirror_account_type != section_type` retiré ; copie intacte verte (10/10). Comptés : 8 → 10 `#[test]` dans
  `presentation.rs` (de `6878a873` à `HEAD`).
- `docs/manual/fr/user-manual.tex` : la phrase de l'accueil dit que l'exception ne naît que d'un solde **de sens
  inverse** (un miroir dont la source garde un solde normal ne change rien) ; « la présentation que le bilan de
  l'art. 959a CO attend » devient « la présentation usuelle ». PDF compilé dans une copie (`make user`, 3 passes) et
  aplati : les deux phrases y sont, « 959a CO attend » n'y est plus (0). La figure de la balance a été remplacée après
  cette compilation : le PDF contrôlé montre encore l'ancienne image, le texte est le même.
- `frontend/tests/e2e/reports-mirror.spec.ts` : 1100, 3000 et 4000 du seed prennent aussi leur nom du plan PME
  (« Débiteurs », « Ventes de marchandises », « Achats de marchandises »), aucune assertion n'en dépend ; la capture
  reprise de l'E2E complet remplace `docs/manual/fr/screenshots/mi-balance.png` (vue : noms du plan PME, soldes
  inchangés ; l'exercice du seed reste « Exercice CI 2020-2030 »).
- Catalogues i18n (quatre locales) : les deux clés `reports-mirror-of-*` et leur commentaire déplacés, valeurs
  inchangées, en bloc après `reports-archived-label` ; `sitesTotal` inchangé (3515).
- `frontend/tests/e2e/invoice-qr-caracteres.spec.ts` : `tjDe` au lieu de la chaîne reconstruite (L6 de la 47-4).
- Rien de visible à l'écran ni à l'API : aucune capture d'écran neuve hors la figure ; ligne des *Notes de version*
  écrite (manuel).

**T2 bis — second temps (décision du responsable de l'epic du 22.11).** Les LOW reportés des 47-1 (revue P1) et 47-2
(revue P2) se corrigent dans la 47-5 quand ils sont faciles et sans risque ; le plafond de deux issues du kickoff est
ainsi dépassé, par cette décision. Corrigés :
- **47-1 L5** — `crates/kesh-api/src/errors.rs` : `account_is_mirror_reactivate_message` rend la clé et le repli selon
  le nombre de sources **archivées** : `error-account-is-mirror-reactivate-first` (une) ou
  `error-account-is-mirror-reactivate-first-many` (plusieurs, neuve dans les quatre catalogues : « sont archivés :
  réactivez-les d'abord, puis retirez leur compte miroir », DE, IT, EN accordés). Test : deux sources archivées dans
  `mirror_refusals_map_to_their_status_code_and_details`. Sondes hors dépôt (`mut2-*.log`) : toujours le singulier,
  rouge ; toujours le pluriel, rouge ; copie intacte verte. La borne de la liste des sources (second point de L5) reste
  en issue : choisir une borne et la forme du reste est une décision d'interface.
- **47-1 L9** — les treize clés des refus du miroir entrent dans la garde commune G9
  (`crates/kesh-api/tests/textes_coherents.rs`, paramètres `{ $number }`, `{ $sources }`, `{ $archived }`) ; la
  comparaison des replis, doublon, est retirée du test d'`errors.rs`, qui garde la présence dans les quatre catalogues
  et l'absence de chiffre.
- **47-1 L2, 47-2 P2-L2** — doc-comments de `create` (refus du miroir juste après `MULTI_CURRENCY_DISABLED`, avant le
  parent ; verrou de la cible avant celui du parent), d'`update` (compte miroir puis parent, avant les refus
  existants) et d'`archive_guarded` (compte miroir entre gain / perte / conversion et désignation), lus contre le code.
- **47-1 L3** — commentaire de `guard_mirror_update` : un lien forgé reposé tel quel n'est pas revalidé, sans effet
  comptable ; tests neufs dans `les_regles_de_la_source_se_rejouent_quand_le_miroir_reste` : miroir reposé tel quel
  vers une cible forgée accepté, retypage refusé (`accountType`), cible introuvable sans effet sur un retypage.
- **47-1 L4** — commentaire de la garde des lignes (`journal_entries.rs`) : le verrou d'intervalle et l'attente
  possible d'une création de compte, raisonnés, non mesurés (la revue recommandait de documenter).
- **47-1 L6** — `aucune_ecriture_ne_se_passe_sur_un_compte_miroir` exerce aussi `journal_entries::update` : une ligne
  déplacée vers le miroir est refusée (`ACCOUNT_NOT_POSTABLE`, « 1179 »).
- **47-1 L1** — `course_pose_contre_archivage` attend les **deux** témoins, chacun par le texte de sa requête
  (`CIBLE_DE_LA_POSE`, `CIBLE_DE_L_ARCHIVAGE`) ; `course_ecriture_contre_pose` attend la requête de la cible, non la
  lecture de la source.
- **47-1 L7** — annotation datée au Dev Agent Record de la 47-1 (échecs attendus par récurrence, non par la liste ;
  depuis tracés en #1282), sans réécrire le relevé.
- **47-1 L8** — `docs/migrations-idempotence-audit.md` : « testé en 0.19.0, la 0.20.0 par la même règle, non testée »
  (le fichier de migration n'est pas touché).
- **47-2 P2-L1** — commentaire du rejeu de l'archivage (`routes/accounts.rs`) : la raison « sans cycle » est que la
  pose verrouille la cible avant d'écrire le lien, et qu'aucun geste qui tient la source n'attend la cible quand le
  lien existe déjà.
- **47-2 P2-L3** — test neuf `a_la_creation_le_refus_du_miroir_passe_avant_celui_du_parent` (parent compte de gains de
  change et cible imputable : `MIRROR_ACCOUNT_INVALID` ; sans miroir : `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`).
- Comptés (de `6878a873` à `HEAD`) : `account_mirror.rs` 11 → 12 tests `#[sqlx::test]` (un neuf, trois étendus) ;
  `errors.rs` 38 → 38 (un étendu).
- Grep des symptômes : « réactivez-le » (code, quatre locales, manuels, `docs/api-external.md` : aucune autre copie),
  « 0.19.0 ou 0.20.0 » (une autre copie : la note de version de la 47-1, laissée telle quelle — vraie par la règle
  de la colonne facultative, elle ne prétend pas à un test).

**T3 — gates (AC 4, 5).** Premier temps — dernier commit de code `7fdb0fa9` (le commit suivant, `fef4de33`, ne porte que la figure PNG
tirée de l'E2E) :
- gate ciblé (sans base) : `cargo fmt` rouge au premier passage (formatage des tests neufs), corrigé par `rustfmt`
  avant le commit ; clippy workspace vert ; `kesh-report` lib 151 passés, 1 ignoré ;
- backend complet (`scripts/test-fast.sh --ci`, base remise à zéro) : fmt et clippy verts, nextest **6252 passés,
  15 ignorés** ;
- frontend : `check` 0 erreur (29 avertissements, préexistants), `lint-i18n-ownership` vert, Vitest **4871 passés
  (284 fichiers)**, `build` vert ;
- E2E complet (`LANG=fr_CH.UTF-8`) : **494 passés, 10 échecs, 18 ignorés** ; les dix sont attendus : les sept de la
  KF-029 (#97 ; `mode-expert` ×2, `onboarding-path-b` ×2, `onboarding` ×3), `backup-restore-new-install.spec.ts:262`
  et `:381` (#1282), `opening-balances.spec.ts:156` ; `reports-mirror.spec.ts` et `invoice-qr-caracteres.spec.ts`
  verts ; `bank-import.spec.ts:130` vert à ce passage ;
- `scripts/gate-ci.sh` sur une copie `git archive` de `7fdb0fa9` : **interrompu** (code 143) sur la décision du
  responsable de l'epic du 22.11 (gate-ci et E2E complet joués une seule fois, sur la branche d'intégration de tous les
  epics) ; avant l'arrêt : fmt, build et clippy verts, 39 binaires de test joués sur environ 260, 1483 passés,
  0 échoué, 7 ignorés. **Le gate-ci n'est donc pas joué jusqu'au bout pour cette story** : il reste dû à
  l'intégration.
- L'E2E complet ci-dessus a été joué avant cette décision ; la nouvelle règle (E2E des seules specs ajoutées) ne
  demandait ici aucun E2E (aucune spec neuve, deux specs modifiées).
- **Second temps — dernier commit de code `acf670b8`** (règle des gates de la nuit du 22.11 : fmt, clippy, nextest complet,
  frontend complet ; E2E des seules specs ajoutées ; pas de gate-ci) : backend complet (`scripts/test-fast.sh --ci`,
  base remise à zéro ; un premier essai est tombé sur un serveur de test arrêté avant tout test, rejoué) — fmt et
  clippy verts, nextest **6253 passés, 15 ignorés** (`gate-backend-2.log`) ; frontend — `check` 0 erreur,
  `lint-i18n-ownership` vert, Vitest **4871 passés (284 fichiers)**, `build` vert ; **E2E : aucun** (le second temps
  n'ajoute ni ne modifie aucune spec) ; gate-ci et E2E complet dus à la branche d'intégration de tous les epics.
- Décompte d'issues : non relevé (`gh` hors de portée) ; à relever par le Project Lead à côté du compte au tag v0.20.0
  et de l'objectif de l'epic (§ *Décomptes d'issues*). Aucune issue à fermer par cette story (les deux lignes traitées
  n'ont pas été ouvertes) : aucune ligne `closes #N` à préparer. Messages de commit sans mot-clé de fermeture.

### File List

- `crates/kesh-api/src/errors.rs`, `crates/kesh-api/src/routes/accounts.rs`, `crates/kesh-api/tests/textes_coherents.rs`
- `crates/kesh-db/src/repositories/accounts.rs`, `crates/kesh-db/src/repositories/journal_entries.rs`,
  `crates/kesh-db/tests/account_mirror.rs`
- `docs/migrations-idempotence-audit.md`, `_bmad-output/implementation-artifacts/47-1-miroir-modele-et-depot.md`
  (annotation datée)
- `crates/kesh-report/src/presentation.rs`
- `crates/kesh-i18n/locales/{de-CH,en-CH,fr-CH,it-CH}/messages.ftl`
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/screenshots/mi-balance.png`
- `frontend/tests/e2e/reports-mirror.spec.ts`, `frontend/tests/e2e/invoice-qr-caracteres.spec.ts`
- `_bmad-output/implementation-artifacts/47-5-solde-dette-epic.md`, `_bmad-output/implementation-artifacts/sprint-status.yaml`,
  `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md`

### Change Log

- 2026-11-21 — développement (Opus 5.5) : registre reporté (huit lignes), deux corrigées (47-3 P2, 47-4 P1), cinq reportées en issues, une ligne déjà tracée hors epic ;
  gates au dernier commit de code `7fdb0fa9` (backend, frontend, E2E complet verts ; gate-ci interrompu, dû à
  l'intégration) ; revue de code P1 à lancer (prompt
  `s47-5/revue-p1-prompt.md` du dossier des journaux).

- 2026-11-19 — validation P1 (Sonnet 5.5) : 0 C / 0 H / 0 M / 1 L, remédié (en-tête *Modules touchés* ajouté). Références (`release-0.20.0.md`, 48-14, § Cadence, § Décomptes, § Registre) vérifiées à `origin/main`.
- 2026-11-19 — validation P1 (Sonnet 5.5) : 0 C / 0 H / 4 M / 3 L, remédiés (voir le rapport de passe).
- 2026-11-19 — validation P2 (Opus 5.5) : 0 C / 0 H / 4 M / 4 L, remédiés (rapport
  `kesh-gate-logs/epic-47-validation/47-5-solde-dette-epic-P2.md`). Trois des quatre MEDIUM viennent de la remédiation
  P1 : registre que rien n'alimentait dans les fiches sœurs (ligne ajoutée aux *References* des 47-1a à 47-4),
  entrée par les seules issues ouvertes (le registre fait foi désormais), choix des issues confié à la story alors
  qu'il revient au Project Lead ; le quatrième porte sur la règle de coupure du fichier d'epic (qui coupe un epic de contributeur,
  sort d'une 47-5 sortie, epic qui attend entier plutôt que de sortir amputé).
- 2026-11-19 — validation P3 (Sonnet 5.5, ciblée sur la remédiation P2) : 0 C / 0 H / 0 M / 3 L, remédiés (rapport `kesh-gate-logs/epic-47-validation/47-5-solde-dette-epic-P3.md`).
- 2026-11-19 — **Refonte du 2026-11-19 (miroir seul, KISS)** : l'epic passe à quatre stories avant le solde (47-1
  modèle et dépôt, 47-2 API et écran du plan, 47-3 présentation, 47-4 écrans des rapports, manuel et E2E ; K10 retiré) ;
  renvois « 47-1a à 47-4 » devenus « 47-1 à 47-4 » ; plafond inchangé (deux issues). Six critères (inchangés).
  **Statut remis à `drafted`** en attendant la revalidation.
- 2026-11-22 — second temps (Opus 5.5), décision du responsable de l'epic du 22.11 : les LOW reportés des 47-1 (L1 à
  L9) et 47-2 (P2-L1 à P2-L3) corrigés, sauf la borne de la liste des sources du refus `ACCOUNT_IS_MIRROR` (issue,
  décision d'interface) ; registre et liste des issues à ouvrir mis à jour ; gates au dernier commit de code `acf670b8`
  (backend 6253, frontend 4871, aucun E2E) ; revue de code P1 à lancer sur le nouveau HEAD (prompt réécrit).
- 2026-11-22 — revue de code P1 (Sonnet 5.5) : 0 C / 0 H / 0 M / 6 L (rapport `kesh-gate-logs/s47-5/revue-p1-rapport.md`), remédiés sans toucher la production : commentaires d'`accounts.rs` (ordre des refus, verrous), test de la clause « miroir, toujours » sur une cible imputable, tests du repli et de l'écart dans les deux sens de `present_balance_sheet`, numéros d'issue du registre, annotations du plafond. Les chemins fixes `/tmp/kesh-e2e-47-4-*` de `reports-mirror.spec.ts` (47-4 L6), sans effet en série, sont notés ici faute de mieux. Gates ciblés au dernier commit de code : fmt, clippy workspace, `kesh-report` lib 152 passés, binaire `account_mirror` 13 passés ; sondes par mutation hors dépôt (`mut3-*.log`) : clause miroir limitée au flux automatique, repli de `present_balance_sheet` jamais pris et `!=` devenu `>`, toutes trois tuées par un test en échec. Story `done`.
