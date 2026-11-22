# Story 39-14 : l'import Wise branché sur le cœur commun des prestataires

Status: drafted

*Fiche rédigée le 2026-11-22 au gabarit de `bmad-create-story`, branche `story/39-14` partie d'`epic/39-wise`
(`1f03f180`, qui porte la 39-11e et `epic/38-paypal` à `8b8cd338`, fusionnée par `aaaf7875`). Epic 39. **Dépend
de** : 39-11b (route `POST /api/v1/wise-imports`), 39-11e (zip), 38-12a (cœur extrait `routes/provider_imports.rs`),
toutes `done`. **Aucune migration**, aucune route neuve, **aucun comportement visible changé**. Liste « À raccorder »
de la fiche 39-11b (§ Dev Agent Record), points 1 à 10.*

## Story

En tant que **développeur de Kesh**, je veux que l'import Wise passe par le cœur commun des imports de prestataire
(`provider_imports::import_lines`) comme PayPal, afin qu'il n'existe qu'**un** cœur (verrous, relecture, empreinte,
doublons, entrée `bank_imports`, traces, rejeu) et que la copie de la 39-11b, restée dans `paypal_imports.rs`,
disparaisse.

## État de départ (relevé au code, `1f03f180`)

- PayPal (fichier et téléchargement) passe déjà par `provider_imports::import_lines` (38-12a).
- `routes/paypal_imports.rs` porte **en plus** le cœur de la 39-11b, que seul Wise appelle (l. 454-826) :
  `file_currency_hash`, `resolve_currency_balances`, `LockedTarget`, `WriteFuture`, `ProviderWrite`,
  `run_provider_import`, `import_under_locks`, `recheck_targets_locked`, `write_under_locks`, `CurrencyImport`,
  `insert_currency_import` — doublon de `provider_imports.rs`. Les bornes `DETAILS_MAX_CHARS`, `NAME_MAX_CHARS`,
  `PROVIDER_TYPE_MAX_CHARS`, `TXID_COLUMN_MAX_CHARS` et `truncate_chars`/`truncate_bytes` y sont `pub(crate)` pour
  Wise : elles restent (Wise s'en sert pour ses lignes).
- `routes/wise_imports.rs` : `create` appelle `resolve_currency_balances` puis `run_provider_import` avec un
  `WiseWriter` (doublons par (identifiant, sens), continuité, entrée par fichier, trace `wise_import.created` avec
  `files` et `ignoredEntries`).
- `provider_imports::AUDIT_ACTIONS` ne porte pas `wise_import.created` (point 6) : `import_lines` le refuserait en
  500. `audit_labels::ACTIONS` et les quatre `.ftl` le portent déjà (39-11b) ; **`SITES_INDIRECTS`
  d'`audit_label_registry.rs` non** (son entrée `provider_imports.rs` ne liste que les deux codes PayPal) : une fois le
  littéral retiré de `wise_imports.rs`, trois tests de ce registre rougiraient (validation P1, H1).

## Ce que le cœur ne sait pas faire, et que Wise exige

1. **Un fichier par devise** : nom (`bank_imports.filename`) et octets (empreinte `SHA-256(octets ‖ ":" ‖ devise)`)
   propres à chaque devise ; `ImportRequest` n'a qu'un `filename` et une `ImportSource::File` pour toutes.
2. **Période, ouverture et clôture du fichier entier** à l'entrée `bank_imports` (point 8 : la continuité de la 39-11c
   relit `period_from` et `opening_balance` stockés) ; le cœur prend la période des lignes **gardées**, `opening_balance:
   None` et la clôture de `balance_after`.
3. **IBAN de la contrepartie** (`ProviderLine` n'en a pas ; `into_new_bank_transaction` pose `None`).
4. **Doublons par (identifiant, sens)** (39-11a2) : le cœur juge par identifiant seul (un recrédit au même
   identifiant qu'un débit déjà en base serait ignoré à tort) — absent de la liste « À raccorder », relevé ici.
5. **Avertissements de continuité** par devise (`bank_imports::continuity_gap`, sous les verrous, avant l'insertion).
6. **Ordre du formulaire** : le cœur parcourt les devises dans l'ordre de `BTreeMap` (alphabétique) ; Wise juge les
   refus `PROVIDER_CURRENCY_DUPLICATE`, `BANK_ACCOUNT_CLOSED`, `BANK_IMPORT_DUPLICATE_FILE`, crée ses entrées (donc
   leurs identifiants) et rend `files` **dans l'ordre du formulaire** (zips développés à leur place).
7. **La trace** `wise_import.created` porte `files` (les comptes rendus de fichier, tels que la réponse) et
   `ignoredEntries`, sans `currencies` (point 7).

## Décisions

- **D1 — Une variante de source, non un champ de plus** : `ImportSource::Statements(&[CurrencyStatement])`, un relevé
  par devise, dans l'ordre de l'appelant. `CurrencyStatement { currency, filename, bytes, period_from, period_to,
  opening_balance, closing_balance, trace }`. Pour cette source, le cœur : parcourt les devises dans l'ordre des
  relevés (refus et écritures) ; prend nom, empreinte (des octets du relevé, même formule que `File`), période et
  soldes du relevé (même pour un relevé à doublons partiels) ; refuse l'empreinte déjà importée comme `File` ; juge la
  continuité et rend `warnings` ; écrit à la trace de la source `files` (pour chaque devise, `trace` du relevé complété
  du compte rendu de la devise) au lieu de `currencies`. `ImportRequest` ne change pas de forme (pas de champ neuf :
  les constructions de PayPal et de Stripe, epic/40, restent valides) ; son `filename` est ignoré pour cette source.
  Les devises de `by_currency` et celles des relevés doivent coïncider, une fois chacune : sinon `AppError::Internal`
  (faute de Kesh), testé. **Ordre dans l'écriture, par devise** (celui de l'ancien `WiseWriter`) : lecture des
  transactions existantes, doublons, **continuité jugée pour tout relevé, avant de savoir s'il reste une ligne** (un
  relevé tout en doublons rend aussi ses avertissements), puis l'entrée s'il reste une ligne.
- **D2 — `ProviderLine` gagne deux champs** : `counterparty_iban: Option<String>` et `direction:
  Option<LineDirection>` (`Debit`/`Credit`). `None` (PayPal, Stripe) : doublon par identifiant, comme aujourd'hui.
  `Some` : doublon par (identifiant, sens) — une transaction en base vaut débit si son montant est ≤ 0, crédit s'il est
  ≥ 0 (zéro : les deux), règle de la 39-11a2 déplacée telle quelle ; dans la source, l'unité du doublon est
  (`group_id`, sens). Coût assumé : les littéraux `ProviderLine { … }` d'epic/40 (Stripe) ne compileront plus à sa
  fusion — correction mécanique, `counterparty_iban: None, direction: None` (à noter au registre des choix).
- **D3 — Doublons** : `dedupe_groups(groups, &ExistingKeys)` ; `ExistingKeys::of(&[ExistingTransaction])`, lue par
  `bank_transactions::existing_transactions_by_ids` (même filtre SQL que `existing_transaction_ids`, montant en plus)
  pour tous les prestataires. Une ligne **sans** sens est présente si une transaction de même identifiant existe, quel
  que soit son montant (ensemble des identifiants, distinct des deux ensembles par sens) : doublons inchangés pour
  PayPal. Les deux tests de `paypal_imports.rs` et celui de `provider_imports.rs` qui appellent `dedupe_groups` passent
  à `ExistingKeys::from_ids` (tests seuls). `existing_transaction_ids` (`kesh-db`) perd son dernier appelant de
  `kesh-api` ; il reste (fonction publique du dépôt, lue par d'autres branches d'epic), noté pour la story de solde.
- **D4 — `CurrencyReport.warnings: Option<Vec<ImportWarning>>`**, omis quand `None` : la réponse PayPal reste octet
  pour octet ; `Some` (éventuellement vide) pour `Statements`. `ImportWarning` = l'ancien `FileWarning` de Wise
  (`code`, `expected`, `found`, `previousImportId`), déplacé au cœur avec `WARNING_GAP` ; `wise_imports` les
  ré-exporte sous leurs noms.
- **D5 — Wise construit ses lignes et son compte rendu** : `line_of_row(row) -> ProviderLine` (un groupe = une ligne,
  `group_id = transaction_id = id`, `direction` du lecteur, `rank` = rang de la ligne, `balance_after: None` — les
  soldes du relevé priment) ; `transaction_of_row` devient `line_of_row(row).into_new_bank_transaction(…)` (API et
  tests inchangés). `FileReport` = la partie fichier (nom, archive, lignes lues, soldes) + le `CurrencyReport` de la
  devise ; la `trace` du relevé porte ces mêmes champs fichier, si bien que `files` de la trace égale `files` de la
  réponse.
- **D6 — Point 2e (point d'accroche `rules` de la 39-12a3) hors périmètre** : la 39-12a3 est sortie de l'epic
  (branche `futur/wise-api`) ; rien n'est écrit qui l'empêche.
- **D7 — Textes du journal serveur** : ceux du cœur 38-12a (« import : libération des verrous en échec », « import :
  verrous des comptes non acquis », « import : trop de transactions ») remplacent pour Wise ceux de la copie
  39-11b (« import de prestataire : … », « import wise : trop de transactions »). Journal seul : ni réponse, ni trace.

## Critères d'acceptation

- **AC-1 — Un seul cœur** : `paypal_imports.rs` ne porte plus aucune des onze fonctions et types du cœur 39-11b ;
  `wise_imports.rs` appelle `provider_imports::load_balances` et `import_lines` ; `grep -rn
  "run_provider_import\|insert_currency_import\|ProviderWrite\|LockedTarget\|resolve_currency_balances\|file_currency_hash"
  crates/` ne rend que des mentions historiques datées (fiches, commentaires « avant la 39-14 »).
- **AC-2 — Réponses inchangées** : pour Wise, mêmes statuts, codes, corps (`files` dans l'ordre du formulaire,
  `rows`, `inserted`, `duplicates`, `importId`, `openingBalance`, `closingBalance`, `warnings` toujours présent,
  `archive`, `ignoredEntries`) et même ordre des refus (en-tête du module `wise_imports`) ; pour PayPal, fichier et
  téléchargement, corps octet pour octet (pas de clé `warnings`). Tenu par les suites existantes
  (`wise_import_e2e`, `paypal_import_e2e`, `paypal_download*`) vertes **sans modification de leurs assertions**, et
  par un test neuf (AC-5).
- **AC-3 — Écritures inchangées** : par fichier Wise, une entrée `bank_imports` (nom du fichier, empreinte octets ‖
  devise, période et soldes du **fichier entier**, `CSV`) s'il reste une ligne ; transactions avec IBAN de la
  contrepartie ; doublons par (identifiant, sens) ; traces `bank_import.created` (mêmes `details`) et
  `wise_import.created` (`provider`, `provider_account`, `files`, `ignoredEntries`, sans `currencies`) ; identifiants
  des entrées croissants dans l'ordre du formulaire ; rejeu sur interblocage sous l'opération `wise_imports::create`.
- **AC-4 — Registre des actions** : `AUDIT_ACTIONS` porte `wise_import.created`, et l'entrée `provider_imports.rs` de
  `SITES_INDIRECTS` (`tests/audit_label_registry.rs`) aussi ; verts : `action_de_trace_inscrite_ou_refusee` (cœur),
  `les_actions_de_la_liste_sont_exactement_celles_que_le_code_ecrit`,
  `les_actions_du_coeur_d_import_sont_inventoriees_et_libellees`,
  `les_codes_de_l_inventaire_sont_verifies_dans_le_fichier_de_leur_site`.
- **AC-5 — Test de non-régression neuf** (`wise_import_e2e`) : deux relevés envoyés dans l'ordre USD puis CHF, le
  second à doublons partiels et à écart de continuité : corps entier comparé à une valeur attendue écrite en dur,
  `files` de la trace égal à `files` de la réponse, lignes `bank_imports` (nom, période, ouverture, clôture) du fichier
  entier, identifiant de l'entrée USD inférieur à celui de CHF ; et deux comptes clos, envoyés USD puis EUR : le refus
  nomme le compte USD ; un relevé CHF **tout en doublons** à écart de continuité : `importId` nul, avertissement rendu
  et tracé, aucune entrée. **Vert avant le raccord** (joué sur `1f03f180`) **et après** ; sondes par mutation hors dépôt
  (ordre alphabétique rétabli, période des lignes gardées, `direction` ignorée, `warnings` omis) tuées par une
  assertion nommée.
- **AC-6 — Doc** : en-têtes de `wise_imports.rs` et `provider_imports.rs` à jour ; `audit_route_registry.rs` point
  (vi) : l'enveloppe de rejeu de Wise vit dans `provider_imports::import_lines` (comptes de routes inchangés — aucune
  route ajoutée ; recompte déclaré) ; `docs/api-external.md` inchangé (aucun contrat ne bouge).

## Liste « À raccorder » de la 39-11b

| Point | Sort |
|---|---|
| 1 | T3 (cœur 39-11b retiré de `paypal_imports.rs` ; PayPal y était déjà branché à la fusion `aaaf7875` et avant) |
| 2a–2d | D1, D2, D4 |
| 2e | écarté, D6 (39-12a3 sortie de l'epic) |
| 3 | D5, T2 |
| 4 | AC-6 (commentaire du point (vi)) ; comptes de routes et phrase du manuel **déjà recomptés** à la fusion d'`epic/38` (gate vert de la 39-11e à `ac3e1fe4`) — rien à recompter, aucune route ajoutée |
| 5, 9 | résolus aux fusions précédentes d'`epic/38` dans `epic/39` (blocs i18n et conflits de texte présents des deux côtés) ; rien ici |
| 6 | AC-4 |
| 7 | D1, D5 : `files` gardé, sans `currencies` |
| 8 | D1 (période et soldes du relevé entier), AC-5 |
| 10 | hors périmètre (Epic 40 : `served()`, et `json.paypal.or(json.wise)`, `kesh-db/src/entities/bank_account.rs:215`) : **à tracer en issue** par l'orchestrateur (cette story n'a pas le droit d'ouvrir d'issue) |

## Tâches

1. T1 — Cœur : `LineDirection`, deux champs de `ProviderLine`, `ExistingKeys`, `CurrencyStatement`,
   `ImportSource::Statements`, ordre des cibles, empreinte, période et soldes, continuité, `warnings`, trace `files` ;
   `wise_import.created` à `AUDIT_ACTIONS` ; tests unitaires (empreinte d'un relevé = celle de `File`, doublons par
   sens, zéro des deux sens).
2. T2 — Wise : `line_of_row`, `create` par `load_balances` + `import_lines`, `FileReport` depuis `CurrencyReport` ;
   retrait de `WiseWriter`, `Target`, `existing_keys` (son test passe au cœur).
3. T3 — PayPal : retrait du cœur 39-11b et des imports devenus inutiles ; littéraux `ProviderLine` complétés.
4. T4 — Test AC-5 écrit **d'abord**, joué vert sur le code d'avant.
5. T5 — Doc (AC-6), grep des symboles retirés, fiche.

## Stratégie de tests

Non-régression avant tout : suites e2e Wise et PayPal inchangées + test AC-5 joué des deux côtés du raccord ; tests
unitaires du cœur. Gates : fmt, clippy workspace, nextest complet ; aucun écran, aucune spec E2E neuve, donc aucun E2E
(règle de la nuit du 22.11) ; frontend non touché.

## Modules touchés (1)

`kesh-api` : `routes/provider_imports.rs`, `routes/wise_imports.rs`, `routes/paypal_imports.rs`,
`tests/wise_import_e2e.rs`, `tests/audit_label_registry.rs` (inventaire), `tests/audit_route_registry.rs`
(commentaire). E2E partiel : aucun — aucune spec neuve, aucun écran (règle de la nuit du 22.11) ; selon la règle du
22.11 00:30, les specs des écrans d'import Wise et PayPal auraient été retenues.

## Fichiers communs (39-11c en remédiation sur `story/39-11c`)

`tests/audit_route_registry.rs` (commentaire du point (vi) ; la 39-11c y ajoute des routes) et `sprint-status.yaml`
(chacun sa ligne). La 39-11c ne touche ni `wise_imports.rs`, ni `provider_imports.rs`, ni `paypal_imports.rs`.

## Note de version

Aucune ligne au CHANGELOG : réorganisation interne, sans effet visible (journal serveur seul, D7).

## Dev Agent Record

*(à remplir)*

## Change Log

- 2026-11-22 — Validation P1 (Claude Sonnet 5.5, contexte séparé, lecture seule, `kesh-gate-logs/s39-14/validation-p1-rapport.md`) :
  0 C, 1 H, 3 M, 7 L. H1 (`SITES_INDIRECTS`) : prémisse corrigée, AC-4 ; M1 : tableau des points 1 à 10 ; M2 : D3
  précisé ; M3 : ordre de la continuité en D1, cas « tout en doublons » ajouté à AC-5. LOW : `flatten` écarté (D5
  construit `FileReport` champ à champ, ordre des clés gardé), cas `Internal` testé, E2E dit ; les autres sans suite
  (clone des groupes, grep de l'AC-1 qui vise l'état final).
- 2026-11-22 — Fiche rédigée (raccord, liste « À raccorder » de la 39-11b, points 1 à 10 ; points 4 et 6 relevés au
  code ; doublon par sens ajouté).
