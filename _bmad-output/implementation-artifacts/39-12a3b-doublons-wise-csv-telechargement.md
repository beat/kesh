# Story 39-12a3b : doublons Wise entre un CSV et un téléchargement — frais de conversion inclus, montant différent, conversion automatique, paiement par carte converti

Status: deferred

**En attente (22.11) : téléchargement par l'API suspendu jusqu'à la réponse du support Wise ; validation P3 non remédiée (rapport `kesh-gate-logs/s39-12a-specs/validation-p3-rapport.md`).**

*Fiche née le 2026-11-21 du découpage de 39-12a3 (amendement du 21.11, A-2 : règle des petites stories). Son AC-1 est
l'AC-4 de 39-12a3, **reprise telle quelle** (validée en P1 à P5 et aux passes ciblées de la voie activités, le
2026-11-20) ; l'AC-2 est neuve (paiement par carte converti, formes relevées le 2026-11-21 sur les relevés réels :
`kesh-gate-logs/epic-39-40-kiss/wise-conversion-partielle-recherche.md`, § « Mesure contre les relevés réels »).
Branche `epic/39-wise`. Epic 39, **phase 2**. **Dépend de** : 39-12a3 (téléchargement), 39-12a2 (amendement A-1 :
totale par carte, frais inclus dans le débit), 39-11b (import CSV), **39-11a2** (clé (identifiant, sens) à l'import CSV,
et la lecture `existing_transactions_by_ids` que cette story réutilise ; branche `story/39-11a2`), **39-14** (raccord de
39-11b au cœur commun de 38-12a : le point d'accroche `rules` se pose dans ce cœur), 39-11c (rapprochement, écriture
groupée d'un paiement par carte converti). **Suivie par** : 39-12a4b, 39-12b.*

## Story

En tant que **Comptable**, je veux mélanger dans les deux ordres un relevé CSV de Wise et un téléchargement par l'API
sans qu'un mouvement ni ses frais comptent deux fois, y compris pour les conversions de solde, les conversions
automatiques et les paiements par carte payés dans une autre devise.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-12a3b-1` | CSV et téléchargement se mélangent dans les deux ordres sans doublon, frais de conversion compris (R1 à R3). | AC-1 |
| `FR-39-12a3b-2` | Les mêmes règles valent pour un paiement par carte converti (`CARD-`, `FEE-CARD-`). | AC-2 |
| `NFR-39-12a3b-1` | PayPal et Stripe inchangés, octet pour octet. | AC-1, AC-3 |

## Critères d'acceptation

1. **AC-1 — Doublons avec le CSV, dans les deux ordres** (AC-4 de 39-12a3 avant l'amendement du 21.11, reprise, sa règle de base alignée sur la clé (identifiant, sens) ; « cette story » y désigne aussi le téléchargement de 39-12a3, qui passe désormais `WiseDuplicateRules`). Règle de base (39-11b AC-5, clé (identifiant, sens) de 39-11a2, au lecteur comme à l'import, lue par `existing_transactions_by_ids` (identifiant, montant, date) : même `transaction_id` **et même sens** sur le même compte bancaire → ligne ignorée ; un montant à 0.00 vaut pour les deux sens).
   S'y ajoutent trois règles **propres à Wise**, qui ne s'écrivent **pas** dans le chemin
   commun tel quel : le cœur `provider_imports.rs` reçoit un paramètre `rules: Option<&dyn DuplicateRules>` (rien par défaut),
   appelé **sous les verrous et dans la transaction d'écriture**, après le test de la clé (identifiant, sens) et **avant** le calcul de la période et de la clôture de la devise et le cas « aucune ligne gardée » (qui jugent les lignes restantes) ; **PayPal (38-11b, 38-12a) et
   Stripe (40-7b) passent `None` et se comportent comme avant, octet pour octet** — un doublon de montant différent y reste
   ignoré, test de non-régression sur une entrée PayPal ; Wise (la route CSV de 39-11b, `wise_imports::create`, **et** cette
   story) passe `WiseDuplicateRules` (module neuf `crates/kesh-api/src/wise_duplicates.rs`, préfixes d'identifiant Wise
   seuls). **Contrat du point d'accroche** (sans dépendance neuve) : le paramètre est un champ de la demande au cœur
   (`ImportRequest` de 38-12a), et chaque prestataire qui la construit le pose — **PayPal (`paypal_imports.rs`,
   `paypal_downloads.rs`) et Stripe (`stripe_imports.rs`) sont donc touchés d'une ligne**, `None` ; le trait est
   **objet-sûr sans `async_trait`**, sa méthode rendant une future en boîte (patron de `mail::MailFuture`,
   `crates/kesh-api/src/mail/mod.rs:12`, et de `exchange_rates/source.rs:23`), et reçoit, par devise : la connexion de la
   transaction d'écriture, le compte bancaire, la **source** (`ImportSource::File` ou `Download` : R2 ne vaut qu'au
   téléchargement), les lignes **gardées** et les lignes **écartées comme doublons** par le test de la clé (identifiant, sens) ; elle rend
   pour chaque ligne gardée « garder » ou « ignorer » (R1, R3), un refus (R2, la tâche échoue), un compte `fees_included`
   et des avertissements. Le cœur reporte ces deux derniers au compte rendu de la devise (`CurrencyReport`) en champs
   **omis quand ils sont vides** (`skip_serializing_if`) : la réponse de PayPal et de Stripe garde sa forme, octet pour
   octet. Une ligne ignorée par R1 ou R3 compte dans `duplicates` (elle n'est pas insérée) **et**, pour R1, dans
   `feesIncluded`. *(Relevé au code des worktrees de 38-12a et de 40-7b le 2026-11-20, P3 : le cœur de 40-7b **reprend** celui de 38-12a —
   `ProviderLine`, `ImportRequest`, `import_lines` — et n'y ajoute que `ProviderLine.value_date` (`None` pour Wise), plus des
   aides de formulaire ; il n'y a plus qu'une forme, au contraire de ce que la P2 avait relevé. `dedupe_lines` garde sa
   signature et ses tests : les lignes écartées pour le point d'accroche se déduisent des lignes gardées ; **le test de doublon du cœur** (`dedupe_groups(groups, existing: &HashSet<String>)`, identifiant seul à ce jour) **passe à la clé (identifiant, sens) par le raccord de 39-14**, signature à elle : cette story le lit tel que 39-14 le laisse, sans le réécrire. Les copies
   privées de PayPal que 40-7b dit « à replier » ne concernent pas Wise.)* **Lectures du dépôt des transactions bancaires** (amendé à la validation P1, finding M13 : aucune lecture en double) :
   cette story **réutilise** `existing_transactions_by_ids(conn, company, compte, ids) -> Vec<ExistingTransaction {
   transaction_id, amount, booking_date }>` qu'ajoute 39-11a2 (même nom, même forme : ne pas la recréer), et n'ajoute
   que `transactions_by_prefix_window(company, compte, préfixe, du, au)`, rendant la même `ExistingTransaction` ;
   39-12a4b étend cette structure de deux champs (`import_id`, `booked`) au lieu d'écrire une troisième lecture. *Point d'accroche*
   : dans cette branche, le dédoublonnage Wise est encore fait par `WiseWriter` (`routes/wise_imports.rs`) et le
   `dedupe_groups` du cœur est celui de PayPal (`provider_imports.rs`) ; le point d'accroche se pose dans le cœur que
   39-14 raccorde à Wise — sans 39-14 faite, cette story ne commence pas. Les tests de 39-11b restent verts sans changement
   d'assertion (un ré-import à l'identique ne remplit aucune condition ci-dessous).
   - **(R1) Frais de conversion inclus.** Une ligne de frais `FEE-BALANCE-x` ou `FEE-CONVERSION_ORDER-x` d'un CSV dont la
     ligne d'origine `x` est un doublon, et dont l'origine en base a pour montant `montant de x dans le fichier + montant
     des frais` (débit téléchargé, frais inclus, 39-12a2 AC-4), est **ignorée** et comptée dans `feesIncluded` — sans quoi
     les frais compteraient deux fois. « L'origine en base » est la ligne de même identifiant **et de même sens** que l'origine `x` du fichier (clé de 39-11a2 ; un montant à 0.00 vaut pour les deux sens), ou celle que (R3) a
     reconnue pour un `CONVERSION_ORDER-x` du CSV (une `AUTO_CONVERSION-y` téléchargée). Le compte est rendu par fichier à la réponse de l'import CSV sous la clé
     `feesIncluded`, **absente quand elle vaut 0** (les réponses de 39-11b gardent leur forme) ; l'écran d'import CSV
     (39-11d) l'affiche avec le libellé de 39-12b.
   - **(R2) Montant différent.** Hors (R1), un doublon de **téléchargement** dont le montant diffère de la ligne en base →
     la tâche échoue, `PROVIDER_RESPONSE_INVALID`, `details.reason = "amount_mismatch"` (`currency`, `id`) ; rien n'est
     écrit. **Exemption — CSV d'abord, puis téléchargement** : le débit téléchargé d'une conversion porte les frais, la
     ligne du CSV en base non ; si la ligne en base est un débit `BALANCE-x` ou `CONVERSION_ORDER-x`, que le compte porte sa
     ligne `FEE-BALANCE-x` ou `FEE-CONVERSION_ORDER-x` de montant `f` et que `montant téléchargé = montant en base + f`, le
     doublon est **ignoré** (sans cette exemption, le mélange CSV puis téléchargement des conversions échouerait à chaque
     fois, en contradiction avec cette AC) ; sans ligne de frais en base ou avec un autre écart, `amount_mismatch`. L'import
     CSV, lui, garde le comportement de 39-11b (doublon ignoré).
   - **(R3) Conversion automatique, faute de numéro commun** (39-12a2 AC-3 : l'identifiant `AUTO_CONVERSION-y` est **définitif** ;
     la recette 39-12a1 AC-0 (a)(2) n'a trouvé le numéro du CSV nulle part dans l'API) : une ligne `AUTO_CONVERSION-y` est un doublon si le compte porte une ligne
     `CONVERSION_ORDER-n` de date à un jour près et de même montant (crédit), ou dont le montant plus celui de
     `FEE-CONVERSION_ORDER-n` égale le sien (débit) ; symétriquement, une ligne CSV `CONVERSION_ORDER-n` (et sa ligne de
     frais, par R1) est un doublon d'une ligne `AUTO_CONVERSION-y` en base qui remplit la même condition. Plusieurs
     candidates → aucune n'est retenue et la ligne est importée, avec l'avertissement `conversion_ambiguous` au compte
     rendu, **qui nomme les identifiants des candidates** (jamais un rattachement deviné : une conversion peut alors figurer
     deux fois, visible et à rapprocher à la main). La recherche se fait par `transactions_by_prefix_window`.
   - **Rapprochement** (39-11c) : le préfixe `AUTO_CONVERSION` s'ajoute aux préfixes de conversion reconnus (même
     identifiant sur deux soldes du même compte Wise) ; l'écart `fee_without_origin` ne vise que les lignes `FEE-x`, une
     conversion téléchargée sans ligne de frais n'en est donc pas un (aucun écart « origine sans frais » n'existe).
2. **AC-2 — Paiement par carte converti** (neuve). Le relevé CSV « avec frais indiqués séparément » porte un paiement
   par carte converti — total ou partiel — comme une conversion de solde : dans le fichier de chaque solde débité dans une
   autre devise que celle du marchand, une ligne `CARD-x` **hors frais** (*Exchange From* = la devise du solde,
   *Exchange To* = celle du marchand) et, une milliseconde avant, une ligne `FEE-CARD-x` ; la part payée dans la devise du
   marchand, sans conversion, n'a pas de ligne de frais. Le téléchargement porte une ligne `CARD-x` par solde débité,
   **frais compris** (39-12a2, amendement A-1 ; 39-12a4a1 et 39-12a4a2 pour les partielles). Les règles de l'AC-1 s'étendent ainsi,
   sans autre changement :
   - **(R1)** une ligne `FEE-CARD-x` d'un CSV dont l'origine `CARD-x` **de la même devise** est en base avec le montant
     `montant de CARD-x dans le fichier + montant des frais` est ignorée et comptée dans `feesIncluded` ;
   - **(R2)**, exemption : un doublon téléchargé `CARD-x` dont le compte porte `CARD-x` et `FEE-CARD-x` de montants `m` et
     `f`, avec `montant téléchargé = m + f`, est ignoré ; un doublon téléchargé `CARD-x` de la devise du marchand (sans
     ligne de frais) suit la règle générale (même montant : ignoré ; sinon `amount_mismatch`) ;
   - la clé d'une ligne Wise à l'import est **(identifiant, sens)** (39-11a2) : un `CARD-x` débité puis recrédité
     (réserve expirée, vérification de carte) est deux lignes en base ; R1 et R2 ne comparent qu'aux lignes **de débit**
     de même identifiant, la ligne de crédit restant soumise à la règle de base (même identifiant et même sens : ignorée).
   - **Échec muet de (R1), à fermer** (validation P2, M5) : si l'origine `CARD-x` est en base mais que son montant n'est pas
     `montant de CARD-x dans le fichier + montant des frais` (un téléchargement dont la part estimée est fausse d'un centime,
     39-12a4a2), R1 ne s'applique pas : `FEE-CARD-x` (identifiant neuf) serait **insérée** alors que `CARD-x` est ignorée
     (règle de base, ou R2 qui ne vaut qu'au téléchargement), et les frais compteraient **deux fois**, sans aucun signal. Règle :
     une ligne de frais `FEE-CARD-x`, `FEE-BALANCE-x` ou `FEE-CONVERSION_ORDER-x` dont l'origine est en base **sans remplir la
     condition de R1** n'est jamais insérée en silence : l'import est **refusé** (aucune écriture, code `WISE_FEE_ORIGIN_MISMATCH`,
     `details` : identifiants, montants lu et attendu), ce que le manuel (39-12b) explique et dont la sortie est d'annuler
     l'import téléchargé ou de corriger le montant. Cas de test : origine à un centime de l'attendu → refus. À reprendre à
     la lumière de la décision C de 39-12a4a2 (règle du dernier centime) : si elle rend la part exacte, ce refus reste le filet.
   **R3** ne vise pas les cartes (identifiant commun aux deux sources). Le rapprochement (39-11c, amendement A-1) groupe les
   lignes `CARD-x` et `FEE-CARD-x` en attente d'un même paiement : qu'elles viennent du CSV, du téléchargement ou des deux,
   l'écriture groupée garde la charge au montant du marchand (somme des lignes dans sa devise et des montants des segments
   de conversion) ; les frais d'un téléchargement, inclus dans la ligne de chaque solde, ne changent que la différence de
   change.
3. **AC-3 — Tests** (`crates/kesh-api/tests/wise_download_e2e.rs` pour le téléchargement, et tests de la route CSV de
   39-11b ; activités et CSV **fictifs**) : les cas de l'AC-1, repris de l'AC-7 de 39-12a3 — **téléchargement puis CSV** →
   rien de doublé, `FEE-BALANCE-…` comptée dans `feesIncluded`, solde du compte égal dans les deux ordres ; conversion
   automatique dans les deux ordres (R3), et deux candidates → `conversion_ambiguous` ; **CSV d'abord puis téléchargement
   d'une conversion à frais** → doublon ignoré par l'exemption de R2, non `amount_mismatch` ; **non-régression** : le cœur
   appelé avec `None` (PayPal) ignore un doublon de montant différent comme avant ; doublon de montant différent →
   `amount_mismatch`, rien d'écrit ; tests de 39-11b et 39-11c verts sans changement d'assertion (la clé `feesIncluded`
   n'apparaît pas dans leurs réponses). Neufs (AC-2) : une totale par carte, puis une partielle à deux devises, dans les
   deux ordres (CSV puis téléchargement, téléchargement puis CSV) → mêmes transactions en attente au même solde de
   compte, `FEE-CARD-…` comptée dans `feesIncluded` ; un `CARD-x` débité puis recrédité dans le CSV, puis le
   téléchargement → aucune `amount_mismatch` ; un `CARD-x` téléchargé de montant différent sans ligne de frais en base →
   `amount_mismatch`.

## Tâches

- [ ] T1 — Point d'accroche `rules` du cœur commun ; appelants PayPal et Stripe à `None` (AC-1).
- [ ] T2 — `wise_duplicates.rs` : R1 à R3, préfixes `BALANCE`, `CONVERSION_ORDER`, `AUTO_CONVERSION`, `CARD` (AC-1, AC-2).
- [ ] T3 — Lecture `transactions_by_prefix_window` (celle par identifiants vient de 39-11a2) ; `feesIncluded` aux deux routes (AC-1).
- [ ] T4 — Préfixe `AUTO_CONVERSION` au rapprochement (AC-1).
- [ ] T5 — Tests (AC-3).

## Stratégie de tests

Gate ciblé : `binary(wise_download_e2e)`, les binaires de 39-11b et 39-11c, ceux de PayPal et de Stripe qui appellent le
cœur ; **gate complet** à chaque patch qui touche un dépôt de `kesh-db` (règle d'exception de `CLAUDE.md`) et au dernier
commit. Aucune migration : P5 à P8 sans objet.

## Modules touchés (3)

`kesh-api` (`routes/provider_imports.rs`, `wise_duplicates.rs`, routes CSV et téléchargement de Wise, une ligne aux
appelants PayPal et Stripe), `kesh-db` (une lecture du dépôt des transactions bancaires ; l'autre est celle de 39-11a2),
`kesh-reconciliation` (un préfixe de conversion).

## Fichiers communs

`routes/provider_imports.rs` (cœur commun : un paramètre `rules` facultatif), `routes/paypal_imports.rs`,
`routes/paypal_downloads.rs` (38-11b, 38-12a), `routes/stripe_imports.rs` (40-7b) : une ligne chacun, `None` ;
`routes/wise_imports.rs` (39-11b, 39-11a2) ; dépôt des transactions bancaires : à relire à la reprise de `main`.

## Note de version

*Ajouté* — Un relevé CSV de Wise et un téléchargement par l'API se mélangent dans les deux ordres sans doublon, frais de
conversion compris, y compris pour les paiements par carte payés dans une autre devise.

## Notes de développement

- Les règles R1 à R3 ne touchent que des identifiants Wise et vivent côté Wise ; le cœur commun ne gagne qu'un point
  d'accroche, que PayPal et Stripe laissent à `None`.
- À vérifier à l'étape 0, au code de 39-11a2 (`existing_keys`) : la ligne de **capture** d'un `CARD-x` (second débit
  du CSV après une réserve expirée et recréditée) a la même clé (identifiant, sens débit) que le premier débit : elle est
  elle-même un doublon du premier à l'import CSV ; R1 et R2 la comparent comme toute ligne de débit.
- Le CSV « sans frais indiqués séparément » porte les frais dans la ligne d'origine (même montant que le téléchargement) :
  il se dédoublonne par la règle de base, sans R1 ni R2. Le manuel (39-12b) recommande le CSV « avec ».

## Change Log

- 2026-11-22 — **remédiation de la validation P2** (rapport `kesh-gate-logs/s39-12a-specs/validation-p2-rapport.md`) : **M1** — la clé (identifiant, sens) propagée aux résidus (AC-1 : reprise, test de la clé, signature du test de doublon renvoyée à 39-14) ; **M5** — l'échec muet de R1 sur une origine connue devient un refus `WISE_FEE_ORIGIN_MISMATCH` (AC-2) ; **L1** — deux champs ajoutés par 39-12a4b. Statut `drafted`, validation P3 à lancer.

- 2026-11-22 — alignement sur 39-11a2, terminée (branche `story/39-11a2`) : règle de base et origine d'un `FEE-x` (R1) lues par la clé (identifiant, sens), montant à 0.00 pour les deux sens ; `existing_transactions_by_ids` réutilisée (M13). Statut `drafted`, validation P2 à lancer.

- 2026-11-21 — **remédiation de la validation P1 des fiches 39-12a** (Claude Sonnet 5.5, contexte frais ; rapport
  `kesh-gate-logs/s39-12a-specs/validation-p1-rapport.md` ; ici 0 C, 0 H, 1 M, 2 L). **M13** — dépend de 39-14 (raccord
  du cœur commun, où se pose le point d'accroche) et de 39-11a2, dont elle réutilise `existing_transactions_by_ids` (une
  seule lecture neuve, `transactions_by_prefix_window` ; 39-12a4b étend la structure au lieu d'une troisième lecture) ;
  **L8** — ligne de capture d'un `CARD-x` à vérifier contre `existing_keys` de 39-11a2. Statut `drafted`, validation P2
  à lancer.

- 2026-11-21 — fiche née du découpage de 39-12a3 (amendement du 21.11) : AC-1 = AC-4 de 39-12a3 reprise telle quelle ;
  AC-2 neuve (`CARD-`/`FEE-CARD-`, clé (identifiant, sens) de 39-11a2) ; tests de l'AC-7 de 39-12a3 qui en relevaient
  déplacés ici. Statut `drafted`, validation P1 à lancer.
