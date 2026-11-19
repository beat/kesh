# Story 46-2 : écrans du bilan et du compte de résultat par groupes

Status: draft

*Revue de Beat (2026-11-19) : approuvée, avec les exigences ajoutées ci-dessous ; approbation de Guy attendue.*

*Esquisse du 2026-11-18. Epic 46, D3, D9. Suit 46-1. Aucune migration.*

## Story

En tant que comptable, je veux choisir à l'écran le niveau de détail du bilan et du compte de résultat, afin de voir
d'un coup d'œil mes groupes ou le détail de chaque compte.

## Critères d'acceptation (esquisse)

1. **AC-1 — Sélecteur.** « À plat · Niveau 1 · … · Tous les comptes » au-dessus du bilan et du compte de résultat ;
   les niveaux proposés vont jusqu'à la profondeur réelle du plan.
2. **AC-2 — Rendu.** Groupes en gras avec leur sous-total, retrait par profondeur, ligne « solde propre » ; solde en
   devise d'un groupe quand l'API le donne ; lien vers le grand livre sur les seules lignes de compte.
3. **AC-3 — À plat inchangé.** Le mode « à plat » rend l'écran actuel, sous-titres de rôle compris (D4).
4. **AC-4 — Mémoire.** Le niveau choisi est retenu par visiteur (`localStorage`, lecture et écriture sous `try`) et
   transmis aux exports (niveau, position des sous-totaux).
5. **AC-5 — Hiérarchie visible (Q10, Beat).** La hiérarchie est visible par défaut, **accordéons ouverts par défaut** ;
   contrôles « **tout ouvrir / tout fermer** » et « **hiérarchie masquée / montrée** » (masquée : retraits et
   accordéons supprimés ; sens exact à préciser à la validation). L'état est retenu par visiteur.
6. **AC-6 — Position des sous-totaux (Beat).** Choix, à l'écran, de la position du sous-total d'un groupe : **sur la
   ligne du titre, au-dessus des comptes correspondants**, ou **au-dessous des comptes, par une ligne de sous-total
   supplémentaire**. Le choix est transmis aux exports (46-3) et pilotable par l'API et le MCP (46-1, 46-4).
7. **AC-7 — Total d'un groupe non affiché (Q2).** Un groupe non affiché laisse son total à son parent ; solde en
   devise : devise + CHF si une seule devise, sinon CHF seul, aucune ligne supplémentaire (Q3).
8. **AC-8 — Maquette.** Une maquette des deux positions de sous-total et des contrôles est jointe au Dev Agent Record
   avant le développement des écrans.
9. **AC-9 — Textes et tests.** Quatre locales ; Vitest des vues ; un E2E Playwright traverse un changement de niveau ;
   captures d'écran (thème clair) au Dev Agent Record ; manuel utilisateur FR (`.tex`).

## Tâches principales

- [ ] Types et appel API (`detailLevel`) ; sélecteur
- [ ] Rendu arborescent dans `BalanceSheetView` et `IncomeStatementView`
- [ ] i18n, Vitest, E2E, captures, manuel

## Fichiers

`frontend/src/lib/features/reports/{BalanceSheetView,IncomeStatementView}.svelte`, `reports.api.ts`,
`reports.types.ts`, tests associés ; `frontend/src/routes/(app)/reports/+page.svelte` ; locales i18n ;
`frontend/tests/e2e/` ; `docs/manual/fr/user-manual.tex`.

## Note de version

*(à écrire au développement)*
