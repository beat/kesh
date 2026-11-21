# Story 39-12a4a2 : paiements par carte partiellement convertis — estimation au cours, confirmation, import des partielles exactes (`kesh-import`)

Status: drafted

*Fiche née le 2026-11-21 du découpage de 39-12a4a à la validation P1 (finding M15, règle des petites stories) : elle
reprend la partie « estimation » de l'ancienne fiche (AC-4, l'étalonnage et l'AC-9), **recalibrée** (findings M3 à M6,
M10). Branche `epic/39-wise`. Epic 39, **phase 2**. **Dépend de** : 39-12a4a1 (continuité : ancrages, segments, parts
déduites, refus). **Suivie par** : 39-12a4b (le téléchargement lit les cours et appelle le moteur avec `split: Some`).
**Porte une décision ouverte** (§ « Décision à prendre »), à soumettre au responsable de l'epic avant le développement.
Fonction pure.*

## Story

En tant que **Comptable**, je veux qu'un paiement par carte que Wise a payé avec plusieurs soldes soit importé par
l'API quand Kesh en établit **exactement** la part de chaque solde, et proposé « à vérifier » avec une estimation au
cours de Wise quand il ne le peut pas.

## Ce que la mesure établit (rappel, montants réels hors dépôt)

- Le cours `GET /v1/rates` à la minute est celui du relevé (*Exchange Rate*) pour 9 partielles sur 10 (écart de 0.0001
  pour la dixième) ; **montant converti = arrondi au centime, demi supérieur, de la part couverte dans la devise du
  marchand divisée par le cours** (26 lignes sur 26) ; frais à part, au moins 0.01, règle d'arrondi non élucidée.
- Frais rapportés au montant converti, sur les lignes converties de carte des relevés 2025–2026 de **20 ou plus** :
  USD → CHF de 0.27 à 0.31 %, EUR → CHF de 0.42 à 0.44 % ; en dessous de 20, l'arrondi des frais domine (jusqu'à 0.68 %
  sur une ligne d'environ 1.5).
- Parts estimées au cours (frais étalonnés) contre le relevé : 6 sur 11 au centime, 5 à un centime (trois estimations au
  cours, une part de reste et un centime reporté de la conversion cachée de juillet 2025) ; ce que la continuité
  déclarait exact l'était, 11 parts sur 11.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-12a4a2-1` | Les frais de conversion s'étalonnent par couple sur les totales par carte de la lecture, à défaut sur des constantes mesurées. | AC-2 |
| `FR-39-12a4a2-2` | Les parts d'un groupe s'estiment au cours de Wise ; une part déduite par la continuité se confirme au cours, avec une tolérance qui croît avec le montant et l'incertitude des frais. | AC-3, AC-4 |
| `FR-39-12a4a2-3` | Une partielle dont toutes les parts sont connues ou déduites et confirmées est importée ; les autres sont « à vérifier », jamais importées. | AC-5 |
| `NFR-39-12a4a2-1` | Sans I/O, dans `kesh-import`, sans dépendance neuve ; aucune donnée réelle dans le dépôt. | AC-6, AC-7 |

## Critères d'acceptation

1. **AC-1 — Entrées, erreur, liste des cours.** `WiseApiSplitInput` (39-12a4a1) gagne `rates: Vec<WiseApiRate {
   activity_id, source: String, target: String, rate: Decimal }>` — un par devise convertie de chaque partielle (cible :
   la devise du marchand) et un par paiement par carte à conversion **totale** (étalonnage, AC-2). Un cours nécessaire
   absent, ou `rate ≤ 0` → `MissingRate { activity_id, source }` (code `WISE_API_MISSING_RATE`, que 39-12a4b place dans
   `details.reason` de `PROVIDER_RESPONSE_INVALID`) : défaut de l'appelant, jamais une partielle exacte. **Liste des cours
   à lire** : fonction publique `rates_needed(&[WiseApiActivity]) -> Vec<WiseApiRateRequest { activity_id, source,
   target, at_updated: bool }>`, qui applique les règles de 39-12a2 (partielle, totale, capture tardive A-2 :
   `at_updated` dit quel instant de l'activité prendre) — 39-12a4b l'appelle au lieu de les recopier.
2. **AC-2 — Étalonnage des frais, par couple (X → devise du marchand).** Sur les totales par carte du couple dans la
   lecture dont le montant converti `conv_i = arrondi_demi_sup(primaire / cours_i)` vaut **20 ou plus** : `t_i =
   (secondaire − conv_i) / conv_i` ; taux `t̂` = la médiane (moyenne des deux valeurs du milieu en nombre pair) ;
   incertitude `Δt = max((max t_i − min t_i) / 2, 0.02 %)`. Sans telle totale : constantes **mesurées** sur les relevés
   2025–2026 (observées, non documentées par Wise) — USD → CHF : `t̂` = 0.29 %, `Δt` = 0.02 % ; EUR → CHF : `t̂` = 0.43 %,
   `Δt` = 0.02 %. Autre couple sans totale : pas d'étalonnage (motif `no_calibration`). Les taux se calculent en
   `Decimal` sans arrondi intermédiaire ; seuls les montants s'arrondissent au centime (demi supérieur). Le même
   étalonnage vaut à deux et à trois devises (aucune borne de vraisemblance à part).
3. **AC-3 — Formules (sans circularité).** Soit M le montant du marchand, c sa part dans la devise du marchand (connue ou
   déduite), r_X le cours, et pour une devise X convertie : `conv_X` le montant converti, `frais_X = max(0.01,
   arrondi_demi_sup(conv_X × t̂_X))`, `couv_X` la part de M couverte.
   - *Deux devises* : `couv_X = M − c` ; `conv_X = arrondi_demi_sup(couv_X / r_X)` ; part attendue `conv_X + frais_X`.
   - *Trois devises (CHF, USD, EUR)* : l'USD n'est pas recalculé depuis M (sa part se déduit de R2) ; sa couverture se
     tire de sa part, en forme fermée : `conv_USD = arrondi_demi_sup(p_USD / (1 + t̂_USD))`, `couv_USD =
     arrondi_demi_sup(conv_USD × r_USD)` ; puis `couv_EUR = M − c − couv_USD`, `conv_EUR = arrondi_demi_sup(couv_EUR /
     r_EUR)`, part EUR attendue `conv_EUR + frais_EUR`.
   - *Tolérance* (arrondie au centime supérieur) : `T_X = 0.02 + conv_X × (Δt_X + 0.02 %)` — 0.02 pour l'arrondi du
     montant converti et des frais, `Δt` pour la règle des frais, 0.02 % pour un cours décalé de 0.0001 ; à trois
     devises, celle de l'EUR s'élargit de l'incertitude reportée de l'USD : `T_EUR = 0.02 + conv_EUR × (Δt_EUR + 0.02 %)
     + (0.01 + p_USD × (Δt_USD + 0.02 %)) × r_USD / r_EUR`.
   - *Mesuré* : les 10 partielles réelles, aux valeurs vraies du relevé, passent la confirmation de l'AC-4 avec l'un et
     l'autre étalonnage (totales ; constantes), cours décalé de ±0.0001 compris : écart au plus 0.04, tolérance de 0.03 à
     0.12 selon le montant.
4. **AC-4 — Parts déduites : confirmées par le cours.** Une part **déduite** par la continuité (39-12a4a1 AC-3, seule
   inconnue de son segment) est **confirmée** quand |part − part attendue| ≤ T. À deux devises, la part convertie se
   confirme ; à trois, l'**EUR** (son contrôle porte aussi l'USD, par `M − c − couv_USD`) ; l'USD d'une partielle à
   trois devises n'est pas confirmé à part (sa couverture vient de lui-même). Une part **connue** (relevé CSV en base)
   ne se confirme pas. Non confirmée → la partielle est « à vérifier », motif `rate_mismatch` (jamais un refus de la
   lecture : un mouvement manquant ou un cours décalé, le message renvoie au site de Wise et au CSV de la période).
   *Limite assumée* : un mouvement absent de l'API, absorbé par la seule inconnue de son segment et plus petit que T (de
   0.03 à 0.12 pour des parts de 10 à 210), passe la confirmation ; la continuité ne le voit pas, le cours non plus.
5. **AC-5 — Exactitude, lignes, parts retenues.** Une partielle est **exacte** quand chacune de ses parts est connue ou
   déduite (39-12a4a1 AC-4) **et** que chaque part déduite à confirmer l'est (AC-4) ; ses lignes sont celles de
   39-12a4a1 AC-5, avec `exchange_to_amount` = `couv_X` et `exchange_rate` = r_X (la charge reste M : c + Σ couv_X = M ;
   un écart d'un centime de `couv_USD` à trois devises ne change que la différence de change, 39-11c). Toute autre
   partielle va à `held_back` (39-12a4a1 AC-5), motifs : `split_group` (part dans un groupe), `rate_mismatch`,
   `no_calibration`, `part_already_booked` (39-12a4a1), `unsupported_shape` (le motif `unconfirmed` de 39-12a4a1 disparaît). **Estimation d'un groupe**
   (pour la proposition seulement, jamais importée) : chaque part du groupe, sauf la dernière, prend sa part attendue
   (AC-3) quand c est connu ou déduit ; la dernière prend le reste de la somme exacte ; un reste inférieur à 0.01, ou un
   c indéterminé, laisse les parts du groupe sans montant (`amount: None`). Les soldes courants qui dépendent d'une part
   estimée sont **estimés** ; un solde estimé n'est un trou (`BalanceGap`) que s'il est inférieur à moins la somme des
   tolérances des parts estimées dont il dépend (un solde déterminé négatif reste un trou, 39-12a4a1 AC-6).
6. **AC-6 — Tests** (unitaires dans `wise_api/continuite.rs` ou un module voisin `wise_api/cours.rs`, intégration
   `crates/kesh-import/tests/wise_api.rs`), sur des activités, cours et lignes en base **fictifs** (fixtures de
   39-12a4a1) : étalonnage (médiane, nombre pair, totales de moins de 20 écartées, `Δt` plancher, constantes faute de
   totale, autre couple → `no_calibration`) ; partielle à deux devises déduite et confirmée → lignes, champs d'échange,
   c + couv = M ; à 157 unités et taux bas de l'étalonnage → confirmée (cas du finding M3) ; cours décalé de 0.0001 →
   confirmée ; à trois devises, EUR déduite et confirmée, la sensibilité d'un centime des frais USD absorbée (cas du
   finding M4) ; mouvement manquant de 0.50 absorbé par la seule inconnue → `rate_mismatch` (cas du finding M5) ; écart
   de cours de deux centimes au-delà de T → `rate_mismatch` ; groupe de quatre parts → estimation, dernière au reste,
   somme exacte, reste < 0.01 → parts sans montant ; solde estimé légèrement négatif dans la tolérance → accepté, au-delà
   → `BalanceGap` ; cours absent, `rate ≤ 0` → `MissingRate` ; `rates_needed` (partielle, totale, capture tardive).
7. **AC-7 — Recette locale, non versionnée** (aucun appel réel : réponses déjà sauvées par la recherche du 21.11 sous
   `data-not-committed/`, test local hors dépôt, comme 39-12a1 AC-0 (b)) : le moteur rejoué sur 2025–2026 contre les
   relevés du site, par fenêtres mensuelles avec le point de contrôle de fin du mois précédent pris aux relevés. Attendu
   et constaté au Change Log, **en métadonnées seulement** (nombres, dates de mois, motifs ; aucun montant ni nom) :
   aucune part importée fausse ; chaque partielle importée égale au relevé ; les fenêtres qui couvrent juillet 2025 →
   `BalanceGap` ; nombre de partielles importées et « à vérifier ». Condition de livraison.

## Décision à prendre (finding M10 de la validation P1, à soumettre au responsable de l'epic)

**La question.** La consigne du 21.11 (décision (a), « essayer encore ») dit qu'une part est **exacte** avec « deux
ancrages indépendants, **ou** un ancrage et le cours à 0.01 près ». Cette fiche la **durcit** : le cours **confirme** une
part déduite par la continuité, il ne **fixe** jamais une part. Le choix est à faire par le responsable de l'epic ; la
fiche est écrite pour l'option 1, l'option 2 change l'AC-4 et l'AC-5 (une part estimée et confirmée devient exacte).

| | Option 1 — le cours confirme (fiche) | Option 2 — le cours fixe (consigne) |
|---|---|---|
| Règle | importée seulement si chaque part est connue ou déduite par deux ancrages, et confirmée | importée aussi quand une part n'a qu'un ancrage et que son estimation au cours tient à 0.01 près |
| Téléchargement mensuel (point de contrôle à la fin du mois précédent) | **10 importées, 0 à vérifier** | **10 importées, 0 à vérifier** (identique : chaque partielle est la seule inconnue de sa devise dans son segment) |
| Une lecture de 2025–2026 d'un coup (ancrages : totales et solde actuel) | **1 importée, 9 à vérifier** | **10 importées, dont 4 fausses d'un centime** (5 parts) |
| Erreur écrite | aucune | un centime, **figé** : le CSV importé ensuite est ignoré comme doublon (même identifiant, 39-11b), il ne le corrige pas |

*D'où viennent ces chiffres* : les 10 partielles réelles de 2025–2026 et le tableau « vrai contre reconstitué » de la
recherche (§ « Mesure contre les relevés réels ») ; le scénario mensuel est **déduit** de ce tableau (une partielle par
mois, deux en novembre 2026 séparées par R2) et de la confirmation mesurée à l'AC-3, non rejoué — la recette AC-7 le
mesure ; la fenêtre de juillet 2025 est refusée dans les deux options (`BalanceGap`, conversion absente de l'API). Dans
la lecture d'un coup, la conversion cachée de juillet 2025 est supposée connue, sinon la lecture entière est refusée.

**Recommandation** : l'option 1, complétée — les parts **exactes par continuité** (et confirmées) sont importées ; les
parts **estimées au cours** sont proposées « à vérifier » **pré-remplies** (estimation de l'AC-5, somme exacte du groupe),
et l'utilisateur les **valide d'un geste**. Rendement sur la lecture d'un coup : 1 importée, 9 proposées pré-remplies,
dont 5 justes au centime et 4 à corriger d'un centime (l'écran ne peut pas dire lesquelles : il invite à comparer au site
de Wise, ou à importer le CSV de la période). Rien n'est écrit sans le geste de l'utilisateur (règle « un appariement
automatique propose, il ne crée jamais »), et le centime faux d'une estimation n'est jamais figé à son insu.
*Conséquence si elle est retenue* : le geste « valider la répartition proposée » (route de `kesh-api`, écran) n'est dans
aucune fiche ; une story de plus (39-12a4c) le porterait, après 39-12a4b. Sans elle, la sortie d'une partielle « à
vérifier » reste le relevé CSV de sa période.

## Tâches

- [ ] T1 — Cours en entrée, `MissingRate`, `rates_needed` (AC-1).
- [ ] T2 — Étalonnage (AC-2).
- [ ] T3 — Formules, tolérance, confirmation (AC-3, AC-4).
- [ ] T4 — Exactitude, estimation des groupes, soldes estimés (AC-5).
- [ ] T5 — Tests (AC-6) ; recette locale (AC-7).

## Stratégie de tests

Gate ciblé `binary(wise_api)` et tests de `kesh-import` ; `cargo publish --dry-run -p kesh-import` ; gate complet au
dernier commit. Aucune migration.

## Modules touchés (1)

`kesh-import` seul (`split` n'est construit à `Some` que par 39-12a4b : rien à raccorder dans `kesh-api`).

## Fichiers communs

`crates/kesh-import/src/wise_api.rs` et ses modules (39-12a2, 39-12a4a1), `tests/fixtures/README.md` : à relire à la
reprise de `main`.

## Note de version

*Ajouté* — Un paiement par carte que Wise a payé avec plusieurs soldes s'importe par l'API quand Kesh en établit
exactement la part de chaque solde (continuité des soldes, confirmée par le cours de Wise) ; sinon, il est proposé à
vérifier avec ses parts estimées, jamais importé d'office ; un trou dans les soldes fait refuser le téléchargement.

## Notes de développement

- Arithmétique en `Decimal` ; montants arrondis au centime demi supérieur (`RoundingStrategy::MidpointAwayFromZero`,
  montants positifs), tolérances au centime supérieur.
- Les constantes de l'AC-2 sont des **mesures** (relevés 2025–2026 d'un compte), non des règles de Wise : une totale du
  couple dans la lecture les remplace. Si Wise change ses frais, la confirmation échoue (`rate_mismatch`) : rien de faux
  n'est écrit.
- Le Change Log ne reporte de la recette AC-7 que des métadonnées.

## Change Log

- 2026-11-21 — fiche née du découpage de 39-12a4a à la remédiation de la validation P1 (Claude Sonnet 5.5, rapport
  `kesh-gate-logs/s39-12a-specs/validation-p1-rapport.md`) : **M3** — tolérance qui croît avec le montant et
  l'incertitude des frais (`T`), arrondi des taux fixé, cas à 157 unités ; **M4** — formule fermée à trois devises,
  l'USD non confirmé à part, tolérance de l'EUR élargie ; **M5** — plus de borne de vraisemblance de 1 % : même
  étalonnage à deux et à trois devises, constantes mesurées faute de totale ; **M6** — solde estimé négatif toléré dans
  la somme des tolérances ; **M7** — part nulle ou négative : refus (39-12a4a1) ; **M9** — `MissingRate` ; **M10** —
  décision ouverte écrite avec son rendement ; **L10** — « cinq fois sur onze » ventilé ; **L12** — `rates_needed` ;
  **L13** — recette en métadonnées ; **L14** — `rate ≤ 0`. Vérifié sur les relevés réels (hors dépôt) : les 10
  partielles passent la confirmation recalibrée. Statut `drafted`, validation P2 à lancer.
