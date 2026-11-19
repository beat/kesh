# Story 47-5 : solde de la dette de l'Epic 47

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md` — règle « une story de solde à la fin de chaque
epic » (`CLAUDE.md` § *Priorités des défauts*, *Tarir la source, puis solder*). Fin d'epic, avant le tag.

## Story

En tant que mainteneur de Kesh,
je veux solder, avant la release, la dette ouverte par les revues des stories de l'Epic 47, dans la limite fixée au
kickoff,
afin que l'epic ne fasse pas monter le compte d'issues plus que nécessaire.

## Critères d'acceptation

1. **Plafond : deux issues**, fixé au kickoff (fichier d'epic) : au plus deux issues ouvertes par les revues des
   stories 47-1a à 47-4 sont traitées ; le choix (priorité, puis effet visible) est écrit au Change Log avec la liste
   de toutes les issues de l'epic.
2. Les issues non retenues restent ouvertes et tracées (« on trace toujours ») ; aucune n'est fermée sans correctif.
3. Chaque correctif vient avec son test (rouge avant, vert après) et suit la boucle de revue
   (`CLAUDE.md` § *Review Iteration Rule*).
4. **Gates complets** (backend, frontend) et **E2E complet** au dernier commit de code ; le compte d'issues et de bugs
   au moment de la story est écrit (relevé par le Project Lead si `gh` n'est pas accessible à l'agent).
5. Les issues traitées se ferment par le corps de la PR de release (`closes #N`), jamais par un message de commit.

## Tâches / sous-tâches

- [ ] Recenser les issues ouvertes par les revues de l'epic (liste fournie par le Project Lead si besoin).
- [ ] Retenir au plus deux ; corriger, tester, revoir.
- [ ] Gates, E2E, décompte.

## Dev Notes

Fichiers selon les issues retenues. Une issue qui exigerait une migration sort du solde (elle irait à un epic
ultérieur). Un LOW de revue qui touche la production peut aussi se corriger dans sa story d'origine, sur passe ciblée,
plutôt qu'attendre ce solde.

### References

- `CLAUDE.md` § *Priorités des défauts* ; § *Issue Tracking Rule*.

## Notes de version

Selon les issues retenues (une ligne par correctif visible).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
