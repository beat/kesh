# Story 39-12a4a2 : paiements par carte partiellement convertis — estimation au cours, confirmation, import des partielles exactes (`kesh-import`)

Status: deferred

**En attente (22.11) : téléchargement par l'API suspendu jusqu'à la réponse du support Wise ; validation P3 non remédiée (rapport `kesh-gate-logs/s39-12a-specs/validation-p3-rapport.md`).**

*Fiche née le 2026-11-21 du découpage de 39-12a4a à la validation P1 (finding M15, règle des petites stories) : elle
reprend la partie « estimation » de l'ancienne fiche (AC-4, l'étalonnage et l'AC-9), **recalibrée** (findings M3 à M6,
M10). Branche `epic/39-wise`. Epic 39, **phase 2**. **Dépend de** : 39-12a4a1 (continuité : ancrages, segments, parts
déduites, refus). **Suivie par** : 39-12a4b (le téléchargement lit les cours et appelle le moteur avec `split: Some`).
**Porte la décision C du 22.11** (§ « Décision C du 22.11 » : résolution d'un groupe au centime, AC-4b), prise par le responsable de l'epic.
Fonction pure.*

## Story

En tant que **Comptable**, je veux qu'un paiement par carte que Wise a payé avec plusieurs soldes soit importé par
l'API quand Kesh en établit **exactement** la part de chaque solde, et proposé « à vérifier » avec une estimation au
cours de Wise quand il ne le peut pas.

## Ce que la mesure établit (rappel, montants réels hors dépôt)

- Le cours `GET /v1/rates` à la minute est celui du relevé (*Exchange Rate*) pour presque toutes les partielles (écart d'au
  plus 0.0001 pour les autres ; la recherche écrit « 9 sur 10 » et nomme deux écarts : non tranché, sans effet, la tolérance couvre ±0.0001) ; **montant converti = arrondi au centime, demi supérieur, de la part couverte dans la devise du
  marchand divisée par le cours** (26 lignes sur 26) ; frais à part, `max(0.01, arrondi_demi_sup(p × montant converti))` dans la devise source, **p variable dans le temps** (par couple, d'environ 0.28 % en USD → CHF et 0.43 % en EUR → CHF), publié nulle part (recherche « option C » du 22.11).
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
| `FR-39-12a4a2-4` | Un groupe de parts inconnues se résout au centime par la somme exacte, l'atteignabilité des montants et une bande de pourcentage de frais élargie jusqu'à la première solution, retenue si elle est unique (décision C du 22.11). | AC-4b |
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
4b. **AC-4b — Résolution d'un groupe au centime (décision C du 22.11).** Un **groupe** (39-12a4a1 AC-3 : plusieurs parts inconnues d'une même devise convertie X entre deux ancrages, somme exacte `Σ`) se résout ainsi, sans I/O. Règle des montants (mesurée sur les relevés 2025–2026, hors dépôt) : le montant converti d'une part est `conv = arrondi_demi_sup(couv / cours)` où `couv` est la part couverte, un **montant au centime de la devise du marchand** ; les frais sont `max(0.01, arrondi_demi_sup(p × conv))` dans la devise X, `p` étant le pourcentage de Wise, **inconnu et variable dans le temps**. Pour une bande de `p` donnée, on énumère les valeurs de `couv` de chaque part (c, déduit ou connu, fixe la somme `M − couv_X` des autres) et on ne garde que les combinaisons où : (1) la somme des parts (`conv + frais`) vaut `Σ` ; (2) chaque `conv` est **atteignable** (image par l'arrondi de `couv / cours` d'un `couv` au centime : un centime de la devise du marchand vaut parfois plus d'un centime de X, si bien que certains montants ne sont l'image d'aucun `couv`) ; (3) à trois devises, la part EUR suit du reste en CHF (formules de l'AC-3). **Bande** : elle part de la plage observée du couple (constantes mesurées de l'AC-2 : USD → CHF 0.2793–0.2826 %, EUR → CHF 0.428–0.436 %, ou l'étalonnage par les totales de la lecture) et **s'élargit par pas de 0.0001 point de pourcentage, de chaque côté, jusqu'à la première bande qui donne au moins une solution** (plafond d'élargissement : 0.05 point, à fixer à l'étape 0 d'après les mesures). **Exactement une solution** → les parts du groupe sont **exactes** et importées ; **zéro ou plusieurs** → le groupe reste « à vérifier », pré-rempli (AC-5), avec sa somme exacte. Un groupe à une seule inconnue est exact sans bande (continuité, AC-4). Le cas « aucune solution tant que la bande reste étroite » est un signal qu'une part sort de la règle : on élargit, on ne refuse pas. **Cas non résolus, toujours « à vérifier »** : groupe à plusieurs solutions ; part EUR dont la part USD est incertaine ; mouvement absent de l'API dans le segment, quand il touche la devise du groupe ; **part seule hors d'un groupe fermé** (sans ancrage qui la ferme : estimation à ±0.01 sur les frais). **Limite assumée, écrite au manuel (39-12b)** : la bande et la règle d'élargissement ont été **réglées sur les relevés mêmes qui les mesurent** (le rendement ci-dessous vaut pour ces données-ci, deux groupes seulement ayant éprouvé l'élargissement) ; une solution unique n'est pas prouvée juste. La recette (AC-7) la **mesure sur de nouvelles données** avant la livraison, et la protection contre un centime faux figé est celle de 39-12a3b (un téléchargement dont une part a un centime faux fait refuser l'import du CSV « avec frais séparés » ensuite, `WISE_FEE_ORIGIN_MISMATCH`, jamais des frais comptés deux fois en silence).
5. **AC-5 — Exactitude, lignes, parts retenues.** Une partielle est **exacte** quand chacune de ses parts est connue ou
   déduite (39-12a4a1 AC-4) **et** que chaque part déduite à confirmer l'est (AC-4), **ou** que ses parts sont résolues par un groupe à solution unique (AC-4b) ; ses lignes sont celles de
   39-12a4a1 AC-5, avec `exchange_to_amount` = `couv_X` et `exchange_rate` = r_X (la charge reste M : c + Σ couv_X = M ;
   un écart d'un centime de `couv_USD` à trois devises ne change que la différence de change, 39-11c). Toute autre
   partielle va à `held_back` (39-12a4a1 AC-5), motifs : `split_group` (part dans un groupe sans solution unique), `rate_mismatch`,
   `no_calibration`, `part_already_booked` (39-12a4a1), `unsupported_shape` (le motif `unconfirmed` de 39-12a4a1 disparaît). **Estimation d'un groupe non résolu**
   (pour la proposition seulement, jamais importée) : chaque part du groupe, sauf la dernière, prend sa part attendue
   (AC-3) quand c est connu ou déduit ; la dernière prend le reste de la somme exacte ; un reste inférieur à 0.01, ou un
   c indéterminé, laisse les parts du groupe sans montant (`amount: None`). Les soldes courants qui dépendent d'une part
   estimée sont **estimés** ; un solde estimé n'est un trou (`BalanceGap`) que s'il est inférieur à moins la somme des
   tolérances des parts estimées dont il dépend (un solde déterminé négatif reste un trou, 39-12a4a1 AC-6).
6. **AC-6 — Tests** (unitaires dans `wise_api/continuite.rs` ou un module voisin `wise_api/cours.rs`, intégration
   `crates/kesh-import/tests/wise_api.rs`), sur des activités, cours et lignes en base **fictifs** (fixtures de
   39-12a4a1) : étalonnage (médiane, nombre pair, totales de moins de 20 écartées, `Δt` plancher, constantes faute de
   totale, autre couple → `no_calibration`) ; partielle à deux devises déduite et confirmée → lignes, champs d'échange,
   c + couv = M ; à plus de 150 unités et taux bas de l'étalonnage → confirmée (cas du finding M3) ; cours décalé de 0.0001 →
   confirmée ; à trois devises, EUR déduite et confirmée, la sensibilité d'un centime des frais USD absorbée (cas du
   finding M4) ; mouvement manquant de 0.50 absorbé par la seule inconnue → `rate_mismatch` (cas du finding M5) ; écart
   de cours de deux centimes au-delà de T → `rate_mismatch` ; groupe de quatre parts → résolution (AC-4b : une seule solution, parts exactes et importées ; bande étroite sans solution puis élargie ; deux solutions → « à vérifier » pré-rempli ; montant non atteignable écarté ; part EUR à trois devises ; mouvement absent → « à vérifier ») ; groupe non résolu → estimation, dernière au reste,
   somme exacte, reste < 0.01 → parts sans montant ; solde estimé légèrement négatif dans la tolérance → accepté, au-delà
   → `BalanceGap` ; cours absent, `rate ≤ 0` → `MissingRate` ; `rates_needed` (partielle, totale, capture tardive).
7. **AC-7 — Recette locale, non versionnée** (aucun appel réel : réponses déjà sauvées par la recherche du 21.11 sous
   `data-not-committed/`, test local hors dépôt, comme 39-12a1 AC-0 (b)) : le moteur rejoué sur 2025–2026 contre les
   relevés du site, par fenêtres mensuelles avec le point de contrôle de fin du mois précédent pris aux relevés. Attendu
   et constaté au Change Log, **en métadonnées seulement** (nombres, dates de mois, motifs ; aucun montant ni nom) :
   aucune part importée fausse ; chaque partielle importée égale au relevé ; les fenêtres qui couvrent juillet 2025 → refusées (`BalanceGap` ou `BalanceMismatch`, selon les ancrages) ; **mesure de la résolution par groupe sur de nouvelles données** (AC-4b, limite) ; nombre de partielles importées et « à vérifier ». Condition de livraison.

## Décision C du 22.11 (finding M10 de la validation P1, tranchée par le responsable de l'epic)

**Historique de la question.** La consigne du 21.11 (décision (a)) disait qu'une part est exacte avec « deux ancrages indépendants, ou un ancrage et le cours à 0.01 près ». Deux options avaient été chiffrées : A (le cours confirme, une estimation reste « à vérifier ») et B (le cours fixe : importée, avec un centime faux possible). **Le 22.11, le responsable de l'epic a répondu : ni A ni B, option C — chercher encore la règle du dernier centime.** La recherche (`wise-conversion-partielle-recherche.md`, § « Option C : le dernier centime », hors dépôt) a établi : montant converti = `arrondi(montant du marchand / cours)` ; frais = `max(0.01, arrondi(p × montant converti))` dans la devise source, au plus proche ; **p varie dans le temps et n'est publié nulle part**, si bien qu'une règle seule ne reproduit que 70 lignes de conversion sur 81. Combinée à la somme exacte d'un groupe entre deux soldes à zéro et à l'atteignabilité des montants (AC-4b), la résolution par élargissement de la bande de p rend les 22 parts et les 10 partielles exactes — **sur ces relevés seulement** (bande réglée sur les mêmes données).

**Ce que la fiche prescrit** : l'AC-4b ; une solution unique est **importée**, tout autre cas est « à vérifier » **pré-rempli** (estimation de l'AC-5, somme exacte du groupe), que l'utilisateur ne valide jamais d'office.

| Rendement (sur les 10 partielles réelles de 2025–2026, ces relevés seulement) | Résultat |
|---|---|
| Téléchargement mensuel (point de contrôle de fin de mois) | 10 importées, 0 à vérifier (chaque partielle est la seule inconnue de sa devise dans son segment : continuité, sans bande) |
| Lecture d'un coup (ancrages : totales et solde actuel) | **10 importées, 0 à vérifier** avec la résolution par groupe (contre 1 importée et 9 à vérifier à l'option A, 10 dont 4 fausses d'un centime à l'option B) ; la fenêtre de juillet 2025 est refusée, la conversion cachée n'étant pas dans l'API ; si elle est connue par un relevé, la part CHF de la partielle suivante est exacte par continuité |
| Cas à « à vérifier » | groupe à plusieurs solutions ; part EUR dont la part USD est incertaine ; mouvement absent de l'API dans le segment ; part seule hors groupe fermé (±0.01) |

**Limites et risque résiduel** (finding M5 de la validation P2) : (1) le rendement est mesuré sur les données qui ont réglé la bande ; la recette AC-7 le mesure sur de nouvelles données avant la livraison ; (2) une solution unique n'est pas une preuve : si elle est fausse d'un centime et importée, le CSV « avec frais séparés » importé ensuite ne la corrige pas — sa ligne `FEE-CARD-x` n'est pas reconnue par R1 (l'origine en base n'a pas le montant « fichier + frais »), `FEE-CARD-x` serait insérée et `CARD-x` ignorée (même identifiant, même sens) : **frais comptés deux fois, sans signal**. 39-12a3b ferme ce chemin : une ligne de frais dont l'origine est en base sans remplir R1 fait **refuser l'import** (`WISE_FEE_ORIGIN_MISMATCH`), ce que le manuel dit avec sa sortie (annuler l'import téléchargé ou l'importer du CSV seul) ; (3) la sortie d'un « à vérifier » reste le relevé CSV de sa période. Le geste « valider la répartition proposée » (39-12a4c) n'est pas pris par cette décision : sans lui, la sortie reste le CSV.

## Décision du 22.11 (règle du centime, remplace l'AC-4b de la décision C)

*Prise par le responsable de l'epic le 22.11, après la validation P3 (H1 : la bande de p élargie jusqu'à la première solution rendait des solutions uniques fausses). Écrite ici, **non encore propagée** aux AC (la fiche est en attente) : à la reprise, elle remplace l'AC-4b et la ligne `FR-39-12a4a2-4`, et les findings M1 à M5 et L1 à L9 de la P3 restent à traiter.*

Une part estimée au cours peut être **ajustée d'au plus un centime, vers le haut ou vers le bas**, pour que la continuité des soldes tombe juste sur le solde connu suivant. **Ancrages** : ceux de 39-12a4a1 AC-2 (solde actuel, zéros de règle dont le zéro après une conversion totale, point de contrôle placé). Règle, sans I/O et sans énumération :

1. **Estimation** — chaque part d'un groupe (39-12a4a1 AC-3 : au moins deux parts inconnues d'une devise entre deux ancrages, somme exacte `Σ`) est estimée au cours : `conv = arrondi_demi_sup(couv / cours)`, `e = conv + max(0.01, arrondi_demi_sup(conv × t̂))`, `t̂` de l'AC-2 ; sa valeur non arrondie est `v = couv / cours + max(0.01, couv / cours × t̂)`. Une part n'a d'estimation propre que si sa couverture `couv` se calcule à partir de parts connues, déduites ou déjà résolues (à trois devises, l'USD vidé d'une partielle n'est estimable que si sa part EUR l'est, et inversement) ; les groupes se résolvent dans l'ordre où leurs parts deviennent estimables, jusqu'à ce que plus rien ne change (au plus un tour par groupe).
2. **Écart** — `k = (Σ − Σ e) / 0.01` (entier). Si `|k| ≤ n` (n = nombre de parts du groupe), les `|k|` parts au plus fort reste d'arrondi (`v − e` le plus grand si `k > 0`, le plus petit si `k < 0` ; à égalité, la plus ancienne dans l'ordre de 39-12a2 AC-5) bougent de 0.01 dans le sens de `k` ; chaque part bouge d'au plus 0.01 ; le groupe est alors importé.
3. **« À vérifier », pré-rempli, jamais importé** — `|k| > n` ; aucun ancrage après la part (segment non refermé) ; une part du groupe sans estimation propre. Un trou de solde reste un refus `BalanceGap` (39-12a4a1 AC-6).
4. **Marque** — une part ajustée est marquée comme telle au compte rendu (champ `adjusted: bool` de la part, ou motif), pour que l'utilisateur sache qu'elle peut différer d'un centime de la répartition de Wise ; le solde aux ancrages reste exact. Le CSV « avec frais séparés » importé ensuite la signale (`WISE_FEE_ORIGIN_MISMATCH`, 39-12a3b).

Une part **seule** inconnue de son segment reste déduite par la continuité et confirmée par l'AC-4 (tolérance T) : elle n'est pas une estimation.

**Rejeu sur les données de la recherche** (métadonnées seulement ; détail au fichier de recherche, § « Règle du centime du 22.11 », hors dépôt) : téléchargement mensuel 10 importées sur 10, aucune ajustée ; lectures plus longues : beaucoup de parts « à vérifier » faute d'estimation propre (partielle à trois devises dans un groupe), et, en trimestriel, deux partielles importées différentes d'un centime du relevé (dont une ajustée) — le risque que la décision accepte, à mesurer de nouveau à la reprise.

## Tâches

- [ ] T1 — Cours en entrée, `MissingRate`, `rates_needed` (AC-1).
- [ ] T2 — Étalonnage (AC-2).
- [ ] T3 — Formules, tolérance, confirmation (AC-3, AC-4) ; résolution d'un groupe, bande de p, atteignabilité (AC-4b).
- [ ] T4 — Exactitude, estimation des groupes, soldes estimés (AC-5).
- [ ] T5 — Tests (AC-6) ; recette locale (AC-7).

## Stratégie de tests

Gate ciblé `binary(wise_api)` et tests de `kesh-import` ; `cargo publish --dry-run -p kesh-import` ; gate complet au
dernier commit. Aucune migration.

## Modules touchés (1)

`kesh-import` seul (`split` n'est construit à `Some` que par 39-12a4b ; `MissingRate` ne demande aucun raccord dans `kesh-api` : `WiseApiError` est `#[non_exhaustive]` et traduit par `code()` avec un bras générique, 39-12a2 AC-6 et 39-12a3 AC-6).

## Fichiers communs

`crates/kesh-import/src/wise_api.rs` et ses modules (39-12a2, 39-12a4a1), `tests/fixtures/README.md` : à relire à la
reprise de `main`.

## Note de version

*Ajouté* — Un paiement par carte que Wise a payé avec plusieurs soldes s'importe par l'API quand Kesh en établit
exactement la part de chaque solde (continuité des soldes confirmée par le cours de Wise, ou résolution au centime d'un groupe de parts par la somme exacte et la règle d'arrondi de Wise) ; sinon, il est proposé à
vérifier avec ses parts estimées, jamais importé d'office ; un trou dans les soldes fait refuser le téléchargement.

## Notes de développement

- Arithmétique en `Decimal` ; montants arrondis au centime demi supérieur (`RoundingStrategy::MidpointAwayFromZero`,
  montants positifs), tolérances au centime supérieur.
- Les constantes de l'AC-2 sont des **mesures** (relevés 2025–2026 d'un compte), non des règles de Wise : une totale du
  couple dans la lecture les remplace. Si Wise change ses frais, la confirmation échoue (`rate_mismatch`) : rien de faux
  n'est écrit ; la résolution d'un groupe (AC-4b) élargit la bande de `p` jusqu'à la première solution et s'arrête au plafond (« à vérifier »).
- Le Change Log ne reporte de la recette AC-7 que des métadonnées.

## Change Log

- 2026-11-22 — **décision du 22.11** (règle du centime) écrite en section propre, AC non encore alignés ; **fiche en attente** (téléchargement par l'API suspendu jusqu'à la réponse du support Wise), validation P3 non remédiée. Statut `deferred`.

- 2026-11-22 — **remédiation de la validation P2** (rapport `kesh-gate-logs/s39-12a-specs/validation-p2-rapport.md`) et **décision C du responsable de l'epic** (recherche « option C » : règle du dernier centime) : « Décision à prendre » devient « Décision C du 22.11 », AC-4b neuf (résolution d'un groupe par somme exacte, atteignabilité et bande de p élargie jusqu'à la première solution, solution unique importée, sinon « à vérifier » pré-rempli), rendement « lecture d'un coup » recalculé (10 importées sur ces relevés, limite : bande réglée sur les mêmes données, mesure sur nouvelles données à la recette) ; **M5** — mécanisme du centime faux décrit au vrai (R1 échoue, `FEE-CARD-x` insérée, frais doublés), refus `WISE_FEE_ORIGIN_MISMATCH` en 39-12a3b ; **M4** — `rien à raccorder` vrai par `non_exhaustive` et `code()` (39-12a2 AC-6) ; **L6**, **L7**, **L8**, **L12**. Statut `drafted`, validation P3 à lancer.

- 2026-11-21 — fiche née du découpage de 39-12a4a à la remédiation de la validation P1 (Claude Sonnet 5.5, rapport
  `kesh-gate-logs/s39-12a-specs/validation-p1-rapport.md`) : **M3** — tolérance qui croît avec le montant et
  l'incertitude des frais (`T`), arrondi des taux fixé, cas à plus de 150 unités ; **M4** — formule fermée à trois devises,
  l'USD non confirmé à part, tolérance de l'EUR élargie ; **M5** — plus de borne de vraisemblance de 1 % : même
  étalonnage à deux et à trois devises, constantes mesurées faute de totale ; **M6** — solde estimé négatif toléré dans
  la somme des tolérances ; **M7** — part nulle ou négative : refus (39-12a4a1) ; **M9** — `MissingRate` ; **M10** —
  décision ouverte écrite avec son rendement ; **L10** — « cinq fois sur onze » ventilé ; **L12** — `rates_needed` ;
  **L13** — recette en métadonnées ; **L14** — `rate ≤ 0`. Vérifié sur les relevés réels (hors dépôt) : les 10
  partielles passent la confirmation recalibrée. Statut `drafted`, validation P2 à lancer.
