# Story 47-3a : écran du plan comptable — poser un miroir ou une paire

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md` — D2, D3 ; Q8, Q9, Q10 ; FR-47-1 (écran),
FR-47-6, FR-47-7, FR-47-8 (manuel utilisateur, partie « poser un lien »). Née du découpage de l'esquisse 47-3
(2026-11-19) ; les écrans des rapports, l'E2E du parcours complet et le reste du manuel : **47-3b**. Dépend de la
**47-1b** (API). Maquettes : formulaire d'approbation hors dépôt (`data-not-committed/epic-47-stories-revue.html`,
§ M1 à M3).

## Story

En tant que comptable,
je veux, dans le dialogue de modification d'un compte, choisir son compte miroir ou le compte avec lequel il forme une
paire, être averti avant de créer une paire, et comprendre pourquoi un archivage est refusé,
afin de poser ces liens sans passer par l'API.

## Critères d'acceptation

1. **Section « Présentation dans les rapports »** du dialogue *Modifier le compte* (`routes/(app)/accounts/+page.svelte`),
   sous les champs existants (et sous `AccountCurrencyFields` en multi-devises) : un choix à trois options —
   **Aucun** · **Compte miroir** · **Compenser avec (paire)** — puis un sélecteur de compte. Absente du dialogue de
   création (47-1b AC-8) et pour un compte de rôle de fonds propres (texte d'aide à la place : « Les comptes de fonds
   propres ne se présentent pas autrement. »).
2. **Sélecteur filtré** sur les comptes admis par la règle de la 47-1a AC-5 (même société, actifs, autre côté pour un
   miroir, type opposé pour une paire, hors fonds propres, hors comptes qui jouent déjà un autre rôle), par une
   fonction pure `presentation-link-options.ts` testée seule ; la règle reste celle de l'API, qui seule fait foi.
   Texte d'aide sous le sélecteur, qui dit ce que fait le choix (miroir : « Quand le solde de ce compte est de sens
   inverse, il est présenté sur le compte choisi, de l'autre côté. » ; paire : « Seul le net des deux comptes paraît,
   sur le compte du sens du net. »).
3. **Avertissement de la paire (Q9)** : à l'enregistrement d'une **nouvelle** paire, un dialogue de confirmation
   s'ouvre avant l'appel (patron du dialogue de retypage) : titre « Compenser 6960 et 6961 ? », texte CO 958c al. 1
   ch. 7 (la compensation d'une charge et d'un produit n'est pas admise dans les comptes annuels ; Kesh présentera le
   net ; la présentation brute reste disponible), boutons **Annuler** (focus) et **Compenser quand même**. Aucun
   avertissement pour un miroir ni pour un retrait.
4. **Refus de l'API au champ** : `PRESENTATION_LINK_INVALID` affiché sous le sélecteur, message traduit selon la
   raison (avec le compte qui occupe déjà le rôle, `linkedAccountNumber`) ; `ACCOUNT_PRESENTATION_LINKED` au retypage
   ou à la pose d'un rôle de fonds propres, au pied du dialogue, comptes liés nommés.
5. **Archivage refusé (Q10)** : le dialogue *Archiver le compte* affiche, sur `ACCOUNT_PRESENTATION_LINKED`
   (`reason: archive`), un avis `role="alert"` qui nomme le ou les comptes liés et dit le remède (« Retirez d'abord
   le lien depuis le compte 2270. »), lien vers ce compte ; **aucun bouton « Archiver quand même »** (refus sans
   recours).
6. **Marque au plan comptable** : un compte lié porte une pastille discrète (patron des pastilles de devise et de
   « Non postable ») — « Miroir → 1179 », « Miroir de 2270, 2271 », « Paire ↔ 6961 » — avec `title` explicatif.
7. **Types et API front** : `accounts.types.ts` (`mirrorAccountId`, `offsetAccountId`, `warnings`), `accounts.api.ts` ;
   Vitest du filtre, du dialogue (états), du mapping des refus.
8. **Captures d'écran** (thème clair) de chaque état : section vide, miroir choisi, paire choisie, avertissement de la
   paire, refus au champ, archivage refusé, pastilles — jointes au Dev Agent Record et au prompt de revue
   (`kesh-gate-logs/`).
9. **E2E** (`frontend/tests/e2e/accounts.spec.ts` ou fichier neuf) : poser un miroir, poser une paire (avertissement
   confirmé), refus d'archivage nommant le compte lié.
10. **Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, section *Plan comptable* `:421`) : nouvelle sous-section
    « Présenter un compte de l'autre côté : miroirs et paires » — poser, déplacer, retirer un miroir ou une paire,
    l'avertissement CO 958c, le refus d'archivage ; renvoi à la sous-section des rapports écrite par la 47-3b. Le mot
    « miroir » est celui de l'écran (Q8). `.tex` seulement ; **le prompt de chaque passe de revue nomme le manuel**
    et exige le contrôle du PDF compilé localement.
11. **i18n** : clés dans les quatre locales ; `sitesTotal` de `i18n-keys.test.ts` **recompté** ;
    `lint-i18n-ownership`, `i18n-libelle-en-dur` verts.

## Tâches / sous-tâches

- [ ] **T0** : reprise de `main` ; `fix/api-parent-compte` change le même dialogue (champ parent) : partir de son état.
- [ ] **T1** : types, API, `presentation-link-options.ts` + tests (AC 2, 7).
- [ ] **T2** : section du dialogue, aide, refus (AC 1, 2, 4).
- [ ] **T3** : dialogue d'avertissement (AC 3) ; archivage refusé (AC 5) ; pastilles (AC 6).
- [ ] **T4** : Vitest, E2E, captures (AC 7–9).
- [ ] **T5** : manuel, i18n (AC 10, 11). **T6** : gates frontend et E2E complets au dernier commit de code.

## Dev Notes

### Code lu (à `origin/main` `fd04099f`)

- `frontend/src/routes/(app)/accounts/+page.svelte:349` (`submitEdit`), `:448-475` (`submitArchive`, cas
  `ACCOUNT_DESIGNATED_IN_SETTINGS`), `:569-645` (lignes et pastilles du plan), `:745-822` (dialogue *Modifier*,
  `AccountCurrencyFields`), `:824-866` (dialogue d'avertissement de retypage : patron de l'AC-3), `:868-925`
  (dialogue d'archivage).
- `frontend/src/lib/features/accounts/AccountCurrencyFields.svelte`, `fx-accounts.ts` (filtre de comptes admis, patron
  de l'AC-2), `account-options.ts`, `accounts.types.ts:45-170`.
- Catalogue : `crates/kesh-i18n/locales/fr-CH/messages.ftl:247-262` (`accounts-edit-*`, `account-field-*`).

### Stratégie de tests

Vitest (filtre, états du dialogue, refus), E2E du parcours, captures ; mutations : afficher l'avertissement pour un
miroir, laisser « Archiver quand même » sur un refus `archive`.

### Risques

Conflit avec `fix/api-parent-compte` (même dialogue, même `+page.svelte`) ; densité du dialogue sur mobile (la section
se replie sous un titre si besoin, à juger aux captures).

### References

- Fiches 47-1a, 47-1b ; maquettes M1 à M3 du formulaire d'approbation ; `CLAUDE.md` § *Recette visuelle*.

## Notes de version

Le dialogue d'un compte permet de choisir son **compte miroir** ou de le **compenser avec** un autre compte (avec un
avertissement avant de créer une paire) ; le plan comptable marque les comptes liés ; un archivage refusé nomme le
compte lié.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
