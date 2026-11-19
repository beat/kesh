# Story 47-5 : solde de la dette de l'Epic 47

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md` — règle « une story de solde à la fin de chaque
epic » (`CLAUDE.md` § *Priorités des défauts*, puce « Une story de solde à la fin de chaque epic »). Fin d'epic, avant
le tag. **Dépend de** la clôture des stories 47-1 à 47-4 ; part du **registre de la dette** du fichier d'epic
(§ *Registre de la dette de l'epic*). Dans l'ordre des coupures, elle sort la première.

## Modules touchés (règle de splitting préventif)

Selon les issues retenues (deux au plus) ; au plus cinq modules, sinon la ligne n'est pas retenue.

## Story

En tant que mainteneur de Kesh,
je veux solder, avant la release, la dette ouverte par les revues des stories de l'Epic 47, dans la limite fixée au
kickoff,
afin que l'epic ne fasse pas monter le compte d'issues plus que nécessaire.

## Critères d'acceptation

0. **Entrée** : le registre de la dette du fichier d'epic porte, pour chaque story 47-1 à 47-4, les findings non
   remédiés de ses revues (chaque fiche sœur l'alimente à la clôture de sa boucle, § *References*), et le Project
   Lead ouvre les issues d'après lui (l'agent de Beat n'en ouvre pas) ; la liste des lignes (story d'origine, passe,
   sévérité, objet, numéro d'issue s'il existe, priorité, étiquettes) est copiée au Change Log. **Le registre, non
   la liste des issues, fait foi** : une ligne dont l'issue n'est pas encore ouverte reste candidate (AC 1). Le
   registre **vide** à la date de la story, la 47-5 est close sans code, le constat écrit (AC 3 et 5 sans objet ;
   le décompte de l'AC 4 s'écrit quand même).
1. **Plafond : deux issues**, fixé au kickoff (fichier d'epic, § *Décomptes d'issues*) : au plus deux lignes du
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
   ouverte et l'état des issues vérifié après la fusion par le Project Lead (l'agent de Beat n'a ni PR ni `gh`).

## Tâches / sous-tâches

- [ ] **T0** (AC 0) : relire le registre de la dette et les numéros d'issues fournis par le Project Lead ; reporter au Change Log.
- [ ] **T1** (AC 1, 2) : proposer au plus deux lignes selon le critère, choix et motif écrits ; décision du Project Lead reportée.
- [ ] **T2** (AC 3) : corriger, tester (rouge puis vert), i18n, manuel `.tex`, note de version ; boucle de revue.
- [ ] **T3** (AC 4, 5) : gates, E2E, décompte ; `closes #N` préparés pour le corps de la PR de release.

## Dev Notes

Fichiers selon les issues retenues. Une issue qui exigerait une migration sort du solde (elle irait à un epic
ultérieur). Un LOW de revue qui touche la production peut aussi se corriger dans sa story d'origine, sur passe ciblée,
plutôt qu'attendre ce solde.

### References

- `CLAUDE.md` § *Priorités des défauts* ; § *Issue Tracking Rule* ; § *Cadence* (coupure : n'entrent que les stories closes).
- Précédent d'une story de solde : la 48-14 de la release 0.20.0 (plafond de quatre issues), `release-0.20.0.md`.
- Fichier d'epic : § *Décomptes d'issues*, § *Registre de la dette de l'epic*, § *Ordre de développement*.

## Notes de version

Selon les issues retenues (une ligne par correctif visible).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log

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
