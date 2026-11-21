# Story 46-6a : balance par groupes — modèle, API et MCP

Status: ready-for-dev

Epic : `_bmad-output/planning-artifacts/epic-46-rapports-groupes.md` — D11 modifiée (Q6 : la balance dans le
périmètre), D1 à D7, D10 ; FR-46-6, FR-46-11 ; NFR-46-1 à NFR-46-4, NFR-46-6. Release visée **0.21.0** (prévision).
Fiche écrite le 2026-11-20 sur la remarque 2 de Guy (« décrire la story de la balance, laisser le solde en dernier »),
à valider en boucle. **Suit la 46-1** ; en parallèle des 46-2 et 46-3, **après la 46-4** pour le MCP. Aucune migration.
**Corrigée le 2026-11-21, avant développement**, sur l'API réelle de la 46-1 (relevée sur `story/46-1` à `de0a582f` :
§ *L'API de la 46-1*, Dev Notes) et sur les comptes miroirs de la 47-3 (`story/47-3` à `b4d17d70`) ; validation close le 2026-11-21 (P1 Sonnet, P2 Opus ; Change Log).

**Découpage** : la balance par groupes (la « 46-6 » de la revue 1) touche plus de cinq modules (`kesh-report` modèle
et rendus, service et route, MCP, écran, i18n, manuel) : règle de splitting préventif, d'où **46-6a** (modèle, API,
MCP) et **46-6b** (écran, PDF, CSV). Le grand livre reste hors périmètre (Beat : « pas utile »).

## Story

En tant que comptable,
je veux la balance des comptes par groupes, avec pour chaque groupe ses sous-totaux d'ouverture, de mouvements et de
clôture, au niveau et avec les groupes ouverts ou fermés de mon choix,
afin de contrôler mes comptes au niveau des groupes, comme au bilan.

## Modules touchés (règle de splitting préventif)

Cinq, la crate `kesh-report` comptant pour un : `kesh-report` (dans `groupes.rs`, à côté de `bilan` et
`compte_de_resultat` : `impl LigneAgregeable for TrialBalanceRow`, `LigneBalanceGroupee` et le constructeur
`AvecGroupes::<TrialBalance, Vec<LigneBalanceGroupee>>::balance` — placés là pour lire les fonctions privées du module,
`ArbreComptes::ancetres` et `Grouping::depuis`, sans en élargir la visibilité ; une retouche d'`Agregat::ajouter_ligne`
pour l'AC-3 (c) ; l'enveloppe `AvecGroupes<T, S>` de la 46-1, `S = Vec<LigneBalanceGroupee>` puisque la balance n'a
pas de sections — `TrialBalance` et `trial_balance.rs` **ne changent pas**, comme `BalanceSheet`, C-46-1-4), `kesh-api/services/rapports`,
`kesh-api/routes/reports` (`GET /reports/trial-balance`), `kesh-api/mcp/outils/rapports.rs` (`kesh_balance`), la
documentation (`docs/api-external.md`, manuel d'administration, tableau MCP).

**Ordre** (validation P1) : suit la 46-1 ; **T1, T2 et T4 en parallèle des 46-2 et 46-3 ; T3 (le MCP) après la 46-4**,
dont elle reprend les arguments, le schéma et la fonction de conversion en demande typée (aucune copie). Le tableau du
fichier d'epic le porte (« Dépend de : 46-1, 46-4 »).

## Critères d'acceptation

1. **AC-1 — Réutilisation.** La balance emploie l'API de la 46-1, sans copie, dans l'ordre où `regrouper_sections` le
   fait pour le bilan : `Dissolution::calculer(arbre, …)` avec la **nature** pour clé (AC-2), puis
   `regrouper(&lignes, arbre, &dissous, options) -> SectionGroupee` sur sa liste unique (écart E2 de la 46-1), puis
   `Grouping::depuis(options, &[section])` pour l'écho. `TrialBalanceRow` implémente `LigneAgregeable` avec **cinq
   montants** — `montants()` dans l'ordre `[opening_balance, total_debit, total_credit, balance, closing_balance]` —
   et `montants_devise()` = les cinq `…_currency` dans le même ordre (un champ absent valant zéro) si `currency` est
   `Some`, `None` sinon ; mêmes règles de groupes non affichés (Q2), de devise (Q3), de niveau, de règle d'ouverture
   (`OptionsGroupes::est_ouvert`), de position des sous-totaux et d'arbre défensif que la 46-1, **sous réserve des
   AC-2 et AC-3**. `regrouper` rend des `LigneGroupee` **neutres** (vecteur de montants, `source` = **indice dans la
   liste passée à `regrouper`**) : la balance les convertit, par **une** fonction (`LigneBalanceGroupee::depuis(l,
   &lignes)`, champs nommés de l'AC-5, `active` lu par `source` comme `LigneRapport::depuis`), en
   `LigneBalanceGroupee`, que porte `AvecGroupes<TrialBalance, Vec<LigneBalanceGroupee>>` (`grouped_rows:
   Option<Vec<LigneBalanceGroupee>>`) — `TrialBalance` et ses littéraux de tests restent intacts. ⚠️ Les lignes passées
   à `regrouper` sont celles de `rows` **moins** les lignes propres nulles des comptes de groupe (AC-3 (d)) : la
   conversion lit `source` dans **cette** liste filtrée, jamais dans `rows`. Les lignes retenues sont **clonées** dans
   un `Vec<TrialBalanceRow>` (`TrialBalanceRow` dérive `Clone`, `trial_balance.rs:89`), que reçoit `regrouper(lignes:
   &[L], …)` : seul `TrialBalanceRow` implémente `LigneAgregeable` ici, aucune impl pour une référence n'est posée.
2. **AC-2 — Nature, au lieu de section.** La balance n'a pas de sections ; ses soldes sont signés du côté naturel de
   chaque compte (`trial_balance.rs:91`, `opening::signed`). La clé de partition est la **nature**
   (`opening::is_debit_natured(row.account_type)` : `Asset` et `Expense` débitrices, `Liability` et `Revenue`
   créditrices), passée à `Dissolution::calculer` comme paire `(account_id, nature)` : un groupe dont la descendance
   mêle les deux natures est **dissous**, ses enfants remontent ; tout sous-total additionne donc des soldes de même
   signe. Débit et crédit d'un groupe sont la somme exacte des mouvements de sa descendance. **Seules les lignes non
   nulles comptent pour la dissolution** (C-46-6a-3) : une ligne dont les dix montants (cinq, et leurs cinq en devise)
   sont nuls n'entre pas dans les paires passées à `Dissolution::calculer` — un zéro n'additionne aucun signe
   contraire, et la balance, qui rend les comptes actifs à zéro, se comporte alors comme le bilan, où une ligne nulle
   n'existe pas (`HAVING`). La dissolution se lit donc, comme à la 46-1, sur les **lignes présentes et non nulles**.
   **Dans le plan PME** (relevé du 2026-11-20, `pme.json`, 86 comptes) les groupes qui peuvent être mixtes sont `6`
   (6950 `Revenue` parmi des charges), `7`, `70`, `8` et `80` — non le seul `7` ; ils sont dissous dès que des comptes
   des deux natures y portent un montant non nul (ouverture ou mouvement), non d'office ; `2 Passifs` (dettes et fonds
   propres, tous `Liability`) **reste** un groupe à la balance, alors que le bilan le dissout (46-1 AC-4) ;
   **conséquence assumée** : quand `6`, `7` et `8` sont dissous, au niveau 1 de la balance du plan PME, les comptes des
   classes 6 à 8 (dix-huit : les treize comptes directs de `6`, dont 6950, `7000`, `7010`, `8000`, `8010`, `8900`)
   paraissent à la profondeur 1, avec le groupe `60`, faute de groupe qui les réunisse — à dire au manuel (46-6b).
   **Comptes miroirs (Epic 47).** La balance n'est **pas** présentée (Epic 47, D7 : la balance donne les soldes réels)
   : un compte miroir y paraît comme tout compte actif, à zéro (il ne porte pas d'écriture). Rangé sous un groupe d'un
   **autre type** que le sien — état admis, avertissement non bloquant, décision du responsable de l'Epic 47 du
   2026-11-21 —, il ne dissout pas ce groupe à la balance, puisque sa ligne est nulle ; au bilan présenté, le même
   groupe est dissous dès que le miroir reçoit un solde (46-1 AC-4, fiche 47-3). Aucun avertissement n'est porté par
   cette story (C-46-6a-4).
3. **AC-3 — Inclusion, groupes ouverts, devise.** La balance rend aussi des comptes actifs **à zéro** (règle
   d'inclusion, `trial_balance.rs:13-18`) : une ligne d'entrée à cinq montants nuls est permise, ce que ne connaît pas
   la 46-1 (une ligne de bilan n'est jamais nulle). Règles propres à la balance :
   (a) un groupe **fermé par le niveau** dont les dix montants (cinq, et leurs cinq en devise) sont nuls n'est pas
   affiché ; son zéro reste dans le sous-total de son parent ; un groupe **fermé à la main** (`closed`) reste affiché,
   même nul (AC-5 de la 46-1, validation P1 de la 46-1) ;
   (b) un groupe **ouvert** est affiché **avec toutes ses lignes d'entrée, même nulles**, comme à plat : aucun compte
   rendu à plat ne disparaît à `all` ;
   (c) pour la règle de devise unique, une ligne d'entrée dont les dix montants sont nuls **ne compte pas** (D7 : « tous
   ses comptes à solde non nul ») — un compte CHF à zéro ne retire pas sa devise à un groupe de comptes USD. ⚠️ La
   46-1 ne le fait pas : `Agregat::ajouter_ligne` (privé, `groupes.rs`) marque `ligne_principale` pour toute ligne
   sans devise, nulle ou non, et ouvre une entrée `par_devise` pour toute ligne en devise. Cette story le retouche :
   une ligne dont `montants()` et `montants_devise()` sont entièrement nuls n'agit ni sur `ligne_principale` ni sur
   `par_devise` (ses montants, nuls, ne changent ni la somme ni `est_nul`) ; le commentaire du champ
   `ligne_principale` se met à jour avec la retouche. Sans effet sur le bilan et le compte de
   résultat, qui n'ont jamais de ligne entièrement nulle (`HAVING balance != 0 OR balance_currency != 0` ; la ligne
   d'un miroir de la 47-3 est non nulle) : les tests et les fixtures de la 46-1 restent verts **sans modification** ;
   (d) **la ligne propre d'un compte de groupe** : la balance rend aussi les comptes de groupe eux-mêmes (actifs, donc
   rendus même sans mouvement — `1`, `10`, `14`… du plan PME), alors qu'au bilan un compte de groupe n'entre que s'il
   porte un solde. Une telle ligne dont les dix montants sont nuls **n'est pas** une ligne d'entrée du regroupement :
   elle n'est pas passée à `regrouper` (filtre de l'AC-1), si bien qu'elle ne donne pas de ligne `own` (D5 vise un
   groupe qui porte lui-même des écritures) ; elle ne compte ni pour la dissolution (AC-2) ni pour la devise ((c)) ; le
   groupe est montré par sa ligne `group`. Est « compte de groupe » au sens de ce filtre un compte qui est ancêtre
   (`ArbreComptes::ancetres`, depuis `parent_id`) d'**une autre ligne** de la balance : un compte de groupe dont aucun
   sous-compte n'est rendu reste une ligne d'entrée (ligne `account` à zéro), conformément à (b). Un groupe **dissous**
   dont la ligne propre est nulle (`6`, `7`, `70`, `8`, `80` du plan PME, quand ils sont dissous) ne paraît donc pas
   du tout en mode groupé : c'est la seule exception à (b), écrite au manuel (46-6b). Une ligne propre non nulle
   (ouverture ou mouvement) suit la règle de la 46-1 (ligne `own`, ou ligne `account` si le groupe est dissous).
   La ligne calculée du résultat reporté reste **hors** des groupes, à sa place d'aujourd'hui.
4. **AC-4 — Totaux et contrôles inchangés.** `total_debit`, `total_credit`, `balanced`, `retained_earnings`,
   `opening_balanced` identiques à tout niveau ; la somme des débits (crédits) des lignes de profondeur 1 égale
   `total_debit` (`total_credit`), lignes `subtotal` **exclues** (en `footer`, le pied d'un groupe de profondeur 1
   porte la même profondeur et le même montant que sa ligne `group`) — test de propriété de la 46-1 étendu à la
   balance. (Ouverture, solde et clôture ne
   se totalisent pas entre groupes de natures contraires : aucun contrôle n'est demandé sur eux au niveau de la racine.)
5. **AC-5 — API.** `GET /reports/trial-balance` (`get_trial_balance`, `routes/reports.rs`) accepte `detailLevel`,
   `subtotalPosition`, `open`, `closed`, mêmes refus et même validation que la 46-1, **paramètre répété compris**, lus
   comme aux routes JSON du bilan : second extracteur `Query<Vec<(String, String)>>`, `acces.exiger(
   rapports::EXIGENCES_BALANCE.affichage)?` **avant** `groupes_de(&paires)?` (ordre de la revue P1 de la 46-1, L-5 ;
   le handler n'a aujourd'hui aucun `exiger`, le service seul contrôle), puis `rapports::balance_par_groupes`. Au
   service, comme l'écart E1 de la 46-1 : `balance(acces, lecture, demande, usage)` garde ses quatre arguments et
   délègue à `balance_par_groupes(acces, lecture, demande, Option<DemandeGroupes>, usage)` avec `None` (appelants
   inchangés : `services_motif.rs:1116`, `export_trial_balance`, `kesh_balance`) ; les deux rendent
   `Rapport<AvecGroupes<TrialBalance, Vec<LigneBalanceGroupee>>, U>`, et `export_trial_balance` rend
   `&report.rapport` (export groupé : 46-6b), `kesh_balance` lit `.rapport` jusqu'à T3. L'arbre se lit par
   `arbre_des_comptes` de la 46-1, seulement avec une demande de groupes. La réponse (absente de tout champ neuf sans
   `detailLevel`) gagne `grouping`
   (écho de la 46-1) et `groupedRows` — **un tableau JSON** (la balance n'a pas de sections ; au bilan, `groupedRows`
   est un objet par section) — de lignes `{ kind, depth, accountId, accountNumber, accountName, openingBalance,
   totalDebit, totalCredit, balance, closingBalance, currency?, openingCurrency?, totalDebitCurrency?,
   totalCreditCurrency?, balanceCurrency?, closingCurrency?, open?, active? }` (les noms de `TrialBalanceRow`, plus
   ceux de la 46-1 ; ni `role` ni `mirrorOf` : `TrialBalanceRow` n'en a pas, et la balance n'est pas présentée) ;
   `rows`, plat, est toujours rendu. Sans paramètre : JSON identique à l'octet (NFR-46-1), prouvé contre une **fixture écrite au commit de base** (T0),
   comme la 46-1 AC-8. `GET /reports/trial-balance` partage `ReportQuery` et **ignorait** ces paramètres (46-1) : il les lit et les refuse
   désormais comme les deux autres rapports. Une seule requête de plus (l'arbre), aucune sans paramètre.
6. **AC-6 — MCP.** `kesh_balance` (`ArgumentsBalance`, `deny_unknown_fields`, schéma en ligne `rapports.rs:208-209`)
   gagne les quatre arguments de la 46-4, **déclarés dans la structure** (pas de `flatten`) et convertis par la
   fonction de la 46-4. La sortie `Balance` gagne un champ `groupes` (`null` sans `detailLevel`, convention C-37-9,
   comme 46-4 AC-3). En mode groupé, `groupes` = l'écho de la 46-4 (`niveau`, `tousLesNiveaux`, `positionSousTotaux`,
   `profondeurMax`, `ouverts`, `fermes`), **repris par une fonction de construction partagée extraite par la 46-4** (`sortie!` interdit `flatten` : la 46-6a n'en recopie ni les six champs ni leur conversion ; si la 46-4 ne l'a pas extraite, la 46-6a le fait, sans copie) et **`groupes.lignes`**, les lignes groupées (genre, profondeur, `compteId`,
   `numero`, `nom`, les cinq montants `ouverture`, `debit`, `credit`, `solde`, `cloture` et les cinq `…Devise`,
   `devise`, `ouvert`) ; le champ plat existant `lignes` (au niveau supérieur, lu par `mcp_outils_e2e.rs:753`) est
   alors vide ; la **tranche** (`limit`, `offset`,
   défaut 23, maximum 42) porte sur les **lignes groupées**, `total` en compte le nombre ; totaux du rapport entier
   inchangés. *Pourquoi la tranche et non le niveau seul (contraire du bilan, 46-4 C-46-4-3)* : une ligne de balance
   pèse ≈ 540 o (dix montants), et le plan PME rend, en `footer`, **42 lignes groupées au niveau 2 et 97 à `all`**
   (calcul du 2026-11-20 sur `pme.json`, tous les comptes actifs, lignes propres nulles des groupes exclues, AC-3 (d))
   — au-delà du plafond (`MCP_ENVOI_OCTETS_MAX`, 24 576 o) sans tranche **à `all`** (97 à 107 lignes, ≈ 52 à 58 Ko) ;
   au niveau 2 (42 ou 43 lignes, ≈ 22,7 à 23,2 Ko à 540 o la ligne), à mesurer au T3, la ligne groupée pouvant peser
   plus que la plate. *Recalculé le 2026-11-21 sous la règle C-46-6a-3* (simulation
   de `regrouper` sur `pme.json`, comptes de groupe sans solde propre, script
   `kesh-gate-logs/s46-specs/sim-balance-pme.py`) : 42 au niveau 2 et 97 à `all` quand des comptes des deux natures
   portent un montant dans `6` à `8` (ces groupes dissous) ; **au plus 43 au niveau 2 et 107 à `all`** quand aucun
   groupe n'est dissous (p. ex. balance entièrement nulle à `all`). La conclusion ne change pas. Bornes : le schéma est **unique** (`limit` maximum 42, défaut 23) pour les deux modes ; la
   ligne groupée est mesurée (`tailles_mesurees_en_devise` étendu, groupe et pied compris) ; si elle dépasse la plate,
   les **deux** bornes baissent ensemble, avec `LIMITE_DEFAUT_BALANCE` / `LIMITE_MAX_MCP_BALANCE`
   (`rapports.rs:25-29`), `bornes_attendues` et la table `mesurees` de `bornes_des_schemas`
   (`mcp_outils_e2e.rs:98`, `:161-181`) ; chiffre daté dans la doc du module. **Sans argument neuf** (C-46-6a-5) : la
   sortie est celle d'aujourd'hui **au seul champ `groupes: null` près**. Le littéral du commit de base (dates de la
   période en jetons, comme 46-4 AC-2) **reste tel qu'écrit à T0** ; le test en analyse la partie JSON, y ajoute la
   clé `groupes` à `null` au niveau supérieur, la re-sérialise par le même chemin que la sortie
   (`serde_json::to_value` puis `to_string_pretty`, `sortie.rs:489-493`), et compare le texte rendu **entier**, en-tête
   compris, à ce texte : toute autre différence rougit. L'argument « complet » de `kesh_balance` dans `TEMOINS`
   (`tests/common/mcp_pieges.rs:186` ; entrée `kesh_balance` `:238-243` sur `story/46-1` à `de0a582f`) gagne les
   quatre arguments
   (`proprietes_du_schema_acceptees`), et `kesh_balance` recopie ses quatre champs dans l'`ArgumentsPeriode` qu'il
   construit (`rapports.rs:225-229`), que lit la fonction de conversion de la 46-4. L'outil appelle
   `rapports::balance_par_groupes` avec la demande que rend cette conversion (`DemandeGroupes::depuis_valeurs`, entrée
   typée de la 46-1), et lit les lignes plates dans `.rapport`.
7. **AC-7 — Trace d'audit inchangée** (comme 46-1 AC-11) ; `services_motif.rs`, `audit_route_registry.rs` verts.
8. **AC-8 — Arrondi.** Cette story ne formate aucun montant (JSON `Decimal`, MCP `Montant`) ; aucun `{:.2}` neuf dans
   `kesh-report`, **tel que la garde de la 48-11 l'interdira** (la 48-11 n'a pas encore de fiche : nom de la garde et
   des fonctions relevés à T0).
9. **AC-9 — Tests et documentation.** Unitaires (`LigneAgregeable` de la balance, ordre des cinq montants ; groupes
   mixtes du plan PME dissous quand les deux natures portent un montant et `2` conservé ; plan PME entièrement nul :
   aucun groupe dissous ; compte à zéro de nature contraire sous un groupe : groupe conservé (C-46-6a-3) ; compte
   miroir de passif à zéro sous un groupe d'actif : groupe conservé, et le même miroir avec un montant non nul — état
   forgé, 47-1 AC-8 — : groupe et ancêtres dissous ; compte à zéro dans un groupe ouvert et dans un groupe fermé ;
   ligne propre nulle d'un groupe sans ligne `own` et sans effet sur la dissolution ; compte de groupe nul sans
   sous-compte rendu : ligne `account` ; ligne propre non nulle en `own` ; `source` lu dans la liste filtrée — un
   groupe filtré **avant** un compte dans `rows` ne décale pas l'`active` de ce compte ; devise avec un compte CHF nul
   et un compte EUR nul parmi des comptes USD ; les tests de la 46-1 inchangés et verts après la retouche
   d'`ajouter_ligne`) ; intégration (`reports_e2e.rs`) : refus, niveau 2 et `all`, égalité à l'octet sans paramètre,
   sonde négative (sans dissolution par nature, le test d'un groupe mixte rougit) ; `mcp_outils_e2e.rs` (cas groupé,
   tranche sur lignes groupées, `total`), `mcp_contrat`, `mcp_gardes_lexicales`, `mcp_matrice_manuel` verts ; un
   paramètre de groupes répété refusé (400 `VALIDATION_ERROR`) sur `trial-balance` — le test `groupes_parametre_repete`
   de la 46-1, qui affirme aujourd'hui que la balance l'**ignore**, est adapté (déclaré au Change Log) — et toujours
   ignoré par `vat` et par l'export de la balance (ce dernier cas **bascule** en refus à la 46-6b, qui adapte le test) ;
   `docs/api-external.md` (balance groupée : `groupedRows` en tableau, partition par **nature** et non par section,
   dissolution sur les seules lignes non nulles, ligne propre nulle d'un compte de groupe sans ligne `own` ; aucun
   numéro de compte hors du plan PME, gardes G4-bis et G4-ter de `textes_coherents.rs`), dont la phrase posée par la
   46-1 « `GET /reports/trial-balance` et `GET /reports/vat` ignorent ces paramètres, même répétés » devient : les autres routes (`vat`, et les exports de la balance — jusqu'à la 46-6b —, des journaux et de la TVA : `ExportQuery` est commun) les ignorent, la balance les lit et les refuse comme le bilan ; le test de l'ignorance par l'export de la balance est demandé aussi par la 46-3 AC-8 : écrit par la première des deux stories fusionnée, l'autre le reprend ; manuel d'administration (tableau MCP, `kesh_balance`), `.tex` seul.

## Tâches / sous-tâches

- [ ] **T0** — Reprise de la branche d'epic avec la 46-1 (et la 46-4 pour T3) fusionnées ; relever au Change Log,
      ligne lue citée, les noms du § *L'API de la 46-1* tels qu'ils sont dans la branche, et si la 47-3 est fusionnée.
      **Avant toute modification**, écrire la fixture JSON de la balance sans paramètre (plan PME, un compte en
      devise, deux exercices, dates de période fixes), comme la 46-1 T0.
- [ ] **T1** — dans `groupes.rs` : `impl LigneAgregeable for TrialBalanceRow` ; retouche d'`Agregat::ajouter_ligne`
      (AC-3 (c)) ; `LigneBalanceGroupee` (`Serialize`, camelCase) et `LigneBalanceGroupee::depuis` ;
      `AvecGroupes::<TrialBalance, Vec<LigneBalanceGroupee>>::balance(rapport, &arbre, &options)` — filtre des lignes
      propres nulles des groupes (AC-3 (d)), paires `(id, nature)` des seules lignes non nulles pour
      `Dissolution::calculer` (AC-2), `regrouper`, `Grouping::depuis`, conversion ; `TrialBalance` intact ; tests (AC 1
      à 4, 8, 9).
- [ ] **T2** — service `balance_par_groupes` et `balance` délégant avec `None` (`services/rapports.rs`, `balance`) ;
      route `get_trial_balance` (second extracteur, `exiger`, `groupes_de`) ; `export_trial_balance` rend
      `&report.rapport` (AC 5, 7).
- [ ] **T3** — MCP `kesh_balance` (AC 6), **après la 46-4** ; appelle `balance_par_groupes` ; la sortie lit l'enveloppe
      (`.rapport` pour les lignes plates, `grouping` et `grouped_rows` pour `groupes` et `groupes.lignes`) ; littéral de
      base du témoin sans argument (AC-6) **écrit au T0**, avant toute modification du MCP (T2 modifie déjà `rapports.rs` du MCP par la lecture de `.rapport`).
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
- **C-46-6a-3** (correction du 2026-11-21) : **une ligne entièrement nulle ne compte pas pour la dissolution par
  nature** (AC-2). *Pourquoi* : le motif de la dissolution est de ne pas additionner des soldes de signes contraires,
  ce qu'un zéro ne fait pas ; la balance rend les comptes actifs à zéro, que le bilan ignore (`HAVING`), et sans cette
  règle un compte jamais mouvementé — en particulier un **compte miroir** de l'Epic 47, qui ne porte pas d'écriture,
  rangé sous un groupe d'un autre type (état admis par la décision du responsable de l'Epic 47 du 2026-11-21) —
  dissoudrait ce groupe et ses ancêtres à la balance, en permanence. La règle généralise celles de l'AC-3 (c) et (d)
  (une ligne nulle ne compte ni pour la devise ni pour la dissolution). *Coût* : la dissolution de `6`, `7`, `70`, `8`,
  `80` du plan PME dépend des montants, non plus de la seule présence de 6950 actif (l'AC-2 d'avant disait « `6` l'est
  toujours ») ; c'est la règle des lignes présentes de la 46-1 (AC-4). *Écarté* : garder les lignes nulles dans la
  dissolution et dire au manuel qu'un miroir mal rangé défait des groupes de la balance.
- **C-46-6a-5** (validation P1 de la nouvelle boucle, 2026-11-21) : **`groupes: null` est admis** dans la sortie
  sans argument neuf, au lieu d'omettre le champ. *Pourquoi* : la convention C-37-9 veut un champ présent à `null` (précédent de la 37-4 au MCP, `rapports.rs:77-79`). *Écarté* : deux structures de sortie, avec et sans `groupes` (une forme de plus à tenir pour un seul champ). Le littéral du commit de base n'est pas réécrit : le test y ajoute la seule clé neuve (AC-6). Même décision
  à la 46-4 (AC-2), pour `kesh_bilan` et `kesh_compte_de_resultat`. *Écarté* : `skip_serializing_if` (impossible sans
  toucher la macro) ; réécrire le littéral en cours de story (il ne prouverait plus rien).
- **C-46-6a-4** (correction du 2026-11-21) : **aucun avertissement** pour un miroir rangé sous un groupe d'un autre
  type : son porteur n'est pas désigné (fichier de l'Epic 47, § *Décision … du 2026-11-21, point e*) ; sa place est
  l'édition du plan, et, par C-46-6a-3, l'état n'a aucun effet sur la balance.

### L'API de la 46-1 (relevée sur `story/46-1` à `de0a582f`, à revérifier au T0 dans la branche d'epic)

- `kesh_report::groupes` : trait `LigneAgregeable { compte_id, numero, nom, devise, montants, montants_devise }` ;
  `ArbreComptes::depuis(&[Account])` (parents validés, cycles coupés ; `parent`, `noeud` publics, `ancetres`
  **privé**) ; `OptionsGroupes { niveau, position, ouverts, fermes }` et `est_ouvert(id, profondeur)` ;
  `Dissolution::calculer<K>(arbre, impl IntoIterator<Item = (i64, K)>)`, `contient(id)` ; `regrouper(lignes: &[L],
  arbre, dissous: &Dissolution, options) -> SectionGroupee { lignes: Vec<LigneGroupee>, profondeur_max, groupes }` —
  la largeur des montants est lue sur la **première** ligne, d'où cinq montants pour toute ligne ; `LigneGroupee {
  genre, profondeur, compte_id, numero, nom, montants, devise, montants_devise, ouvert, source }` ; `Grouping`
  (constructeur **privé** `Grouping::depuis(options, &[SectionGroupee])`, qui filtre `open`/`closed` sur les groupes de
  l'arbre affiché) ; `AvecGroupes<T, S>` (`sans_groupes`, `bilan`, `compte_de_resultat`) ; `Agregat` privé (somme
  exacte, `devise_unique`, `est_nul`). Groupe nul fermé par le niveau masqué, fermé à la main affiché (`emettre`) :
  c'est déjà l'AC-3 (a) ; une ligne d'entrée nulle d'un groupe ouvert est émise (`emettre`) : c'est déjà l'AC-3 (b).
- `services/rapports.rs` : `DemandeGroupes { options }`, `depuis_requete`, `depuis_valeurs` ; `arbre_des_comptes`
  (privé, une requête, archivés compris) ; `bilan_par_groupes` / `compte_de_resultat_par_groupes` et l'écart E1 (les
  fonctions à quatre arguments délèguent avec `None`) ; `balance` (à `de0a582f` : sans groupes, `exiger` dans le
  service seulement).
- `routes/reports.rs` : `groupes_de(&[(String, String)])` (privé, même module que `get_trial_balance`) ;
  `ReportQuery` revenu à `fiscal_year_id`, `period_start`, `period_end` (revue P1 de la 46-1, L-4) ;
  `get_trial_balance` prend aujourd'hui `Query<ReportQuery>` seul.

### References

Fiches 46-1 (§ *Le contrat*, Dev Agent Record : écarts E1 à E4), 46-4 ; rapports de revue de la 46-1 (P1 L-4, L-5 ;
P2 L-6) ; fiche 47-3 (AC-1, § *Recâblage*) et fichier de l'Epic 47 (D7 ; § *Décision … du 2026-11-21, point e*) ;
`CLAUDE.md` § *Le prompt d'une passe doit nommer le manuel*.

## Note de version

*(à écrire au développement)* — La balance des comptes se demande par groupes (API, MCP), avec les sous-totaux
d'ouverture, de mouvements et de clôture de chaque groupe ; sans paramètre, l'API ne change pas ; au MCP, la sortie de `kesh_balance` gagne seulement `groupes: null`.

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
- 2026-11-20 — **validation P2 (Claude Opus 5.5, contexte frais, au code de `origin/main` `d29aa557`) : 0 C, 0 H, 1 M,
  4 L, remédiés.** M : la balance rend les comptes de groupe eux-mêmes (actifs, à zéro : règle d'inclusion,
  `trial_balance.rs:13-18`) — sans règle, chaque groupe aurait une ligne `own` nulle, et un groupe typé autrement que
  ses enfants se dissoudrait sur son propre zéro → AC-3 (d), tests (AC-9), conséquence au manuel (46-6b). L : enveloppe
  `AvecGroupes<TrialBalance, Vec<LigneBalanceGroupee>>` (46-1 à deux paramètres, C-46-1-4) ; « une cinquantaine de
  lignes » → 42 au niveau 2 et 97 à `all` (calcul sur `pme.json`) ; classes 6 à 8 au niveau 1 : dix-huit comptes et le
  groupe `60` ; contrôle de l'AC-4 sans les lignes `subtotal`. Propagation de la 46-4 : témoin en jetons de date,
  `TEMOINS`, champs recopiés dans l'`ArgumentsPeriode` de `rapports.rs:225`. Rapport :
  `kesh-gate-logs/epic-46/validation/46-6a-P2.md`.
- 2026-11-21 — **Corrections avant développement** (orchestrateur de l'epic, sur les rapports de revue de la 46-1, l'API
  relevée sur `story/46-1` à `de0a582f` et la fiche 47-3 à `b4d17d70`). AC-1 et T1 : enchaînement réel
  `Dissolution::calculer` → `regrouper` → `Grouping::depuis`, ordre des cinq montants, `source` lu dans la liste
  filtrée, code placé dans `groupes.rs` pour les fonctions privées ; AC-3 (c) : la 46-1 compte une ligne nulle pour la
  devise, retouche d'`Agregat::ajouter_ligne` déclarée, sans effet sur le bilan ; AC-3 (d) : « compte de groupe »
  défini par l'arbre ; AC-2 et C-46-6a-3 : **une ligne nulle ne compte pas pour la dissolution** (comptes miroirs de
  l'Epic 47 à zéro sous un groupe d'un autre type, décision du responsable de l'Epic 47 du 2026-11-21) ; C-46-6a-4 :
  aucun avertissement ; AC-5 et T2 : `groupes_de` et second extracteur (revue P2 de la 46-1, L-6 ; la consigne posée
  sur `story/46-1` par `de0a582f` est reprise ici), `exiger` avant les groupes, `balance_par_groupes` et `balance`
  à quatre arguments (E1), fixture au commit de base, pas de `role` ni de `mirrorOf` ; AC-6 : recalcul des tailles
  (au plus 43 lignes au niveau 2 et 107 à `all`) ; AC-9 : tests neufs, `groupes_parametre_repete` de la 46-1 adapté.
  Propagation : 46-6b AC-6 (dissolution « dès que les deux natures portent un montant »). Validation à reprendre (P1
  d'une nouvelle boucle, prompt `kesh-gate-logs/s46-specs/validation-p1-prompt.md`).
- 2026-11-21 — **Validation P1 de la nouvelle boucle** (Claude Sonnet 5.5, contexte frais, à `fe8f0eeb`) : 0 C / 0 H /
  1 M / 5 L sur cette fiche, remédiés ; C-46-6a-3 jugé juste sur le fond. M (F2) : « sortie identique sans argument
  neuf » contredisait « `groupes` à `null` sinon » (`sortie!` n'omet aucun champ) → C-46-6a-5 : `groupes: null`
  admis, littéral de base inchangé, la clé ajoutée par le test avant comparaison (AC-6) ; même décision à la 46-4.
  L : justification de la tranche (au-delà du plafond à `all`, niveau 2 à mesurer, F6) ; `groupes.lignes` distinct du
  champ plat `lignes` (F7) ; lignes filtrées clonées, `TrialBalanceRow` dérivant `Clone` (F8, AC-1) ; phrase du guide
  d'API de la 46-1 à réécrire et règles propres à la balance nommées (F9, AC-9) ; entrée `kesh_balance` de `TEMOINS`
  (F10) ; bascule du test d'ignorance de l'export de la balance à la 46-6b (F11). Aussi : commentaire de
  `ligne_principale` à mettre à jour (AC-3 (c)). Rapport : `kesh-gate-logs/s46-specs/validation-p1-rapport.md`.
- 2026-11-21 — **Validation** (boucle close) : P1 (Claude Sonnet 5.5, 0 C / 0 H / 0 M au-dessus de LOW ; tendance 0/0/2/10 → 0/0/0/10) puis P2 (Claude Opus 5.5, contexte frais, à `e61c9029`) 0 C / 0 H / 0 M / 10 L sur les trois fiches (46-3 : 4, 46-6a : 5, 46-4 : 1) ; aucun finding au-dessus de LOW. LOW de la P2 appliqués dans la fiche : G5 (note de version), G6 (littéral du témoin au T0), G7 (phrase du guide d'API complète), G8 (motif de C-46-6a-5 : convention C-37-9), G9 (écho de la 46-4 repris par une fonction partagée). Rapport : `kesh-gate-logs/s46-specs/validation-p2-rapport.md`. `ready-for-dev`.
