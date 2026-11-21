# Story 46-6b : balance par groupes — écran, PDF et CSV

Status: ready-for-dev

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

**Ce que la balance montre aujourd'hui** (relevé du 2026-11-20 à `origin/main`) : **quatre** colonnes de montants —
ouverture, débit, crédit, clôture — à l'écran (`TrialBalanceView.svelte`), au PDF (`render_trial_balance_pdf`) et au
CSV (`NumeroCompte;NomCompte;SoldeOuverture;TotalDebit;TotalCredit;SoldeCloture`) ; le **solde net des mouvements**
(`balance`) en a été retiré à la 25-5-b (#385) et ne paraît **nulle part** hors du JSON et du MCP (46-6a). Les
extensions ci-dessous portent sur ces quatre colonnes ; aucune cinquième n'est créée.

1. **AC-1 — Écran.** L'onglet Balance gagne les contrôles de la 46-2 (hiérarchie montrée / masquée, niveau, tout
   ouvrir / tout fermer, position des sous-totaux, accordéons, règle d'ouverture, mémoire par visiteur **propre à la
   balance**) et réutilise le composant `GroupedReportRows.svelte` de la 46-2 — **aucune copie** : le composant se
   paramètre par un descripteur de **colonnes** (étiquette et accès au montant), que la 46-2 pose pour le bilan (une
   colonne) et que cette story étend à quatre colonnes et à leurs montants en devise. Hiérarchie masquée : l'écran
   d'aujourd'hui (`TrialBalanceView.svelte`), ligne calculée du résultat reporté et contrôle d'ouverture compris. Par
   défaut, comme au bilan : hiérarchie montrée, `all`, `header` (Q10, décision 5). En mode groupé : la ligne du
   résultat reporté (libellé « Perte reportée » si négatif) vient **après** les groupes, hors d'eux ; un groupe en
   devise unique porte la même seconde ligne en devise qu'un compte (`reports-fx-trial-balance`) ; un sous-total en
   pied s'intitule « Total <nom du groupe> ». Les tests axe de la page, à zéro violation, portent aussi l'onglet Balance par défaut (groupé). Les `data-testid` d'aujourd'hui (`tb-row`, `tb-fx-row-<numéro>`,
   `tb-retained`, `tb-opening-check`, `tb-movements-check`) sont conservés sur les lignes de compte.
2. **AC-2 — PDF et CSV fidèles à l'écran.** `GET /reports/trial-balance/export` (`routes/reports.rs:745`) accepte les
   quatre paramètres (validation de la 46-1) ; PDF (`render_trial_balance_pdf`, `pdf.rs:1042`) et CSV
   (`render_trial_balance_csv`, `csv.rs:355`) rendent `groupedRows` par **deux fonctions neuves**
   (`render_trial_balance_pdf` et `_csv` à plat restent **inchangées** avec leurs appelants et tests, comme la 46-3 le
   fait pour le bilan), qui reçoivent `AvecGroupes<TrialBalance, Vec<LigneBalanceGroupee>>` de la 46-6a et appellent **la fonction de ligne groupée
   de la 46-3**, étendue aux quatre colonnes ; la route choisit selon que `grouping` est présent — au PDF, le groupe en devise unique porte la seconde ligne en petit
   (`write_fx_line`) comme un compte ; en-tête du PDF comme la 46-3 (niveau, et « détail ajusté à la main » si `open` ou `closed` n'est pas vide) ; **au CSV**, deux colonnes
   `Niveau;TypeLigne` en **tête** (la balance n'a pas de colonne `Section`, que la 46-3 AC-5 suit pour le bilan et le
   compte de résultat : elles se placent là où se placerait cette colonne, avant `NumeroCompte`), mêmes valeurs
   (`groupe`, `compte`, `propre`, `soustotal`) et même règle de rendu ; les lignes `Total` et du résultat reporté
   restent ce qu'elles sont, colonnes neuves vides ; quand les colonnes en devise sont présentes (un compte en devise
   au rapport, `csv.rs:364`), une ligne `groupe` ou `soustotal` en devise unique les remplit comme un compte, les autres
   les laissent vides. **Rapport vide** : les rendus groupés reprennent la garde des rendus à plat (`tb.rows.is_empty()`,
   message « rapport vide » au PDF, en-tête seul au CSV), comme la 46-3 pour le bilan. Test de fidélité JSON ↔ PDF comme la 46-3 AC-4. Les boutons
   d'export de l'onglet passent les paramètres de la dernière requête affichée.
3. **AC-3 — Sans paramètre (NFR-46-1).** CSV octet pour octet, PDF aux mêmes lignes de texte, contre des fixtures
   écrites **au commit de base de cette story** (après la 46-3, qui ne touche pas la balance), avant toute modification.
4. **AC-4 — Arrondi.** Fonctions communes posées par la 48-11 seules (noms relevés à T0 : elle n'a pas encore de
   fiche) ; aucun `{:.2}` neuf dans `kesh-report`, tel que sa garde l'interdira.
5. **AC-5 — Totaux.** Total débit = total crédit et contrôle d'ouverture affichés comme aujourd'hui, à tout niveau.
6. **AC-6 — Textes, tests, manuel.** Libellés neufs en quatre locales (bloc `reports-groups-*`), `i18n-keys`
   (`sitesTotal` recompté) et `i18n-libelle-en-dur` verts ; Vitest de la vue (`TrialBalanceView.test.ts` adapté) ; les
   specs d'aujourd'hui qui lisent l'**écran** de la balance — `frontend/tests/e2e/reports.spec.ts` (`tb-row`…),
   `reports-currency.spec.ts` (`tb-fx-row-…`) — passent en « hiérarchie masquée » ou s'adaptent, **sans perdre
   d'assertion** (comme la 46-2 AC-1) ; `opening-balances.spec.ts` ne lit que l'API sans paramètre (`:227`) et ne
   change pas ; E2E neuf (niveau 1, ouverture manuelle, export CSV aux mêmes paramètres) ; captures (thème clair)
   au Dev Agent Record ; manuel utilisateur FR, § *Balance des comptes* (`user-manual.tex:3972`), `.tex` seul : il dit
   aussi qu'à la balance les groupes qui mêlent des comptes de natures contraires sont dissous dès que des comptes des
   deux natures y portent un montant non nul (plan PME : classes 6 à 8 ; un compte à zéro, un compte miroir par
   exemple, ne dissout rien : 46-6a AC-2, C-46-6a-3), que `2 Passifs` y reste un groupe, et qu'un compte de groupe sans solde ni mouvement n'y a pas
   de ligne propre (46-6a AC-3 (d) : un groupe dissous à zéro, `6`, `7`, `70`, `8`, `80` du plan PME, n'y paraît pas).

## Tâches / sous-tâches

- [ ] **T0** — Reprise de la branche d'epic (46-2, 46-3, 46-6a fusionnées ; 48-11 dans `main`).
- [ ] **T1** — fixtures sans paramètre (AC-3) ; **T2** — route et deux rendus neufs PDF/CSV (AC 2, 4) ; **T3** — écran (AC 1, 5) ;
      **T4** — i18n, tests, captures, manuel (AC 6) ; **T5** — gates complets, E2E complet au dernier commit de code.

## Dev Notes

- `frontend/src/lib/features/reports/TrialBalanceView.svelte` (138 lignes) : colonnes ouverture, débit, crédit,
  clôture, « Perte reportée » si négatif, devise par `formatCurrencyFigure`, seconde ligne en devise.
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
- 2026-11-20 — **validation P1 (Sonnet 5.5) : 0 C, 1 H, 1 M, 1 L, remédiés.** H : « cinq colonnes » (ouverture, débit,
  crédit, solde, clôture) — l'écran, le PDF et le CSV n'en montrent que **quatre**, le solde net ayant été retiré à la
  25-5-b : critères, CSV et fixtures recadrés sur les quatre. M : placement de `Niveau;TypeLigne` (la balance n'a pas
  de `Section`), composant paramétré par colonnes, résultat reporté et seconde ligne en devise d'un groupe, specs
  d'aujourd'hui à adapter (`reports.spec.ts`, `reports-currency.spec.ts`, `opening-balances.spec.ts`,
  `TrialBalanceView.test.ts`), conséquence du plan PME écrite au manuel. L : 48-11 sans fiche (noms et garde à relever
  à T0). Axes exercés : `TrialBalanceView.svelte`, `csv.rs`, `pdf.rs`, `reports/+page.svelte`, 46-2/46-3 ; non
  exercés : PDF compilé du manuel, rendu réel.
- 2026-11-20 — **validation P2 (Claude Opus 5.5, contexte frais, au code de `origin/main` `d29aa557`) : 0 C, 0 H, 0 M,
  4 L, remédiés.** L : enveloppe `AvecGroupes<TrialBalance, Vec<LigneBalanceGroupee>>` ; colonnes en devise du CSV
  (`csv.rs:364`) pour un groupe en devise unique, et garde « rapport vide » (`tb.rows.is_empty()`) des rendus groupés,
  comme la 46-3 ; `opening-balances.spec.ts` ne lit que l'API sans paramètre (`:227`), il ne change pas ; manuel : les
  groupes à zéro sans ligne propre (46-6a AC-3 (d)). Rapport : `kesh-gate-logs/epic-46/validation/46-6b-P2.md`.
- 2026-11-21 — Propagation de la correction de la 46-6a (C-46-6a-3 : une ligne nulle ne compte pas pour la
  dissolution par nature) à l'AC-6 (phrase du manuel). Aucune autre modification.
