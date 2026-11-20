# Story 39-11a : lecteur du CSV « relevé » de Wise (`kesh-import`)

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/39-wise`, numéros de ligne relevés au code de `origin/main` (`fd04099f`,
fusionné dans `epic/39-wise` au commit `9c8bc452`), revérifiés à la validation P1. Epic 39, **phase 1 KISS**
(décision de Beat du 2026-11-20, § *Phase 1 KISS* du fichier d'epic, G1, G3, G4, G10). **Dépend de** : 38-11a
(lecteur PayPal, validée P1 : en-tête par noms et forme fermée des montants, que cette fiche partage). **Suivie
par** : 39-11b (import), 39-11c (rapprochement).*

## Story

En tant que **Comptable**, je veux que Kesh lise le relevé CSV qu'exporte Wise pour un solde (une devise) et une
période, ligne par ligne, avec le montant, la devise, le solde courant, les parties, la conversion et l'identifiant
Wise, afin que l'import (39-11b) n'ait plus qu'à ranger ces lignes au compte bancaire de la devise.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-11a-1` | Lire le CSV Wise par les **noms** de colonnes, quel que soit leur ordre ; toutes les colonnes requises manquantes sont nommées dans le refus. | AC-1, AC-2 |
| `FR-39-11a-2` | Rendre chaque ligne avec identifiant, date, horodatage, montant, devise, solde courant, description, référence de paiement, payeur, bénéficiaire, numéro de compte du bénéficiaire, marchand, conversion, frais indiqués, sens et type. | AC-3 |
| `FR-39-11a-3` | Refuser un fichier à deux devises, un identifiant en double, ou un solde courant discontinu, en nommant la ligne. | AC-4, AC-5 |
| `FR-39-11a-4` | Classer l'identifiant par son **préfixe** (`TRANSFER`, `CARD`, `BALANCE`, `CONVERSION_ORDER`, …, et `FEE-` suivi du préfixe d'origine). | AC-3 |
| `NFR-39-11a-1` | Fonction pure, sans I/O, dans `kesh-import`, publiable sans dépendance interne ; aides communes partagées avec 38-11a, non recopiées. | AC-2, AC-4, AC-6 |
| `NFR-39-11a-2` | Aucune donnée réelle dans le dépôt : fixtures fictives. | AC-7 |

## Critères d'acceptation

1. **AC-1 — Entrée et décodage** (même patron que 38-11a AC-1). `kesh_import::wise::parse_wise_csv(bytes: &[u8]) ->
   Result<WiseCsv, WiseCsvError>` (module neuf `crates/kesh-import/src/wise.rs`, `pub mod wise;` et réexport à côté
   de `parse_csv`, `crates/kesh-import/src/lib.rs:45-55`). `detect_encoding(bytes)` rend `(encodage, bom_len)` ;
   `decode_bytes` (`crates/kesh-import/src/csv/encoding.rs:48`, `:107`) reçoit `&bytes[bom_len..]` (le relevé de
   Wise n'a pas de BOM — relevé sur les fichiers 2024 —, mais un BOM est admis et retranché). Crate `csv` déjà en
   dépendance : `has_headers(false)`, virgule, guillemets doubles, `flexible(false)` (nombre de cellules ≠ en-tête →
   erreur de ligne `FIELD_COUNT`) ; le numéro de ligne d'une erreur est celui du fichier (`record.position().line()`,
   1 = l'en-tête) ; lignes entièrement vides sautées. Fichier vide ou en-tête seul → `WiseCsvError::Empty`.
   **Type d'erreur fermé** : `WiseCsvError` = `Empty`, `Encoding(CsvError)`, `MissingColumns(Vec<String>)`,
   `DuplicateColumn(String)`, `MixedCurrencies(Vec<String>)`, `DuplicateId { id, lines }`, `RunningBalanceBreak {
   line, expected, found }`, `Lines(Vec<WiseLineError>)` ; chaque variante a un code stable (`WISE_CSV_EMPTY`,
   `…_ENCODING`, `…_MISSING_COLUMNS`, `…_DUPLICATE_COLUMN`, `…_MIXED_CURRENCIES`, `…_DUPLICATE_ID`,
   `…_RUNNING_BALANCE_BREAK`, `…_LINES`) que 39-11b place dans `details` de son refus et que 39-11d traduit.
2. **AC-2 — En-tête par noms.** Comparaison après `trim`, sans casse, par l'**aide commune** de 38-11a (G10) : 38-11a
   l'écrit dans `paypal.rs` ; cette story la **déplace** dans un module partagé `crates/kesh-import/src/csv/by_name.rs`
   (index des colonnes par nom, requises manquantes listées dans l'ordre donné, doublon de colonne) et l'appelle des
   deux côtés, tests de 38-11a inchangés en assertions. **Requises** : `TransferWise ID`, `Date`, `Amount`,
   `Currency`, `Description`, `Running Balance`, `Transaction Type`, `Transaction Details Type`. **Facultatives**
   (absentes = vides) : `Date Time`, `Payment Reference`, `Exchange From`, `Exchange To`, `Exchange Rate`, `Exchange
   To Amount`, `Payer Name`, `Payee Name`, `Payee Account Number`, `Merchant`, `Total fees`, `Note`. `Card Last Four
   Digits`, `Card Holder Full Name`, `Attachment` et toute autre colonne sont **ignorées** (jamais lues ni rendues :
   quatre chiffres de carte et nom du porteur n'entrent pas dans Kesh). Requises absentes → `MissingColumns`, toutes
   listées dans l'ordre ci-dessus ; doublon de colonne → `DuplicateColumn`.
3. **AC-3 — Ligne rendue.** `WiseRow { line: usize, id: String, id_prefix: String, is_fee: bool, fee_of:
   Option<String>, date: NaiveDate, date_time: Option<NaiveDateTime>, amount: Decimal, currency: String,
   description: Option<String>, payment_reference: Option<String>, running_balance: Decimal, payer_name:
   Option<String>, payee_name: Option<String>, payee_account: Option<String>, merchant: Option<String>,
   exchange_from: Option<String>, exchange_to: Option<String>, exchange_rate: Option<Decimal>, exchange_to_amount:
   Option<Decimal>, total_fees: Option<Decimal>, direction: WiseDirection, details_type: String, note:
   Option<String> }` ; cellule vide (après `trim`) → `None`, texte gardé tel quel sinon (les descriptions de Wise
   portent des espaces fines insécables U+202F : ni retirées ni remplacées, aucune comparaison ne les lit). `id` : non
   vide, ASCII `[A-Z0-9_-]` après `trim` et majuscules, ≤ 64 caractères (le plus long des relevés 2024 en compte 28) ;
   `id_prefix` = la partie avant le premier chiffre, sans tiret final (`TRANSFER`, `CARD`, `BALANCE`,
   `CONVERSION_ORDER`, `FEE-TRANSFER`, `FEE-BALANCE`, `FEE-CONVERSION_ORDER`, …) ; `is_fee` = l'identifiant commence
   par `FEE-` ; `fee_of` = l'identifiant sans `FEE-`. `direction` = `Credit` | `Debit` (colonne `Transaction Type`,
   `CREDIT` / `DEBIT` dans les relevés, comparée sans casse ; autre → erreur de ligne) ; `currency` trois lettres
   ASCII, majuscules ; `details_type` tel quel (`TRANSFER`, `CARD`, `CONVERSION`, `DEPOSIT` relevés). Le lecteur **ne
   classe rien d'autre** : conversions, virements, versements Stripe sont l'affaire de 39-11c.
4. **AC-4 — Dates, montants, signes.** `Date` au format `JJ-MM-AAAA` (relevés 2024) ou `AAAA-MM-JJ` ; autre → erreur de
   ligne `DATE` ; année hors `ANNEE_MIN..=ANNEE_MAX` (`lib.rs:35-43`, `annee_recevable`) → erreur. `Date Time` au
   format `JJ-MM-AAAA HH:MM:SS` suivi ou non de `.fff` (relevés : millisecondes), lu en `NaiveDateTime` **sans
   conversion de fuseau** (limite au manuel par 39-11d) ; autre forme → erreur `DATE_TIME`. **Montants** : la forme
   fermée de 38-11a AC-5 (`^-?(\d+|\d{1,3}(,\d{3})+)(\.\d{1,2})?$`, virgule seulement en séparateur de milliers),
   par la **même fonction**, déplacée dans `csv/by_name.rs` avec l'aide d'en-tête (⚠️ **pas** `parse_amount`,
   `csv/parser.rs:816`, privé, qui retire toute virgule sans la contrôler : 38-11a AC-5 l'écarte pour cette raison,
   et ne le rend pas `pub(crate)`) — pour `Amount`, `Running Balance`, `Total fees`, `Exchange To Amount` (les
   relevés portent `-9.99`, `9.99` et des entiers sans décimale) ; `Exchange Rate` : `^\d+(\.\d{1,10})?$` (cinq
   décimales relevées). Contrôles : `Credit` avec `amount < 0` ou `Debit` avec `amount > 0` → `SIGN_MISMATCH` ;
   ligne de frais créditée → `FEE_SIGN` (aucune dans les relevés 2024). Erreurs agrégées (`WiseLineError { line,
   field, code, value }`, valeur tronquée à `MAX_CSV_LINE_ERROR_VALUE_CHARS`, plafond `MAX_CSV_LINE_ERRORS`,
   `crates/kesh-import/src/error.rs:55`, `:49`) ; une erreur refuse le fichier entier.
5. **AC-5 — Contrôles du fichier.** Toutes les lignes de la même devise, sinon `MixedCurrencies` ; un identifiant en
   double dans le fichier → `DuplicateId` (une conversion porte le même identifiant dans **deux** fichiers, jamais deux
   fois dans un) ; lignes **triées par ordre chronologique** : `date_time` (valeur lue, jamais la chaîne : `JJ-MM-AAAA`
   ne se trie pas en texte) quand **toutes** les lignes en ont un, à égalité, ou dès qu'une seule en manque (colonne
   absente comprise), ordre **inverse** du fichier (Wise écrit la plus récente en
   tête ; une ligne de frais précède d'une milliseconde sa ligne d'origine) ; puis **continuité** : chaque
   `running_balance` = le précédent + `amount`, sinon `RunningBalanceBreak`. `WiseCsv { currency, rows
   (chronologiques), opening_balance = premier solde − premier montant, closing_balance = dernier solde, first_date,
   last_date }`.
6. **AC-6 — Pureté et publication.** Aucune dépendance neuve ni interne ; `cargo publish --dry-run -p kesh-import`
   vert ; tests de 38-11a et de `parser.rs` inchangés en assertions ; `parse_amount` non touché.
7. **AC-7 — Fixtures fictives.** `crates/kesh-import/tests/fixtures/wise-releve-{chf,eur,usd}-fictif.csv` (et leur
   ligne au catalogue `crates/kesh-import/tests/fixtures/README.md`, qui dit tous ses fichiers synthétiques) : 23
   colonnes dans l'ordre de Wise (G1), **aucune donnée réelle** (noms inventés, IBAN d'exemple publiés comme `CH93
   0076 2011 6238 5295 7`, identifiants inventés), **toutes les dates en 2020** (exercice du seed `with-company`
   ouvert au 01.01.2020, comme la fixture de 38-11a : l'E2E de 39-11d importe sans créer d'exercice), solde courant
   continu, et au moins : une conversion CHF↔EUR (même `BALANCE-…` dans deux fichiers, le montant cible égal à
   `Exchange To Amount` de la ligne source) avec sa ligne `FEE-BALANCE-…` dans la devise source, horodatée une
   milliseconde avant ; une conversion EUR→USD avec ses frais ; une conversion `CONVERSION_ORDER-…` ; un virement
   sortant vers l'IBAN fictif de la société avec sa ligne `FEE-TRANSFER-…` ; un virement sortant vers un numéro de
   compte non IBAN ; deux crédits `DEPOSIT` « Argent reçu de Stripe » (payeur `Stripe Payments Europe`, fictif) ;
   deux achats par carte (marchands inventés) ; un paiement à un tiers ; une description avec une espace U+202F ; deux
   lignes de jours différents dont l'ordre en texte (`JJ-MM-AAAA`) est l'inverse de l'ordre chronologique. Elles
   servent à 39-11b, 39-11c et à l'E2E de 39-11d.

## Tâches

- [ ] T1 — Aide « en-tête par noms » et forme fermée des montants, déplacées de `paypal.rs` dans `csv/by_name.rs`,
  appelées des deux côtés (AC-2, AC-4, G10).
- [ ] T2 — `wise.rs` : types, erreurs fermées, lecture, préfixes (AC-1, AC-3).
- [ ] T3 — Dates et horodatages, montants, signes ; tri et continuité (AC-4, AC-5).
- [ ] T4 — Fixtures fictives, tests, `cargo publish --dry-run -p kesh-import` (AC-6, AC-7).

## Stratégie de tests

Unitaires dans `wise.rs` ; intégration `crates/kesh-import/tests/wise_csv.rs` sur les fixtures : nombre de lignes,
valeurs de chacune, préfixes, `fee_of`, soldes d'ouverture et de clôture ; colonnes dans un autre ordre ; colonnes
de carte absentes du résultat ; requises manquantes ; doublon de colonne ; deux devises ; identifiant en double ;
rupture du solde courant ; tri sur l'horodatage lu (cas des deux jours de la fixture) ; signe contraire ; date
`MM/JJ/AAAA` refusée ; montant `1.500,00` et `1,50` refusés ; cours à onze décimales refusé ; BOM admis ; U+202F gardé
dans la description ; plafond d'erreurs. Une ligne sans `Date Time` dans un fichier qui en a ailleurs → tri sur l'ordre inverse du fichier seul. Tests de 38-11a verts sans
changement d'assertion après le déplacement des aides. Gate ciblé `binary(wise_csv)` et tests de `kesh-import` ; gate complet au push.

## Modules touchés (1)

`kesh-import` seul.

## Note de version

*Ajouté* — Kesh lit le relevé CSV d'un solde Wise (colonnes en anglais) ; un fichier d'une autre forme est refusé en
nommant les colonnes qui manquent, et un relevé dont le solde courant ne se suit pas est refusé.

## Notes de développement

- Format relevé sur les relevés 2024 de Beat (notes sans donnée personnelle, `kesh-gate-logs/epic-39-40-kiss/`) :
  aucune rupture du solde courant (ni dans l'ordre des horodatages, ni dans l'ordre inverse du fichier), frais
  toujours à part, dans la devise source pour une conversion, et égaux au `Total fees` de leur ligne ; `Total fees`
  n'est donc jamais compté (39-11b). Aucun montant à séparateur de milliers relevé.
- Ancienne 39-3 (relevé JSON par l'API, `superseded`) : seule l'idée du contrôle du solde courant survit, ici.

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
- 2026-11-20 — validation P1 (Claude Opus 5.5, contexte frais) : 0 CRITICAL, 1 HIGH, 3 MEDIUM, 3 LOW, remédiés dans la
  fiche (rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-11a-lecteur-csv-wise-P1.md`). HIGH : montants par
  `parse_amount` « rendu `pub(crate)` par 38-11a », contraire à 38-11a validée (forme fermée, `parse_amount` privé et
  dangereux pour la virgule) → forme fermée partagée. MEDIUM : tri sur `Date Time` en texte (`JJ-MM-AAAA` ne se trie
  pas) → horodatage lu ; fixtures sans exercice d'accueil en E2E → dates de 2020 ; BOM, numéros de ligne et type
  d'erreur fermé non définis → repris de 38-11a. LOW : U+202F des descriptions, lignes citées de `lib.rs`
  (`MAX_CSV_LINE_ERRORS` défini à `error.rs:49`), catalogue des fixtures.
- 2026-11-20 — validation P2 (Claude Sonnet 5.5, contexte frais) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW, remédié (rapport
  `kesh-gate-logs/epic-39-40-kiss/validation/39-11a-lecteur-csv-wise-P2.md`). Remédiation P1 relue au code d'`origin/main`
  (`fd04099f`) et aux relevés 2024 en lecture seule (agrégats : montants et cours conformes aux formes fermées, identifiants
  `[A-Z0-9_-]`, aucun doublon d'identifiant par fichier, `Date` = date de `Date Time`, solde courant continu dans l'ordre des
  horodatages, aucun horodatage double) : rien à reprendre. LOW : tri sur `Date Time` quand une seule ligne en manque,
  règle écrite (ordre inverse du fichier seul).
