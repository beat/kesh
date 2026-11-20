# Story 39-11a : lecteur du CSV « relevé » de Wise (`kesh-import`)

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/39-wise`, numéros de ligne relevés au code de `origin/main` (`fd04099f`,
fusionné dans `epic/39-wise` au commit `9c8bc452`). Epic 39, **phase 1 KISS** (décision de Beat du 2026-11-20,
§ *Phase 1 KISS* du fichier d'epic, G1, G3, G4, G10). **Dépend de** : 38-11a (lecteur PayPal, dont les aides
communes sont réutilisées). **Suivie par** : 39-11b (import), 39-11c (rapprochement).*

## Story

En tant que **Comptable**, je veux que Kesh lise le relevé CSV qu'exporte Wise pour un solde (une devise) et une
période, ligne par ligne, avec le montant, la devise, le solde courant, les parties, la conversion et l'identifiant
Wise, afin que l'import (39-11b) n'ait plus qu'à ranger ces lignes au compte bancaire de la devise.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-11a-1` | Lire le CSV Wise par les **noms** de colonnes, quel que soit leur ordre ; toutes les colonnes requises manquantes sont nommées dans le refus. | AC-1, AC-2 |
| `FR-39-11a-2` | Rendre chaque ligne avec identifiant, date, montant, devise, solde courant, description, référence de paiement, payeur, bénéficiaire, numéro de compte du bénéficiaire, marchand, conversion, frais indiqués, types. | AC-3 |
| `FR-39-11a-3` | Refuser un fichier à deux devises, un identifiant en double, ou un solde courant discontinu, en nommant la ligne. | AC-4, AC-5 |
| `FR-39-11a-4` | Classer l'identifiant par son **préfixe** (`TRANSFER`, `CARD`, `BALANCE`, `CONVERSION_ORDER`, …, et `FEE-` suivi du préfixe d'origine). | AC-3 |
| `NFR-39-11a-1` | Fonction pure, sans I/O, dans `kesh-import`, publiable sans dépendance interne ; aides communes partagées avec 38-11a, non recopiées. | AC-6 |
| `NFR-39-11a-2` | Aucune donnée réelle dans le dépôt : fixtures fictives. | AC-7 |

## Critères d'acceptation

1. **AC-1 — Entrée et décodage.** `kesh_import::wise::parse_wise_csv(bytes: &[u8]) -> Result<WiseCsv, WiseCsvError>`
   (module neuf `crates/kesh-import/src/wise.rs`, réexporté par `crates/kesh-import/src/lib.rs:47-55`). Encodage par
   `detect_encoding` puis `decode_bytes` (`crates/kesh-import/src/csv/encoding.rs:48`, `:107`) — le relevé de Wise
   n'a pas de BOM, mais un BOM est admis ; virgule, guillemets doubles, crate `csv` déjà en dépendance. Fichier vide ou
   sans ligne de données → `WiseCsvError::Empty`.
2. **AC-2 — En-tête par noms.** Comparaison après `trim`, sans casse, par l'aide commune de 38-11a (G10 : si 38-11a
   l'a écrite dans `paypal.rs`, la déplacer dans un module partagé `crates/kesh-import/src/csv/by_name.rs` et
   l'utiliser des deux côtés, sans changer le comportement de 38-11a). **Requises** : `TransferWise ID`, `Date`,
   `Amount`, `Currency`, `Description`, `Running Balance`, `Transaction Type`, `Transaction Details Type`.
   **Facultatives** (absentes = vides) : `Date Time`, `Payment Reference`, `Exchange From`, `Exchange To`, `Exchange
   Rate`, `Exchange To Amount`, `Payer Name`, `Payee Name`, `Payee Account Number`, `Merchant`, `Total fees`, `Note`.
   `Card Last Four Digits`, `Card Holder Full Name`, `Attachment` et toute autre colonne sont **ignorées** (jamais
   lues ni rendues : quatre chiffres de carte et nom du porteur n'entrent pas dans Kesh). Requises absentes →
   `MissingColumns(Vec<String>)`, toutes listées dans l'ordre ci-dessus ; doublon de colonne → `DuplicateColumn`.
3. **AC-3 — Ligne rendue.** `WiseRow { line, id: String, id_prefix: String, is_fee: bool, fee_of: Option<String>,
   date: NaiveDate, date_time: Option<String>, amount: Decimal, currency: String, description: Option<String>,
   payment_reference: Option<String>, running_balance: Decimal, payer_name, payee_name, payee_account, merchant:
   Option<String>, exchange_from, exchange_to: Option<String>, exchange_rate: Option<Decimal>, exchange_to_amount:
   Option<Decimal>, total_fees: Option<Decimal>, direction: WiseDirection, details_type: String, note: Option<String> }`.
   `id` : non vide, ASCII `[A-Z0-9_-]` après `trim` et majuscules, ≤ 64 caractères (le plus long relevé en 2024 en
   compte 28) ; `id_prefix` = la partie avant le premier chiffre, sans tiret final (`TRANSFER`, `CARD`, `BALANCE`,
   `CONVERSION_ORDER`, `FEE-TRANSFER`, `FEE-BALANCE`, …) ; `is_fee` = l'identifiant commence par `FEE-` ; `fee_of` =
   l'identifiant sans `FEE-`. `direction` = `Credit` | `Debit` (colonne `Transaction Type`, sans casse ; autre →
   erreur de ligne) ; `currency` trois lettres majuscules. Le lecteur **ne classe rien d'autre** : conversions,
   virements, versements Stripe sont l'affaire de 39-11c.
4. **AC-4 — Dates, montants, signes.** `Date` au format `JJ-MM-AAAA` (relevé 2024) ou `AAAA-MM-JJ` ; autre → erreur de
   ligne ; année hors `ANNEE_MIN..=ANNEE_MAX` (`lib.rs:35-41`) → erreur. `Date Time` gardé tel quel, sans conversion
   de fuseau (limite au manuel par 39-11d). Montants par `parse_amount(raw, '.')`
   (`crates/kesh-import/src/csv/parser.rs:816`, rendu `pub(crate)` par 38-11a), deux décimales au plus pour `Amount`,
   `Running Balance`, `Total fees`, `Exchange To Amount` ; `Exchange Rate` jusqu'à dix décimales. Contrôles :
   `Credit` avec `amount < 0` ou `Debit` avec `amount > 0` → `SIGN_MISMATCH` ; ligne de frais créditée →
   `FEE_SIGN`. Erreurs agrégées (`WiseLineError { line, field, code, value }`, plafonds `MAX_CSV_LINE_ERRORS`,
   `MAX_CSV_LINE_ERROR_VALUE_CHARS`, `lib.rs:51-54`) ; une erreur refuse le fichier entier.
5. **AC-5 — Contrôles du fichier.** Toutes les lignes de la même devise, sinon `MixedCurrencies(Vec<String>)` ; un
   identifiant en double dans le fichier → `DuplicateId { id, lines }` ; lignes **triées par ordre chronologique**
   (`Date Time`, à défaut `Date` puis ordre inverse du fichier, Wise écrivant la plus récente en tête), puis
   **continuité** : chaque `running_balance` = le précédent + `amount`, sinon `RunningBalanceBreak { line, expected,
   found }`. `WiseCsv { currency, rows (chronologiques), opening_balance = premier solde − premier montant,
   closing_balance = dernier solde, first_date, last_date }`.
6. **AC-6 — Pureté et publication.** Aucune dépendance neuve ni interne ; `cargo publish --dry-run -p kesh-import`
   vert ; tests de 38-11a et de `parser.rs` inchangés en assertions.
7. **AC-7 — Fixtures fictives.** `crates/kesh-import/tests/fixtures/wise-releve-{chf,eur,usd}-fictif.csv` : 23
   colonnes dans l'ordre de Wise, **aucune donnée réelle** (noms inventés, IBAN de test valides `CH93 0076 2011 6238
   5295 7` et voisins publiés comme exemples, identifiants inventés), et au moins : une paire de conversion CHF→EUR
   (même `BALANCE-…` dans deux fichiers) avec sa ligne `FEE-BALANCE-…`, une conversion EUR→USD, un virement sortant
   vers l'IBAN fictif de la société avec sa ligne `FEE-TRANSFER-…`, un virement reçu de la société, deux crédits
   « Argent reçu de Stripe » (payeur `Stripe Payments Europe`, fictif), deux achats par carte (marchands inventés), un
   paiement à un tiers ; solde courant continu. Elles servent à 39-11b, 39-11c et à l'E2E de 39-11d.

## Tâches

- [ ] T1 — Aide « en-tête par noms » partagée avec 38-11a (AC-2, G10).
- [ ] T2 — `wise.rs` : types, lecture, préfixes (AC-1, AC-3).
- [ ] T3 — Dates, montants, signes ; tri et continuité (AC-4, AC-5).
- [ ] T4 — Fixtures fictives, tests, `cargo publish --dry-run -p kesh-import` (AC-6, AC-7).

## Stratégie de tests

Unitaires dans `wise.rs` ; intégration `crates/kesh-import/tests/wise_csv.rs` sur les fixtures : nombre de lignes,
valeurs de chacune, préfixes, `fee_of`, soldes d'ouverture et de clôture ; colonnes dans un autre ordre ; colonnes
de carte absentes du résultat ; requises manquantes ; doublon de colonne ; deux devises ; identifiant en double ;
rupture du solde courant ; signe contraire ; date `MM/JJ/AAAA` refusée ; montant `1.500,00` refusé ; plafond
d'erreurs. Gate ciblé `binary(wise_csv)` et tests de `kesh-import` ; gate complet au push.

## Modules touchés (1)

`kesh-import` seul.

## Note de version

*Ajouté* — Kesh lit le relevé CSV d'un solde Wise (colonnes en anglais) ; un fichier d'une autre forme est refusé en
nommant les colonnes qui manquent, et un relevé dont le solde courant ne se suit pas est refusé.

## Notes de développement

- Format relevé sur les relevés 2024 de Beat (notes sans donnée personnelle, `kesh-gate-logs/epic-39-40-kiss/`) :
  aucune rupture du solde courant, frais toujours à part et égaux au `Total fees` de leur ligne ; `Total fees` n'est
  donc jamais compté (39-11b).
- Ancienne 39-3 (relevé JSON par l'API, `superseded`) : seule l'idée du contrôle du solde courant survit, ici.

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
