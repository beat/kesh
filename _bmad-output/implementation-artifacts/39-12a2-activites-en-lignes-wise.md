# Story 39-12a2 : activités Wise en lignes de relevé — montants, sens, frais, soldes reconstruits et contrôlés (`kesh-import`)

Status: ready-for-dev

*Fiche rédigée le 2026-11-20 sur la **voie activités** (recherche du 2026-11-20, `data-not-committed/wise-recherche.md`,
métadonnées seulement). Le numéro 39-12a2 désignait jusqu'à ce jour la route et l'import, renumérotés **39-12a3**
(même contenu, adapté). Branche `epic/39-wise`. Epic 39, **phase 2** (H2 du fichier d'epic). **Dépend de** : 39-11a
(`WiseRow`, `WiseCsv`, contrôle des identifiants), la **recette préalable 39-12a1 AC-0 (a)** (faite le 2026-11-20 : règles de l'AC-3 devenues définitives).
**Suivie par** : 39-12a3 (qui l'appelle), la recette finale 39-12a1 AC-0 (b). Indépendante du client HTTP de 39-12a1 :
elle ne lit que des structures neutres.*

## Story

En tant que **Comptable**, je veux que les mouvements que Kesh lit dans l'API de Wise deviennent **exactement les mêmes
lignes** que celles du relevé CSV de Wise — mêmes identifiants, mêmes montants, mêmes frais de virement, même solde
courant —, afin qu'un relevé importé à la main et un téléchargement se dédoublonnent et se rapprochent de la même façon.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-12a2-1` | Une fonction pure convertit les activités, frais de transfert et soldes courants en un `WiseCsv` par devise (modèle de 39-11a). | AC-1 |
| `FR-39-12a2-2` | Les montants, donnés en texte balisé, se lisent par une forme fermée ; toute autre forme refuse l'ensemble. | AC-2 |
| `FR-39-12a2-3` | Identifiants = ceux du CSV (`TRANSFER-`, `CARD-`, `BALANCE-`, `FEE-TRANSFER-`) ; frais des virements sortants en lignes séparées ; frais de conversion inclus dans le débit. | AC-3, AC-4 |
| `FR-39-12a2-4` | Le solde courant de chaque ligne est reconstruit depuis le solde actuel, puis contrôlé contre les soldes de clôture déjà connus de Kesh ; un écart refuse l'ensemble. | AC-5 |
| `NFR-39-12a2-1` | Sans I/O, dans `kesh-import`, publiable sans dépendance interne ni paquet neuf. | AC-1, AC-7 |
| `NFR-39-12a2-2` | Aucune donnée réelle dans le dépôt : fixtures fictives. | AC-7 |

## Critères d'acceptation

1. **AC-1 — Entrée et sortie.** Module neuf `crates/kesh-import/src/wise_api.rs` (`pub mod wise_api;` à côté de
   `pub mod wise;`, `crates/kesh-import/src/lib.rs`), **à part de `wise.rs`** (39-11a) pour ne pas grossir le lecteur CSV ;
   les aides de 39-11a dont il a besoin (contrôle de l'identifiant `parse_id`, calcul de `id_prefix`) passent `pub(crate)`,
   et le calcul de `is_fee`/`fee_of`, écrit en ligne dans `read_row` de 39-11a, en sort en une petite aide `pub(crate)`
   appelée des deux côtés — sans changement de comportement ni d'assertion. Fonction `rows_from_activities(input: &WiseApiInput) ->
   Result<WiseApiOutput, WiseApiError>`. **Entrée neutre** (types propres au module, aucun type de `kesh-api`) :
   `activities: Vec<WiseApiActivity { id, activity_type, resource_type, resource_id, title, description, primary_amount,
   secondary_amount, status, created_on: NaiveDateTime, updated_on: NaiveDateTime }>` (**instants déjà ramenés à l'heure de Zurich — choix de la recette 39-12a1 AC-0 (a)(1) — par 39-12a3, qui a `kesh_core::horloge::heure_suisse` : `kesh-import` n'a aucune dépendance interne et ne connaît aucun fuseau**) ; `transfer_fees:
   Vec<WiseApiTransferFee { transfer_id, fee_currency, fee_total: Decimal, reference: Option<String> }>` (un par virement
   sortant de la période lue, rempli par 39-12a3 depuis les transferts et devis de 39-12a1) ; `balances: Vec<(currency,
   Decimal)>` (soldes actuels, lus **avant** les activités, 39-12a3) ; `period: (from: NaiveDate,
   to: NaiveDate)` (période à rendre : la période demandée, son début avancé par 39-12a3 au dernier jour du
   relevé précédent, `importFrom`) ; `checkpoints: Vec<WiseApiCheckpoint { currency, date: NaiveDate, closing: Decimal }>`
   (AC-5). **Sortie** : `WiseApiOutput { statements: Vec<WiseCsv>, ignored: { not_completed, card_checks } }` — **un
   `WiseCsv` de 39-11a par devise** ayant au moins une ligne dans la période (`currency`, `rows` chronologiques, `opening_balance`,
   `closing_balance`, `first_date`, `last_date`), si bien que 39-12a3 le passe au cœur d'import exactement comme un fichier
   CSV lu. `WiseRow.line` = rang de la ligne dans l'ordre chronologique de sa devise (1, 2, …), documenté comme n'étant pas
   une ligne de fichier.
2. **AC-2 — Montants : forme fermée.** `primary_amount` et `secondary_amount` sont du texte (« 1,234.56 USD »,
   balises, `+` pour un crédit ; doc de Wise : « not meant to be parsed as a number »). Lecture : (i) retrait des seules
   balises `<positive>`, `</positive>`, `<negative>`, `</negative>`, `<strong>`, `</strong>`, `<strikethrough>`,
   `</strikethrough>` (tout autre `<` → erreur) ; (ii) `trim` ; (iii) forme
   `^([+-])?[ \u{00A0}\u{202F}]?(\d{1,3}(,\d{3})+|\d+)(\.\d{1,2})?[ \u{00A0}\u{202F}]([A-Z]{3})$` (virgule seulement en
   séparateur de milliers ; deux décimales au plus ; code ISO à trois lettres) ; signe : `+` ou balise `<positive>` →
   crédit, `-` ou `<negative>` → débit, rien → sens par la règle de l'AC-3 ; `+` dans `<negative>` (ou l'inverse) → erreur.
   Montant rendu en `Decimal`, positif au crédit, négatif au débit. `secondary_amount` vide (après `trim`) → absent. Toute
   forme autre → `AmountFormat { activity_id, field }` (**jamais** la valeur dans l'erreur : donnée réelle) ; une seule
   erreur refuse l'ensemble (rien n'est importé, 39-12a3).
3. **AC-3 — Types, sens, identifiants** (règles **confirmées** sur 2024 par la recette 39-12a1 AC-0 (a) du 2026-11-20 : 176 mouvements sur 176 au montant, à la devise et au sens, soldes reconstruits égaux au CSV). Seules les activités `status = COMPLETED` deviennent des lignes ; `CANCELLED`, `IN_PROGRESS`,
   `UPCOMING`, `REQUIRES_ATTENTION` sont comptées dans `ignored.not_completed` (et reprises au prochain téléchargement).
   | `activity_type` | Lignes | Identifiant |
   |---|---|---|
   | `TRANSFER` avec crédit (`+`) | un crédit, `primary` | `TRANSFER-<resource_id>` |
   | `TRANSFER` sinon | un débit (montant hors frais) ; plus la ligne de frais de l'AC-4 | `TRANSFER-<resource_id>` |
   | `CARD_PAYMENT` | un débit (un crédit si `+` : remboursement) | `CARD-<resource_id>` |
   | `INTERBALANCE` | crédit `primary` dans sa devise, débit `secondary` dans la sienne (**frais de conversion inclus**) | `BALANCE-<resource_id>` sur les **deux** lignes |
   | `AUTO_CONVERSION` | comme `INTERBALANCE` | `AUTO_CONVERSION-<resource_id>`, **définitif** : le numéro `CONVERSION_ORDER-<n>` du CSV n'est nulle part dans l'API (AC-0 (a)(2)) ; le doublon avec un CSV déjà importé ne se détecte pas par identifiant (39-12a3 R3) |
   | `CARD_CHECK` | aucune (vérification de carte, sans mouvement) ; comptée dans `ignored.card_checks` | — |
   | tout autre type `COMPLETED` (`BALANCE_INTEREST`, `BALANCE_CASHBACK`, `FEE_REFUND`, `DIRECT_DEBIT_TRANSACTION`, `CASH_WITHDRAWAL`, …) | **refus de l'ensemble** : `UnsupportedActivity { activity_type, date }` (`activity_type` rendu tel quel s'il a la forme `^[A-Z_]{1,40}$`,
   sinon `UNKNOWN` : c'est un texte de Wise, borné avant d'entrer dans une erreur), **même si son `date` est hors de `period`** (postérieure à `to` ou antérieure à `from`) : le solde se reconstruit depuis aujourd'hui, un mouvement inconnu n'importe où entre `readFrom` et maintenant fausse tous les soldes ; le refus dit donc la date du mouvement et que **tout** téléchargement qui remonte avant cette date est refusé tant que le type n'est pas pris en charge (le manuel, 39-12b : importer le CSV) | — |
   **Montant d'un débit simple** : `primary` si `secondary` est absent ; sinon celui des deux dont la devise est un solde du
   profil (`balances`) ; si les deux le sont et que l'activité est un **débit `TRANSFER`**, celui qui est dans la devise `fee_currency` de son entrée `transfer_fees` (la devise source du devis, donc la devise débitée : un virement du solde CHF vers un bénéficiaire en EUR, alors que le profil porte aussi un solde EUR, est le cas courant ; cette entrée est exigée, AC-4 : absente, c'est `MissingTransferFee`, jamais `AmbiguousAmount`) ; sinon, les deux ou aucun → `AmbiguousAmount { activity_id }`. Une ligne dans une devise sans solde au
   profil → `UnknownCurrency { currency }` (limite assumée : un solde **fermé** chez Wise, absent de `balances`, fait
   refuser tout téléchargement qui remonte à ses mouvements ; importer alors le CSV). Un même identifiant deux fois dans une devise → `DuplicateId { id, currency }`, **sauf** deux activités **distinctes** (`id` d'activité différent) qui partagent un `resource_id` — un **virement remboursé** peut revenir en seconde activité `TRANSFER` de même `resource_id` : elles donnent **une ligne chacune**, la première (tri par `created_on`, puis `id` d'activité) gardant l'identifiant `<PREFIX>-<resource_id>`, les suivantes `<PREFIX>-<resource_id>-<n>` (n = 2, 3, … dans cet ordre, stable d'un téléchargement à l'autre) ; la forme exacte, et l'identifiant que le CSV donne au remboursement, sont ce que l'AC-0 (a)(8) de 39-12a1 devait relever : **non relevé** (aucun remboursement en 2024 ni en 2025), le suffixe **provisoire** est maintenu et le manuel avertit de ne pas mélanger CSV et téléchargement sur un virement remboursé ; un même `id` d'activité deux fois dans la réponse ne parvient plus jusqu'ici : le client de 39-12a1 a déjà fondu les répétitions d'un même `id` (dernière version gardée), de sorte que `DuplicateId` pour ce cas ne garde que la fonction pure. Aucune ligne de frais pour le crédit de remboursement (AC-4 ne vise que le débit).
   Les identifiants passent le contrôle de 39-11a (ASCII `[A-Z0-9_-]`, ≤ 64) ; `id_prefix`, `is_fee`, `fee_of` sont calculés
   par les mêmes aides (AC-1). **Autres champs** de `WiseRow` : `date` = le **jour de Zurich** de `created_on` pour les débits (`TRANSFER` sortant, `CARD_PAYMENT`) et les conversions (`INTERBALANCE`, `AUTO_CONVERSION`), de `updated_on` pour les entrants (`TRANSFER` avec `+`) ; la date d'un crédit de carte est non relevée, à confirmer à l'AC-0 (c) : règle tranchée par AC-0 (a)(1), qui redonne la colonne `Date` du CSV pour 206 lignes sur 206 ; la ligne de frais prend la `date` de son origine. `date_time` = le même instant retenu (`created_on` ou `updated_on`), converti à l'heure de Zurich, choix de Kesh : le CSV est en **UTC+1 toute l'année**, même en été, et donne aux entrants l'instant d'achèvement, que l'API n'expose pas ; **jamais** de comparaison ni de dédoublonnage sur l'heure, le jour et l'ordre (débits par `created_on`, entrants par `updated_on`, frais une milliseconde avant leur origine) suffisent aux soldes ; `description` = `title` sans
   balises ; la contrepartie = le texte du premier `<strong>…</strong>` de `title`, en `payer_name` (crédit), `merchant`
   (`CARD_PAYMENT`) ou `payee_name` (débit `TRANSFER`) ; `payment_reference` = `reference` du transfert, égale à la colonne `Payment Reference` du CSV (AC-0 (a)(6), 123 sur 123) ; aucune pour cartes et conversions ;
   conversions : `exchange_from` = devise du débit, `exchange_to` = devise du crédit, sur les deux lignes,
   `exchange_to_amount` = montant crédité sur la ligne de débit seule, `exchange_rate` = `None` ; `details_type` = la valeur
   du CSV pour ce type (AC-0 (a)(5)) : `TRANSFER` entrant → `DEPOSIT` / `CREDIT` ; `TRANSFER` sortant → `TRANSFER` / `DEBIT` ; `CARD_PAYMENT` → `CARD` / `DEBIT` (un crédit de carte, remboursement : sa date et son `details_type` ne sont **pas relevés**, aucun n'existe en 2024 ; **à confirmer à l'AC-0 (c)**) ; `INTERBALANCE` et `AUTO_CONVERSION` → `CONVERSION` (`CREDIT`, `DEBIT`) ; ligne de frais d'un virement → `TRANSFER` / `DEBIT` ; `payee_account`, `note` = `None` ; `direction` du signe.
4. **AC-4 — Frais.** **Virement sortant** : pour chaque débit `TRANSFER-<id>`, l'entrée `transfer_fees` de même
   `transfer_id` est **exigée** (sinon `MissingTransferFee { transfer_id }`) ; `fee_total` > 0 → une ligne
   `FEE-TRANSFER-<id>` (`is_fee`, `fee_of = TRANSFER-<id>`), débit de `fee_total`, devise = `fee_currency`, qui doit égaler
   celle du virement (sinon `FeeCurrencyMismatch { transfer_id }`), horodatée **une milliseconde avant** sa ligne
   d'origine (comme le CSV) mais de **la même `date` que son origine** (même si la milliseconde la fait passer sous minuit : la
   revue de 39-11c lit `FEE-x` et `x` sur le même compte, et la période, filtre de l'AC-5, garde ou écarte la ligne de frais
   avec son origine) ; `total_fees` de la ligne d'origine = `fee_total` ; `fee_total` = 0 → aucune ligne.
   **Conversion** : aucune ligne de frais, `total_fees = None` — le débit porte les frais (l'API ne les ventile pas) ; la
   ligne `FEE-BALANCE-…` d'un CSV du même mouvement est traitée par 39-12a3 (AC-4). C'est la seule différence de contenu
   prévue entre l'API et le CSV, écrite au manuel (39-12b).
5. **AC-5 — Solde courant reconstruit, puis contrôlé.** Par devise, les lignes de **toute la période lue** (toutes les
   activités reçues : du début de lecture, `readFrom`, à l'instant de lecture, 39-12a3) sont triées par `date_time` (à égalité : identifiant), ligne de frais avant son
   origine ; le solde courant de la **dernière** ligne = le solde actuel de la devise (`balances`), puis, en remontant,
   `running_balance[i−1] = running_balance[i] − amount[i]`. **Contrôle** : pour chaque `checkpoint` (le solde de clôture
   d'un relevé déjà connu de Kesh pour ce compte, à sa date de fin, fourni par 39-12a3), le solde reconstruit à la fin de
   ce jour (dernière ligne de date ≤ `date`, ou, sans ligne, le solde précédant la première ligne) doit égaler `closing`
   — **ou**, parce qu'un relevé a pu être arrêté **en cours de journée** (téléchargement jusqu'à l'instant de la lecture,
   CSV exporté dans la journée, téléchargement dont les dernières lignes étaient des doublons : la clôture du cœur est
   celle de la dernière ligne **insérée**), le solde reconstruit à **l'un des instants du jour `date`** (avant sa première
   ligne, ou après l'une de ses lignes) ; sans cette tolérance, un mouvement survenu le même jour après la coupure ferait
   refuser **tout** téléchargement suivant, durablement (relancer n'y change rien) — ; sinon
   `BalanceMismatch { currency, date, expected, found }` (`found` = le solde reconstruit à la fin du jour ; **une devise sans aucune ligne lue** a pour solde reconstruit son solde actuel, **une devise absente de `balances` sans ligne** n'a ni `WiseCsv` ni contrôle : rien à importer, aucune erreur) — **un format de montant qui aurait changé, une activité
   manquée ou une activité en cours déjà portée au solde se voient ici** ; rien n'est importé. Puis seules les lignes dont
   `date` ∈ `period` sont rendues (une ligne de frais suit son origine) ; `opening_balance` = solde courant de la première rendue − son montant ;
   `closing_balance` = solde courant de la dernière ; une devise sans ligne dans la période n'a pas de `WiseCsv`. Sans
   `checkpoint` pour une devise, le contrôle n'a pas lieu (39-12a3 le dit au compte rendu, `balanceUnchecked`).
6. **AC-6 — Erreurs fermées.** `WiseApiError` = `AmountFormat`, `UnsupportedActivity`, `AmbiguousAmount`,
   `UnknownCurrency`, `DuplicateId`, `MissingTransferFee`, `FeeCurrencyMismatch`, `BalanceMismatch` ; chaque variante a un
   code stable (`WISE_API_AMOUNT_FORMAT`, `…_UNSUPPORTED_ACTIVITY`, `…_AMBIGUOUS_AMOUNT`, `…_UNKNOWN_CURRENCY`,
   `…_DUPLICATE_ID`, `…_MISSING_TRANSFER_FEE`, `…_FEE_CURRENCY_MISMATCH`, `…_BALANCE_MISMATCH`) que 39-12a3 place dans
   `details.reason` de `PROVIDER_RESPONSE_INVALID` (sauf `UnsupportedActivity`, qui devient `WISE_ACTIVITY_UNSUPPORTED`,
   39-12a3). Aucune variante ne porte de texte venu de Wise (titre, montant brut).
7. **AC-7 — Tests et pureté.** Unitaires dans `wise_api.rs` ; intégration `crates/kesh-import/tests/wise_api.rs` sur des
   activités **fictives** (Rust ou `tests/fixtures/wise-api-activites-fictives.json`, ligne au catalogue
   `tests/fixtures/README.md`), **construites pour redonner les fixtures CSV de 39-11a** (`wise-releve-{chf,eur,usd}-fictif.csv`) :
   la sortie, comparée au `WiseCsv` lu de ces fixtures, a les mêmes identifiants, dates, montants, devises, sens et soldes
   courants, **sauf** les lignes `FEE-BALANCE-…`/`FEE-CONVERSION_ORDER-…` (absentes) et les débits de conversion (frais
   inclus), écarts assertés un par un. Cas : une ligne de frais à `date` égale à celle de son origine alors que son horodatage passe sous minuit (période qui coupe entre les deux : les deux lignes suivent l'origine) ; un type inconnu hors de `period` refuse quand même ; chaque forme de montant admise (milliers, sans décimale, espace insécable,
   balises) et refusée (`1.234,56 USD`, `1,23 USD`, trois décimales, balise inconnue, signe contradictoire) ; chaque ligne du
   tableau de l'AC-3 ; activité non `COMPLETED` et `CARD_CHECK` comptées ; type non pris en charge ; débit ambigu ; frais
   manquant (y compris quand les deux montants d'un débit `TRANSFER` sont dans des soldes du profil : `MissingTransferFee`), nul, d'une autre devise ; virement CHF vers un bénéficiaire en EUR avec un solde EUR au profil (devise débitée = celle du devis, pas d'`AmbiguousAmount`) ; virement remboursé (débit puis crédit `TRANSFER` de même `resource_id`, deux `id` d'activité : deux lignes aux identifiants `TRANSFER-<id>` et `TRANSFER-<id>-2`, relance au même résultat, solde reconstruit juste ; même `id` d'activité deux fois, à la fonction pure seule : `DuplicateId` ; le client l'a déjà fondu, 39-12a1) ; contrôle juste, faux (`BalanceMismatch`), absent, **juste à un instant du jour
   seulement** (relevé coupé en cours de journée, un mouvement plus tard le même jour : accepté) ; période qui ne garde qu'une partie
   des lignes lues (solde d'ouverture exact) ; devise sans ligne dans la période. Aucune dépendance neuve ni interne ;
   `cargo publish --dry-run -p kesh-import` vert ; tests de 39-11a inchangés en assertions.

## Tâches

- [ ] T1 — Types d'entrée et de sortie, aides de 39-11a en `pub(crate)` (AC-1).
- [ ] T2 — Forme fermée des montants (AC-2).
- [ ] T3 — Types, sens, identifiants, frais (AC-3, AC-4).
- [ ] T4 — Reconstruction et contrôle du solde, période (AC-5), erreurs (AC-6).
- [ ] T5 — Fixtures fictives et tests (AC-7).

## Stratégie de tests

Gate ciblé `binary(wise_api)` et tests de `kesh-import` ; `cargo publish --dry-run -p kesh-import` ; gate complet au
dernier commit. Aucune migration.

## Modules touchés (1)

`kesh-import` seul.

## Fichiers communs

`crates/kesh-import/src/lib.rs` (une ligne), `crates/kesh-import/src/wise.rs` (visibilité de deux aides, une troisième extraite de `read_row`, AC-1),
`tests/fixtures/README.md` : à relire à la reprise de `main`.

## Note de version

*Ajouté* — Kesh convertit les mouvements lus dans l'API de Wise en lignes de relevé identiques à celles du CSV de Wise
(mêmes identifiants, frais de virement en lignes séparées) et contrôle le solde reconstruit contre les relevés déjà
importés.

## Notes de développement

- Recherche du 2026-11-20 (2024 entière comparée au CSV du site) : 176 des 177 identifiants hors frais retrouvés ; mouvement
  net de chaque devise égal au CSV au centime, à l'exception exacte des frais de virement (retrouvés 22 sur 22 par les
  devis) ; frais de conversion inclus dans le débit ; le seul identifiant absent est un `CONVERSION_ORDER-…` (activité
  `AUTO_CONVERSION`). 2025 non comparée : recette 39-12a1 AC-0 (c), non bloquante ; la recette (a) est faite (2026-11-20, concordance exacte), (b) se fait sur 2024 (CSV local).
- Les règles de l'AC-3 sont des **hypothèses validées sur 2024** ; un type d'activité nouveau refuse le téléchargement
  plutôt que de deviner (fermé par construction), et le contrôle de l'AC-5 attrape un montant mal lu.
- Limite assumée : la ventilation conversion / frais n'est pas donnée par l'API (voie Open Banking, H6, hors périmètre). Les frais
  de conversion étant dans le débit, l'écriture d'une conversion téléchargée ne les passe pas en charge à part (39-11c : écriture
  au montant de la transaction) ; avec le CSV d'abord, la ligne `FEE-…` se propose, elle, au compte de frais : le traitement
  comptable dépend de l'ordre (écrit au manuel, 39-12b).
- Limite assumée : l'horodatage retenu (AC-3) peut **bouger après coup** — un mouvement terminé après la fin d'un relevé déjà importé, ou une activité mise à jour après son import — ; le contrôle de l'AC-5 le voit alors comme un écart (refus, jamais une écriture fausse), et le refus **dure** tant que le relevé fautif reste dans Kesh (sortie écrite au manuel, 39-12b). De même, un mouvement **en cours** chez Wise (compté dans `ignored.notCompleted`) peut être déjà porté au solde actuel sans être une ligne : sans point de contrôle (`balanceUnchecked`), les soldes d'ouverture et de clôture du relevé importé peuvent en différer du montant de ce mouvement ; le compte rendu et le manuel le disent. **Avec** un point de contrôle, ce même mouvement décale **tous** les soldes reconstruits : le contrôle refuse jusqu'à ce qu'il soit terminé, et le relevé de Kesh n'y est pour rien — la sortie est d'attendre la fin du mouvement puis de relancer, **jamais** de supprimer un relevé (39-12b).
- Limite assumée : le solde se reconstruit depuis le solde **actuel**, d'où un devis par virement sortant lu de `readFrom` à
  maintenant et un refus pour tout type inconnu de cette plage ; pour un usage mensuel (relevé de clôture récent), la plage est
  courte. Reconstruire en avant depuis le dernier relevé de Kesh serait un perfectionnement futur, hors périmètre.

## Amendement du 21.11

Sources : `kesh-gate-logs/epic-39-40-kiss/wise-conversion-carte-ecarts.md` (formes réelles des paiements par carte
convertis, 2025–2026) et `wise-conversion-partielle-recherche.md` (§ « Reconstitution par continuité des soldes » et
« Mesure contre les relevés réels » : relevés CSV, CAMT et PDF du site, 2025 et 2026) ; décisions du responsable de
l'epic du 21.11 (journal des décisions de signature des epics du 21.11, sous `kesh-gate-logs/`, (a) et (b)). Elles l'emportent sur les AC ci-dessus
là où elles diffèrent. La répartition d'une conversion **partielle** par continuité des soldes, suivi de la décision
(a), n'est **pas** ici : elle forme les stories **39-12a4a** (moteur, `kesh-import`) et **39-12a4b** (téléchargement),
qui lèvent le refus de A-3 ; cette story livre le refus seul, pour qu'un téléchargement sûr existe sans elles.

- **A-1 — Paiement par carte à conversion totale** (écart 1 de la note des écarts). Un `CARD_PAYMENT` dont le
  `secondary_amount` est un **montant** (forme de l'AC-2) : `primary` = montant et devise **du marchand** (sa devise
  n'est pas débitée), `secondary` = montant débité dans un autre solde, **frais de conversion compris**. Règle, à la
  place de « Montant d'un débit simple » pour ce type : la ligne est un débit du **secondaire**, dans sa devise, jamais
  du primaire — même quand les deux devises sont des soldes du profil (sans quoi les 14 totales de 2025–2026 rendaient
  `AmbiguousAmount`) ; `exchange_from` = devise du secondaire, `exchange_to` = devise du primaire, `exchange_to_amount`
  = montant du primaire, `exchange_rate` = `None` ; aucune ligne de frais (comme une conversion de solde, AC-4 : le
  débit porte les frais ; le CSV « avec frais indiqués séparément » porte, lui, une ligne `FEE-CARD-<id>`, traitée par
  39-12a3b). Mesuré : chaque ligne carte convertie du relevé est dans le fichier du solde débité, *Exchange To Amount* =
  montant du marchand (26 lignes sur 26). Le segment de conversion que l'import écrit de ces champs
  (`transaction_of_row`, `routes/wise_imports.rs`) est celui que lit l'écriture groupée de 39-11c (amendement A-1 de
  39-11c). Un `CARD_PAYMENT` sans secondaire garde la règle de l'AC-3 ; un remboursement (`+`) aussi.
- **A-2 — Réserve de carte expirée puis capturée.** Un `CARD_PAYMENT` dont `updated_on − created_on` dépasse **7
  jours** est daté et ordonné à `updated_on` (date = jour de Zurich de `updated_on`, `date_time` et place dans l'ordre
  de l'AC-5 au même instant), non à `created_on` : la réserve a expiré, le solde a été recrédité, et le débit a lieu à
  la capture, sur les soldes du jour de la capture (cas réel : achat du 2025-12-20 capturé le 2026-01-06 sur l'EUR ;
  ordonné à la création, il tombait sur un solde qui ne pouvait pas le payer). Le relevé du site porte trois lignes
  (débit, recrédit, débit à la capture), l'API l'état final seul : une ligne. L'écart est sans effet sur les soldes ;
  un CSV importé après le téléchargement apporte le débit et le recrédit du premier solde (39-11a2 : même identifiant,
  sens opposés), de somme nulle.
- **A-3 — Paiement par carte à conversion partielle : refus fermé dédié** (écart 2 ; décision (a) du 21.11). Un
  `CARD_PAYMENT` dont le `secondary_amount`, balises retirées et `trim` faits, a la forme
  `^[A-Z]{3}(, [A-Z]{3})+$` (« USD, CHF », « USD, CHF, EUR » : une **liste de devises sans montant**) est une
  conversion partielle : l'API ne donne pas sa répartition par solde. Il ne relève **pas** de `AmountFormat` (motif
  trompeur : le format n'a pas changé) mais d'une variante neuve `PartialCardConversion { date }` (date de la ligne,
  règle de l'AC-3 et de A-2), code `WISE_API_PARTIAL_CARD_CONVERSION`, que 39-12a3 rend en `WISE_CARD_PARTIAL_CONVERSION`
  (renvoi au CSV de la période). Comme `UnsupportedActivity`, le refus vaut **même hors de `period`** (le solde se
  reconstruit depuis aujourd'hui) ; avec plusieurs partielles, la date rendue est celle de la **plus récente**, qui
  borne tout téléchargement : il ne peut pas remonter avant elle. Un `CARD_PAYMENT` non `COMPLETED` reste compté dans
  `ignored.not_completed`, quelle que soit sa forme. 39-12a4a remplace ce refus par la répartition.
- **A-4 — Erreurs et tests.** `WiseApiError` gagne `PartialCardConversion` (AC-6 : neuf variantes) ; l'AC-7 gagne :
  une totale CHF ← USD et une CHF ← EUR, devises toutes deux soldes du profil (débit du secondaire, champs d'échange,
  aucune ligne de frais, aucun `AmbiguousAmount`) ; une capture à plus de 7 jours (date et ordre à `updated_on`, solde
  reconstruit juste) et une à 7 jours tout juste (date à `created_on`) ; une partielle à deux et à trois devises →
  `PartialCardConversion` avec la date de la plus récente, y compris hors de `period` ; une liste de devises mal formée
  (« USD,CHF », « usd, chf ») → `AmountFormat`. Fixtures fictives seulement (modèle :
  `kesh-gate-logs/epic-39-40-kiss/wise-fixtures-conversion-carte/`, à recopier sous `tests/fixtures/` avec sa ligne au
  catalogue).
- **Hors de cet amendement, relevé** (note des écarts, § 3 et § 4) : un virement sortant dont le secondaire est dans la
  **même devise**, frais compris (31 virements sur 46 en 2025–2026 ; débit = `primary`, contrôle `secondary − primary =
  fee_total`) ; deux types `COMPLETED` refusés par l'AC-3, `DIRECT_DEBIT_TRANSACTION` et `BALANCE_DEPOSIT`, présents en
  2026 — tant qu'ils sont refusés, aucun téléchargement ne remonte avant le dernier d'entre eux. À trancher avant la
  recette 39-12a1 AC-0 (c) ; non traités ici.

## Change Log

- 2026-11-21 — **amendement du 21.11** (section ci-dessus) : totale par carte (débit du secondaire), capture tardive,
  refus dédié d'une partielle (décision (a)) ; la répartition par continuité va aux stories neuves 39-12a4a et 39-12a4b.
  À revalider (validation P1 de l'amendement).
- 2026-11-20 — validation ciblée AC-0 (a) « décisions » P2 sur le commit `b2d272e0` (Claude Sonnet 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-ac0-decisions-ciblee-2.md`) : 0 C, 0 H, 0 M, 2 L, remédiés : la table de l'AC-3 ne cite plus de ligne de frais d'une conversion (jamais produite, AC-4) ; la date d'un crédit de carte est marquée non relevée, à confirmer à l'AC-0 (c).
- 2026-11-20 — **décisions de la recette AC-0 (a), 2026-11-20** (source `kesh-gate-logs/epic-39-40-kiss/wise-ac0-decisions.md`) : règles de l'AC-3 devenues définitives. Date : jour de Zurich de `createdOn` pour débits et conversions, de `updatedOn` pour les entrants (UTC : 7 débits faux ; `createdOn` seul pour les entrants : 1 faux ; `updatedOn` seul pour les débits : 15 faux) ; heure : le CSV est en UTC+1 constant, jamais comparer ni dédoublonner sur l'heure ; ligne de frais à la date de son origine ; `AUTO_CONVERSION-<resource_id>` définitif ; solde `amount.value` (égal à `cashAmount.value`) ; sens : `+` sur `primary` pour les entrants, devise débitée = `sourceCurrency` du devis (22 sur 22) ; table des types de détail ; `reference` = `Payment Reference` ; virement sans frais = option `BALANCE` à `fee.total` 0, aucune ligne de frais ; virement remboursé non relevé, suffixe provisoire maintenu ; cartes en devise étrangère non relevées (règle inchangée). Les mentions « à reconfirmer / tranché par la recette » sont retirées ; les tests de l'AC-6 (date des entrants et des débits proches de minuit, frais à la date de l'origine, sans frais, remboursé) restent cohérents, avec ces règles pour attendus. Aucun code touché.
- 2026-11-20 — validation P5 ciblée sur le commit `06a8f336` (Claude Sonnet 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P5-ciblee.md`) : 0 C, 0 H, 0 M, 1 L, remédié. Ici : deux activités distinctes de même `resource_id` (virement remboursé) donnent deux lignes, identifiants suffixés `-<n>`, au lieu d'un `DuplicateId` durable (AC-3) ; cas ajouté au test de l'AC-7. Limite sur le mouvement en cours, `MissingTransferFee` (AC-3, AC-4, AC-7) relus, cohérents.
- 2026-11-20 — validation P4 ciblée sur le commit `a98da37a` (Claude Opus 5.5, contexte frais ; commune à 39-12a1, 39-12a2, 39-12a3, 39-12b ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P4-ciblee.md`) : 0 C, 0 H, 1 M, 2 L, remédiés. Ici : **M** — un mouvement en cours porté au solde, avec un point de contrôle, fait refuser jusqu'à sa fin sans que le relevé soit en cause (limite complétée : attendre, ne rien supprimer) ; **L** — une entrée `transfer_fees` absente rend `MissingTransferFee` et non `AmbiguousAmount` quand les deux montants sont dans des soldes du profil (AC-3, test AC-7).
- 2026-11-20 — validation P3 de la voie activités (Claude Sonnet 5.5, contexte frais ; commune à 39-12a1, 39-12a3, 39-12b ;
  rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P3.md`) : 0 C, 0 H, 3 M, 4 L, remédiés. Ici : **M1** — un
  virement entre deux soldes détenus (CHF vers EUR, solde EUR au profil) faisait refuser tout le téléchargement en
  `AmbiguousAmount` : la devise du devis lève l'ambiguïté ; **M2** — horodatage qui bouge après coup et mouvement en cours
  porté au solde, écrits en limite ; devise sans ligne ou absente de `balances` précisée (AC-5).
- 2026-11-20 — validation P2 de la voie activités (Claude Opus 5.5, contexte frais ; commune à 39-12a1, 39-12a3, 39-12b ;
  rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P2.md`) : 0 C, 1 H, 2 M, 9 L, remédiés. Ici : **H1** — le
  contrôle « solde à la fin du jour » refusait durablement tout téléchargement après un relevé arrêté en cours de journée
  (téléchargement jusqu'à l'instant, CSV du jour, dernières lignes en doublon) : tolérance « à l'un des instants du jour »,
  test ; `is_fee`/`fee_of` extraits de `read_row` (aucune aide n'existait) ; `activity_type` borné ; solde fermé
  (`UnknownCurrency`) écrit en limite.
- 2026-11-20 — validation P1 de la voie activités (Claude Sonnet 5.5, contexte frais ; commune à 39-12a1, 39-12a3, 39-12b ;
  rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P1.md`) : horodatages en `NaiveDateTime` ramenés par 39-12a3
  (`kesh-import` sans fuseau, la recette (a)(1) pouvait retenir l'heure de Zurich), `read_until` retiré (sans emploi dans la
  fonction pure), date de la ligne de frais = celle de son origine et période qui les garde ensemble, refus du type inconnu
  même hors période (portée dite), 2024 pour la recette.
- 2026-11-20 — **réécrite sur la voie activités (2026-11-20)** : fiche neuve sous ce numéro (conversion des activités en
  `WiseRow`, forme fermée des montants, frais des virements par les devis, solde reconstruit depuis le solde actuel et
  contrôlé) ; la route et l'import qui portaient ce numéro sont renumérotés 39-12a3. Statut `drafted`, non validée.
