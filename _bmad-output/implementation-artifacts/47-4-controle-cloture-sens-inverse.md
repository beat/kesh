# Story 47-4 : contrôle de clôture — solde de sens inverse sans miroir ni paire

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md` — FR-47-5 ; Q6 (avertissement **non bloquant**,
Guy, 2026-11-19), D8. Dépend de la **47-1a** (colonnes de lien). Maquette : § M6 du formulaire d'approbation.

## Story

En tant que comptable qui prépare une clôture,
je veux voir les comptes dont le solde paraîtrait négatif au bilan ou au compte de résultat faute de miroir ou de
paire,
afin de poser le lien ou de vérifier l'écriture avant d'arrêter les comptes, sans être empêché de clore.

## Critères d'acceptation

1. **Contrôle K10 `REVERSED_BALANCES_WITHOUT_PRESENTATION`**, sévérité **`warning`**, ajouté **en dernier** à
   `closing_checks::for_fiscal_year` (les index de K1 à K9 ne bougent pas) ; la doc du module (« exactement neuf
   contrôles ») passe à dix, table comprise.
2. **Comptes relevés** : comptes de la société, hors rôles de fonds propres (D8), **sans** lien de présentation qui
   les couvre — ni source d'un miroir, ni membre d'une paire —, dont le solde à la fin de l'exercice Y est **de sens
   inverse** et non nul : actif et passif cumulés jusqu'à `end_date` (tous exercices, comme le bilan) ; produit et
   charge sur `[start_date, end_date]` de Y (comme le compte de résultat). Signes : actif et charge = débit − crédit,
   passif et produit = crédit − débit. Un **compte miroir** dont le solde propre est de sens inverse est relevé (il
   n'a pas de miroir lui-même).
3. **Détails** : `count` = nombre de comptes relevés ; liste triée par numéro des 50 premiers `{ accountId, number,
   name, accountType, balance }` (et `truncated: true` au-delà), à l'image des listes de K6.
4. **Non bloquant** (Q6) : `fiscal_years::close` est inchangé ; un test clôt un exercice dont K10 est en `attention`.
5. **Une seule vue** : K10 lit sur le `ReadSnapshot` de la route, comme les neuf autres ; aucune écriture, aucun
   verrou ; deux requêtes au plus (bilan, résultat).
6. **API** : `GET` des contrôles de clôture (route existante de `kesh-api/src/routes/fiscal_years.rs`) rend K10 ;
   `closing_checks_e2e.rs` étendu (dix contrôles, K10 `ok` sans compte relevé, `attention` avec un passif débiteur
   sans miroir, `ok` dès que le miroir est posé).
7. **Écran** `FiscalYearChecks.svelte` : K10 listé avec titre, aide et lignes « 2270 Dettes sociales : solde −1 200.00
   (passif à solde débiteur) », lien **« Ouvrir le plan comptable »** ; icône et libellé « À vérifier » des
   `warning`. Vitest dédié (`FiscalYearChecks.k10.test.ts`, patron de `.k9.test.ts`). Capture d'écran (thème clair).
   La phrase d'intro (« deux d'entre eux, marqués Bloquant ») reste juste (K10 n'est pas bloquant).
8. **Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, *Préparer la clôture* `:940`) : le contrôle K10, ce qu'il
   relève, le remède (poser un miroir ou une paire, renvoi à la sous-section de la 47-3a) ; `.tex` seulement ; le
   prompt de revue nomme le manuel et exige le contrôle du PDF. `docs/api-external.md` : code et détails de K10.
9. **i18n** : quatre locales ; `sitesTotal` recompté.

## Tâches / sous-tâches

- [ ] **T1** : `CheckCode::ReversedBalancesWithoutPresentation`, variante de `CheckDetails`, requêtes, ordre (AC 1–3,
  5).
- [ ] **T2** : tests de dépôt (`crates/kesh-db/tests/closing_checks.rs` : dix contrôles, cas de l'AC-2 — actif
  créditeur, passif débiteur, produit débiteur, charge créditrice, fonds propres exclus, source de miroir exclue,
  membre de paire exclu, miroir à solde propre inverse relevé, troncature à 50) ; clôture non bloquée (AC 4).
- [ ] **T3** : API (AC 6). **T4** : écran, Vitest, capture (AC 7). **T5** : manuel, `api-external.md`, i18n (AC 8, 9).
- [ ] **T6** : gate complet (dépôt touché : jamais ciblé), E2E complet au dernier commit de code.

## Dev Notes

### Code lu (à `origin/main` `fd04099f`)

- `crates/kesh-db/src/repositories/closing_checks.rs:1-40` (table des contrôles, règle « ajouter en dernier »),
  `:131-160` (`ClosingChecks`, `ClosingCheck`), `:168` (`CheckCode`), `:195` (`Severity`), `:222` (`CheckDetails`),
  `:608-640` (`for_fiscal_year`), `:641` (`status_of`), `:690-800` (K2, K3 : patron d'un `warning` à liste).
- `frontend/src/lib/features/fiscal-years/FiscalYearChecks.svelte:60-240` ; `FiscalYearChecks.k9.test.ts`.
- Catalogue : `fiscal-years-checks-*` (`messages.ftl:1093-1154`).

### Stratégie de tests

Dépôt (cas de l'AC-2), API, Vitest, capture ; mutations : oublier l'exclusion des fonds propres, inverser le signe
d'un type, compter un membre de paire — chacune doit rougir.

### Risques

Coût de la requête sur un grand plan (une agrégation par type, bornée par l'index des lignes d'écriture : mesurer sur
la base de démonstration) ; ordre des contrôles si une autre story en ajoute un entre-temps (le second arrivé prend
K11).

### References

- Fiche 47-1a (colonnes) ; `CLAUDE.md` § *Priorités des défauts* (aucune).
- Registre de la dette du fichier d'epic (§ *Registre de la dette de l'epic*) : à la clôture de la boucle de revue,
  tout finding non remédié y est consigné, une ligne par problème ; la 47-5 en part (« on trace toujours »).

## Notes de version

« Préparer la clôture » signale, sans bloquer, les comptes dont le solde paraîtrait négatif au bilan ou au compte de
résultat faute de compte miroir ou de paire.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
