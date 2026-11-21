# Story 39-12a4a : paiements par carte partiellement convertis — répartition par continuité des soldes (`kesh-import`)

Status: drafted

*Fiche rédigée le 2026-11-21 (amendements du 21.11 de 39-12a2 et 39-12a3). C'est le suivi « essayer encore » de la
décision (a) du 21.11 du responsable de l'epic (journal des décisions de signature des epics, sous `kesh-gate-logs/`) : la recherche du même
jour, `kesh-gate-logs/epic-39-40-kiss/wise-conversion-partielle-recherche.md` (§ « Reconstitution par continuité des
soldes » et « Mesure contre les relevés réels »), montre que la répartition par solde d'un paiement partiellement
converti, absente de l'API ouverte au jeton personnel, se **reconstitue** par la continuité des soldes. Branche
`epic/39-wise`. Epic 39, **phase 2**. **Dépend de** : 39-12a2 (amendée : totale par carte, capture tardive, refus
`PartialCardConversion` que cette story **lève**). **Suivie par** : 39-12a4b (le téléchargement lui fournit cours, lignes
déjà dans Kesh et place des points de contrôle). Indépendante du client HTTP : fonction pure.*

## Story

En tant que **Comptable**, je veux qu'un paiement par carte que Wise a payé avec plusieurs soldes soit importé par
l'API quand Kesh peut en établir **exactement** la part de chaque solde, proposé « à vérifier » quand il ne le peut pas,
et que le téléchargement soit refusé quand les soldes révèlent un mouvement que l'API ne montre pas.

## Ce que la mesure établit (rappel, montants réels hors dépôt)

- Wise **vide** le solde de la devise du marchand (CHF dans tous les cas vus), puis, à trois devises, l'USD, et prend le
  reste dans la dernière (EUR) ; une conversion **totale** n'a lieu que si la devise du marchand est à zéro (si elle
  part de l'EUR, l'USD l'est aussi). Ordre : `created_on`, ou `updated_on` après une réserve expirée (39-12a2, A-2).
- Le cours `GET /v1/rates` à la minute est celui du relevé (*Exchange Rate*) ; **montant converti = arrondi au centime,
  demi supérieur, de la part couverte dans la devise du marchand divisée par le cours** (26 lignes sur 26) ; frais à part,
  environ 0.28 % (USD) et 0.43 % (EUR), au moins 0.01, règle d'arrondi non élucidée.
- Sur 10 partielles réelles de 2025–2026, contre les relevés du site : **ce que la continuité déclarait exact l'était (11
  parts sur 11)** ; les parts estimées au cours : 6 sur 11 exactes, 5 à un centime. Une conversion EUR → CHF de juillet
  2025 est **absente de l'API** ; elle se voit comme un **trou de solde** (CHF négatif, solde avant une totale non nul).

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-12a4a-1` | Une partielle dont toutes les parts sont exactes devient une ligne `CARD-<id>` par solde débité, que le rapprochement réunit en une écriture groupée (39-11c, A-1). | AC-1 à AC-5 |
| `FR-39-12a4a-2` | Une partielle qui n'est pas exacte n'est jamais importée : elle est rendue « à vérifier », avec ses parts estimées et la somme exacte de son groupe. | AC-6 |
| `FR-39-12a4a-3` | Une lecture où la continuité révèle un trou de solde est refusée en entier, avec un code propre. | AC-7 |
| `NFR-39-12a4a-1` | Sans I/O, dans `kesh-import`, sans dépendance neuve ; aucune donnée réelle dans le dépôt. | AC-8, AC-9 |

## Critères d'acceptation

1. **AC-1 — Entrées et sorties.** `rows_from_activities` (39-12a2) reçoit en plus, dans `WiseApiInput` :
   `rates: Vec<WiseApiRate { activity_id, source: String, target: String, rate: Decimal }>` — un par devise convertie
   de chaque partielle (cible : la devise du marchand) et un par paiement par carte à conversion **totale** (étalonnage
   des frais, AC-4) ; `known_lines: Vec<WiseApiKnownLine { currency, id, debit: bool, amount: Decimal, statement_end:
   NaiveDate }>` — les lignes **déjà dans Kesh** sur le compte de la devise dont l'identifiant est lu (ou est `FEE-` d'un
   identifiant lu), avec la date de fin (`period_to`) du relevé qui les a importées. `WiseApiOutput` gagne
   `held_back: Vec<WiseApiHeldBack>` (AC-6) et `split_groups: Vec<WiseApiSplitGroup>` (AC-6). La variante
   `PartialCardConversion` de 39-12a2 est **retirée** (son code aussi).
2. **AC-2 — Inconnues et ancrages.** Une **partielle** (39-12a2, A-3) d'activité `COMPLETED` a une **part** par devise
   de sa liste ; une part est **connue** quand `known_lines` porte, dans cette devise, le débit `CARD-<resource_id>` (sa
   valeur = ce débit plus le débit `FEE-CARD-<resource_id>` de la même devise s'il y est : le relevé CSV « avec frais
   séparés » importé avant), sinon **inconnue**. Par devise, les mouvements sont dans l'ordre de l'AC-5 de 39-12a2
   (amendement A-2 compris). Un **ancrage** est un solde connu à une place de cet ordre :
   - **observés** — (O1) le solde actuel (`balances`), après la dernière ligne ; (O2) un point de contrôle (39-12a2 AC-5)
     **placé** : juste après la dernière ligne lue de sa devise, datée de son jour `date`, qui figure dans `known_lines`
     (même identifiant, même sens) avec `statement_end ≤ date`, **à condition** que toutes les lignes lues de ce jour qui
     y figurent ainsi précèdent toutes celles qui n'y figurent pas ; sinon (lignes mêlées, relevé saisi à la main sans
     identifiant Wise, aucune ligne de ce jour dans Kesh) le point de contrôle n'est **pas** un ancrage et garde le seul
     contrôle de l'AC-5 de 39-12a2, avec sa tolérance « à un instant du jour » ;
   - **de règle** — (R1) zéro dans la devise du marchand juste après une partielle ; (R2) zéro en USD juste après une
     partielle dont la liste est exactement {CHF (marchand), USD, EUR} ; (R3) zéro dans la devise du marchand juste avant
     une totale par carte, et zéro en USD aussi quand elle part de l'EUR vers le CHF. Une partielle d'une autre forme
     (marchand hors CHF, quatre devises ou plus, trois devises autres que CHF, USD, EUR) ne pose **aucun** ancrage de
     règle et n'est jamais exacte (AC-6, motif `unsupported_shape`).
3. **AC-3 — Résolution par segments.** Par devise, entre deux ancrages consécutifs (un **segment**), la somme des parts
   inconnues = ancrage de fin − ancrage de début − somme des montants connus du segment (une part est un débit : montant
   négatif). Aucune inconnue : l'égalité est un **contrôle** (écart : AC-7). **Une** inconnue : sa valeur est **exacte**
   (« deux ancrages »). **Plusieurs** : elles forment un **groupe** de somme exacte ; chaque part est estimée au cours
   (AC-4), la dernière du segment prend le reste. Avant le premier ancrage d'une devise (segment **ouvert**), une part
   inconnue n'a ni valeur ni somme. Un même instant portant deux ancrages (p. ex. O2 et R1) : ils doivent être égaux
   (sinon AC-7). Les soldes de chaque ligne s'en déduisent ; ceux qui dépendent d'une part estimée sont **estimés**.
4. **AC-4 — Exactitude d'une partielle.** Soit M le montant du marchand (`primary`), c sa part dans la devise du
   marchand, et, pour chaque devise convertie X, p_X sa part (frais compris), r_X le cours (`rates`), `couv_X` la part de
   M qu'elle couvre. Les covertures vérifient c + Σ couv_X = M ; à deux devises, couv = M − c ; à trois, couv_USD =
   arrondi_demi_sup((p_USD − frais_USD) × r_USD) et couv_EUR = M − c − couv_USD. Montant converti de X =
   arrondi_demi_sup(couv_X / r_X), frais estimés de X = max(0.01, arrondi_demi_sup(converti × taux_X)), où taux_X est
   l'**étalonnage** du couple (X → devise du marchand) : la médiane basse, sur les totales par carte du même couple de
   la lecture, de (secondaire − arrondi_demi_sup(primaire / cours)) / arrondi_demi_sup(primaire / cours). La partielle est
   **exacte** quand : (i) chacune de ses parts est connue (AC-2) ou exacte par deux ancrages (AC-3) ; (ii) 0 < c < M et
   chaque p_X > 0 ; (iii) **confirmation par le cours, à un centime près** : pour chaque X, |p_X − (converti_X + frais
   estimés_X)| ≤ 0.01 — sans totale du même couple dans la lecture (pas d'étalonnage), à deux devises seulement, les frais
   implicites p_X − converti_X doivent être compris entre 0.01 et arrondi_demi_sup(converti_X × 1 %) + 0.01 (borne de
   vraisemblance, plus du double des frais mesurés), et une partielle à trois devises sans étalonnage n'est pas exacte
   (motif `no_calibration`). La confirmation garde contre ce que la continuité seule ne voit pas : un mouvement absent de
   l'API **absorbé** par l'unique inconnue d'un segment. Une part connue (relevé CSV en base) est exacte sans
   confirmation. *(La recherche énonçait « deux ancrages, ou un ancrage et le cours à un centime près » ; la mesure montre
   qu'un cours seul se trompe d'un centime cinq fois sur onze : le cours confirme ici une part, il ne la fixe jamais.)*
5. **AC-5 — Lignes d'une partielle exacte.** Une ligne `CARD-<resource_id>` par devise de part non nulle, dans le
   `WiseCsv` de cette devise, au **même** instant et à la **même** date (39-12a2, AC-3 et A-2) : débit de c dans la devise
   du marchand, sans champ d'échange ; débit de p_X (frais compris, aucune ligne de frais : 39-12a2 AC-4) avec
   `exchange_from` = X, `exchange_to` = devise du marchand, `exchange_to_amount` = couv_X, `exchange_rate` = r_X ;
   `details_type` `CARD` / `DEBIT`, `merchant` et description comme un `CARD_PAYMENT` (39-12a2 AC-3). Une part connue garde
   sa valeur en base (la ligne sera un doublon, 39-12a3b). Le rapprochement (39-11c, A-1) réunit ces lignes en **une**
   écriture : charge = c + Σ couv_X = M, chaque solde crédité de sa part, différence de change sur sa ligne (décision
   (b) du 21.11).
6. **AC-6 — Partielle « à vérifier » : proposée, jamais importée.** Une partielle qui n'est pas exacte n'a **aucune**
   ligne dans `rows`, dans **aucune** devise (une écriture groupée ne se forme qu'avec toutes ses parts). Elle va à
   `held_back` si sa date est dans `period` : `WiseApiHeldBack { card_id, date, merchant_amount, merchant_currency,
   parts: [{ currency, amount, exact: bool, cover: Option<Decimal> }], reason }` (`reason` ∈ `split_group`,
   `rate_mismatch`, `no_calibration`, `unsupported_shape` ; `amount` = la valeur exacte, connue ou estimée). Chaque
   groupe de l'AC-3 qui contient une part retenue va à `split_groups` : `{ currency, from: NaiveDate, to: NaiveDate, sum,
   card_ids }` (dates des deux ancrages, **somme exacte**). Rien n'est écrit d'office (règle « un appariement automatique
   propose, il ne crée jamais ») : la partielle s'importe ensuite par le relevé CSV de sa période — ses lignes y sont
   neuves —, ou se saisit à la main (manuel, 39-12b). Les soldes d'ouverture et de clôture de chaque `WiseCsv` restent
   ceux de la continuité : `closing − opening` = somme de ses `rows` + somme des parts retenues de sa devise dans
   `period` (asserté).
7. **AC-7 — Refus.** Toute la lecture est refusée, rien n'est rendu :
   - **`BalanceGap { currency, from, to }`** (code `WISE_API_BALANCE_GAP`) — trou de solde : un solde reconstitué
     négatif ; un contrôle de segment sans inconnue faux dont un ancrage est **de règle** ; deux ancrages d'un même instant
     différents ; une part hors de (ii) de l'AC-4 ; des frais implicites hors de la borne de vraisemblance de l'AC-4, avec
     ou sans étalonnage. `from`, `to` = les dates des deux ancrages qui bornent le segment en cause. Le refus vaut même
     hors de `period` (le solde de la période se reconstitue à travers lui) ; c'est ainsi que la conversion de juillet
     2025 absente de l'API se voit. Le contrôle faux entre **deux ancrages observés** reste `BalanceMismatch` (39-12a2
     AC-5) ;
   - **`SplitUnresolved { currency, date, reason }`** (code `WISE_API_SPLIT_UNRESOLVED`) — `reason = unanchored` : une
     part inconnue dans un segment ouvert (aucun ancrage avant elle dans sa devise : premier téléchargement sans relevé
     antérieur, ou point de contrôle non placé) ; `reason = period_boundary` : le solde d'ouverture ou de clôture d'une
     devise (avant la première ligne de `period`, après la dernière) dépend d'une part estimée. `date` = celle de la
     partielle en cause.
   Un écart de confirmation (iii) **dans** la borne de vraisemblance ne refuse rien : la partielle est « à vérifier »
   (`rate_mismatch`).
8. **AC-8 — Tests** (unitaires dans `wise_api.rs` ou un module voisin `wise_api/continuite.rs`, intégration
   `crates/kesh-import/tests/wise_api.rs`), sur des activités, cours et lignes en base **fictifs** (forme des fixtures
   `kesh-gate-logs/epic-39-40-kiss/wise-fixtures-conversion-carte/`, recopiées sous `tests/fixtures/` avec leur ligne au
   catalogue) : partielle à deux devises exacte (lignes, champs d'échange, c + couv = M) ; à trois devises dont l'EUR est
   seule inconnue entre un point de contrôle placé et le solde actuel (exacte) ; quatre parts USD dans un segment (groupe,
   `split_group`, somme exacte, ouverture et clôture exactes) ; part connue par un CSV en base (exacte, valeur de la base,
   frais compris) ; mouvement manquant (solde CHF négatif ; totale sur un solde non nul) → `BalanceGap`, y compris hors
   de `period` ; mouvement manquant absorbé par une seule inconnue → frais implicites hors borne → `BalanceGap` ; écart
   de cours de deux centimes dans la borne → `rate_mismatch` ; sans totale du couple : deux devises dans la borne
   (exacte), trois devises (`no_calibration`) ; capture tardive (ordre à `updated_on`) ; point de contrôle placé, puis
   non placé (lignes mêlées ; relevé saisi à la main) ; segment ouvert → `unanchored` ; fin de période entre deux parts
   d'un groupe → `period_boundary` ; part nulle → pas de ligne ; partielle à marchand en EUR → `unsupported_shape` ;
   l'invariant de l'AC-6 sur chaque `WiseCsv`. Tests de 39-12a2 verts, sauf ceux du refus `PartialCardConversion`,
   remplacés.
9. **AC-9 — Recette locale, non versionnée** (aucun appel réel : réponses déjà sauvées par la recherche du 21.11 sous
   `data-not-committed/`, test local hors dépôt, comme 39-12a1 AC-0 (b)) : le moteur rejoué sur 2025–2026 contre les
   relevés du site, par fenêtres mensuelles avec le point de contrôle de fin du mois précédent pris aux relevés. Attendu
   et constaté au Change Log, en métadonnées : aucune part importée fausse ; chaque partielle importée égale au relevé ;
   les fenêtres qui couvrent juillet 2025 → `BalanceGap` ; nombre de partielles importées et « à vérifier ».

## Tâches

- [ ] T1 — Entrées et sorties, retrait de `PartialCardConversion` (AC-1).
- [ ] T2 — Inconnues, ancrages, placement des points de contrôle (AC-2).
- [ ] T3 — Segments, groupes, estimation au cours, étalonnage, exactitude (AC-3, AC-4).
- [ ] T4 — Lignes, parts retenues, refus (AC-5 à AC-7).
- [ ] T5 — Tests (AC-8) ; recette locale (AC-9).

## Stratégie de tests

Gate ciblé `binary(wise_api)` et tests de `kesh-import` ; `cargo publish --dry-run -p kesh-import` ; gate complet au
dernier commit. Aucune migration.

## Modules touchés (1)

`kesh-import` seul.

## Fichiers communs

`crates/kesh-import/src/wise_api.rs` (39-12a2), `tests/fixtures/README.md` : à relire à la reprise de `main`.

## Note de version

*Ajouté* — Un paiement par carte que Wise a payé avec plusieurs soldes s'importe par l'API quand Kesh en établit
exactement la part de chaque solde (continuité des soldes, confirmée par le cours) ; sinon, il est proposé à vérifier
avec ses parts estimées, jamais importé d'office ; un trou dans les soldes fait refuser le téléchargement.

## Notes de développement

- Arithmétique en `Decimal`, arrondi au centime demi supérieur (`RoundingStrategy::MidpointAwayFromZero` sur des
  montants positifs).
- Limite assumée : les règles de Wise (R1 à R3, ordre de prélèvement) sont **observées** sur un compte, non documentées ;
  une forme non vue n'est jamais exacte. La confirmation par le cours et le refus `BalanceGap` sont les garde-fous.
- Limite assumée : à trois devises, l'EUR n'a d'ancrage qu'aux points de contrôle et au solde actuel ; un téléchargement
  mensuel (point de contrôle à la fin du mois précédent) est ce qui le rend exact.
- Hors périmètre : le webhook `balances#update` (répartition exacte pour l'avenir, recherche du 21.11, piste 4 a) —
  exposition publique et décision d'architecture à prendre d'abord.

## Change Log

- 2026-11-21 — fiche rédigée (amendements du 21.11 de 39-12a2 et 39-12a3 ; suivi de la décision (a)). Statut `drafted`,
  validation P1 à lancer.
