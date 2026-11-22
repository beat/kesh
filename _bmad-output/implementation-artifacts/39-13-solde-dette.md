# Story 39-13 : solde de la dette de l'Epic 39 (Wise)

Status: ready-for-dev

*Fiche rédigée le 2026-11-22 au gabarit de `bmad-create-story`, branche `story/39-13` partie d'`epic/39-wise`
(`0a414376f`). Epic 39. Story de solde (`CLAUDE.md` § *Priorités des défauts*, « une story de solde à la fin de chaque
epic ») : elle ne prend que la dette des stories de l'epic. **Aucune migration**, aucune route neuve, aucun texte
d'interface, **aucun comportement visible changé**.*

## Plafond

Le fichier d'epic (`_bmad-output/planning-artifacts/epic-39-wise.md`) **ne fixe pas** de nombre d'issues pour la story
de solde : le plafond est pris à **deux** points de dette, faute de mieux (décision de l'orchestrateur, à confirmer par
le responsable de l'epic). Choisis parmi la dette de l'Epic 39 — ligne de registre de la 39-13, entrées « (39-…) » et
« (intégration 39/40) » de la liste des issues à ouvrir, LOW des Dev Agent Records des 39-11x et de la 39-14 — selon
la consigne « les plus faciles et sans risque d'abord ».

## Story

En tant que **développeur de Kesh**, je veux solder deux dettes laissées par les stories Wise — un contrôle de devise
quadratique et deux fonctions de lecture des doublons dont les commentaires ne disent plus vrai, l'une dupliquant
l'autre —, afin que le code de l'epic parte en PR sans ces restes.

## État de départ (relevé au code, `0a414376f`)

1. **`check_single_currency`** (`crates/kesh-import/src/wise.rs:530-541`, Story 39-11a, LOW L2 de la revue P2) :
   pour chaque ligne, `Vec::contains` sur les devises déjà vues — quadratique en nombre de devises distinctes. Le refus
   `WiseCsvError::MixedCurrencies(Vec<String>)` porte les devises **dans l'ordre de leur première apparition** dans le
   fichier (doc de la variante, `wise.rs:298-301`), ordre relu par `routes/wise_imports.rs:358` (détail
   `currencies` du `422 WISE_CSV_INVALID`) et figé par `tests/wise_csv.rs:308-314` (deux devises seulement).
2. **`existing_transaction_ids` et `existing_transactions_by_ids`** (`crates/kesh-db/src/repositories/bank_transactions.rs:131-218`,
   relevé par la 39-14, § *Fichiers communs et points relevés*) :
   - `existing_transaction_ids` dit servir « la recherche de doublon de l'import PayPal » : depuis la 39-14, le cœur
     commun (`routes/provider_imports.rs:677`) lit `existing_transactions_by_ids` pour **tous** les prestataires ;
     `existing_transaction_ids` n'a plus d'appelant dans le dépôt (`git grep`), et aucun test ne l'exerce ;
   - `existing_transactions_by_ids` dit que « l'import Wise juge un doublon » et renvoie à 39-12a3 (sortie de l'epic) ;
   - les deux portent **la même requête** recopiée (paquets de 500, `QueryBuilder`, filtre société et compte) :
     duplication contraire à DRY.

## Décisions

- **D1 — Contrôle de devise linéaire, même erreur.** Comparer chaque ligne à la devise de la première ligne ; tant
  qu'elles sont égales, aucune collection. À la première différence, construire la liste des devises distinctes dans
  l'ordre de première apparition par un `HashSet` des devises vues (un passage). Sortie **identique** à l'actuelle,
  pour tout fichier.
- **D2 — `existing_transaction_ids` écrite sur `existing_transactions_by_ids`.** Elle garde sa signature publique
  (`HashSet<String>` des identifiants tels que stockés) — une branche d'epic non fusionnée l'appelle encore — mais ne
  porte plus sa propre requête : elle projette les identifiants du résultat de `existing_transactions_by_ids`. Même
  filtre SQL, mêmes paquets, même collation ; deux colonnes lues en plus, sans effet.
- **D3 — Commentaires rendus vrais** : `existing_transactions_by_ids` nomme son appelant réel (le cœur commun des
  imports de prestataire, Story 39-14, doublons par (identifiant, sens) via `ExistingKeys`), sans renvoi à 39-12a3 ;
  `existing_transaction_ids` dit qu'elle n'a plus d'appelant dans `kesh-api` depuis la 39-14 et pourquoi elle reste
  (lecture par identifiant seul, sans sens).

## Critères d'acceptation

- **AC-1** — `check_single_currency` ne fait qu'un passage sur les lignes, sans `Vec::contains` ; un fichier à une
  devise ne construit aucune liste.
- **AC-2** — Un fichier à trois devises, dont une revient après une autre (p. ex. CHF, EUR, CHF, USD), est refusé par
  `MixedCurrencies(vec!["CHF", "EUR", "USD"])` : ordre de première apparition, sans doublon. Test neuf dans
  `crates/kesh-import/tests/wise_csv.rs`, qui rougit si l'ordre ou l'unicité se perdent (mutations).
- **AC-3** — `two_currencies_are_refused` reste vert sans modification.
- **AC-4** — `existing_transaction_ids` ne contient plus de requête SQL ; elle délègue à
  `existing_transactions_by_ids`. Test neuf `#[sqlx::test(migrations = "./test-schema")]` dans
  `crates/kesh-db/tests/bank_transactions_repository.rs` : identifiant présent au débit et au crédit rendu une fois,
  identifiant absent non rendu, autre compte et autre société exclus.
- **AC-5** — Les commentaires des deux fonctions décrivent leurs appelants réels (D3) ; aucun renvoi à 39-12a3 ni à
  « l'import PayPal » comme seul appelant.
- **AC-6** — Liste des issues à ouvrir (hors dépôt) : les deux entrées soldées barrées, avec le commit.

## Tâches

- [ ] T1 (AC-1, AC-2, AC-3) — `check_single_currency` réécrite (D1), test neuf d'ordre et d'unicité.
- [ ] T2 (AC-4, AC-5) — `existing_transaction_ids` sur `existing_transactions_by_ids` (D2), commentaires (D3), test
  neuf.
- [ ] T3 — Sondes par mutation hors dépôt, une à la fois, tuées sur assertion nommée.
- [ ] T4 — Gates : `cargo fmt --check`, clippy workspace `-D warnings`, nextest complet ; frontend non touché ; aucune
  spec E2E ajoutée (règle de la nuit du 22.11 : E2E limité aux specs neuves de la story).
- [ ] T5 (AC-6) — Barrer les entrées soldées ; registre `39-13-solde-dette: review` ; prompt de revue P1.

## Stratégie de tests

- Unitaires (`kesh-import`, sans base) : AC-2, AC-3 ; le test `wise_csv.rs` passe par `parse_wise_csv`, chemin public.
- Intégration (`kesh-db`, base éphémère) : AC-4.
- Mutations visées : (a) ne pas dédoublonner (pousser chaque devise différente de la première) ; (b) perdre l'ordre
  (trier) ; (c) `existing_transaction_ids` qui rend un ensemble vide ; (d) filtre de compte retiré (déjà gardé par le
  test d'`existing_transactions_by_ids`, rejoué par délégation).

## Modules touchés (2)

`kesh-import` (`src/wise.rs`, `tests/wise_csv.rs`), `kesh-db` (`src/repositories/bank_transactions.rs`,
`tests/bank_transactions_repository.rs`).

## Fichiers communs

Aucun fichier de la liste commune (CHANGELOG, PDF, catalogues i18n, compteurs, migrations). `sprint-status.yaml` :
ligne de la 39-13 seule. La revue d'intégration d'`epic/39-wise` en cours lit `routes/provider_imports.rs` et les
fusions : non touchés ici.

## Hors périmètre (dette de l'epic non prise, plafond atteint)

Restent à ouvrir en issues (liste hors dépôt) : 39-11a L2 et L8 (numéro de ligne après une ligne vide, ordre des 100
erreurs) ; 39-11b P3 (rognage du libellé) ; 39-11a2 (même identifiant deux fois dans le même sens) ; 39-11c P1 L5, L8,
L9, M1 ; 39-11e P1 L3 ; 39-14 (`json.paypal.or(json.wise)`) ; 39-11d2 (avertissement d'annulation d'un groupe) ; 39-11d
(garde des codes traduits), 39-11d P2 L1 et L2 ; intégration 39/40 (chaîne Stripe → Wise sans test).

## Note de version

Aucune ligne de CHANGELOG : rien de visible ne change (dette interne).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-22 — Fiche rédigée (plafond pris à deux, faute de nombre au fichier d'epic).
