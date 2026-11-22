# Story 39-12a4a1 : paiements par carte partiellement convertis — continuité des soldes : ancrages, segments, refus (`kesh-import`)

Status: deferred

**En attente (22.11) : téléchargement par l'API suspendu jusqu'à la réponse du support Wise ; validation P3 non remédiée (rapport `kesh-gate-logs/s39-12a-specs/validation-p3-rapport.md`).**

*Fiche rédigée le 2026-11-21 sous le numéro 39-12a4a (amendements du 21.11 de 39-12a2 et 39-12a3), **scindée le même
jour** à la validation P1 (finding M15, règle des petites stories) : cette story porte la **continuité** (ancrages,
segments, refus) ; l'**estimation au cours** (étalonnage, confirmation, import des partielles exactes) va à
**39-12a4a2**. C'est le suivi « essayer encore » de la décision (a) du 21.11 du responsable de l'epic (journal des
décisions de signature des epics, sous `kesh-gate-logs/`) : la recherche du même jour,
`kesh-gate-logs/epic-39-40-kiss/wise-conversion-partielle-recherche.md` (§ « Reconstitution par continuité des
soldes » et « Mesure contre les relevés réels »), montre que la répartition par solde d'un paiement partiellement
converti, absente de l'API ouverte au jeton personnel, se **reconstitue** par la continuité des soldes. Branche
`epic/39-wise`. Epic 39, **phase 2**. **Dépend de** : 39-12a2 (amendée : totale par carte, capture tardive, refus
`PartialCardConversion`), 39-12a2b (types d'activité de 2026), 39-12a3 (raccord d'une ligne, AC-1). **Suivie par** :
39-12a4a2 (estimation au cours), 39-12a4b (le téléchargement fournit les lignes déjà dans Kesh et l'identifiant du
relevé de chaque point de contrôle). Indépendante du client HTTP : fonction pure.*

## Story

En tant que **Comptable**, je veux que Kesh reconstitue, par la continuité des soldes, la part de chaque solde d'un
paiement par carte que Wise a payé avec plusieurs soldes, qu'il me rende « à vérifier » ce qu'il ne peut pas établir, et
qu'il refuse le téléchargement quand les soldes révèlent un mouvement que l'API ne montre pas.

## Ce que la mesure établit (rappel, montants réels hors dépôt)

- Wise **vide** le solde de la devise du marchand (CHF dans tous les cas vus), puis, à trois devises, l'USD, et prend le
  reste dans la dernière (EUR) ; une conversion **totale** n'a lieu que si la devise du marchand est à zéro (si elle
  part de l'EUR, l'USD l'est aussi). Ordre : `created_on`, ou `updated_on` après une réserve expirée (39-12a2, A-2).
  Règles **observées** sur un compte (R1 sur le seul CHF, R2 sur le seul ordre USD puis EUR), non documentées par Wise.
- Sur 10 partielles réelles de 2025–2026, contre les relevés du site : **ce que la continuité déclarait exact l'était (11
  parts sur 11)**. Une conversion EUR → CHF de juillet 2025 est **absente de l'API** ; elle se voit comme un **trou de
  solde** (CHF négatif, solde avant une totale non nul).

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-12a4a1-1` | Les parts d'une partielle se déduisent par segments entre ancrages (soldes observés, soldes que les règles de Wise mettent à zéro) ; un point de contrôle n'est un ancrage que si sa place est sûre. | AC-1 à AC-3 |
| `FR-39-12a4a1-2` | Une partielle n'est jamais importée par cette story (sauf si toutes ses parts sont déjà dans Kesh) : elle est rendue « à vérifier », avec ses parts déduites et la somme exacte de son groupe. | AC-4, AC-5 |
| `FR-39-12a4a1-3` | Une lecture où la continuité révèle un trou de solde est refusée en entier, avec un code propre. | AC-6 |
| `NFR-39-12a4a1-1` | Sans I/O, dans `kesh-import`, sans dépendance neuve ; aucune donnée réelle dans le dépôt ; 39-12a3 compile et se comporte comme avant. | AC-1, AC-7 |

## Critères d'acceptation

1. **AC-1 — Entrées et sorties, sans changer 39-12a3.** `WiseApiInput` (39-12a2) gagne `split:
   Option<WiseApiSplitInput>` : **`None`** — comportement de 39-12a2 inchangé (refus `PartialCardConversion`, que 39-12a3
   rend en `WISE_CARD_PARTIAL_CONVERSION`) ; **`Some`** — le moteur de cette story. `WiseApiSplitInput { known_lines:
   Vec<WiseApiKnownLine { currency, id, debit: bool, amount: Decimal, import_id: i64, booked: bool }> }` : les lignes
   **déjà dans Kesh** sur le compte de la devise dont l'identifiant est lu (ou est `FEE-` d'un identifiant lu), avec
   l'identifiant de l'import qui les porte (`bank_transactions.import_id`, croissant dans le temps) et `booked` = la
   transaction n'est plus en attente (déjà rapprochée d'une écriture). `WiseApiCheckpoint` (39-12a2
   AC-1) gagne `import_id: Option<i64>` : l'identifiant du relevé (`bank_imports.id`) dont la clôture fait le point ;
   `None` : jamais placé (AC-2). `WiseApiOutput` gagne `held_back: Vec<WiseApiHeldBack>` et `split_groups:
   Vec<WiseApiSplitGroup>` (AC-5 ; vides si `split` est `None`). **Raccord de 39-12a3, sans effet** (une ligne par site
   de construction, `kesh-api`) : le point de contrôle reçoit l'identifiant que ses deux lectures rendent déjà
   (`previous_statement_closing` → `(id, period_to, closing_balance)`, `overlapping_statements` → `BankImport.id`),
   l'entrée `split: None`. `BalanceGap` et `SplitUnresolved` s'ajoutent à `WiseApiError` (`#[non_exhaustive]`, `code()`, 39-12a2 AC-6) : 39-12a3 les traduit par son bras générique, sans raccord de plus. 39-12a3 lit la sortie par ses champs (à vérifier à l'étape 0 : une déstructuration exhaustive
   de `WiseApiOutput` prend une ligne de plus). Ainsi le gate complet de cette story est vert avec 39-12a3 telle quelle ;
   le passage à `Some` est de 39-12a4b.
2. **AC-2 — Inconnues et ancrages.** Le moteur ne traite que les **devises qui portent au moins une part d'une
   partielle** de la lecture ; les autres gardent le seul contrôle de 39-12a2 AC-5. Une **partielle** (39-12a2, A-3)
   d'activité `COMPLETED` a une **part** par devise de sa liste ; une part est **connue** quand `known_lines` porte,
   dans cette devise, le débit `CARD-<resource_id>` (sa valeur = ce débit plus le débit `FEE-CARD-<resource_id>` de la
   même devise s'il y est : le relevé CSV « avec frais séparés » importé avant), sinon **inconnue**. Par devise, les
   mouvements sont dans l'ordre de l'AC-5 de 39-12a2 (amendement A-2 compris). Un **ancrage** est un solde connu à une
   place de cet ordre :
   - **observés** — (O1) le solde actuel (`balances`), après la dernière ligne ; (O2) un point de contrôle K (39-12a2
     AC-5) **placé**. Une ligne lue est **connue de K** quand `known_lines` la porte (même identifiant, même sens) avec
     `import_id ≤ K.import_id` : elle était dans Kesh quand K a été écrit (un relevé ne crée de transactions que pour
     ses lignes neuves ; les plus anciennes, doublons, viennent d'un import antérieur). K se place **juste après la
     dernière ligne lue de sa devise, datée de son jour `K.date`, insérée par K** (`import_id == K.import_id`), à condition que,
     parmi les lignes lues de ce jour, **toutes celles connues de K** (`import_id ≤ K.import_id`) **précèdent toutes les autres** ; sinon (lignes mêlées, `import_id` absent,
     aucune ligne de ce jour insérée par K, **ou une ligne connue de K mais d'un import antérieur, placée après la dernière ligne insérée par K**) K n'est **pas** un ancrage et garde le seul contrôle de 39-12a2 AC-5, avec sa
     tolérance « à un instant du jour ». **Pourquoi la dernière ligne insérée, et pourquoi on retombe sur la tolérance** (validation P2, M3) : le cœur commun, après 39-14 (`provider_imports.rs`), stocke pour un relevé la clôture de sa dernière ligne **insérée** (`period_to`, `closing_balance` calculés sur les lignes gardées), non celle de la dernière ligne **rendue** ; un relevé dont les dernières lignes du jour étaient des doublons (import antérieur, ou CSV exporté avant un téléchargement puis importé après lui) a donc une clôture d'**avant** ces doublons, et le placer après eux ferait un faux ancrage, donc un `BalanceMismatch` ou un `BalanceGap` durable dans un cas que 39-12a2 AC-5 accepte. Choix : retomber sur la tolérance plutôt que d'imposer au cœur commun de stocker la clôture de la dernière ligne rendue, ce qui changerait un cœur partagé avec PayPal et Stripe ; le coût est un ancrage de moins (une partielle devient « à vérifier » au lieu d'être déduite), jamais un refus à tort. Un relevé enregistré avant 39-14 (clôture de la dernière ligne rendue) dont la fin du jour est en doublon n'est de même pas un ancrage. *Deux relevés finissant le même jour* (téléchargé à 10 h, puis, après un
     mouvement de 11 h, à 12 h) se placent chacun après ses propres lignes : la ligne de 11 h, importée par le second,
     n'est pas connue du premier ;
   - **de règle** — (R1) zéro dans la devise du marchand juste après une partielle ; (R2) zéro en USD juste après une
     partielle dont la liste est exactement {CHF (marchand), USD, EUR} ; (R3) zéro dans la devise du marchand juste avant
     une totale par carte, et zéro en USD aussi quand elle part de l'EUR vers le CHF. Une partielle d'une autre forme
     (marchand hors CHF, quatre devises ou plus, trois devises autres que CHF, USD, EUR) ne pose **aucun** ancrage de
     règle et n'est jamais exacte (motif `unsupported_shape`). *Limite* : une ligne que l'API et le CSV nomment différemment (`AUTO_CONVERSION-y` téléchargée pour un `CONVERSION_ORDER-n` du CSV, 39-12a3b R3 ; remboursement suffixé, 39-12a2 AC-3) est « inconnue de K » : K se place avant elle alors que sa clôture l'inclut ; cas rare (aucune `AUTO_CONVERSION` dans les mesures), à traduire par la reconnaissance de R3 si le cas se présente. *Limite* : R1 n'est observée qu'avec l'USD comme seconde
     devise ; la fiche l'étend à tout couple (X, CHF) — la confirmation par le cours (39-12a4a2) et le refus de l'AC-6
     gardent.
3. **AC-3 — Résolution par segments.** Par devise traitée, entre deux ancrages consécutifs (un **segment**), la somme
   des parts inconnues = ancrage de fin − ancrage de début − somme des montants connus du segment (une part est un
   débit : montant négatif). Aucune inconnue : l'égalité est un **contrôle** (écart : AC-6). **Une** inconnue : sa valeur
   est **déduite** (« deux ancrages »), exacte si les mouvements du segment sont complets. **Plusieurs** : elles forment
   un **groupe** de somme exacte ; leurs valeurs ne sont pas établies ici (39-12a4a2 les résout au centime quand la solution est unique, AC-4b, sinon les estime au cours). Avant le premier
   ancrage d'une devise (segment **ouvert**), une part inconnue n'a ni valeur ni somme. Un même instant portant deux
   ancrages (p. ex. O2 et R1), dans une devise traitée : ils doivent être égaux (sinon AC-6). Le solde courant de chaque
   ligne s'en déduit ; il est **indéterminé** entre deux parts d'un même groupe (39-12a4a2 : estimé).
4. **AC-4 — Parts déduites et forme.** Soit M le montant du marchand (`primary`) et c la part dans sa devise. Toute part
   **connue** ou **déduite** doit vérifier 0 < c < M et, pour chaque autre devise, part > 0 : la liste de devises de
   Wise dit que chaque solde a été débité, un zéro ou une valeur négative la contredit et révèle un mouvement manquant —
   refus `BalanceGap` (AC-6), jamais une ligne omise. Une partielle dont **toutes** les parts sont **connues** (relevé
   CSV en base) est exacte : ses lignes sont rendues (AC-5), doublons ignorés à l'import (39-12a3b). **Part déjà
   rapprochée** (finding M12) : une partielle dont une part connue est `booked` et une autre part n'est pas connue (le
   fichier d'un seul solde importé, puis sa ligne acceptée comme une dépense ordinaire avant que l'autre solde n'arrive)
   ne forme plus d'écriture groupée juste : elle est « à vérifier », motif `part_already_booked` (sortie : annuler le
   rapprochement de la ligne en base, puis relancer ; dit au manuel, 39-12b). Toute autre partielle est rendue « à
   vérifier » (AC-5) : dans cette story, aucune part déduite n'est importée sans la confirmation par le cours de
   39-12a4a2.
5. **AC-5 — Lignes, parts retenues, bornes des relevés.**
   - *Partielle exacte* : une ligne `CARD-<resource_id>` par devise de sa liste, dans le `WiseCsv` de cette devise, au
     **même** instant et à la **même** date (39-12a2, AC-3 et A-2) : débit de c dans la devise du marchand, sans champ
     d'échange ; débit de p_X (frais compris, aucune ligne de frais : 39-12a2 AC-4) avec `exchange_from` = X,
     `exchange_to` = devise du marchand, `exchange_to_amount` = la part couverte quand elle est établie (39-12a4a2),
     sinon `None`, `exchange_rate` idem ; `details_type` `CARD` / `DEBIT`, `merchant` et description comme un
     `CARD_PAYMENT` (39-12a2 AC-3). Une part connue garde sa valeur en base. Le rapprochement (39-11c, A-1) réunit ces
     lignes en **une** écriture (charge = M ; décision (b) du 21.11).
   - *Partielle « à vérifier »* : **aucune** ligne dans `rows`, dans **aucune** devise (une écriture groupée ne se forme
     qu'avec toutes ses parts). Elle va à `held_back` si sa date est dans `period` : `WiseApiHeldBack { card_id, date,
     merchant_amount, merchant_currency, parts: [{ currency, amount: Option<Decimal>, exact: bool, cover:
     Option<Decimal> }], reason }` (`reason` ∈ `unconfirmed` — toutes les parts déduites, confirmation par le cours à
     venir (39-12a4a2) —, `split_group`, `part_already_booked`, `unsupported_shape` ; `amount` = la valeur connue ou déduite, `None` dans un
     groupe). Chaque groupe de l'AC-3 qui contient une part retenue va à `split_groups` : `{ currency, from:
     NaiveDate, to: NaiveDate, sum, card_ids }` (dates des deux ancrages, **somme exacte**). Rien n'est écrit d'office
     (règle « un appariement automatique propose, il ne crée jamais ») : la partielle s'importe ensuite par le relevé CSV
     de sa période (`POST /api/v1/wise-imports`, 39-11b : ses lignes y sont neuves) — un relevé **saisi à la main** est
     refusé sur un compte Wise (`409 BANK_ACCOUNT_IS_PROVIDER`, `refuse_provider_account`).
   - *Bornes de chaque `WiseCsv`* (définition de la **sortie** `WiseCsv`, la même pour les deux lectures ; le cœur commun stocke, lui, la clôture de la dernière ligne **insérée**, ce que l'AC-2 retient pour le placement) : `rows` = les lignes
     **rendues** ; `opening_balance` = solde avant la première rendue ; `closing_balance` = solde **après la dernière
     rendue** ; `first_date`, `last_date` = dates de la première et de la dernière **rendues** — une part retenue ne fixe
     jamais aucune de ces valeurs, si bien que la clôture stockée et le `period_to` du relevé décrivent le même instant et
     que le point de contrôle qu'il devient se place au téléchargement suivant (sauf lignes de fin de jour en doublon : AC-2). Invariant, asserté : `closing − opening`
     = somme des `rows` + somme des parts retenues de la devise placées **entre** la première et la dernière ligne
     rendues. Si l'ouverture ou la clôture d'une devise est indéterminée (elle suit une part d'un groupe non encore
     refermé), refus `SplitUnresolved` `period_boundary` (AC-6). Le solde courant d'une ligne rendue n'est pas écrit en
     base (`transaction_of_row`, 39-11b) : seules l'ouverture et la clôture le sont.
6. **AC-6 — Refus.** Toute la lecture est refusée, rien n'est rendu :
   - **`BalanceGap { currency, from, to }`** (code `WISE_API_BALANCE_GAP`) — trou de solde : un solde reconstitué
     **déterminé** négatif ; un contrôle de segment sans inconnue faux dont un ancrage est **de règle** ; deux ancrages
     d'un même instant différents ; une part connue ou déduite hors de l'AC-4. `from`, `to` = les dates des deux
     ancrages qui bornent le segment en cause. Le refus vaut même hors de `period` (le solde de la période se reconstitue
     à travers lui) ; c'est ainsi que la conversion de juillet 2025 absente de l'API se voit. Le contrôle faux entre
     **deux ancrages observés** reste `BalanceMismatch` (39-12a2 AC-5). Causes possibles, que le message et le manuel
     disent (39-12a4b, 39-12b) : un mouvement absent de l'API ; un mouvement **en cours** déjà porté au solde (39-12a2,
     limite) ; un ordre de prélèvement des soldes autre que celui observé (réglage de l'utilisateur chez Wise, R2) ;
   - **`SplitUnresolved { currency, date, reason }`** (code `WISE_API_SPLIT_UNRESOLVED`) — `reason = unanchored` : une
     part inconnue dans un segment ouvert (aucun ancrage avant elle dans sa devise : premier téléchargement sans relevé
     antérieur, ou point de contrôle non placé) ; `reason = period_boundary` (AC-5). `date` = celle de la partielle en
     cause.
7. **AC-7 — Tests** (unitaires dans un module voisin `crates/kesh-import/src/wise_api/continuite.rs`, intégration
   `crates/kesh-import/tests/wise_api.rs`), sur des activités et lignes en base **fictives** (forme des fixtures
   `kesh-gate-logs/epic-39-40-kiss/wise-fixtures-conversion-carte/`, recopiées sous `tests/fixtures/` avec leur ligne au
   catalogue) : `split: None` → refus `PartialCardConversion` inchangé (tests de 39-12a2 verts sans changement) ;
   partielle à deux devises dont la part USD est seule inconnue entre un point de contrôle placé et le solde actuel
   (déduite, `unconfirmed`) ; à trois devises (R2 : USD déduite ; EUR seule inconnue jusqu'au solde actuel) ; quatre parts
   USD dans un segment (groupe, `split_group`, somme exacte, ouverture et clôture déterminées aux ancrages) ; toutes les
   parts connues par un CSV en base (exacte, lignes rendues, valeur de la base, frais compris) ; part CHF connue et
   `booked`, part USD inconnue → `part_already_booked` ; part CHF connue non rapprochée, part USD déduite → `unconfirmed` ; mouvement manquant (solde
   CHF négatif ; totale sur un solde non nul) → `BalanceGap`, y compris hors de `period` ; part déduite nulle ou
   négative, c ≥ M → `BalanceGap` ; capture tardive (ordre à `updated_on`) ; **deux téléchargements le même jour**
   (relevés K1 puis K2 de même `period_to`, clôtures différentes, un mouvement entre les deux) → les deux placés, aucune
   erreur ; **dernières lignes du jour en doublon** (relevé dont la fin de journée est faite de lignes connues d'un import antérieur, ou CSV exporté avant un téléchargement et importé après lui) → K non placé, contrôle toléré de 39-12a2 AC-5, aucun refus ; point de contrôle non placé (lignes mêlées ; `import_id` absent) ; segment ouvert → `unanchored` ; fin de
   période entre deux parts d'un groupe → `period_boundary` ; dernière ligne de la période = part retenue → clôture
   et `last_date` de la dernière ligne rendue, point de contrôle placé au téléchargement suivant ; devise sans partielle
   → seul le contrôle de 39-12a2 AC-5 (aucun ancrage de règle ne s'y applique) ; partielle à marchand en EUR →
   `unsupported_shape` ; l'invariant de l'AC-5 sur chaque `WiseCsv`.

## Tâches

- [ ] T1 — Entrées et sorties, `split`, `import_id`, raccord de 39-12a3 (AC-1).
- [ ] T2 — Inconnues, ancrages, placement des points de contrôle par identifiant d'import (AC-2).
- [ ] T3 — Segments, groupes, parts déduites (AC-3, AC-4).
- [ ] T4 — Lignes, parts retenues, bornes, refus (AC-5, AC-6).
- [ ] T5 — Tests (AC-7).

## Stratégie de tests

Gate ciblé `binary(wise_api)` et tests de `kesh-import` ; `cargo build` du workspace (raccord de 39-12a3) ;
`cargo publish --dry-run -p kesh-import` ; gate complet au dernier commit. Aucune migration.

## Modules touchés (2)

`kesh-import` ; `kesh-api` pour le seul raccord de l'AC-1 (identifiant du relevé au point de contrôle, `split: None`),
sans changement de comportement.

## Fichiers communs

`crates/kesh-import/src/wise_api.rs` (39-12a2), `tests/fixtures/README.md`, le site d'appel de `rows_from_activities`
dans la route de 39-12a3 : à relire à la reprise de `main`.

## Note de version

*(Interne, sans effet visible seule : publiée avec 39-12a4a2 et 39-12a4b.)* — Kesh reconstitue par la continuité des
soldes la part de chaque solde d'un paiement par carte payé avec plusieurs soldes, et refuse un téléchargement dont les
soldes révèlent un mouvement absent de l'API.

## Notes de développement

- Arithmétique en `Decimal`, arrondi au centime demi supérieur (`RoundingStrategy::MidpointAwayFromZero` sur des
  montants positifs).
- Limite assumée : les règles de Wise (R1 à R3, ordre de prélèvement) sont **observées** sur un compte, non documentées ;
  une forme non vue n'est jamais exacte. Le refus `BalanceGap` et la confirmation par le cours (39-12a4a2) sont les
  garde-fous.
- Limite assumée : à trois devises, l'EUR n'a d'ancrage qu'aux points de contrôle et au solde actuel ; un téléchargement
  mensuel (point de contrôle à la fin du mois précédent) est ce qui le rend déductible.
- Les identifiants d'import croissent dans le temps (`bank_imports.id`, auto-incrément) : c'est ce qui fait de
  `import_id ≤ K.import_id` le critère « dans Kesh quand K a été écrit ». Un relevé supprimé emporte ses transactions
  (Story 27-1) : aucune ligne ne survit à son import.
- Hors périmètre : le webhook `balances#update` (répartition exacte pour l'avenir, recherche du 21.11, piste 4 a) —
  exposition publique et décision d'architecture à prendre d'abord.

## Change Log

- 2026-11-22 — **remédiation de la validation P2** (rapport `kesh-gate-logs/s39-12a-specs/validation-p2-rapport.md`) : **M3** — le point de contrôle se place après la dernière ligne **insérée** par lui (le cœur commun stocke la clôture de la dernière ligne insérée, `provider_imports.rs`) ; une ligne connue d'un import antérieur après elle le fait retomber sur la tolérance de 39-12a2 AC-5 (choix motivé à l'AC-2), test ajouté ; **M4** — raccord des variantes d'erreur par `code()` ; **L5** — limite des identifiants nommés différemment ; résolution par groupe : renvoi à 39-12a4a2 AC-4b. Statut `drafted`, validation P3 à lancer.

- 2026-11-21 — **remédiation de la validation P1** (Claude Sonnet 5.5, rapport
  `kesh-gate-logs/s39-12a-specs/validation-p1-rapport.md` : 0 C, 2 H, 15 M, 18 L sur les six fiches). Ici : **M15** —
  fiche scindée : la continuité reste sous le numéro 39-12a4a1, l'estimation au cours passe à 39-12a4a2 (fichier
  renommé) ; **H1** — point de contrôle placé par identifiant d'import (`import_id ≤ K.import_id`), et non par
  `statement_end ≤ date` (deux relevés du même jour posaient deux ancrages contradictoires), test de deux téléchargements
  le même jour (L15) ; **M7** — une part connue ou déduite nulle ou négative, ou c ≥ M, est un `BalanceGap`, jamais une
  ligne omise ; **M8** — bornes d'un `WiseCsv` définies par les lignes rendues, test « dernière ligne = part retenue » ;
  **M9** — `split: Option<…>` et raccord d'une ligne de 39-12a3, gate vert sans 39-12a4b ; **M11** — plus de « relevé
  saisi à la main » (refusé sur un compte Wise) ; **L9** — trois causes d'un `BalanceGap` ; **L11** — extension de R1
  écrite en limite ; **M12** — part déjà rapprochée (`booked`) → `part_already_booked` ; **L18** — moteur restreint aux devises qui portent une part. Statut `drafted`, validation P2 à
  lancer.
- 2026-11-21 — fiche rédigée sous le numéro 39-12a4a (amendements du 21.11 de 39-12a2 et 39-12a3 ; suivi de la décision
  (a)). Statut `drafted`, validation P1 à lancer.
