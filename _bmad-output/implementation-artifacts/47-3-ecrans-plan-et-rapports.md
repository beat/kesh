# Story 47-3 : écrans — plan comptable et rapports

Status: draft

Epic : `epic-47-comptes-miroirs.md`. Dépend de 47-2.

## Intention

Poser le lien à l'écran du compte ; lire, dans les rapports, d'où vient une ligne présentée.

## Critères d'acceptation

1. Formulaire de compte : section *Présentation* — « Compte miroir » ou « Compenser avec » (sélecteur filtré sur les
   comptes admis), retrait ; refus de l'API affichés au champ ; à la création d'une paire, **avertissement** CO 958c
   à confirmer (Q9) ; archivage d'une cible refusé avec le compte lié nommé (Q10).
2. Plan comptable : marque discrète sur un compte lié.
3. Bilan et compte de résultat : marque sur une ligne présentée ; sources au survol ou au dépliage ; bascule
   « Présentation brute ».
4. Captures d'écran de chaque état (thème clair) ; Vitest ; E2E du parcours (lien posé → bilan sans négatif → brut).
5. **Manuel utilisateur** (`docs/manual/fr/user-manual.tex`) : poser un miroir ou une paire, ce que montrent le bilan
   et le compte de résultat, l'avertissement de la paire (CO 958c) et la mention « présentation compensée », comment
   obtenir la présentation brute, l'exemple chiffré du fichier d'epic ; i18n 4 locales, `sitesTotal` recompté.

## Tâches

- [ ] `AccountForm`, types, sélecteur ; vues de rapports.
- [ ] Tests, E2E, manuel.

## Documentation (exigence de Guy, 2026-11-19)

- Manuel utilisateur (`docs/manual/fr/user-manual.tex`) : critère 5.

Manuels FR (`docs/manual/fr/`, `.tex` seulement, PDF régénérés à la release) ; s'ils sont traduits d'ici la livraison, le mécanisme y figure aussi. **Le prompt de chaque passe de revue de cette story nomme ces manuels** et exige le contrôle du PDF (`CLAUDE.md` § *Le prompt d'une passe doit NOMMER le manuel*).

## Fichiers

`frontend/src/lib/features/accounts/`, `routes/(app)/accounts/+page.svelte`, `frontend/src/lib/features/reports/`,
`frontend/tests/e2e/`, `docs/manual/fr/user-manual.tex`. Conflits attendus : Epic 46, `fix/bilan-resultat-reporte`,
`fix/api-parent-compte`.

## Note de version

Écrans du lien de présentation et des rapports présentés.
