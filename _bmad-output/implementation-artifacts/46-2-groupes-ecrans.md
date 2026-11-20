# Story 46-2 : écrans du bilan et du compte de résultat par groupes

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-46-rapports-groupes.md` — D3, D4, D9 (étendue), Q2, Q3, Q10 ; FR-46-2,
FR-46-7 à FR-46-10 ; NFR-46-1, NFR-46-5. Release visée **0.21.0** (prévision). Approuvée par Beat et par Guy (revue 1,
2026-11-19) ; fiche écrite le 2026-11-20, à valider en boucle. **Suit la 46-1** (contrat de l'API). Aucune migration.

## Story

En tant que comptable,
je veux voir à l'écran le bilan et le compte de résultat dans la hiérarchie de mon plan, ouvrir et fermer les groupes,
choisir un niveau de détail et la place des sous-totaux,
afin de lire mes comptes d'un coup d'œil au niveau qui m'intéresse, et d'exporter exactement ce que je vois.

## Modules touchés (règle de splitting préventif)

Cinq : `frontend/src/lib/features/reports` (vues, types, appels, composant neuf), `frontend/src/routes/(app)/reports`
(barre de contrôles, export), `kesh-i18n` (quatre locales), `docs/manual/fr/user-manual.tex`, `frontend/tests/e2e`.
Aucun code Rust : la 46-1 rend déjà `groupedRows` ; l'export groupé est la 46-3 (cette story y passe les paramètres,
qui sont refusés en `400` tant que la 46-3 n'est pas fusionnée — d'où l'ordre 46-1 → 46-3 → 46-2 recommandé, ou
l'export groupé masqué derrière la fusion de la 46-3).

## Critères d'acceptation

1. **AC-1 — Hiérarchie montrée par défaut (Q10).** À la première visite, le bilan et le compte de résultat s'affichent
   **groupés**, niveau « Tous les comptes » (`detailLevel=all` : tous les groupes ouverts), sous-totaux sur la ligne du
   titre (`header`, décision 5). C'est un changement de l'écran d'aujourd'hui, voulu par Beat ; les E2E existants qui
   lisent l'écran à plat (`frontend/tests/e2e/reports.spec.ts`, `reports-currency.spec.ts`) passent en
   « hiérarchie masquée » ou sont adaptés, sans perdre aucune assertion.
2. **AC-2 — Contrôles.** Au-dessus du bilan et du compte de résultat (pas des autres rapports) :
   - **« Hiérarchie montrée / masquée »** : masquée = l'écran **à plat d'aujourd'hui**, requête **sans** `detailLevel`,
     sous-titres de rôle des fonds propres compris (D4) ; montrée = l'écran groupé. *(Ce contrôle tient lieu de l'entrée
     « À plat » du sélecteur de D9.)* Niveau, position et ouvertures sont gardés pendant que la hiérarchie est masquée ;
   - **sélecteur de niveau** « Niveau 1 · … · Niveau n−1 · Tous les comptes », `n` = `grouping.maxDepth` de la dernière
     réponse ; désactivé quand la hiérarchie est masquée ;
   - **« Tout ouvrir »** = niveau « Tous les comptes » et listes manuelles vidées ; **« Tout fermer »** = niveau 1 et
     listes vidées ;
   - **« Sous-totaux : sur le titre / en pied »** (`header` / `footer`).
   Contrôles accessibles au clavier, libellés et `aria-expanded` sur chaque accordéon.
3. **AC-3 — Accordéons et règle d'ouverture (remarque 4 de Guy).** Chaque ligne `group` porte un accordéon. Le niveau
   fixe l'état de départ ; un clic **prime** ensuite : ouvrir un groupe fermé par le niveau l'ajoute à `open` (et le
   retire de `closed`), fermer un groupe ouvert l'ajoute à `closed` (et le retire de `open`). Changer de niveau vide
   les deux listes. L'écran **redemande** le rapport à l'API avec `detailLevel`, `subtotalPosition`, `open`, `closed`
   (une seule source de la règle : la 46-1 ; aucun repli calculé dans le navigateur). Requêtes concurrentes : seule la
   réponse de la dernière demande s'affiche.
4. **AC-4 — Rendu (D5, Q2, Q3).** Lignes rendues dans l'ordre de `groupedRows` : retrait par `depth` ; `group` en gras,
   son montant **selon la règle de rendu de la 46-1** (sur le titre en `header` ou si fermé ; sinon sur la ligne
   `subtotal`, en gras, libellé « Total <nom du groupe> ») ; `own` libellée « <numéro> <nom> (solde propre) » ; solde en
   devise d'une ligne `group`/`subtotal` quand l'API le donne (devise + CHF), rien sinon (aucune ligne ajoutée) ; lien
   vers le grand livre (`ledgerHref`) sur les lignes `account` et `own` seulement. Totaux de section, lignes calculées
   du bilan et équation : à leur place d'aujourd'hui. Montants par `formatReportAmount` / `formatCurrencyFigure`
   (arrondi au centime de `roundToCents`, `reports.api.ts:306`), aucun formatage neuf.
5. **AC-5 — Mémoire par visiteur.** Hiérarchie montrée ou masquée, niveau, position, et les listes `open`/`closed`
   **par rapport** (bilan, compte de résultat) sont retenus en `localStorage`, toute lecture et écriture sous
   `try`/`catch`, l'écran rendu par défaut (AC-1) si le stockage est vide, illisible ou refusé. Une valeur relue
   invalide (niveau hors bornes, liste non numérique) est ignorée, non transmise.
6. **AC-6 — Export fidèle à l'écran.** Les boutons PDF et CSV du bilan et du compte de résultat passent **les mêmes
   paramètres** que la dernière requête affichée (aucun si la hiérarchie est masquée) : `downloadReport` et
   `getReportExportUrl` (`reports.api.ts:334`, `:446`) gagnent un argument facultatif de groupement. Le rendu exporté
   est la 46-3.
7. **AC-7 — Affichage de travail (remarque 3 de Guy, lecture (a)).** Une phrase du manuel (AC-9) dit que les sections
   légales du bilan restent (Actifs, Dettes, Fonds propres) et qu'un affichage replié — au niveau 1 en particulier —
   est un affichage de travail, non des comptes annuels ; à l'écran, une info-bulle discrète sur le sélecteur de
   niveau le rappelle.
8. **AC-8 — Maquette avant les écrans.** Une maquette (HTML statique, hors dépôt, `kesh-gate-logs/epic-46/`) des deux
   positions de sous-total, d'un groupe fermé par le niveau puis ouvert à la main, et des contrôles, est jointe au Dev
   Agent Record et montrée à Beat avant le développement des vues.
9. **AC-9 — Textes, tests, manuel.** Libellés en quatre locales (bloc `reports-groups-*` propre à l'epic) ;
   `i18n-keys` (`sitesTotal` recompté par la garde) et `i18n-libelle-en-dur` verts ; Vitest des vues (rendu `header`
   et `footer`, groupe fermé, `own`, devise de groupe, mémoire illisible) et du composant ; un E2E Playwright
   (`reports-groupes.spec.ts`) traverse : écran groupé par défaut, niveau 1, ouverture d'un groupe à la main, export
   CSV dont l'URL porte les mêmes paramètres, hiérarchie masquée qui rend l'écran d'aujourd'hui ; captures d'écran
   (thème clair) de chaque état au Dev Agent Record ; manuel utilisateur FR, § *Bilan* et *Compte de résultat*
   (`docs/manual/fr/user-manual.tex:3923`, `:3954`), `.tex` seul, PDF non commité.

## Tâches / sous-tâches

- [ ] **T0** — Reprise de `main` / de la branche d'epic ; vérifier le contrat de la 46-1 tel que fusionné.
- [ ] **T1 — maquette** (AC-8).
- [ ] **T2 — types et appels** (AC 3, 6) : `reports.types.ts` (`Grouping`, `GroupedRow`, champs facultatifs des DTO),
      `reports.api.ts` (`getBalanceSheet`, `getIncomeStatement`, `downloadReport`, `getReportExportUrl` avec options).
- [ ] **T3 — composant** `GroupedReportRows.svelte` (neuf, partagé par les deux vues et réutilisé par la 46-6b) :
      rendu des lignes, accordéons, règle de rendu (AC-4).
- [ ] **T4 — vues et page** (AC 1, 2, 5, 7) : `BalanceSheetView.svelte`, `IncomeStatementView.svelte` (branche
      groupée, branche à plat inchangée), barre de contrôles dans `routes/(app)/reports/+page.svelte`, mémoire.
- [ ] **T5 — i18n, Vitest, E2E, captures, manuel** (AC 9).
- [ ] **T6 — gates** : `npm run check`, `lint-i18n-ownership`, `test:unit`, `build` ; E2E complet au dernier commit de
      code.

## Dev Notes

### État du code (relevé le 2026-11-20 à `29b9ec27`)

- `frontend/src/lib/features/reports/BalanceSheetView.svelte` (283 lignes) : sections Actifs, Passifs, Capitaux
  propres groupés **par rôle** (`equityGroups`, ordre reçu du serveur), lignes calculées (`showsRetainedEarningsLine`),
  total des capitaux propres calculé en `Big`, `AccountCurrencyBalance` pour la devise, `ledgerHref` sur chaque compte.
- `IncomeStatementView.svelte` (113 lignes), `TrialBalanceView.svelte` (138 ; 46-6b).
- `reports.api.ts` : `buildQuery` (`:26`), `getBalanceSheet` (`:36`), `getIncomeStatement` (`:45`), `isReportEmpty`
  (`:158`), `formatReportAmount` (`:280`), `roundToCents` (`:306`), `getReportExportUrl` (`:334`), `downloadReport`
  (`:446`). `routes/(app)/reports/+page.svelte` (893 lignes) : onglets, période, export (`:455-480`).
- `localStorage` : précédents sous `lib/app/stores/` (`mode.svelte.ts`) — même garde `try`/`catch`.

### Ce qui doit rester

La vue à plat **identique** (hiérarchie masquée) : sous-titres de rôle, ordre des fonds propres, lignes calculées,
garde « rapport vide » ; les autres onglets ; l'export sans paramètre.

### Choix et écarts

- **C-46-2-1** : l'écran redemande à chaque changement plutôt que de replier localement — la règle d'ouverture n'existe
  qu'en Rust (DRY), et l'export reprend les paramètres de la dernière requête, ce qui garantit la fidélité (46-3).
- **C-46-2-2** : « hiérarchie masquée » = vue à plat (point à confirmer par Beat à la validation : le contrôle et
  l'entrée « À plat » de D9 feraient sinon double emploi).
- **C-46-2-3** : « tout ouvrir / tout fermer » passe par le niveau, non par la liste de tous les groupes (URL courte).

### References

- Fiche 46-1 (§ *Le contrat*) ; fichier d'epic ; `CLAUDE.md` § *Recette visuelle* (captures, thème clair), § *Le prompt
  d'une passe doit nommer le manuel* (le PDF compilé localement se contrôle à la revue).

## Note de version

*(à écrire au développement)* — Le bilan et le compte de résultat s'affichent dans la **hiérarchie de votre plan** :
groupes ouverts ou fermés, niveau de détail, sous-totaux sur le titre ou en pied ; « Hiérarchie masquée » rend
l'affichage d'avant.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-20 — Fiche écrite depuis l'esquisse : hiérarchie montrée par défaut (Q10), contrôles, règle d'ouverture
  (remarque 4 de Guy), `header` par défaut (remarque 5), export aux mêmes paramètres, phrase « affichage de travail »
  (remarque 3). Neuf critères. `drafted`.
