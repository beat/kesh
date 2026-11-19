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
   `closing_checks::for_fiscal_year` (les index de K1 à K9 ne bougent pas : `checks[8]` reste K9). Doc du module
   (« exactement neuf contrôles », table, paragraphe `count`, « Ce que les contrôles ne disent pas » — coût de K10
   comme celui de K6, devise principale, jugement par compte et non par groupe de l'Epic 46), doc de
   `ClosingChecks::checks` (« Toujours neuf »), de `Severity::Warning` (« K2, K3 »), de `CheckDetails` et de
   `for_fiscal_year` passées à dix.
2. **Comptes relevés — ceux dont la ligne serait négative dans le rapport présenté** (règles D2, D3 et AC-2/AC-3
   de la 47-2a, sur les liens rendus par `accounts::presentation_links`, 47-1a AC-9) : comptes de la société,
   **archivés et non imputables compris** (comme le bilan), hors rôles de fonds propres (D8 ; en SQL
   `(a.role IS NULL OR a.role NOT IN (…))`, jamais `NOT IN` seul, qui écarterait les comptes sans rôle), dont le
   **solde brut** est de sens inverse (`< 0` exact dans le signe de sa nature, comme `HAVING balance != 0` du
   bilan), et qui ne sont **pas couverts** :
   - **membre d'une paire** valide (partenaire rendu par le lecteur) → couvert (le net paraît du côté du net) ;
   - **source d'un miroir** valide → couvert (sa ligne passe de l'autre côté) ;
   - **cible d'un miroir** → relevé si **sa ligne présentée** est négative : solde propre + somme des soldes
     inversés de ses sources de sens inverse `< 0` (un miroir à solde propre inverse que ses sources compensent
     n'est **pas** relevé ; sans source de sens inverse, il est relevé dès que son solde propre est inverse) ;
   - **tout lien que le lecteur écarte** (47-1a AC-8 : cible absente ou d'une autre société, lien vers soi, paire non
     réciproque, chaîne, fonds propres, types qui ne sont plus de l'autre côté) ne couvre rien : le compte est jugé
     comme s'il n'avait pas de lien ; K10 ne relit jamais les colonnes de lien, seulement le vecteur du lecteur ;
   - une cible de miroir dont la ligne présentée vaut **zéro** (propre + sources) n'est pas relevée (0 n'est pas `< 0` ;
     la présentation omet cette ligne, comme une paire à net nul, 47-2a algorithme).
   Soldes **en devise principale** (le solde en devise n'entre pas, comme les totaux du bilan, NFR-37-4-3) :
   actif et passif cumulés `entry_date <= end_date` de Y, tous exercices (`balance_sheet.rs:336-345`) ; produit et
   charge sur les écritures `fiscal_year_id = Y` **et** `entry_date BETWEEN start_date AND end_date`
   (`income_statement.rs:76-88`). Signes : actif et charge = débit − crédit, passif et produit = crédit − débit
   (`opening::is_debit_natured`).
3. **Détails** — `{ "accounts": [{ accountId, number, name, accountType, balance }] }` : `count` = nombre de comptes
   relevés, **jamais borné** ; liste des **`MAX_LISTED` (20)** premiers par numéro, la constante du module, **sans
   champ `truncated`** — à l'image de K4, K6 et K9 (`fx_revaluation::not_revalued`, `:500-530`) : le client lit
   `count > accounts.length`. `accountType` en PascalCase (`"Liability"`), `balance` en chaîne décimale, **signée
   dans la nature du compte** (donc négative) ; pour une cible de miroir, `balance` = sa ligne présentée.
4. **Non bloquant** (Q6) : `fiscal_years::close` est inchangé ; un test (`closing_checks_e2e.rs`) clôt (`POST
   /fiscal-years/{id}/close`, 200) un exercice dont K10 est en `attention`.
5. **Une seule vue, aucune écriture, aucun verrou** : K10 lit sur la connexion du `ReadSnapshot` de la route, comme
   les neuf autres ; **trois requêtes au plus** — le lecteur des liens (`accounts::presentation_links`, ajouté à la
   liste des fonctions externes « vérifiées en revue de code » de la doc du module) et deux agrégations (bilan,
   résultat). Les deux agrégations sont des **constantes `…_SQL` statiques** inscrites à `ALL_SQL` (garde T-a2 :
   premier argument de `query*` = une constante de `ALL_SQL`, nombre de constantes = longueur de `ALL_SQL`, aucun
   mot de verrou) ; le signe s'écrit en SQL par un `CASE` sur `account_type`, sans `format!` ni appel à
   `kesh_report::balance_sheet::sign_exprs` (`kesh-db` ne dépend pas de `kesh-report`).
6. **API** : `GET /api/v1/fiscal-years/{id}/closing-checks` (`routes/fiscal_years.rs:264`, doc `:253-256` passée à
   dix) rend K10 ; `closing_checks_e2e.rs` étendu (dix contrôles, `checks[9]` K10 `warning` ; `ok` sans compte
   relevé ; `attention` avec un passif débiteur sans miroir, `count` 1 ; `ok` dès que le miroir est posé ; tout
   rôle ; autre société jamais lue). Pas d'outil MCP des contrôles de clôture (aucun à `origin/main`) : rien à
   faire au MCP.
7. **Écran** `FiscalYearChecks.svelte` : K10 listé avec titre, aide et une ligne par compte — « 2270 Dettes
   sociales : -1’200.00 CHF (passif à solde débiteur) », montant par `formatChfBalance`, motif par **quatre clés**
   (actif à solde créditeur, passif à solde débiteur, produit à solde débiteur, charge à solde créditeur) ; ligne
   « et N autres » quand `count` dépasse la liste (patron de `fiscal-years-checks-k8-more`) ; lien **« Ouvrir le
   plan comptable »** (`/accounts`, `data-testid="fiscal-year-checks-link-REVERSED_BALANCES_WITHOUT_PRESENTATION"`) ;
   icône et libellé « À vérifier » des `warning`. `checkTitle` et `checkHelp` (`closing-checks-labels.ts`) et le
   type `ClosingCheckCode` (`fiscal-years.types.ts:65-78`, union `accounts` `:156`) étendus. Vitest dédié
   (`FiscalYearChecks.k10.test.ts`, patron de `.k9.test.ts`). **Dialogue de clôture** (`+page.svelte`, rappel
   `pointsToVerify`) : K10 en `attention` y paraît comme point à vérifier et **« Clôturer » reste actif**
   (`closeBlockedByK9` inchangé) — test dans `fiscal-years-page.test.ts`. Capture d'écran (thème clair) du panneau
   et du rappel. La phrase d'intro (« deux d'entre eux, marqués Bloquant ») reste juste (K10 n'est pas bloquant).
8. **Documentation** : manuel utilisateur (`docs/manual/fr/user-manual.tex`, *Préparer la clôture* `:940` ;
   « neuf contrôles » `:942` passé à dix) — le contrôle K10, ce qu'il relève, le remède (poser un miroir ou une
   paire, renvoi par `\ref` à la sous-section de la 47-3a ; si la 47-4 est fusionnée dans la branche d'epic avant
   la 47-3a, le renvoi s'écrit dans la story qui arrive la seconde) ; manuel d'administration (« les neuf contrôles »
   `:1878` passé à dix) ; `.tex` seulement, PDF contrôlé au gate de release ; le prompt de chaque passe de revue
   nomme les deux manuels et exige le contrôle du PDF compilé localement. `docs/api-external.md` (note ¹⁰,
   `:270-272`) : « toujours dix » (avec la version d'avant, au tag), K10 **en dernier** (les neuf premiers gardent
   leur rang), sa sévérité, ses détails, sa liste à 20, et **⚠️ changement de contrat** (la note dit elle-même
   qu'« un contrôle neuf serait un changement de contrat annoncé ici ») ; la note de version le dit aussi.
9. **i18n** : clés `fiscal-years-checks-k10-*` (préfixe du dossier, `lint-i18n-ownership`), quatre locales ;
   `sitesTotal` recompté.
10. **Sites « neuf contrôles » mis à dix** (inventaire à `origin/main`, à recompter par `git grep -nE
    "neuf|\b9\b|toHaveCount\(9\)|len\(\), 9"` sur ces fichiers à la reprise de `main`) :
    `crates/kesh-db/tests/closing_checks.rs:283-305` (`t_a1`, ordre et sévérités) et `:1683-1686` ;
    `crates/kesh-db/tests/fx_revaluation.rs:1205` ; `crates/kesh-api/tests/closing_checks_e2e.rs:246-255` ;
    `frontend/src/lib/features/fiscal-years/closing-checks.test-fixtures.ts` (`ORDER`, `base`, commentaire) ;
    `frontend/tests/e2e/fiscal-year-closing-journey.spec.ts:158-167` (`toHaveCount(9)`, et revoir si ses données
    mettent K10 en `attention`) ; tout test existant qui exige « tout `ok` sauf K7 » sur des données qui créent un
    solde inverse.

## Tâches / sous-tâches

- [ ] **T1** : `CheckCode::ReversedBalancesWithoutPresentation`, variante de `CheckDetails`, deux constantes SQL à
  `ALL_SQL`, appel du lecteur des liens, calcul de la ligne présentée des cibles, ordre, docs (AC 1–3, 5).
- [ ] **T2** : tests de dépôt (`crates/kesh-db/tests/closing_checks.rs` : dix contrôles ; actif créditeur, passif
  débiteur, produit débiteur, charge créditrice ; compte sans rôle relevé, fonds propres exclus ; archivé relevé ;
  source de miroir exclue ; membre de paire exclu ; cible de miroir à solde propre inverse relevée sans source
  inverse, **non** relevée quand ses sources la compensent ni quand elles l'annulent à zéro ; liens écartés sans effet
  (orphelin, chaîne, paire non réciproque, posés par SQL direct : un test par cas, le compte reste relevé ; les sept
  cas de la liste close sont testés au lecteur, 47-1a AC-8, que K10 consomme sans le recoder) ; produit d'une écriture d'un
  autre exercice datée dans Y non compté ; autre société jamais lue ; troncature à 20 avec `count` complet) ; sites
  de l'AC-10 côté dépôt.
- [ ] **T3** : API (AC 4, 6) ; **test de cohérence avec la présentation** (dans la story, 47-2a ou 47-4, qui arrive
  la seconde dans la branche d'epic) : sur le jeu de l'exemple chiffré de la 47-2a AC-11 et ses variantes, l'ensemble
  relevé par K10 égale celui des lignes négatives (hors fonds propres et lignes calculées) du bilan et du compte de
  résultat présentés au dernier jour de Y.
- [ ] **T4** : écran, rappel du dialogue, Vitest, fixtures, E2E `toHaveCount(10)`, capture (AC 7, 10).
- [ ] **T5** : manuels, `api-external.md`, i18n (AC 8, 9).
- [ ] **T6** : gate complet (dépôt touché : jamais ciblé), E2E complet au dernier commit de code.

## Dev Notes

### Code lu (à `origin/main` `fd04099f`)

- `crates/kesh-db/src/repositories/closing_checks.rs:1-40` (table des contrôles, règle « ajouter en dernier »),
  `:60-80` (fonctions externes vérifiées en revue), `:131-160` (`ClosingChecks`, `ClosingCheck`), `:168`
  (`CheckCode`), `:195` (`Severity`), `:222` (`CheckDetails`, `untagged` : le client distingue par `code`), `:461`
  (`MAX_LISTED = 20`), `:585` (`ALL_SQL`), `:608-640` (`for_fiscal_year`), `:641` (`status_of`), `:1017`
  (`foreign_accounts_not_revalued`, patron d'une liste de comptes par numéro), `:1071` (garde T-a2).
- `crates/kesh-db/src/repositories/fx_revaluation.rs:95`, `:484-530` (`MAX_LISTED`, `NotRevaluedAccounts`,
  `count` non borné, liste à 20).
- `crates/kesh-report/src/balance_sheet.rs:183-202` (`sign_exprs`), `:319-360` (`fetch_cumulative_section`) ;
  `income_statement.rs:76-88` (filtre `fiscal_year_id` et dates) ; `opening.rs:36` (`is_debit_natured`).
- `crates/kesh-api/src/routes/fiscal_years.rs:253-275` (`get_closing_checks`).
- `frontend/src/lib/features/fiscal-years/FiscalYearChecks.svelte:150-373` (intro `:155`, K9 `:333`, « … more »
  de K8) ; `closing-checks-labels.ts` ; `fiscal-years.types.ts:65-78`, `:156` ; `fiscal-years.helpers.ts:115`
  (`pointsToVerify`) ; `routes/(app)/settings/fiscal-years/+page.svelte:78-84` (`closePoints`,
  `closeBlockedByK9`), `:296` (contrôles relus après une réévaluation) ; `FiscalYearChecks.k9.test.ts`.
- Catalogue : `fiscal-years-checks-*` (`fr-CH/messages.ftl:1092-1201`).

### Interaction avec K9 (Epic 37)

K10 lit les soldes **en devise principale** à l'instant de la lecture : avant la réévaluation de fin d'exercice, le
solde d'un compte en devise n'est pas encore réévalué, et K10 peut changer après elle. Le panneau relit les
contrôles après une réévaluation (`+page.svelte:296`) ; K9, bloquant, se traite d'abord. K10 ne relève pas les
comptes de K9 à part, et K9 garde son rang (`checks[8]`, tests `fx_revaluation_e2e.rs:775-796` inchangés).

### Stratégie de tests

Dépôt (cas de T2), API, cohérence avec la présentation (T3), Vitest, capture ; mutations : oublier l'exclusion des
fonds propres, écrire `role NOT IN` sans `IS NULL`, inverser le signe d'un type, compter un membre de paire, juger
une cible de miroir sur son seul solde propre, tenir pour couvrant un lien que le lecteur écarte (orphelin, chaîne, paire non réciproque), oublier le filtre
`fiscal_year_id` du résultat — chacune doit rougir.

### Risques

Coût de la requête du bilan sur un long historique (agrégation de tout le grand livre jusqu'à `end_date`, comme le
bilan et comme K6 : mesurer sur la base de démonstration, le dire dans la doc du module) ; ordre des contrôles si une
autre story en ajoute un entre-temps (le second arrivé prend K11) ; deux règles de « couverture » (K10 en `kesh-db`,
présentation en `kesh-report`) qui pourraient diverger — d'où le test de cohérence de T3.

### References

- Fiche 47-1a (AC-8 liens écartés par le lecteur, liste close (a)–(g) ; AC-9 lecteur) ; fiche 47-2a (AC-2, AC-3, algorithme) ; `CLAUDE.md`
  § *Priorités des défauts* (aucune).
- Registre de la dette du fichier d'epic (§ *Registre de la dette de l'epic*) : à la clôture de la boucle de revue,
  tout finding non remédié y est consigné, une ligne par problème ; la 47-5 en part (« on trace toujours »).

## Notes de version

« Préparer la clôture » signale, sans bloquer, les comptes dont le solde paraîtrait négatif au bilan ou au compte de
résultat faute de compte miroir ou de paire. ⚠️ **Changement de contrat de l'API** : `GET
/fiscal-years/{id}/closing-checks` rend dix contrôles, le dixième (`REVERSED_BALANCES_WITHOUT_PRESENTATION`,
`warning`) en dernier ; les neuf premiers gardent leur rang.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-19 — validation P3 ciblée (Opus 5.5, une lentille sur la remédiation P2 `53bf7f00`) : 0 C / 0 H / 0 M /
  3 L, remédiés dans la 47-4. L : référence restée « AC-8 lien orphelin » (résidu du symptôme de P2) → liste close
  (a)–(g) ; L : T2 ne testait que trois des sept cas écartés sans dire que les sept le sont au lecteur (47-1a AC-8) ;
  L : ligne présentée nulle — renvoi à l'omission de la 47-2a. Liste des sept cas identique dans la 47-4 AC-2, la
  47-1a AC-8 et la 47-2a AC-1 ; écarts **signalés, non remédiés** (47-2a en validation P1 par un autre agent, fichier
  modifié non commité ; 47-1a idem) : algorithme de la 47-2a (§ 1) qui omet « soi-même » dans sa liste ; omission
  de la ligne nulle dite au seul § 5 (cas à tester), non à l'AC 2, et sans la règle de devise du brut (`HAVING balance
  != 0 OR balance_currency != 0`) ; 47-1a AC-9 ne dit pas qu'un lien écarté est absent du vecteur (champ à `NULL` ou
  compte omis), ce sur quoi K10 et la 47-2a reposent. Rapport :
  `kesh-gate-logs/epic-47-validation/47-4-controle-cloture-sens-inverse-P3.md`.
- 2026-11-19 — validation P2 (Sonnet 5.5) : 0 C / 0 H / 1 M / 2 L, remédiés. M : K10 ne nommait que le lien orphelin parmi
  les liens que le lecteur écarte (47-1a AC-8 en compte sept cas) : un miroir en chaîne ou une paire non réciproque
  aurait été tenu pour couvrant alors que la présentation n'applique rien (faux négatif, même classe que le M2 de P1) ;
  L : cible dont la ligne présentée vaut zéro (non relevée), tests des cas écartés ; L : 47-2a alignée (§ Propagation).
  Rapport : `kesh-gate-logs/epic-47-validation/47-4-controle-cloture-sens-inverse-P2.md`.
- 2026-11-19 — validation P1 (Opus) : 0 C / 0 H / 7 M / 7 L, remédiés. M1 : une cible de miroir était relevée sur
  son solde propre, non sur sa ligne présentée (faux positif quand ses sources la compensent) ; M2 : un lien orphelin
  comptait comme couvrant (faux négatif : le bilan l'affiche négatif) ; M3 : liste à 50 avec `truncated`, contraire à
  `MAX_LISTED = 20` et à la forme de K4, K6, K9, et ligne « et N autres » absente ; M4 : sites « neuf contrôles »
  non inventoriés (tests dépôt, API, fixtures, E2E `toHaveCount(9)`, manuel d'administration `:1878`, doc de la
  route) — AC-10 ; M5 : changement de contrat de l'API non dit ; M6 : garde T-a2 (`ALL_SQL`, SQL statique, pas de
  `sign_exprs` de `kesh-report`), lecteur des liens à la liste des fonctions externes, « deux requêtes » passé à
  trois ; M7 : rappel du dialogue de clôture non traité (« Clôturer » reste actif). LOW : filtre `fiscal_year_id`
  du compte de résultat ; piège `NOT IN` / `NULL` ; quatre clés de motif, format `formatChfBalance`, clé `accounts`,
  `balance` signé, archivés et devise principale ; renvoi du manuel selon l'ordre avec la 47-3a ; numéros de ligne
  (catalogue `:1092-1201`, composant, patron K9) ; interaction avec K9 écrite ; jugement par compte, non par groupe
  (Epic 46). Propagé à la 47-2a (algorithme, cas du miroir à solde propre inverse). Rapport :
  `kesh-gate-logs/epic-47-validation/47-4-controle-cloture-sens-inverse-P1.md`.
