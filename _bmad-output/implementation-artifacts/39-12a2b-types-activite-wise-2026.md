# Story 39-12a2b : activités Wise de 2026 — approvisionnement (`BALANCE_DEPOSIT`) et prélèvement (`DIRECT_DEBIT_TRANSACTION`) en lignes de relevé (`kesh-import`)

Status: deferred

**En attente (22.11) : téléchargement par l'API suspendu jusqu'à la réponse du support Wise ; validation P3 non remédiée (rapport `kesh-gate-logs/s39-12a-specs/validation-p3-rapport.md`).**

*Fiche née le 2026-11-21 de la validation P1 des fiches 39-12a (finding M1, question Q2 : « dans une story, non dans une
issue », parce que ces deux types arrêtent tout téléchargement qui remonte avant eux — tout l'exercice 2025 et le début
de 2026). Formes relevées le même jour sur les relevés CSV 2026 du site (`data-not-committed/2025/11-wise/`, « avec
frais indiqués séparément ») et les activités déjà sauvées (`data-not-committed/wise-recette-2026/`), métadonnées
seulement. Branche `epic/39-wise`. Epic 39, **phase 2**. **Dépend de** : 39-12a2 (amendée). **Suivie par** : 39-12a3 (qui
appelle la conversion), 39-12a4a1. Fonction pure, à côté de 39-12a2.*

## Story

En tant que **Comptable**, je veux que les approvisionnements et les prélèvements de mon compte Wise deviennent par
l'API les mêmes lignes que dans le relevé CSV de Wise, afin que leur seule présence n'empêche plus le téléchargement de
remonter avant eux.

## Ce que le relevé établit (2026, un cas de chaque, montants hors dépôt)

| Activité (`type`, `resource.type`) | Relevé CSV : identifiant, *Transaction Details Type* / *Transaction Type* | Concordance |
|---|---|---|
| `BALANCE_DEPOSIT` (`TRANSFER`), `primary` avec `+`, sans `secondary` | `TRANSFER-<resource.id>`, `MONEY_ADDED` / `CREDIT`, dans le fichier de la devise du `primary` ; *Payer Name*, *Payment Reference* vides ; *Total fees* 0.00 | montant = `primary` (1/1) ; date = jour de `created_on` et de `updated_on` (même jour, 1/1) |
| `DIRECT_DEBIT_TRANSACTION` (`DIRECT_DEBIT_TRANSACTION`), `primary` sans signe, sans `secondary` | `DIRECT_DEBIT-<resource.id>`, `DIRECT_DEBIT` / `DEBIT` ; *Merchant* = texte du premier `<strong>` du titre ; *Payment Reference* renseignée (absente de l'API) ; *Total fees* 0.00 | montant = `primary` (1/1) ; date = jour de `created_on` et de `updated_on` (même jour, 1/1) |

Un seul cas de chaque : la règle de date (débit à `created_on`, entrant à `updated_on`, celle de 39-12a2 AC-3) est
celle de leur famille, **non départagée** par les données ; un jour de décalage ne se verrait au contrôle de solde de 39-12a2 AC-5 que si un point de contrôle tombait entre `created_on` et `updated_on` (la tolérance « à un instant du jour » et l'ordre ne le signalent pas autrement).

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-12a2b-1` | `BALANCE_DEPOSIT` et `DIRECT_DEBIT_TRANSACTION` terminés deviennent une ligne chacun, à l'identifiant, au type de détail et au sens du relevé CSV. | AC-1, AC-2 |
| `FR-39-12a2b-2` | Une forme non relevée de ces deux types (conversion, sens contraire) reste refusée. | AC-3 |
| `NFR-39-12a2b-1` | Sans I/O, dans `kesh-import`, sans dépendance neuve ; fixtures fictives. | AC-4 |

## Critères d'acceptation

1. **AC-1 — Approvisionnement (`BALANCE_DEPOSIT`).** Activité `COMPLETED`, `primary` crédit (`+` ou `<positive>`), sans
   `secondary` → **une ligne** de crédit de `primary` dans sa devise, identifiant `TRANSFER-<resource_id>`,
   `details_type` `MONEY_ADDED` / `CREDIT`, `date` = jour de Zurich de `updated_on` (règle des entrants de 39-12a2
   AC-3), `date_time` au même instant ; `description` = `title` sans balises (comme toute ligne de 39-12a2 ; le CSV
   porte une description traduite, jamais comparée) ; `payer_name`, `payment_reference` = `None` (vides au CSV) ; aucune
   ligne de frais. Au rapprochement, le préfixe `TRANSFER` le classe comme un virement (39-11c : `ProviderKind::Specific(KIND_TRANSFER)`, `kesh-reconciliation/src/wise.rs`), comme la ligne du CSV de même préfixe : rien à changer à 39-11c.
2. **AC-2 — Prélèvement (`DIRECT_DEBIT_TRANSACTION`).** Activité `COMPLETED`, `primary` sans signe (débit), sans
   `secondary` → **une ligne** de débit de `primary`, identifiant `DIRECT_DEBIT-<resource_id>` (préfixe accepté par le
   contrôle de 39-11a : `[A-Z0-9_-]`), `details_type` `DIRECT_DEBIT` / `DEBIT`, `date` = jour de Zurich de `created_on`
   (règle des débits), `merchant` = texte du premier `<strong>` du titre, `payment_reference` = `None` — **écart de
   contenu assumé** avec le CSV, qui porte une référence que l'API n'expose pas (dit au manuel, 39-12b) ; aucune ligne
   de frais. Au rapprochement : genre `Other` (préfixe inconnu des genres de 39-11c), proposé par règle comme toute
   dépense.
3. **AC-3 — Formes non relevées : refus inchangé.** Un `BALANCE_DEPOSIT` au débit ou avec `secondary`, un
   `DIRECT_DEBIT_TRANSACTION` au crédit (remboursement) ou avec `secondary` (prélèvement converti) → `UnsupportedActivity
   { activity_type, date }` de 39-12a2, avec sa portée (même hors de `period`). Non `COMPLETED` → `ignored.not_completed`.
   La ligne « tout autre type » du tableau de 39-12a2 AC-3 ne cite plus `DIRECT_DEBIT_TRANSACTION`.
4. **AC-4 — Tests** (unitaires à côté de ceux de 39-12a2, intégration `crates/kesh-import/tests/wise_api.rs`) : une
   activité fictive de chaque type, comparée à une ligne de CSV fictive **de la forme relevée** (fixture
   `tests/fixtures/wise-releve-*-fictif.csv` de 39-11a complétée, ou fixture neuve, ligne au catalogue
   `tests/fixtures/README.md`) : même identifiant, date, montant, devise, sens, type de détail ; solde reconstruit juste ;
   chaque forme de l'AC-3 refusée, y compris hors de `period` ; le même prélèvement non `COMPLETED` compté. Tests de
   39-12a2 verts sans changement d'assertion (sauf le cas qui prenait `DIRECT_DEBIT_TRANSACTION` comme type refusé : il
   prend un type fictif).

## Tâches

- [ ] T1 — Deux lignes du tableau des types, refus des formes non relevées (AC-1 à AC-3).
- [ ] T2 — Fixtures fictives et tests (AC-4).

## Stratégie de tests

Gate ciblé `binary(wise_api)` et tests de `kesh-import` ; `cargo publish --dry-run -p kesh-import` ; gate complet au
dernier commit. Aucune migration.

## Modules touchés (1)

`kesh-import` seul.

## Fichiers communs

`crates/kesh-import/src/wise_api.rs` (39-12a2), `tests/fixtures/README.md` : à relire à la reprise de `main`.

## Note de version

*Ajouté* — Le téléchargement Wise reprend les approvisionnements et les prélèvements, qui l'empêchaient jusqu'ici de
remonter avant eux.

## Notes de développement

- Limite assumée : un seul cas de chaque type a été relevé ; un approvisionnement ou un prélèvement avec frais, converti
  ou remboursé n'est pas connu et reste refusé (fermé par construction). Les types `COMPLETED` rares non encore vus
  (intérêts, cashback, remboursement de frais, retrait) restent refusés : à suivre par une issue (on trace toujours).
- La référence d'un prélèvement manque à l'API : un CSV importé avant ou après n'en double pas la ligne (même
  identifiant), mais la ligne téléchargée n'a pas la référence du CSV.

## Change Log

- 2026-11-22 — **remédiation de la validation P2** (rapport `kesh-gate-logs/s39-12a-specs/validation-p2-rapport.md`) : **L3** — classement 39-11c du préfixe `TRANSFER` corrigé ; **L4** — portée du contrôle de solde ramenée au vrai. Statut `drafted`, validation P3 à lancer.

- 2026-11-21 — fiche née de la remédiation de la validation P1 des fiches 39-12a (Claude Sonnet 5.5, rapport
  `kesh-gate-logs/s39-12a-specs/validation-p1-rapport.md`, finding M1 et question Q2) ; préfixes et types de détail
  relevés sur les relevés CSV 2026 du site (`TRANSFER-` / `MONEY_ADDED`, `DIRECT_DEBIT-` / `DIRECT_DEBIT`). Statut
  `drafted`, validation P2 à lancer.
