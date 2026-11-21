# Story 46-6a : balance par groupes — modèle, API et MCP

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-46-rapports-groupes.md` — D11 modifiée (Q6 : la balance dans le
périmètre), D1 à D7, D10 ; FR-46-6, FR-46-11 ; NFR-46-1 à NFR-46-4, NFR-46-6. Release visée **0.21.0** (prévision).
Fiche écrite le 2026-11-20 sur la remarque 2 de Guy (« décrire la story de la balance, laisser le solde en dernier »),
à valider en boucle. **Suit la 46-1** ; en parallèle des 46-2 et 46-3, **après la 46-4** pour le MCP. Aucune migration.

**Découpage** : la balance par groupes (la « 46-6 » de la revue 1) touche plus de cinq modules (`kesh-report` modèle
et rendus, service et route, MCP, écran, i18n, manuel) : règle de splitting préventif, d'où **46-6a** (modèle, API,
MCP) et **46-6b** (écran, PDF, CSV). Le grand livre reste hors périmètre (Beat : « pas utile »).

## Story

En tant que comptable,
je veux la balance des comptes par groupes, avec pour chaque groupe ses sous-totaux d'ouverture, de mouvements et de
clôture, au niveau et avec les groupes ouverts ou fermés de mon choix,
afin de contrôler mes comptes au niveau des groupes, comme au bilan.

## Modules touchés (règle de splitting préventif)

Cinq, la crate `kesh-report` comptant pour un : `kesh-report` (`trial_balance` : implémentation de `LigneAgregeable` ;
`groupes` : clé de partition « nature », déjà prévue par la 46-1, et l'enveloppe `AvecGroupes<T>` que la 46-1 pose —
`TrialBalance` **ne change pas**, comme `BalanceSheet`, C-46-1-4), `kesh-api/services/rapports`,
`kesh-api/routes/reports` (`GET /reports/trial-balance`), `kesh-api/mcp/outils/rapports.rs` (`kesh_balance`), la
documentation (`docs/api-external.md`, manuel d'administration, tableau MCP).

**Ordre** (validation P1) : suit la 46-1 ; **T1, T2 et T4 en parallèle des 46-2 et 46-3 ; T3 (le MCP) après la 46-4**,
dont elle reprend les arguments, le schéma et la fonction de conversion en demande typée (aucune copie). Le tableau du
fichier d'epic le porte (« Dépend de : 46-1, 46-4 »).

## Critères d'acceptation

1. **AC-1 — Réutilisation.** La balance emploie `kesh_report::groupes::regrouper` de la 46-1, sans copie :
   `TrialBalanceRow` implémente `LigneAgregeable` avec **cinq montants** — dans l'ordre `[ouverture, débit, crédit,
   solde de la période, clôture]` — et leurs cinq montants en devise ; mêmes règles de groupes non affichés (Q2), de
   devise (Q3), de niveau, de règle d'ouverture, de position des sous-totaux et d'arbre défensif que la 46-1, **sous
   réserve de l'AC-3**. `regrouper` rend la ligne **neutre** de la 46-1 (un vecteur de montants) : la balance la
   convertit, par **une** fonction (champs nommés de l'AC-5), en `LigneBalanceGroupee`, que porte `AvecGroupes<TrialBalance>`
   (`grouped_rows: Option<Vec<LigneBalanceGroupee>>`) — `TrialBalance` et ses littéraux de tests restent intacts.
2. **AC-2 — Nature, au lieu de section.** La balance n'a pas de sections ; ses soldes sont signés du côté naturel de
   chaque compte (`trial_balance.rs:91`, `opening::signed`). La clé de partition est la **nature**
   (`opening::is_debit_natured` : `Asset` et `Expense` débitrices, `Liability` et `Revenue` créditrices) : un groupe
   dont la descendance mêle les deux natures est **dissous**, ses enfants remontent ; tout sous-total additionne donc
   des soldes de même signe. Débit et crédit d'un groupe sont la somme exacte des mouvements de sa descendance.
   **Dans le plan PME** (relevé du 2026-11-20, `pme.json`, 86 comptes) les groupes mixtes sont `6` (6950 `Revenue`
   parmi des charges), `7`, `70`, `8` et `80` — non le seul `7` ; `2 Passifs` (dettes et fonds propres, tous
   `Liability`) **reste** un groupe à la balance, alors que le bilan le dissout (46-1 AC-4) ; **conséquence assumée** :
   au niveau 1 de la balance du plan PME, les comptes des classes 6 à 8 (une vingtaine) paraissent à la profondeur 1,
   faute de groupe qui les réunisse — à dire au manuel (46-6b). La dissolution se lit, comme à la 46-1, sur les **lignes
   présentes** ; à la balance, les comptes actifs à zéro en font partie, si bien que `6` l'est toujours (6950 actif).
3. **AC-3 — Inclusion, groupes ouverts, devise.** La balance rend aussi des comptes actifs **à zéro** (règle
   d'inclusion, `trial_balance.rs:13-18`) : une ligne d'entrée à cinq montants nuls est permise, ce que ne connaît pas
   la 46-1 (une ligne de bilan n'est jamais nulle). Règles propres à la balance :
   (a) un groupe **fermé par le niveau** dont les dix montants (cinq, et leurs cinq en devise) sont nuls n'est pas
   affiché ; son zéro reste dans le sous-total de son parent ; un groupe **fermé à la main** (`closed`) reste affiché,
   même nul (AC-5 de la 46-1, validation P1 de la 46-1) ;
   (b) un groupe **ouvert** est affiché **avec toutes ses lignes d'entrée, même nulles**, comme à plat : aucun compte
   rendu à plat ne disparaît à `all` ;
   (c) pour la règle de devise unique, une ligne d'entrée dont les dix montants sont nuls **ne compte pas** (D7 : « tous
   ses comptes à solde non nul ») — un compte CHF à zéro ne retire pas sa devise à un groupe de comptes USD ;
   La ligne calculée du résultat reporté reste **hors** des groupes, à sa place d'aujourd'hui.
4. **AC-4 — Totaux et contrôles inchangés.** `total_debit`, `total_credit`, `balanced`, `retained_earnings`,
   `opening_balanced` identiques à tout niveau ; la somme des débits (crédits) des lignes de profondeur 1 égale
   `total_debit` (`total_credit`) — test de propriété de la 46-1 étendu à la balance. (Ouverture, solde et clôture ne
   se totalisent pas entre groupes de natures contraires : aucun contrôle n'est demandé sur eux au niveau de la racine.)
5. **AC-5 — API.** `GET /reports/trial-balance` (`routes/reports.rs:348`) accepte `detailLevel`, `subtotalPosition`,
   `open`, `closed`, mêmes refus et même validation (fonction du service de la 46-1) ; la réponse (`AvecGroupes<TrialBalance>`, absente de tout champ neuf sans `detailLevel`) gagne `grouping`
   (écho de la 46-1) et `groupedRows` — **un tableau JSON** (la balance n'a pas de sections ; au bilan, `groupedRows`
   est un objet par section) — de lignes `{ kind, depth, accountId, accountNumber, accountName, openingBalance,
   totalDebit, totalCredit, balance, closingBalance, currency?, openingCurrency?, totalDebitCurrency?,
   totalCreditCurrency?, balanceCurrency?, closingCurrency?, open?, active? }` (les noms de `TrialBalanceRow`, plus
   ceux de la 46-1) ; `rows`, plat, est toujours rendu. Sans paramètre : JSON identique à l'octet (NFR-46-1). `GET /reports/trial-balance` partage `ReportQuery` et **ignorait** ces paramètres (46-1) : il les lit et les refuse
   désormais comme les deux autres rapports (**par `groupes_de` et `Query<Vec<(String, String)>>`, non par des champs `Option<String>` ajoutés à `ReportQuery`**, que `vat` partage aussi : un paramètre répété y deviendrait un rejet Axum sans `VALIDATION_ERROR` — revue P2 de la 46-1, L-6). Une seule requête de plus (l'arbre), aucune sans paramètre.
6. **AC-6 — MCP.** `kesh_balance` (`ArgumentsBalance`, `deny_unknown_fields`, schéma en ligne `rapports.rs:208-209`)
   gagne les quatre arguments de la 46-4, **déclarés dans la structure** (pas de `flatten`) et convertis par la
   fonction de la 46-4. En mode groupé : la sortie gagne `groupes` (`null` sinon) = l'écho et `lignes` (genre,
   profondeur, `compteId`, `numero`, `nom`, les cinq montants `ouverture`, `debit`, `credit`, `solde`, `cloture` et les
   cinq `…Devise`, `devise`, `ouvert`) ; les lignes plates (`lignes`) sont vides ; la **tranche** (`limit`, `offset`,
   défaut 23, maximum 42) porte sur les **lignes groupées**, `total` en compte le nombre ; totaux du rapport entier
   inchangés. *Pourquoi la tranche et non le niveau seul (contraire du bilan, 46-4 C-46-4-3)* : une ligne de balance
   pèse ≈ 540 o (dix montants), et le niveau 2 du plan PME compte une cinquantaine de lignes avec les pieds — au-delà
   du plafond sans tranche. Bornes : le schéma est **unique** (`limit` maximum 42, défaut 23) pour les deux modes ; la
   ligne groupée est mesurée (`tailles_mesurees_en_devise` étendu, groupe et pied compris) ; si elle dépasse la plate,
   les **deux** bornes baissent ensemble, avec `LIMITE_DEFAUT_BALANCE` / `LIMITE_MAX_MCP_BALANCE`
   (`rapports.rs:25-29`), `bornes_attendues` et la table `mesurees` de `bornes_des_schemas`
   (`mcp_outils_e2e.rs:98`, `:161-181`) ; chiffre daté dans la doc du module. Sans argument neuf : sortie identique
   (littéral du commit de base, comme 46-4 AC-2).
7. **AC-7 — Trace d'audit inchangée** (comme 46-1 AC-11) ; `services_motif.rs`, `audit_route_registry.rs` verts.
8. **AC-8 — Arrondi.** Cette story ne formate aucun montant (JSON `Decimal`, MCP `Montant`) ; aucun `{:.2}` neuf dans
   `kesh-report`, **tel que la garde de la 48-11 l'interdira** (la 48-11 n'a pas encore de fiche : nom de la garde et
   des fonctions relevés à T0).
9. **AC-9 — Tests et documentation.** Unitaires (`LigneAgregeable` de la balance, groupes mixtes du plan PME dissous
   et `2` conservé, compte à zéro dans un groupe ouvert et dans un groupe fermé, devise avec un compte CHF nul parmi
   des comptes USD) ; intégration (`reports_e2e.rs`) : refus, niveau 2 et `all`, égalité à l'octet sans paramètre,
   sonde négative (sans dissolution par nature, le test d'un groupe mixte rougit) ; `mcp_outils_e2e.rs` (cas groupé,
   tranche sur lignes groupées, `total`), `mcp_contrat`, `mcp_gardes_lexicales`, `mcp_matrice_manuel` verts ;
   `docs/api-external.md` (balance groupée ; aucun numéro de compte hors du plan PME, gardes G4-bis et G4-ter de
   `textes_coherents.rs`) ; manuel d'administration (tableau MCP, `kesh_balance`), `.tex` seul.

## Tâches / sous-tâches

- [ ] **T0** — Reprise de la branche d'epic avec la 46-1 (et la 46-4 pour T3) fusionnées.
- [ ] **T1** — `LigneAgregeable` pour `TrialBalanceRow` ; clé « nature » ; `LigneBalanceGroupee` et sa conversion ;
      `AvecGroupes<TrialBalance>` (enveloppe de la 46-1, `TrialBalance` intact) ; tests (AC 1 à 4, 8).
- [ ] **T2** — service `balance` (`services/rapports.rs:640`) et route (AC 5, 7).
- [ ] **T3** — MCP `kesh_balance` (AC 6), **après la 46-4** ; la sortie lit l'enveloppe (`.rapport` pour les lignes plates).
- [ ] **T4** — tests d'intégration, documentation (AC 9) ; **T5** — gate complet, E2E complet au dernier commit de code.

## Dev Notes

- `crates/kesh-report/src/trial_balance.rs` (446 lignes) : `TrialBalance` (`:66`), `TrialBalanceRow` (`:91`, soldes du
  côté naturel), `RawRow`, `trial_balance_sql` (une requête, `ORDER BY a.number`), ligne calculée `retained_earnings`
  (`fetch_retained_earnings`, partagée avec le bilan), `is_opening_balanced`. `crate::opening` : `is_debit_natured`
  (`:36`), `signed`, `debit_sense`.
- Le grand livre (`general_ledger.rs`) n'est pas touché.
- **C-46-6a-1** : partition par nature (AC-2) plutôt que par type de compte : actif et charge (débiteurs) peuvent
  partager un groupe sans fausser une somme ; *écarté* : sommer en sens débit et marquer « D/C », illisible. Le
  coût — les classes 6 à 8 du plan PME restent en comptes au niveau 1 — est écrit à l'AC-2.
- **C-46-6a-2** : tranche MCP sur les lignes groupées (AC-6) : un groupe et son sous-total en pied peuvent tomber
  dans deux tranches — la description de l'outil le dit ; chaque ligne porte `profondeur` et `compteId`, de quoi
  reprendre le fil.

### References

Fiches 46-1 (§ *Le contrat*), 46-4 ; `CLAUDE.md` § *Le prompt d'une passe doit nommer le manuel*.

## Note de version

*(à écrire au développement)* — La balance des comptes se demande par groupes (API, MCP), avec les sous-totaux
d'ouverture, de mouvements et de clôture de chaque groupe ; sans paramètre, rien ne change.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-20 — Fiche écrite (remarque 2 de Guy) ; découpée de la « 46-6 » de la revue 1 par la règle de splitting
  préventif (46-6a modèle, API, MCP ; 46-6b écran, PDF, CSV). Neuf critères. `drafted`.
- 2026-11-20 — **validation P1 (Sonnet 5.5) : 0 C, 0 H, 5 M, 2 L, remédiés** (alignée à la fin de la passe sur la P1 de la 46-1, Opus 5.5 : enveloppe `AvecGroupes<T>`, groupe fermé à la main toujours affiché, dissolution sur les lignes présentes). M : dépendance cachée au MCP de la 46-4
  (« en parallèle » faux pour T3) ; ligne neutre à cinq montants sans conversion vers les champs nommés, et
  `groupedRows` ambigu (tableau, non objet par section) ; règles absentes pour une ligne d'entrée à zéro (groupe
  ouvert, compte à zéro dans la règle de devise unique) ; exemple du plan PME faux (groupes mixtes `6`, `7`, `70`,
  `8`, `80`, relevé de `pme.json`, et `2` conservé à la balance) ; tranche MCP justifiée par les tailles, bornes à
  schéma unique et tests à jour nommés. L : décompte de modules (crate) ; garde et noms de la 48-11 non établis
  (« à relever à T0 »). Axes exercés : `trial_balance.rs`, `opening.rs`, `rapports.rs` MCP, `services/rapports.rs`,
  `pme.json`, tests MCP, concordance 46-1/46-4/46-6b ; non exercés : compilation, mesures.
