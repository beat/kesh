# Story 47-5 : solde de la dette de l'Epic 47

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md` — règle « une story de solde à la fin de chaque
epic » (`CLAUDE.md` § *Priorités des défauts*, puce « Une story de solde à la fin de chaque epic »). Fin d'epic, avant
le tag. **Dépend de** la clôture des stories 47-1a à 47-4 ; part du **registre de la dette** du fichier d'epic
(§ *Registre de la dette de l'epic*). Dans l'ordre des coupures, elle sort la première.

## Story

En tant que mainteneur de Kesh,
je veux solder, avant la release, la dette ouverte par les revues des stories de l'Epic 47, dans la limite fixée au
kickoff,
afin que l'epic ne fasse pas monter le compte d'issues plus que nécessaire.

## Critères d'acceptation

0. **Entrée** : le registre de la dette du fichier d'epic porte, pour chaque story 47-1a à 47-4, les findings non
   remédiés de ses revues, et le Project Lead en a ouvert les issues (l'agent de Beat n'en ouvre pas) ; la liste
   (numéro, story d'origine, sévérité, priorité, étiquettes) est copiée au Change Log. Sans issue ouverte à la
   date de la story, la 47-5 est close sans code (AC 4 sans objet), le constat écrit.
1. **Plafond : deux issues**, fixé au kickoff (fichier d'epic, § *Décomptes d'issues*) : au plus deux issues ouvertes
   par les revues des stories 47-1a à 47-4 sont traitées. **Critère de choix**, dans l'ordre : priorité (P1 avant
   P2…), puis effet visible pour l'utilisateur, puis le plus ancien. Une issue qui exigerait une **migration** ou
   sortirait du périmètre de l'epic n'est pas retenue (elle va à un epic ultérieur, et le dit). Le choix et son
   motif sont écrits au Change Log, avec la liste de **toutes** les issues de l'epic.
2. Les issues non retenues restent ouvertes et tracées (« on trace toujours ») ; aucune n'est fermée sans correctif,
   et aucune n'est fermée pour tenir le chiffre.
3. Chaque correctif vient avec son test (**rouge avant, vert après**, constat écrit) et suit la boucle de revue
   (`CLAUDE.md` § *Review Iteration Rule*), première passe non ciblée. Un correctif visible met à jour, dans la même
   story, les quatre catalogues i18n, le `.tex` du manuel concerné (aucun PDF commité ; il se contrôle au gate de
   release) et sa ligne des *Notes de version*.
4. **Gates complets** (backend, frontend) et **E2E complet** au dernier commit de code (échecs attendus jugés contre
   `docs/testing.md`), `crates/kesh-db/` comprise le cas échéant (gate complet même en boucle) ; le compte d'issues
   et de bugs au moment de la story est écrit (relevé par le Project Lead si `gh` n'est pas accessible à l'agent),
   à côté du compte du tag v0.20.0 et de l'objectif de l'epic.
5. Les issues traitées se ferment par le corps de la PR de release (`closes #N`), jamais par un message de commit
   (grep des mots-clés sur `git log` avant la PR), et leur état se vérifie après la fusion.

## Tâches / sous-tâches

- [ ] **T0** (AC 0) : relire le registre de la dette et la liste d'issues fournie par le Project Lead ; reporter au Change Log.
- [ ] **T1** (AC 1, 2) : retenir au plus deux selon le critère ; écrire le choix et le motif.
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

- 2026-11-19 — validation P1 (Sonnet 5.5) : 0 C / 0 H / 4 M / 3 L, remédiés (voir le rapport de passe).
