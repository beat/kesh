# Story 47-3 : compte miroir — présentation au bilan et au compte de résultat

Status: review

Epic : `_bmad-output/planning-artifacts/epic-47-comptes-miroirs.md` — § *Décision du 2026-11-19 : miroir seul*,
règles R5 à R8 ; D4, D5, D7, D8 ; Q2, Q4, Q5, Q7, Q11 ; FR-47-3, FR-47-4 ; NFR-47-1, NFR-47-2, NFR-47-4. **Refonte du
2026-11-19 (miroir seul, KISS)** de l'ancienne 47-2a (fiche renommée) ; l'ancienne 47-2b (PDF et CSV présentés) est
`superseded` : PDF et CSV n'ont rien de propre au miroir. Dépend de la **47-1** (lecteur `mirror_links`). **Se
développe sans l'Epic 46** : fonction pure appliquée aux lignes de compte **avant** tout regroupement (Q5), recâblée à
sa fusion.

## Story

En tant que comptable,
je veux un bilan et un compte de résultat où le solde de sens inverse d'un compte paraît, positif, sous son compte
miroir de l'autre côté,
afin de présenter mes comptes comme le veut le CO (art. 959a), sans écriture de reclassement ni compensation.

## Modules touchés (règle de splitting préventif)

Cinq : `kesh-report` (module neuf `presentation`, et le champ `mirror_of` des lignes), `kesh-api/services/rapports`,
`kesh-api/mcp` (descriptions seulement), la documentation (`docs/api-external.md`, manuel d'administration), les tests
d'intégration de `kesh-api`. Aucun écran (47-4), aucun rendu PDF ou CSV propre.

## Critères d'acceptation

1. **Fonction pure** `kesh_report::presentation` (module neuf) : `present_balance_sheet(raw: BalanceSheet, links:
   &[MirrorLink]) -> BalanceSheet` et `present_income_statement(raw: IncomeStatement, links: &[MirrorLink]) ->
   IncomeStatement`, sans I/O, testées seules. Les générateurs `balance_sheet::generate` et
   `income_statement::generate` **restent bruts et inchangés** (appelants de production : `services/rapports.rs:619`,
   `:634`, et `balance_sheet.rs:263`, sans effet puisque le résultat net est invariant) ; l'accueil, la balance, le
   grand livre, le journal, la TVA et les projets n'y passent pas (D7, Q4, Q11). La fonction ne lit et n'écrit que les
   sections `assets`, `liabilities` (bilan), `revenues`, `expenses` (résultat) — **jamais** `equity` ni les lignes
   calculées (D8). Liens vides, ou aucune source de sens inverse → le rapport est rendu **tel quel**, sans retri
   (chemin court : la sortie **égale** l'entrée, octet pour octet en JSON).
2. **Règle** (R5) : une source dont le solde est **de sens inverse** (`balance < 0` dans le signe de sa nature) quitte
   sa section ; la ligne de son miroir, insérée dans la section du type du miroir, vaut la **somme des soldes inversés
   de ses sources de sens inverse** (plusieurs sources possibles), plus le solde propre du miroir s'il en a une ligne
   brute (état forgé seulement : 47-1 AC-8) ; numéro et libellé du miroir (Q7), `account_type` et `active` tirés de
   `MirrorLink`, `role` à `None` (un miroir n'a pas de rôle). Une
   source de solde nul ou de sens normal reste à sa place, inchangée ; un miroir qui ne reçoit aucune source de sens
   inverse ne paraît pas (il n'a pas d'écriture). **Aucune compensation** : une source de sens normal n'est jamais
   additionnée à rien.
3. **Traçabilité** (R7) : la ligne du miroir porte `mirror_of: Vec<MirrorSource>` — `{ accountId, accountNumber,
   accountName, balance }`, `balance` = solde **brut** de la source, signé dans sa nature (donc négatif), sources triées
   par numéro ; champ sérialisé `mirrorOf`, **omis quand vide** (`skip_serializing_if = "Vec::is_empty"`,
   `#[sqlx(skip)]`, défaut vide) : un rapport sans miroir appliqué rend **le même JSON qu'aujourd'hui**. Aucun champ de
   rapport neuf.
4. **Devise** (R8) : la ligne du miroir ne porte ni `currency` ni `balanceCurrency` (devise principale seulement,
   quelle que soit la devise des sources ou du miroir) ; une ligne non déplacée garde ses champs de devise. Les totaux
   restent en devise principale.
5. **Invariants** (NFR-47-2) : `total_assets` et `total_liabilities` (bilan) ou `total_revenues` et `total_expenses`
   (résultat) recalculés ; en présenté, l'équation du bilan tient dès qu'elle tient en brut (les deux côtés augmentent
   du même montant), `equation_holds` égal à celui du brut, `net_result` identique ; fonds propres,
   `retained_earnings`, `equity_result` identiques. Si le résultat net recalculé diffère du brut (défaut de code, non
   de données) : `tracing::error!` et rendu du rapport **brut** — jamais de `debug_assert!` ni de panique dans un
   handler. **Test de propriété maison**, sans dépendance neuve : une boucle sur un jeu généré déterministe (graine
   fixe) de soldes et de liens valides vérifie les deux invariants et le chemin court. Une section modifiée est retriée
   par numéro (`String::cmp`) ; d'où le chemin court de l'AC-1, qui ne retrie jamais un rapport inchangé.
6. **Date d'arrêté** (D4, Q2) : la présentation s'applique à toute période et à toute date d'arrêté, sur les soldes
   que rend déjà le générateur ; aucun traitement propre à la clôture.
7. **Service, toujours présenté** (R6) : `services/rapports.rs::bilan` et `::compte_de_resultat` lisent les liens par
   `accounts::mirror_links(lecture.pool(), acces.company_id())` (47-1 AC-8) et appliquent la fonction ; aucun paramètre
   neuf (ni route, ni service, ni trace d'audit : `services_motif.rs` inchangé), aucune présentation brute. Lecture des
   liens **sans instantané commun** avec celle des soldes (comme les sections entre elles, `homepage_summary.rs`,
   *Limite*) : un lien modifié entre les deux ne change qu'une présentation, jamais un solde. Les **exports** PDF et
   CSV du bilan et du compte de résultat passent par le même service : la ligne du miroir y paraît comme un compte
   ordinaire, avec son solde présenté, **sans colonne ni note neuves** (R7) ; aucun code de `pdf.rs` ni de `csv.rs`
   ne change, sauf les littéraux de leurs tests (AC-12).
8. **MCP** `kesh_bilan`, `kesh_compte_de_resultat` : rendent le rapport présenté (même service) ; **aucun argument ni
   champ neuf** (aucune taille à remesurer) ; leurs descriptions disent en une phrase que le solde de sens inverse d'un
   compte à miroir paraît sous son compte miroir, et que `kesh_balance` donne les soldes réels. `mcp_contrat` et
   `mcp_gardes_lexicales` verts (longueur des descriptions comprise).
9. **Accueil resté brut** (Q11) : `GET /reports/homepage-summary` inchangé ; `crates/kesh-api/tests/homepage_summary_e2e.rs`
   gagne un cas avec un miroir au compte de résultat (6950.1 → 6900.1 de l'exemple) : l'accueil garde le résultat net
   du compte de résultat mais **diffère** sur `totalRevenues` / `totalExpenses` (sonde négative : la mutation
   « appliquer la présentation à l'accueil » le rougit) ; ses cas sans miroir restent égaux au compte de résultat.
10. **Exemple chiffré** (NFR-47-4) — celui du fichier d'epic (§ *Exemple chiffré*, source unique), rejoué par un test
    d'intégration (`crates/kesh-api/tests/reports_e2e.rs`) sur une société au plan **`pme`** : comptes créés (6950.1,
    6900.1, 1040.1, 2100.1, tous **sans compte parent** : un parent deviendrait non imputable, règle 14-3a), miroirs posés, écritures contre 2800, puis bilan et compte de résultat présentés **et**
    balance (`trial-balance`, soldes réels inchangés). Le test porte la liste des montants en constante
    `EXEMPLE_PRESENTATION` (chaînes sans signe ni apostrophe) entre `// exemple-presentation:debut` et
    `// exemple-presentation:fin`, et affirme chacun : `13700.00`, `11700.00`, `3800.00`, `1800.00`, `9900.00`,
    `8200.00`, `1200.00`, `800.00`, `2000.00`, `1750.00`, `300.00`, `250.00`, `50.00`, `1700.00` (quatorze). Ces
    marqueurs `// …` sont lus par la garde **G26** de la 47-4 (fonction sœur `bloc_marque_rs`).
11. **Aucune écriture** (NFR-47-1) : générer un rapport présenté ne crée ni ne modifie aucune ligne comptable ; balance
    et grand livre identiques avant et après.
12. **Littéraux** : `AccountBalance` gagne `mirror_of` ; ses littéraux (recomptés à `fd04099f`, `git grep -c
    "AccountBalance {"` : `balance_sheet.rs` 3 dont la définition, soit 2 littéraux, `csv.rs` 8, `pdf.rs` 8,
    `benches/export.rs` 2) prennent le champ vide
    (ou `..Default::default()` si `AccountBalance` dérive `Default`, au choix du développement). Les tests de `pdf.rs`
    et `csv.rs` sont réécrits par #1327 et #1346 : conflit de texte au second arrivé, résolu en gardant les deux.
13. **Documentation** : `docs/api-external.md` — champ `mirrorOf` des lignes du bilan et du compte de résultat, règle
    (R5), devise (R8), exemple JSON aux comptes de l'exemple ; ⚠️ **changement de contrat** : dès qu'un miroir est
    posé, ces routes rendent la présentation (une intégration qui recalcule depuis les lignes de compte lit
    `GET /reports/trial-balance`) ; la phrase de l'accueil (`:305`, « `totalRevenues` et `totalExpenses` égalent ceux
    de `GET /reports/income-statement` ») devient « … égalent ceux du compte de résultat quand aucun compte miroir ne
    s'y applique ; l'accueil reste sur les soldes réels ». Manuel d'administration (`docs/manual/fr/admin-manual.tex`,
    tableau des outils MCP `:3538`, troisième colonne seule) : une phrase pour `kesh_bilan` et
    `kesh_compte_de_resultat`. ⚠️ G4-ter (`textes_coherents.rs`, forme libre « NNNN Mot ») lit `api-external.md`, le
    manuel d'administration et les sources Rust hors `tests/` (`presentation.rs` compris) ; G4-bis, le manuel
    utilisateur, le guide et les catalogues : exemples aux comptes de l'exemple seulement. Un sous-compte `6950.1 « … »`
    passe les deux gardes (seul le parent, 6950, doit être livré ; la forme libre saute un numéro à décimales). `.tex` seulement ; **le
    prompt de chaque passe de revue nomme ces documents** et exige le contrôle du PDF compilé localement.
14. **Inventaire** (47-1 AC-11) : rien à ajouter — la 47-1 ne tient plus de liste close des fichiers qui nomment
    `mirror_account_id` (validation P1 de la 47-1 et de la 47-3) ; la fonction de présentation ne lit que `MirrorLink`.

## Tâches / sous-tâches

- [x] **T0** : reprise de `main` ; si l'Epic 46 est fusionnée, appliquer d'emblée le recâblage (§ ci-dessous).
- [x] **T1 — types** (AC 3, 12) : `MirrorSource`, champ `mirror_of`, littéraux.
- [x] **T2 — fonction** (AC 1, 2, 4, 5, 6) : `presentation.rs`, tests unitaires et de propriété maison.
- [x] **T3 — service** (AC 7) et **MCP** (AC 8).
- [~] **T4 — tests d'intégration** (AC 9, 10, 11) : `reports_e2e.rs`, `homepage_summary_e2e.rs`, `mcp_outils_e2e.rs`
  (le bilan MCP de l'exemple montre 1040.1).
- [x] **T5 — documentation** (AC 13).
- [ ] **T6 — gate complet**, E2E complet au dernier commit de code.

## Dev Notes

### Code lu (à `origin/main` `fd04099f`, relevé de l'ancienne 47-2a)

- `crates/kesh-report/src/balance_sheet.rs:74-99` (`BalanceSheet`), `:113-121` (`is_empty`), `:152-174`
  (`AccountBalance`, `sqlx::FromRow`), `:204-299` (`generate`), `:319-360` (`fetch_cumulative_section`, `INNER JOIN` :
  un compte sans écriture n'y paraît pas — d'où la ligne du miroir construite depuis `MirrorLink`), `:346` (`HAVING`).
- `crates/kesh-report/src/income_statement.rs:21-31`, `:87` ; `homepage_summary.rs:6-11` (invariant avec le compte de
  résultat brut, Q11).
- `crates/kesh-api/src/services/rapports.rs:610-636` (`bilan`, `compte_de_resultat`) ;
  `crates/kesh-api/src/mcp/outils/rapports.rs` (descriptions de `kesh_bilan`, `kesh_compte_de_resultat`).
- `crates/kesh-report/src/trial_balance.rs:87-88` (côté naturel positif de `closingBalance`).

### Algorithme

1. Indexer les liens par source ; `links` ne contient que des liens lisibles (le lecteur de la 47-1 a écarté les cas
   (a)–(c) : cible absente, types qui ne sont pas de l'autre côté, chaîne) — la fonction ne revérifie rien. Une source
   de rôle de fonds propres est dans `equity`, que la fonction ne lit pas : son lien reste sans effet.
2. Pour chaque section modifiable, pour chaque ligne dont le compte est une source et dont le solde est < 0 : retirer
   la ligne, accumuler `−balance` sur le miroir, ajouter la source à son `mirror_of`.
3. Pour chaque miroir qui a reçu au moins une source : ligne = solde propre (si une ligne brute existe, forgée) +
   accumulations ; insérer dans la section du type du miroir.
4. Retrier chaque section modifiée par numéro ; recalculer ses totaux, puis `net_result` (résultat) ou
   `equation_holds` (bilan) ; contrôle défensif de l'AC-5.
5. Cas limites testés : aucune source de sens inverse ; plusieurs sources vers un miroir, dont une de sens normal ;
   source archivée ; source au solde nul ; miroir en devise étrangère ; société sans lien ; source dont la ligne n'a de
   solde qu'en devise (règle du `HAVING` : une ligne brute à CHF nul et devise non nulle n'est pas de sens inverse en
   CHF — elle reste à sa place).

### Recâblage à la fusion de l'Epic 46 (Q5)

L'Epic 46 (`epic/46-rapports-groupes`) prévoit `kesh-report/src/groupes.rs` et un champ `groupedRows` par section.
(1) `groupedRows` se calcule **sur les sections présentées** : si la 46-1 le calcule dans `generate`, déplacer cet
appel après `present_*` ; (2) une ligne de miroir va au groupe du **miroir** (effet naturel de l'ordre, si l'arbre des
groupes lit tous les comptes) ; (3) la ligne `kind: account` de `groupedRows` transporte `mirrorOf` ; (4) les rendus
groupés (46-3, 46-4) n'ont rien de propre au miroir ; (5) l'exemple se rejoue à chaque niveau de détail, et la
propriété « somme des sous-totaux de premier niveau = total de section » (NFR-46-2) en présenté. Recâblage par le
second arrivé, avec revue du code d'intégration.

### Choix KISS (refonte du 2026-11-19)

- **C-47-3-1** : **pas de présentation brute** — ni paramètre `presentation`, ni argument MCP, ni clé d'audit.
  *Écarté* : `presentation=brute|presentee`, qui touchait deux requêtes partagées, le service, la trace d'audit
  (`services_motif.rs`), le schéma, le résumé et la désérialisation MCP, et une bascule à l'écran et aux exports — pour
  un besoin que la balance de vérification couvre déjà (soldes réels), et qui n'est plus réglementaire : le miroir est
  la présentation correcte selon l'art. 959a CO, sans compensation.
- **C-47-3-2** : traçabilité par un champ de ligne `mirrorOf`, omis quand vide. *Écarté* : `sources` à genres
  (`own`, `mirror`, `offset`), drapeau `compensated`.
- **C-47-3-3** : PDF, CSV et MCP sans rien de propre au miroir (la ligne paraît comme un compte ordinaire). *Écarté* :
  marque, note des sources, mention « présentation compensée », colonnes `Presentation` et `Sources` (ancienne 47-2b),
  `sources` et `compense` en MCP.
- **C-47-3-4** : ligne du miroir en devise principale seulement. *Écarté* : la règle Q3 (devise montrée si toutes les
  sources la partagent).
- **C-47-3-5** : test de propriété maison, sans `proptest`.

### Risques

Recâblage avec l'Epic 46 ; le contrat des routes change dès qu'un miroir est posé (AC-13) ; conflit de texte avec
#1327, #1346 et la 48-11 sur les tests de `pdf.rs` / `csv.rs` (littéraux seulement). `fix/bilan-resultat-reporte`
emploie « bilan **présenté** » au sens d'« affiché » : à ne pas confondre.

### References

- Fiche 47-1 (AC-8) ; fichier d'epic (§ *Exemple chiffré*) ; `epic/46-rapports-groupes` (D2, D3, D7, D8, NFR-46-2,
  NFR-46-3).
- Registre de la dette du fichier d'epic : à la clôture de la boucle de revue, tout finding non remédié y est consigné ;
  la 47-5 en part (« on trace toujours »).

## Notes de version

Le bilan et le compte de résultat présentent les **comptes miroirs** : le solde de sens inverse d'un compte (banque à
découvert, dette sociale à solde débiteur, gains de change à solde débiteur) paraît, positif, sous son compte miroir de
l'autre côté, à toute date d'arrêté, à l'écran comme dans les exports et les outils MCP. Aucune écriture n'est passée,
rien n'est compensé ; la balance et le grand livre montrent toujours les soldes réels. ⚠️ Pour les intégrations : dès
qu'un miroir est posé, `GET /reports/balance-sheet` et `/income-statement` rendent la présentation ; les soldes réels
restent à `GET /reports/trial-balance`.

## Change Log

- 2026-11-19 — retouche de la validation P3 des fiches 47-2 et 47-4 (Sonnet 5.5) : AC-10, les sous-comptes de l'exemple
  se créent **sans parent** (propagation de la 47-2 P2 M1 ; signalé par la P2 de la 47-4). Aucune autre modification.
- 2026-11-19 — **Refonte du 2026-11-19 (miroir seul, KISS)** : fiche renommée de `47-2a-presentation-fonction-api-mcp` ;
  l'ancienne 47-2b (PDF et CSV) est superseded. Retirés : paire et `net`, « paire appliquée », `compensated`, `sources`
  à genres, paramètre `presentation=brute|presentee` (route, service, audit, MCP), règle de devise Q3, `proptest`,
  ancien exemple (paire 6900/6950, miroirs vers 1040 et 2100, comptes imputables). Gardés : fonction pure avant les
  groupes, générateurs inchangés, chemin court, invariants et repli défensif, lecture sans instantané commun, accueil
  brut et sa sonde négative, exemple rejoué par un test à marqueurs `//` (nouvel exemple du fichier d'epic, quatorze
  montants), recâblage de l'Epic 46, littéraux des tests de `pdf.rs` / `csv.rs`. Quatorze critères (quatorze avant).
  **Statut remis à `drafted`** en attendant la revalidation.

- 2026-11-19 — validation P1 de la fiche refondue (Opus 5.5, avec la 47-1) : **0 C / 0 H / 0 M / 4 L, remédiés** (les
  MEDIUM de la passe portent sur la 47-1, rapport commun). L1 : la ligne du miroir exige `active` (`AccountBalance`),
  absent de `MirrorLink` → ajouté à la 47-1 AC-8, AC-2 ici. L2 : `balance_sheet.rs` porte 2 littéraux
  `AccountBalance` (le troisième est la définition) (AC-12). L3 : G4-bis et G4-ter mal attribués (G4-bis ne lit ni
  `api-external.md` ni le manuel d'administration) ; `6950.1 « … »` passe les deux (seul 6950 doit être livré ; la
  forme libre saute les décimales) (AC-13). L4 : AC-14 et algorithme § 1 alignés sur la 47-1 (plus de liste close des
  fichiers ; lecteur à trois cas). Exemple chiffré refait au centime : 2800 = 14 000 − 5 800 = 8 200 ; actif brut
  11 700, passif 1 800, fonds propres 8 200 + 1 700 = 9 900, équation vraie ; présenté 13 700 = 3 800 + 9 900 ;
  produits 1 750 → 2 000, charges 50 → 300, résultat 1 700 dans les deux ; comptes et types relus à `pme.json`
  (1040, 2100, 2270, 2800 `EquityCapital`, 6900, 6950). Vérifié : appelants de `generate` (`services/rapports.rs:619`,
  `:634`, `balance_sheet.rs:263`) ; exports et MCP par le service (`routes/reports.rs:393`, `:524`, `mcp/outils/
  rapports.rs:305`, `:357`) ; le MCP projette ses propres lignes (`lignes()`) : `mirrorOf` n'y paraît pas, conforme à
  l'AC-8 ; `api-external.md` (accueil) et `admin-manual.tex:3538-3539`. Rapport :
  `kesh-gate-logs/epic-47-validation/47-3-miroir-presentation-rapports-P1.md`.

### Historique — Change Log de la 47-2a (avant la refonte, laissé tel quel)

- 2026-11-19 — validation P1 (Opus 5.5) : 0 C / 0 H / 6 M / 9 L, remédiés. M : « ligne présentée » mal bornée —
  une cible sans source de sens inverse et une paire à un seul membre positif portaient des `sources` (et
  `compensated`) sur un rapport identique au brut, contre 47-2b AC 3 (AC 2, 3, 6, algorithme) ; liens incohérents
  (paire non réciproque, fonds propres) non écartés — couvert par la 47-1a AC-8 refondue en parallèle, la fonction
  ne touchant jamais `equity` (AC 1) ; trace d'audit non spécifiée alors que `services_motif.rs` fige le détail des
  dix-huit traces (AC 9) ; Q11 : `api-external.md:305` et `homepage_summary_e2e.rs` comparent l'accueil au compte de
  résultat par défaut, devenu présenté (AC 13, AC 14 neuf) ; littéraux `AccountBalance` / `BalanceSheet` dans les
  tests de `pdf.rs` / `csv.rs` (#1327, #1346) non déclarés (T1, fichier d'epic) ; contrat MCP sous-spécifié
  (`ArgumentsPeriode` partagé avec `kesh_balance`, `SCHEMA_PERIODE` qui ne l'est pas, type des sources sous
  `sortie!`, résumé visible de l'Administrateur, mesure des tailles) (AC 10). L : règle de devise (sources comptées,
  signes) ; changement de contrat du défaut à écrire ; exemple (`equationHolds`, `compensated`, `closingBalance`) ;
  `ReportQuery` / `ExportQuery` partagées avec la balance ; `debug_assert!` remplacé ; lecture sans instantané ;
  homonymie `own` avec l'Epic 46 ; omission d'une ligne présentée nulle (règle du `HAVING` du brut, AC 2, AC 3,
  algorithme § 3) et liste des sept cas écartés du lecteur alignée sur 47-1a AC-8 (P3 de la 47-4) ; tri `String::cmp` contre collation SQL (chemin court) ; `is_empty` à aligner
  sur « `compensated` ⇒ jamais vide » de la 47-3b. Rapport :
  `kesh-gate-logs/epic-47-validation/47-2a-presentation-fonction-api-mcp-P1.md`.

- 2026-11-19 — validation P2 (Sonnet 5.5) : 0 C / 0 H / 1 M / 1 L, remédiés. Exemple de l'AC-11 recalculé au centime
  (actif brut 11 700.00 = 500 + 12 000 − 800 ; passif brut 1 800.00 = 3 000 − 1 200 ; contrepartie 2800 = 9 330.00,
  fonds propres 9 900.00 ; présenté 13 700.00 / 3 800.00, équation 13 700 = 3 800 + 9 900 vraie dans les deux ; paire
  1 020 − 450 = 570, résultat net inchangé) : tient ; comptes et noms vérifiés aux trois plans (`assets/charts/*.json`).
  M : plan `pme` requis par 1040 non écrit à l'AC-11 (relevé P3 de la 47-2b), et absence de source `own` de 1040
  (`Sources` = `2270:-1200.00`) — écrits. L : liste `EXEMPLE_PRESENTATION` (ajoutée par la validation P2 de la 47-3b)
  relue et conforme. Concordances : forme du vecteur `sources` ; `compensated` ⇒ jamais vide (47-3b AC-4, `is_empty`
  ici) ; exemple identique 47-2b / 47-3b ; le fichier d'epic ne porte pas l'exemple (source unique, AC-11).
  Rapport : `kesh-gate-logs/epic-47-validation/47-2a-presentation-fonction-api-mcp-P2.md`.

- 2026-11-19 — validation P3 (Opus 5.5) : 0 C / 0 H / 0 M / 7 L, remédiés. Axes que la P2 déclarait non exercés,
  repris : (1) concordance avec la 47-1a AC-8/AC-9 — liens écartés jamais rendus, liste close (a)–(g), compte gardé
  dans le vecteur liens à `None`, archivé non écarté, paire réciproque, attributs du miroir sans écriture
  (`PresentationLinkAccount` : type, rôle, devise, `active`) : concordant ; L1 « `PresentationLinks` vide » précisé
  (un compte du vecteur sans lien ne compte pas, AC 1) ; (2) lignes citées relues à `origin/main` `fd04099f` — toutes
  justes, sauf : L2 `income_statement::generate` a un troisième appelant (`balance_sheet.rs:263`, `equity_result`,
  sans effet : AC 1) ; L3 handler d'export du compte de résultat (`routes/reports.rs:504-530`) hors de la plage
  citée ; L4 `details_periode` (`:595`) et `tracer` (`:414-460`) à modifier, `emit_report_audit` au huitième argument
  (`clippy::too_many_arguments`), appels de `services_motif.rs` (`:1104`, `:1110`, `:1882`) (AC 9) ; L5 `kesh_balance`
  désérialise `ArgumentsBalance` (`:50`) — le couplage n'est que le littéral de `:225` (AC 10) ; (3) manuel —
  matrice MCP gardée par `mcp_matrice_manuel.rs` (nom, portée seuls), seuil `:3564` : L6 exemples de `api-external.md`,
  du manuel d'administration et des doc-comments soumis à G4-bis/G4-ter (AC 13). L7 `own` défini par la ligne brute
  (CHF ou devise, AC 6), pour concorder avec la règle de devise de l'AC 7. Relevé de la P3 de la 47-3b, transmis par
  l'orchestrateur : marqueurs `// exemple-presentation:…` lus par `bloc_marque_rs` (47-3b AC-10), non par
  `bloc_marque` (AC 11). Remédiation P2 relue : exemple au plan `pme`, `sources` de 1040 et 2100, liste
  `EXEMPLE_PRESENTATION` — justes et concordants avec 47-2b AC 1, 4 et 47-3b AC-10. Rapport :
  `kesh-gate-logs/epic-47-validation/47-2a-presentation-fonction-api-mcp-P3.md`.


## Dev Agent Record

### Agent Model Used

Sonnet 5.5.

### Completion Notes List

**Fait (commits `213c66b7`, `4797ba9c` sur `story/47-3`).**
- `crates/kesh-report/src/presentation.rs` (neuf) : `present_balance_sheet`, `present_income_statement`, cœur commun
  `reclasser` (R5 ; sens inverse = `balance < 0` en CHF ; plusieurs sources ; ligne brute forgée du miroir additionnée ;
  ligne nulle omise ; tri par numéro ; `role`/`currency`/`balance_currency` à `None`). Chemin court : sans lien, ou sans
  source de sens inverse, le rapport brut est rendu tel quel. Repli sur le brut avec `tracing::error!` si l'équation
  (`equation_holds`) ou le résultat net recalculés diffèrent. `equity` et lignes calculées jamais lues. Aucun `{:.2}`,
  aucune fonction d'arrondi ajoutée.
- `AccountBalance.mirror_of: Vec<MirrorSource>` (`#[sqlx(skip)]`, omis du JSON si vide) ; littéraux des tests de
  `pdf.rs`, `csv.rs`, `balance_sheet.rs` et de `benches/export.rs` complétés (`Vec::new()`).
- `services/rapports.rs::bilan` et `::compte_de_resultat` : lecture de `accounts::mirror_links` puis présentation,
  toujours ; aucune route, paramètre ni trace d'audit neufs. Générateurs inchangés (l'accueil et `equity_result` du bilan
  restent bruts).
- MCP : phrase ajoutée aux descriptions de `kesh_bilan` et `kesh_compte_de_resultat`.
- Docs : `docs/api-external.md` (paragraphe « Les comptes miroirs au bilan et au compte de résultat », changement de
  contrat, phrase de l'accueil) ; `docs/manual/fr/admin-manual.tex` (tableau MCP) ; PDF non commités.
- Tests : 7 tests de `presentation::tests` dont la propriété maison (500 tours, graine fixe : équation, résultat net,
  fonds propres, totaux recomposés, chemin court octet pour octet) ; `reports_e2e::mirror_example_is_replayed_by_the_reports`
  (exemple chiffré, quatorze montants entre les marqueurs `// exemple-presentation:debut/fin`, balance inchangée, aucune
  écriture) ; `homepage_summary_e2e::mirror_leaves_the_homepage_on_real_balances`.

**Écarts et réserves honnêtes.**
- **Aucun test MCP d'intégration** du bilan présenté (T4 partielle) : le MCP passe par le même service, couvert en HTTP ;
  seules les descriptions changent (contrat MCP à rejouer).
- Le test de l'exemple monte la société avec `OrgType::Independant` (helper existant) et charge le plan `pme` par
  `load_chart("pme")` : le plan est celui de l'exemple, le type d'organisation non.
- **Sondes négatives par mutation, exécutées** : (1) accueil présenté (`homepage_summary`) : rouge sur
  `homepage_summary_e2e::mirror_leaves_the_homepage_on_real_balances` (`kesh-gate-logs/s47-3/mut-accueil.log`) ;
  (2) présentation sans retri : rouge sur `presentation::tests::banque_a_decouvert_passe_au_passif`
  (`mut-retri.log`).
- **Gates rejoués sur la cible propre** `/home/beat/claude/kesh-targets/kesh-wt47s3`, au commit `50a8c906` :
  backend (nextest, base `kesh_s473`) **6202 passés, 14 ignorés** ; E2E complet **480 passés, 10 échecs attendus**
  (les sept de la KF-029 #97, KF-065 #1282 ×2, KF-067 #1284) ; frontend **4658 tests** (`npm run check`,
  `lint-i18n-ownership`, build verts) ; `gate-ci` (`cargo test`, journal `s47-3/gate-ci.log`) **804 passés, 1 échec,
  5 ignorés** : l'échec est `admin_auto_backup_e2e::disque_sature_pendant_l_ecriture` (panique de `zip-2.6.1`
  `write.rs:1078`, `Failed` au lieu de `DiskFull`), **instable déjà vu** sur 39-11a, 40-7a et 38-11d, sans rapport
  avec la story ; **rejeu de 3 passages** (`s47-3/rejeu-disque.log`) : 20 passés, 0 échec, trois fois. Ce gate
  (`cargo test`) ne vaut que pour ce commit ; aucune ligne de code depuis.

**À recâbler à la fusion de l'Epic 46 (Q5)** : (1) `groupedRows` à calculer **après** `present_*` (si la 46-1 le calcule
dans `generate`, déplacer l'appel dans le service) ; (2) la ligne de miroir doit aller au groupe du miroir (l'arbre
lit toutes les lignes présentées) ; (3) la ligne `kind: account` de `groupedRows` doit transporter `mirrorOf` ;
(4) rejouer l'exemple à chaque niveau de détail et la propriété NFR-46-2 en présenté ; revue du code d'intégration.

### File List

- crates/kesh-report/src/presentation.rs (neuf)
- crates/kesh-report/src/{lib,balance_sheet,csv,pdf}.rs, crates/kesh-report/benches/export.rs
- crates/kesh-api/src/services/rapports.rs, crates/kesh-api/src/mcp/outils/rapports.rs
- crates/kesh-api/tests/{reports_e2e,homepage_summary_e2e}.rs
- docs/api-external.md, docs/manual/fr/admin-manual.tex
- _bmad-output/implementation-artifacts/47-3-miroir-presentation-rapports.md, sprint-status.yaml
