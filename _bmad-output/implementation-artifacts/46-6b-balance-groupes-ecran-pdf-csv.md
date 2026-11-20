# Story 46-6b : balance par groupes — écran, PDF et CSV

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-46-rapports-groupes.md` — D11 modifiée (Q6), D9 étendue (Q10) ; FR-46-6,
FR-46-9, FR-46-10 ; NFR-46-1, NFR-46-5. Release visée **0.21.0** (prévision). Fiche écrite le 2026-11-20 (remarque 2
de Guy), à valider en boucle. **Suit la 46-6a, la 46-2 (composant et contrôles) et la 46-3 (rendu PDF/CSV groupé,
après la 48-11).** Aucune migration.

## Story

En tant que comptable,
je veux voir, imprimer et exporter la balance par groupes, avec les mêmes contrôles qu'au bilan,
afin de contrôler mes comptes au niveau des groupes et de remettre ce que je vois.

## Modules touchés (règle de splitting préventif)

Cinq : `frontend` (vue de la balance, page des rapports), `kesh-report/{pdf,csv}` (balance), `kesh-api/routes/reports`
(paramètres de l'export de la balance), `kesh-i18n`, la documentation (`docs/manual/fr/user-manual.tex`,
`docs/api-external.md`). E2E dans `frontend/tests/e2e`.

## Critères d'acceptation

1. **AC-1 — Écran.** L'onglet Balance gagne les contrôles de la 46-2 (hiérarchie montrée / masquée, niveau, tout
   ouvrir / tout fermer, position des sous-totaux, accordéons, règle d'ouverture, mémoire par visiteur **propre à la
   balance**) et réutilise le composant `GroupedReportRows.svelte` de la 46-2, étendu aux cinq colonnes (ouverture,
   débit, crédit, solde, clôture) et à leurs montants en devise — aucune copie. Hiérarchie masquée : l'écran
   d'aujourd'hui (`TrialBalanceView.svelte`), ligne calculée du résultat reporté et contrôle d'ouverture compris. Par
   défaut, comme au bilan : hiérarchie montrée, `all`, `header` (Q10, décision 5).
2. **AC-2 — PDF et CSV fidèles à l'écran.** `GET /reports/trial-balance/export` (`routes/reports.rs:745`) accepte les
   quatre paramètres (validation de la 46-1) ; PDF (`render_trial_balance_pdf`, `pdf.rs:1042`) et CSV
   (`render_trial_balance_csv`, `csv.rs:355`) rendent `groupedRows` par **la fonction de ligne groupée de la 46-3**,
   étendue aux cinq colonnes, avec niveau et écarts manuels dans l'en-tête du PDF, colonnes `Niveau;TypeLigne` au CSV ;
   test de fidélité JSON ↔ PDF comme la 46-3 AC-4. Les boutons d'export de l'onglet passent les paramètres de la
   dernière requête affichée.
3. **AC-3 — Sans paramètre (NFR-46-1).** CSV octet pour octet, PDF aux mêmes lignes de texte, contre des fixtures du
   commit de base.
4. **AC-4 — Arrondi.** Fonctions de `kesh-core` de la 48-11 seules ; aucun `{:.2}` neuf dans `kesh-report`.
5. **AC-5 — Totaux.** Total débit = total crédit et contrôle d'ouverture affichés comme aujourd'hui, à tout niveau.
6. **AC-6 — Textes, tests, manuel.** Libellés neufs en quatre locales (bloc `reports-groups-*`), `i18n-keys`
   (`sitesTotal` recompté) et `i18n-libelle-en-dur` verts ; Vitest de la vue ; E2E (niveau 1, ouverture manuelle,
   export CSV aux mêmes paramètres) ; captures (thème clair) au Dev Agent Record ; manuel utilisateur FR,
   § *Balance des comptes* (`user-manual.tex:3972`), `.tex` seul.

## Tâches / sous-tâches

- [ ] **T0** — Reprise de la branche d'epic (46-2, 46-3, 46-6a fusionnées ; 48-11 dans `main`).
- [ ] **T1** — fixtures sans paramètre (AC-3) ; **T2** — route et rendus PDF/CSV (AC 2, 4) ; **T3** — écran (AC 1, 5) ;
      **T4** — i18n, tests, captures, manuel (AC 6) ; **T5** — gates complets, E2E complet au dernier commit de code.

## Dev Notes

- `frontend/src/lib/features/reports/TrialBalanceView.svelte` (138 lignes) : colonnes ouverture / mouvements /
  clôture, « Perte reportée » si négatif, devise par `formatCurrencyFigure`.
- `pdf.rs:1042` `render_trial_balance_pdf`, `csv.rs:355` `render_trial_balance_csv`.
- Ne rien recopier de la 46-2 ni de la 46-3 : étendre leurs fonctions et composant (DRY).

### References

Fiches 46-1, 46-2, 46-3, 46-6a.

## Note de version

*(à écrire au développement)* — La balance des comptes s'affiche, s'imprime et s'exporte par groupes, comme le bilan.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-20 — Fiche écrite (remarque 2 de Guy) ; découpée de la « 46-6 » de la revue 1 (règle de splitting
  préventif). Six critères. `drafted`.
