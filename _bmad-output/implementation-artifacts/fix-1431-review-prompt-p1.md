# Prompt — revue de code P1 du correctif #1431 (parent à la création et à la réactivation), passe complète

Tu relis le correctif **#1431** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit 5), pour la release
**v0.21.0**. Kesh tient des livres réels : un défaut qui écrit faux ou qui bloque une facture se paie cher. **Aucune
migration.** Contexte frais, première passe complète, rapport en français.

Worktree : `/home/beat/claude/kesh/.claude/worktrees/kesh-1431`, branche `fix/1431-parent-creation-reactivation`. Diff à
relire, **aplati** : `git -C <worktree> diff origin/main..HEAD` (le dernier commit de code est indiqué au Change Log de la
fiche ; les commits suivants ne portent que la fiche et ce prompt — vérifie-le par `git -C <worktree> log --stat
origin/main..HEAD`).

Lis d'abord la fiche `_bmad-output/implementation-artifacts/fix-1431-parent-creation-reactivation.md` (Intention, Contrat
retenu, choix C-1431-1 à 4, Tests, Notes de version, Dev Agent Record), puis le code. Ne prends pas le Dev Agent Record
pour argent comptant : recoupe-le avec les journaux (lecture seule) sous `/home/beat/claude/kesh-gate-logs/s1431/`
(`t1-new-tests.log`, `t2-affected.log`, `t3-fixed.log`, `mutations.log`, `mut/*.log`, `mutations.py`, le texte aplati du
PDF du manuel compilé sur une copie : `user-manual.txt`, journal `manual-make.log`).

## Axes (déclare au rapport ceux que tu as exercés et ceux que tu n'as pas exercés)

1. **Contrat.** Codes, statuts, `details` et messages d'`ACCOUNT_PARENT_HAS_ENTRIES` (désormais aussi à la création et à
   la réactivation) et d'`ACCOUNT_PARENT_DESIGNATED` (neuf) sur `POST /accounts`, `PUT /accounts/{id}`, `PUT
   /accounts/{id}/reactivate`, `POST /accounts/groups` (`details.groupStep`). Ordre des refus. Conventions voisines de
   `crates/kesh-api/src/errors.rs` (`ACCOUNT_PARENT_BANK_LINKED`, `ACCOUNT_CURRENCY_IN_USE`, `ACCOUNT_DESIGNATED_IN_SETTINGS`).
   Les membres d'un groupe restent acceptés. Un chemin qui rend un compte parent et **échappe** encore à la garde ?
   (cherche **tous** les sites qui écrivent `postable = FALSE` sur un parent ou qui posent `parent_id`.)
2. **Transaction et courses.** La garde est-elle dans la transaction, sous le `FOR UPDATE` du parent ? Ordre des verrous
   de `reactivate` (réglages de facturation ajoutés en partage) contre `company_invoice_settings::update`,
   `products::create`/`update`, la liaison d'un compte bancaire, une écriture concurrente sur le parent. Interblocages
   possibles, rejeu (`retry_app_on_deadlock`) ou non. Une désignation posée **pendant** le geste est-elle vue ?
3. **DRY.** Une seule garde partagée (`refuse_unfit_parent_in_tx`) ? Duplication restante, code mort, commentaires
   devenus faux (greper `#1431`, « écart », « plus strict que la création », « ne le refuse pas » dans tout le dépôt —
   code, tests, docs, manuel, frontend).
4. **Docs et manuel contre le code.** `docs/api-external.md` (paragraphe des comptes en devise, paragraphe du `PUT`
   `parentId`, paragraphe de `POST /accounts/groups`, tableau des erreurs § 10) et `docs/manual/fr/user-manual.tex` :
   disent-ils ce que fait le code, ni plus ni moins ? Contrôle le **PDF** (texte aplati fourni), pas seulement le `.tex`.
   Les ⚠️ de changement de contrat sont-ils là où une réponse change ? Quatre locales cohérentes entre elles et avec les
   replis du code ?
5. **Tests qui échouent pour la bonne raison.** Chaque test neuf rougit-il sous la mutation de la garde qu'il couvre
   (`mutations.log`) ? Un test passe-t-il à vide (montage qui ne crée pas la situation visée) ? Le cas qui doit passer
   est-il couvert ? `postable` et `version` du parent vérifiés après chaque refus ? Les quatre tests retournés
   (« #1431 inchangée ») le sont-ils pour la bonne raison ?

Sévérités : CRITICAL, HIGH, MEDIUM, LOW. Pour chaque finding : fichier et ligne lus, extrait, pourquoi, correctif proposé.
Un « 0 finding » sans liste des axes exercés ne compte pas.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport
  `/home/beat/claude/kesh-gate-logs/s1431/review-p1.md`. Aucun fichier temporaire ; aucune redirection (`>`, `>>`, `tee`,
  `2>`, `&>`). Un fichier créé par erreur se déclare au rapport, il ne se supprime pas.
- Aucun `rm`, `pkill`, `killall`, `kill`.
- Aucun `git` qui écrit (`checkout`, `switch`, `restore`, `stash`, `commit`, `reset`, `merge`, `rebase`, `push`) ; aucun
  `gh`.
- Aucun interpréteur ni rien qui compile ou exécute : ni `python3`, `perl`, `node`, `npm`, `npx`, `bash -c`, `sh -c`, `jq`,
  `cargo`, `make`, `latexmk`, `docker`, `mariadb`, `sqlx`, `curl`, `sed -i` ; ni **`scripts/prepare-release.sh`**,
  `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs`
  (ils se lisent).
- Ne lis rien hors du worktree et de `/home/beat/claude/kesh-gate-logs/s1431/`. Aucune donnée réelle.
