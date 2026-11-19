# Story 47-2a : présentation au bilan et au compte de résultat — fonction, API, MCP

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md` — D2–D9 ; Q2, Q3, Q5, Q7, Q11 ; FR-47-3, FR-47-4
(API, MCP) ; NFR-47-1, NFR-47-2, NFR-47-4. Née du découpage de l'esquisse 47-2 (2026-11-19) ; PDF et CSV : **47-2b**.
Dépend de la **47-1a** (lecteur des liens). **Se développe sans l'Epic 46** : la présentation est une fonction pure
appliquée aux lignes de compte **avant** tout regroupement (Q5) ; son recâblage à la fusion de l'Epic 46 est écrit
ci-dessous.

## Story

En tant que comptable,
je veux un bilan et un compte de résultat sans montant négatif dû à un compte de sens inverse ou à une paire, et
pouvoir obtenir à tout moment la présentation brute,
afin de retrouver la présentation de mon ancien logiciel sans écriture de reclassement.

## Critères d'acceptation

1. **Fonction pure** `kesh_report::presentation` (nouveau module) : `present_balance_sheet(raw: BalanceSheet, links:
   &PresentationLinks) -> BalanceSheet` et `present_income_statement(raw: IncomeStatement, links: &PresentationLinks)
   -> IncomeStatement`, sans I/O, testées seules ; `PresentationLinks` se construit depuis
   `accounts::presentation_links` (47-1a AC-9). Les générateurs `balance_sheet::generate` et
   `income_statement::generate` **restent bruts et inchangés** (seuls appelants : `services/rapports.rs:619`, `:634`) :
   l'accueil (`homepage_summary`, invariant `:6-11`), la balance, le grand livre, le journal, la TVA et les projets
   n'y passent pas (D7, Q4, Q11). La fonction ne lit et n'écrit que les sections `assets`, `liabilities` (bilan),
   `revenues`, `expenses` (résultat) — **jamais** `equity` ni les lignes calculées (D8). `PresentationLinks` vide →
   le rapport est rendu **tel quel**, sans retri (chemin court).
   **Liens incohérents écartés** (défense ; la 47-1a les refuse à l'écriture, son AC-8 ne couvre que l'orphelin) : à la
   construction de `PresentationLinks`, un lien est ignoré, avec `tracing::warn!` (compte, raison), s'il touche un
   compte de rôle de fonds propres, si source et cible ne sont pas de l'autre côté du même rapport (miroir) ou de types
   opposés du même rapport (paire), si la paire n'est **pas réciproque** (`A.offset = B` sans `B.offset = A`), ou si
   un compte y joue plus d'un rôle (chaîne). Ainsi un état posé hors de l'API (SQL, restauration) ne compte jamais une
   ligne deux fois, ne déplace jamais un compte de fonds propres, et l'AC 5 tient sans condition sur les liens.
2. **Miroir** (D2) : une source dont le solde est **de sens inverse** (`balance < 0` dans le signe de sa nature) quitte
   sa section ; la ligne de son compte miroir, de l'autre côté, vaut **solde propre du miroir + somme des soldes
   inversés de ses sources** ; libellé et numéro du miroir (Q7), même s'il ne porte aucune écriture (il ne paraît
   alors que par ses sources ; ses attributs — type, rôle, devise, `active` — viennent de `PresentationLinks`). Une
   source de solde nul ou de sens normal reste à sa place, inchangée. **Une cible ne devient ligne présentée que si
   elle reçoit au moins une source de sens inverse** : sinon sa ligne brute (si elle en a une) reste telle quelle,
   sans `sources` (47-2b AC 3 : « liens posés dont aucune source n'est de sens inverse » → rendu identique au brut).
3. **Paire** (D3) : pour A et B de types opposés, `net = solde(A) − solde(B)` dans le sens de A ; `net > 0` → une
   ligne sur A ; `net < 0` → une ligne sur B, montant `−net` ; `net = 0` → aucune ligne ; les deux lignes brutes
   disparaissent. Vaut au compte de résultat (produit / charge) et au bilan (actif / passif hors fonds propres).
   **Paire appliquée** : une paire ne s'applique que si elle change le rapport ; elle est **sans effet** — lignes
   brutes laissées telles quelles, sans `sources`, sans `compensated` — quand les deux soldes CHF sont nuls, ou quand
   un seul est non nul **et positif** (il resterait sur son compte, au même montant). Un seul membre non nul **et
   négatif** passe sur son partenaire (appliquée : la ligne change de compte). Une paire appliquée liste ses **deux**
   membres en `sources` (`offset`), le membre sans écriture à `balance` 0.
4. **Fonds propres et lignes calculées inchangés** (D8) : `equity`, `total_equity`, `retained_earnings`,
   `equity_result` identiques en brut et en présenté (un compte de rôle de fonds propres ne porte jamais de lien,
   47-1a AC-5) ; une perte reste en déduction (CO 959a).
5. **Invariants** (NFR-47-2) : en présenté, `total_assets = total_liabilities + total_equity + retained_earnings +
   equity_result` dès qu'il tient en brut, et `net_result` est identique ; `equation_holds` est recalculé et **égal**
   à celui du brut ; **test de propriété** (`proptest`, absent du workspace à `fd04099f` : dépendance de dev de
   `kesh-report`, `Cargo.lock` régénéré ; ou générateur maison si la dépendance n'est pas admise — à trancher en T0)
   sur des jeux aléatoires de soldes et de liens, **valides et incohérents** (AC 1 : les seconds sont écartés). Une
   section modifiée est retriée par numéro de compte avec la comparaison de chaînes de Rust (`String::cmp`) ; celle du
   SQL (`ORDER BY a.number`, collation de la colonne) peut différer sur la casse d'un numéro alphanumérique — d'où le
   chemin court de l'AC 1 et l'exigence : sans lien, ou avec des liens tous sans effet, la sortie **égale** l'entrée.
6. **Sources** (D9) : chaque ligne présentée porte `sources: [{ accountId, accountNumber, accountName, balance, kind }]`
   — `kind` ∈ `own` (solde propre du miroir), `mirror` (source inversée), `offset` (membre de paire) ; `balance` =
   solde **brut** du compte source, signé dans sa nature. **Ordre** : miroir — `own` d'abord (s'il est non nul), puis
   les sources `mirror` par numéro ; paire — le membre qui porte la ligne, puis l'autre (47-2b écrit `Sources` dans
   cet ordre). Champ omis (`skip_serializing_if = "Vec::is_empty"`) sur une ligne non présentée : un rapport sans
   lien, ou dont les liens sont tous sans effet (AC 2, 3), rend **le même JSON qu'aujourd'hui**. Le rapport porte
   `compensated: true` (omis quand faux) dès qu'**une paire est appliquée** (AC 3) — y compris une paire de net nul,
   qui supprime ses deux lignes (Q9 : mention de la 47-2b) ; un miroir seul ne le pose jamais.
7. **Devise** (Q3) : la ligne présentée porte `currency` et `balanceCurrency` **seulement si** toutes ses sources qui
   avaient une ligne brute (solde CHF **ou** en devise non nul) portent la même devise étrangère ; `balanceCurrency`
   = somme de leurs soldes en devise, signés comme en CHF (`own` et membre porteur `+`, source `mirror` et autre
   membre `−`) ; sinon les deux absents — y compris quand le compte qui porte la ligne est lui-même en devise. Une
   ligne non présentée garde ses champs de devise. Les totaux restent en devise principale (NFR-37-4-3).
8. **Date d'arrêté** (D4, Q2) : la présentation s'applique à toute période et toute date d'arrêté, sur les soldes que
   rend déjà le générateur ; aucun traitement propre à la clôture.
9. **API** : `GET /api/v1/reports/balance-sheet`, `/income-statement` et leurs `/export` acceptent
   `presentation=brute|presentee` (défaut `presentee`) ; valeur inconnue → 400 (même patron que `validate_format`) ;
   `brute` rend exactement le rapport d'aujourd'hui (D6). Le service (`services/rapports.rs::bilan`,
   `::compte_de_resultat`) lit les liens dans la même `Lecture` et applique la fonction ; la trace d'audit du rapport
   porte la présentation demandée.
10. **MCP** `kesh_bilan`, `kesh_compte_de_resultat` : argument facultatif `presentation` (`"brute"` |
    `"presentee"`, défaut `presentee`) au schéma (`SCHEMA_PERIODE` dédoublé si `kesh_balance` le partage) ;
    `LigneDeSolde` gagne `sources` (liste, vide sans présentation) ; `Bilan` et `CompteDeResultat` gagnent
    `compense` ; `mcp_contrat`, `mcp_gardes_lexicales` verts, **tailles re-mesurées**.
11. **Exemple chiffré** (NFR-47-4 ; **source unique** de l'exemple : le fichier d'epic ne le porte pas, et le manuel
    utilisateur, l'E2E de la 47-3b et ce test reprennent **ces nombres et ces comptes**) — **comptes des plans livrés**
    (`plans_livres()`, garde G4-bis de `textes_coherents.rs`, qui rougit sur un numéro de compte absent des trois plans
    ou sous un autre nom : 1021, 1179, 2120, 6960, 6961 n'y sont pas, 2000 s'y nomme « Créanciers » et 2270
    « AVS/AI/APG »). Contrepartie : une écriture par compte contre `2800` *Capital social* (rôle capital) ; solde de
    `2800` = 9 330.00 (équilibre), fonds propres 9 900.00 avec le résultat de 570.00. Bilan brut : 1000 Caisse 500.00,
    1010 Poste 12 000.00, 1020 Banque **−800.00** (miroir 2100 Dettes bancaires à court terme), 2000 Créanciers
    3 000.00, 2270 AVS/AI/APG **−1 200.00** (miroir 1040 Avoirs à court terme) → présenté : 1040 à 1 200.00 à l'actif,
    2100 à 800.00 au passif, total actif 13 700.00 (brut 11 700.00), total passif 3 800.00 (brut 1 800.00), les deux
    côtés +2 000.00 ; compte de résultat : 6950 Produits financiers 1 020.00, 6900 Charges financières 450.00 en paire
    → 6950 à 570.00, 6900 absent, résultat net 570.00 identique ; la balance (`trial-balance`) montre toujours 1020 à
    −800.00 et 2270 à −1 200.00. *(Remplace, à la validation P1 de la 47-3b, un jeu aux numéros de la comptabilité
    de Beat — 1021, 1179, 2120, 6960, 6961 — que le manuel n'aurait pu citer sans rougir G4-bis.)*
12. **Aucune écriture** (NFR-47-1) : générer un rapport présenté ne crée ni ne modifie aucune ligne comptable ;
    balance et grand livre identiques avant et après.
13. **Documentation (exigence de Guy, 2026-11-19)** : `docs/api-external.md` — paramètre `presentation`, champs
    `sources` et `compensated`, règle de devise, exemple JSON, clé `presentation` de la trace d'audit ; ⚠️ **le défaut
    change ce que rend la route dès qu'un lien est posé** : une intégration qui recalcule depuis les lignes de compte
    demande `presentation=brute` (écrit comme les autres changements de contrat du document) ; et la phrase de la
    synthèse de l'accueil (`:305`, « `totalRevenues` et `totalExpenses` égalent ceux de `GET
    /reports/income-statement` ») devient « … de `GET /reports/income-statement?presentation=brute` » — une paire au
    compte de résultat change ses totaux (non son résultat) ; manuel d'administration
    (`docs/manual/fr/admin-manual.tex`, tableau des outils MCP `:3538`) — la présentation brute à l'API et aux outils
    MCP. `.tex` seulement ; **le prompt de chaque passe de revue nomme ces documents** et exige le contrôle du PDF
    compilé localement.
14. **Accueil resté brut** (Q11, D6) : `GET /reports/homepage-summary` est inchangé ; ses totaux égalent ceux du
    compte de résultat **brut**. `crates/kesh-api/tests/homepage_summary_e2e.rs` (`income_totals`, `:76-91`, appel
    sans paramètre) demande désormais `presentation=brute`, et un cas neuf pose une paire au compte de résultat :
    l'accueil égale le brut et **diffère** du présenté sur `totalRevenues` / `totalExpenses` (sonde négative : la
    mutation « appliquer la présentation à l'accueil » le rougit).

## Tâches / sous-tâches

- [ ] **T0** : reprise de `main` ; si l'Epic 46 est fusionnée, appliquer d'emblée le recâblage (§ ci-dessous) ;
  trancher `proptest` (dépendance de dev de `kesh-report`).
- [ ] **T1 — types** (AC 6) : `PresentationSource`, `SourceKind`, champ `sources` d'`AccountBalance`
  (`#[sqlx(skip)]`, défaut vide), `compensated` sur `BalanceSheet` et `IncomeStatement`.
- [ ] **T2 — fonction** (AC 1–5, 7, 8) : `presentation.rs`, algorithme en trois temps (paires, miroirs, totaux),
  tests unitaires et de propriété.
- [ ] **T3 — service et route** (AC 9, 12) : paramètre, lecture des liens, trace d'audit.
- [ ] **T4 — MCP** (AC 10).
- [ ] **T5 — tests d'intégration** (AC 9, 11, 12) : `reports_e2e.rs`, `mcp_outils_e2e.rs`.
- [ ] **T6 — documentation** (AC 13).
- [ ] **T7 — gate complet**, E2E complet au dernier commit de code.

## Dev Notes

### Code lu (à `origin/main` `fd04099f`)

- `crates/kesh-report/src/balance_sheet.rs:74-99` (`BalanceSheet`), `:113-121` (`is_empty`, source unique de la garde
  « vide » : un bilan dont toutes les lignes se compensent reste à juger ainsi, à tester), `:152-174`
  (`AccountBalance`, `sqlx::FromRow`), `:204-299` (`generate` : partition fonds propres par rôle, totaux, équation),
  `:319-360` (`fetch_cumulative_section`, `INNER JOIN` : un compte sans écriture n'y paraît pas — d'où le miroir
  « vide » construit depuis `PresentationLinks`).
- `crates/kesh-report/src/income_statement.rs:21-31` (`IncomeStatement`, `generate`).
- `crates/kesh-report/src/homepage_summary.rs:6-11` (invariant avec le compte de résultat **brut**).
- `crates/kesh-api/src/services/rapports.rs:199-206` (`DemandePeriode`), `:610-636` (`bilan`, `compte_de_resultat`).
- `crates/kesh-api/src/routes/reports.rs:64-79` (`ReportQuery`), `:174-200` (`ExportQuery`, `validate_format`),
  `:235-258`, `:374-420` ; routage `crates/kesh-api/src/lib.rs:1381-1430`.
- `crates/kesh-api/src/mcp/outils/rapports.rs:38-112` (`ArgumentsPeriode` `deny_unknown_fields`, `SCHEMA_PERIODE`,
  `LigneDeSolde`, `Bilan`, `CompteDeResultat`).

### Algorithme (référence pour la fonction)

1. Indexer les lignes brutes par compte ; `links` ne contient que des liens **valides** (le lecteur a écarté ceux de
   47-1a AC-8 : orphelin, autre société, chaîne, paire non réciproque, fonds propres, types) — la fonction ne revérifie
   rien, et un compte à lien écarté reste à sa place, brut ; `links` donne, par compte lié, sa nature et ses attributs (y compris les
   miroirs et partenaires sans écriture).
2. **Paires** (paire dédoublonnée par plus petit id) : calculer `net`, retirer les deux lignes, insérer la ligne du
   sens du net avec `sources` (`offset` × 2) ; `compensated = true` si au moins un membre avait un solde non nul.
3. **Miroirs** : pour chaque source à solde < 0, retirer sa ligne ; accumuler `−solde` sur la cible ; la ligne cible
   = solde propre (source `own` si non nul) + accumulations ; insérer dans la section du type de la cible.
4. Retrier chaque section par numéro, recalculer `total_*` (bilan) ou `total_revenues`, `total_expenses` (résultat ;
   `net_result` recalculé et **asserté** égal au brut en `debug_assert!`, journalisé en `tracing::error!` sinon).
5. Cas limites à tester : miroir dont le solde propre est lui-même de sens inverse (sa ligne reste négative si ses
   sources de sens inverse ne la compensent pas : un miroir n'a pas de miroir — relevée alors par le contrôle K10 de
   la 47-4, qui juge la ligne présentée et non le solde propre ; test de cohérence K10 / présentation : 47-4 T3) ; source archivée ; miroir dont la ligne (propre + sources) vaut zéro : ligne omise, comme une paire à net nul ; compte à lien écarté par le lecteur (inchangé, relevé par K10) ; toutes les sources de sens normal
   (le miroir sans écriture ne paraît pas) ; paire à un seul membre mouvementé ; société sans lien.

### Recâblage à la fusion de l'Epic 46 (Q5)

L'Epic 46 (`epic/46-rapports-groupes`, fiches non validées) prévoit une fonction pure `kesh-report/src/groupes.rs` et
un champ `groupedRows` par section, sous le paramètre `detailLevel` (D2, D8 de l'Epic 46). Ce qu'il faudra recâbler :

1. **Ordre** : `groupedRows` doit être calculé **sur les sections présentées**. Si la 46-1 le calcule dans
   `generate`, déplacer cet appel après `present_*` (dans le service, ou dans une étape `finaliser(rapport, liens,
   niveau)` de `kesh-report`) ; sinon la présentation tourne avant le regroupement, sans autre changement.
2. **Groupe de rattachement** : une ligne présentée va au groupe du **compte qui la porte** (miroir, ou membre du sens
   du net), jamais à celui du compte d'origine — c'est l'effet naturel de l'ordre ci-dessus si l'arbre des groupes
   contient le miroir (lecture de **tous** les comptes, NFR-46-3 : à vérifier).
3. **Lignes groupées** : la ligne `kind: account` de `groupedRows` doit transporter `sources` ; le solde en devise d'un
   groupe (D7 de l'Epic 46) se calcule sur les lignes présentées.
4. **Rendus** : 46-3 (PDF, CSV groupés) et 46-4 (MCP groupé) reprennent la marque et les sources de la 47-2b / 47-2a.
5. **Tests** : rejouer l'exemple chiffré à chaque niveau de détail ; propriété « somme des sous-totaux de premier
   niveau = total de section » (NFR-46-2) en présenté.
Ce recâblage se fait dans la story de l'epic qui fusionne la seconde (le second arrivé résout), avec revue du code
d'intégration (interaction sans conflit de texte : deux règles sur les mêmes lignes).

### Choix proposés

- **C-47-2a-1** : valeurs `brute` / `presentee` (D6 dit `presentation=brute`) ; défaut `presentee`.
- **C-47-2a-2** : champ additif `sources` omis quand vide, plutôt qu'un champ `presentation` en tête de rapport : un
  rapport sans lien reste identique octet pour octet.
- **C-47-2a-3** : la fonction prend et rend le rapport entier (et non des lignes), pour recalculer les totaux en un
  seul lieu.

### Stratégie de tests

Unitaires et propriété (fonction) ; intégration API et MCP (exemple chiffré, `brute` identique à l'actuel, valeur
inconnue en 400, société sans lien identique, autre société jamais lue) ; mutations : inverser le signe d'une source,
oublier le recalcul d'un total, appliquer la présentation à l'accueil — chacune doit rougir.

### Risques

Recâblage avec l'Epic 46 (deux règles sur les mêmes lignes) ; budgets MCP ; `fix/bilan-resultat-reporte` touche
`balance_sheet.rs` (méthode `shows_retained_earnings_line`) : conflit de texte probable, sans conflit de règle.

### References

- Fiche 47-1a (AC-9) ; `epic/46-rapports-groupes:_bmad-output/planning-artifacts/epic-46-rapports-groupes.md` (D2, D3,
  D7, D8, NFR-46-2, NFR-46-3).
- Registre de la dette du fichier d'epic (§ *Registre de la dette de l'epic*) : à la clôture de la boucle de revue,
  tout finding non remédié y est consigné, une ligne par problème ; la 47-5 en part (« on trace toujours »).

## Notes de version

Le bilan et le compte de résultat présentent les **comptes miroirs** (un solde de sens inverse paraît de l'autre côté,
sur son compte miroir) et les **paires compensées** (le net seul, sur le compte du sens du net), à toute date
d'arrêté. Aucune écriture n'est passée ; balance et grand livre montrent toujours les soldes réels. La **présentation
brute** reste disponible : `presentation=brute` à l'API, argument `presentation` des outils MCP.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
