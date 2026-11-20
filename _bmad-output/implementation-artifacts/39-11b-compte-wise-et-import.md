# Story 39-11b : réglages du compte Wise et import des relevés, ventilé par devise

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/39-wise`, numéros de ligne relevés au code de `origin/main` (`fd04099f`),
revérifiés à la validation P1. Epic 39, **phase 1 KISS** (G2, G3, G4, G10, G11). **Dépend de** : 39-11a (lecteur),
**38-11b** (colonnes `provider`, `provider_account`, `fee_account_id`, contrainte qui admet déjà `'wise'`, réglages
prestataire d'un compte bancaire, cœur de l'import ventilé ; fiche validée P1 lue dans le worktree de l'Epic 38 le
2026-11-20). **Suivie par** : 39-11c (rapprochement), 39-11d (écrans). **Aucune migration** (décision de
l'orchestrateur du 2026-11-20, Q-39-10 tranchée).*

## Story

En tant que **Comptable** (les routes des comptes bancaires sont des routes Comptable et plus, `comptable_routes`,
`crates/kesh-api/src/lib.rs:579`, `:1053-1060`), je veux faire d'un compte bancaire Kesh le solde, dans une devise,
de mon compte Wise — sans IBAN obligatoire, lié à un compte du plan en devise, avec le compte de frais — puis importer
d'un geste les relevés CSV de ce compte Wise (une devise par fichier, plusieurs fichiers à la fois), sans doublon,
afin que chaque ligne attende son rapprochement (39-11c).

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-11b-1` | Un compte bancaire porte des réglages de prestataire `wise` : **libellé du compte Wise** et **compte de frais** — les colonnes de 38-11b, sans champ propre à Wise. | AC-1, AC-2 |
| `FR-39-11b-2` | Un compte Wise a au plus un compte bancaire par devise ; sans IBAN, lié à un compte du plan en devise (levée de C-37-9 de 38-11b étendue à `wise`) ; refusé aux imports bancaires génériques et aux règlements comme un compte PayPal. | AC-2 |
| `FR-39-11b-3` | Import à la demande (route et bouton) d'un ou plusieurs relevés d'un compte Wise : chaque fichier va au compte bancaire de sa devise ; une devise sans compte refuse tout l'import en la nommant. | AC-3, AC-4 |
| `FR-39-11b-4` | Une ligne = une transaction ; doublons (même identifiant Wise sur le même compte) ignorés et comptés ; soldes d'ouverture et de clôture posés ; un import qui ne prolonge pas le précédent est signalé. | AC-4, AC-5 |
| `NFR-39-11b-1` | Aucune migration : `chk_bank_accounts_provider` de 38-11b admet `'wise'`. | AC-1 |
| `NFR-39-11b-2` | Code commun avec 38-11b factorisé, non recopié ; PayPal inchangé (ses tests verts sans retouche de leurs attentes). | AC-2, AC-4, AC-6 |

## Critères d'acceptation

1. **AC-1 — `provider = 'wise'`, sans migration.** La migration de 38-11b (`20261120000001_paypal_import.sql`) pose
   `chk_bank_accounts_provider` avec `BINARY provider IN ('paypal', 'wise', 'stripe')` et un test qui admet `'wise'` au
   schéma : **aucune migration** ici, `kesh_version_min_required` et versions Cargo inchangés, aucune ligne à
   `docs/migrations-idempotence-audit.md`. Vérifié au T0 sur 38-11b telle que développée ; si elle n'admettait pas
   `'wise'`, c'est à l'Epic 38 de corriger sa migration **avant toute publication** (P8), non à cette story d'en ajouter
   une. Sauvegarde, réimport et export CSV : rien de neuf (colonnes de 38-11b, AC-7 de 38-11b).
2. **AC-2 — Réglages du compte Wise** (routes existantes `POST /api/v1/bank-accounts` et `PUT
   /api/v1/bank-accounts/{id}`, `lib.rs:1053-1060`, Comptable et plus). Le corps gagne un objet facultatif `wise:
   { account: string, feeAccountId: number | null }`, **de même forme** que `paypal` de 38-11b AC-2 (absent au `PUT` =
   inchangé, `null` = retiré, pose `provider = 'wise'`) ; `paypal` et `wise` ensemble → 400 `VALIDATION_ERROR`. La
   réponse gagne `wise: { account, feeAccountId } | null`. **Factorisation (G10)** : la fonction unique de gardes de
   38-11b AC-2 et AC-3 (état résultant ; libellé rogné 1 à 120 caractères et `refuse_card_number`,
   `crates/kesh-api/src/routes/bank_accounts.rs:387` ; compte lié obligatoire ; un compte bancaire par devise et par
   libellé **du même prestataire** ; compte de frais actif, imputable, de charge, CHF ou de la devise du compte lié ;
   réglages figés dès qu'une transaction existe ; IBAN facultatif mais contrôlé, QR-IBAN refusé ; jamais principal ;
   levée des deux refus `ACCOUNT_IN_FOREIGN_CURRENCY`, `locked_ledger_type` `:435-458` et le pré-vol `:680-691` ;
   `PATCH` refusé `BANK_ACCOUNT_FIELD_NOT_ALLOWED` ; refus `BANK_ACCOUNT_IS_PROVIDER` aux imports CAMT et CSV, au
   relevé saisi, au règlement d'une facture client et au paiement d'une facture fournisseur) est **paramétrée par le
   prestataire** ; chaque garde lit `provider IS NOT NULL` (ou le prestataire passé), jamais `= 'paypal'` en dur —
   chaque site de 38-11b qui le ferait est généralisé, relevé par `grep -n "paypal" crates/kesh-api/src/routes/` au
   Change Log. **Codes d'erreur** : ceux de 38-11b, s'ils sont génériques (`PROVIDER_*`, demande Q-39-10 à l'Epic 38) ;
   s'ils restent `PAYPAL_*`, la même fonction rend les codes `WISE_*` correspondants (le préfixe est un paramètre, la
   logique une seule), messages aux quatre locales (`crates/kesh-i18n/locales/*`), choix au Change Log.
3. **AC-3 — Route d'import** `POST /api/v1/wise-imports` (multipart : un ou plusieurs `file`, `account` = libellé),
   Comptable et plus, clés d'API `read-write` admises (comme `POST /api/v1/bank-imports`, `lib.rs:946`), limite du
   corps entier = `bank_import_max_mib` (même `DefaultBodyLimit` que `lib.rs:946-950`). Inscrite à `LIB_ROUTES`
   (`crates/kesh-api/tests/audit_route_registry.rs:440`) en `Traced, SansEcritureAuJournal` comme celle de 38-11b ; le
   compte de `the_registry_partition_is_what_the_story_declares` (157 au relevé de `origin/main`, plus les routes des
   Epics 38 et suivants à la reprise) relevé de un avec sa raison, et la phrase « N des M routes à verbe mutant » du
   manuel d'administration (`docs/manual/fr/admin-manual.tex`, lue par `le_manuel_dit_le_compte_du_registre`)
   **recomptée** — `.tex` seul, PDF à la release. Si 38-11b a été développée avec une route d'import prestataire
   générique, Wise s'y branche au lieu d'une route neuve (G10), choix au Change Log. Handlers `wise_imports::create` (clé du registre). Ordre des refus, rien d'écrit à
   aucun (codes `WISE_*` nommés ci-dessous, ou leurs équivalents génériques de 38-11b selon la règle de l'AC-2) : 400 `VALIDATION_ERROR` (champ manquant, aucun fichier) → 404 compte Wise introuvable (aucun compte bancaire
   non archivé `provider = 'wise'` de ce libellé ; code de l'AC-2) → 422 `WISE_CSV_INVALID` (erreur de 39-11a,
   `details` : nom du fichier et code fermé de 39-11a, puis colonnes manquantes ou erreurs de ligne, forme de
   `CsvLineErrorPayload`, `crates/kesh-api/src/errors.rs:1541`) → 422 `WISE_DUPLICATE_CURRENCY_FILE` (deux fichiers de
   même devise, `details: { currency, filenames }`) → 422 devise sans compte (`details: { currencies }`, jamais de
   solde créé d'office) → 409 doublon par devise (deux comptes bancaires non archivés de ce libellé pour une devise :
   la course de 38-11b AC-2) → 409 `BANK_ACCOUNT_CLOSED` → 422 `BANK_IMPORT_DUPLICATE_FILE` (empreinte de l'AC-5).
4. **AC-4 — Rangement**, par le **cœur commun de 38-11b** (AC-5 et AC-6 de 38-11b : `with_account_locks` neuf de
   `crates/kesh-reconciliation/src/mutex.rs`, à côté de `with_account_lock` `:75`, verrous pris par identifiant
   croissant ; une transaction de base, tout ou rien ; recherche de doublon par identifiant ; empreinte). Par fichier,
   **une** entrée `bank_imports` (`create_with_transactions`, `crates/kesh-db/src/repositories/bank_imports.rs:72`)
   au compte bancaire de sa devise : `source_format = 'CSV'` (valeur de 38-11b, aucune neuve), `filename`, période =
   `first_date`..`last_date`, `opening_balance` et `closing_balance` de 39-11a AC-5 (colonnes existantes,
   `migrations/20260504000001_bank_imports.sql:29-30`). Chaque ligne donne une `NewBankTransaction`
   (`crates/kesh-db/src/entities/bank_transaction.rs:113`, champs de 38-11b compris) : `amount` = `Amount` (signé),
   `currency`, `booking_date` = `Date`, `value_date = None`, `transaction_id` = identifiant Wise, `reference` =
   `Payment Reference` (sinon `None`), `end_to_end_id = None`, `counterparty_name` = `Merchant`, sinon `Payee Name`
   (débit) ou `Payer Name` (crédit), `counterparty_iban` = `Payee Account Number` sans espaces et en majuscules **s'il
   a la forme d'un IBAN** (deux lettres, deux chiffres, 11 à 30 caractères alphanumériques, ≤ 34, colonne
   `VARCHAR(34)`, `20260504000001_bank_imports.sql:68`) — sinon `None` et le numéro va aux détails (les relevés 2024
   portent aussi des numéros à huit chiffres et des numéros masqués d'étoiles) —, `counterparty_email = None`,
   `provider_type` = `id_prefix` (filtre de règle de 38-11b, G5), `details` = `Description`, `Transaction Details
   Type`, numéro de compte non IBAN, `Exchange From`→`Exchange To` au cours `Exchange Rate`, `Note`, non vides, joints
   par « · », tronqués comme les autres imports (à la frontière d'un caractère : U+202F tient sur trois octets).
   `Total fees` n'est **jamais** une transaction (les frais sont leurs propres lignes, 39-11a). Somme des transactions
   insérées d'un fichier sans doublon = clôture − ouverture (test). Trace d'audit : `bank_import.created` par fichier,
   comme l'import CSV, plus `wise_import.created` (`details` : libellé, fichiers, compte rendu), sous l'acteur
   (`NewAuditLogEntry::for_actor`, utilisateur ou clé d'API ; garde `audit_acteur_garde.rs`).
5. **AC-5 — Doublons, continuité, compte rendu.** Empreinte : la fonction de 38-11b AC-6 (SHA-256 de `fichier ‖ ":" ‖
   devise`), refus de l'AC-3 si `find_by_company_and_hash` (`bank_imports.rs:308`) la trouve. Doublon de transaction :
   même `transaction_id` déjà sur ce compte bancaire → ligne ignorée (la fenêtre date-montant-référence des imports
   bancaires, `find_in_dedup_window`, `crates/kesh-db/src/repositories/bank_transactions.rs:83`, ne s'applique pas,
   comme pour PayPal) ; un fichier tout en doublons ne crée pas d'entrée. **Continuité avec l'import précédent** du
   même compte, **une seule définition** (servie aussi par les écarts de 39-11c, qui ne peut pas la lire des doublons :
   le fichier n'est pas conservé, `transaction_count` ne compte que les lignes insérées), jugée sur le **premier jour** du
   fichier seul : un fichier **prolonge par chevauchement** un relevé du compte quand `overlapping_statements(company, compte,
   first_date, first_date)` (`bank_imports.rs:386`, `period_from <= first_date AND period_to >= first_date`, relevés saisis ou
   importés ; à la revue, l'import jugé exclu) en rend au moins un — pas d'avertissement. Jamais le chevauchement de toute la
   période (`first_date..last_date`) : un relevé **postérieur** qui chevauche la fin d'un import masquerait, à la revue,
   l'écart du début de cet import, et un import rattrapé après coup qui chevauche un relevé plus récent ne serait jamais
   jugé. Sinon, par la fonction
   existante `previous_statement_closing` (`bank_imports.rs:358` : le relevé à solde du compte dont `period_to` <
   `first_date`, le plus tardif) : si elle en rend un et que sa clôture diffère de `opening_balance` du fichier →
   avertissement `gap` au compte rendu (l'import passe) ; aucun relevé antérieur → rien. La clôture lue est celle d'un
   relevé à solde : les imports Wise en ont toujours une, un relevé saisi sans solde est ignoré par la fonction. Réponse `201` : `{ account, files: [{ filename, currency, bankAccountId,
   importId | null, inserted, duplicates, openingBalance, closingBalance, warnings: [ { code: "gap", expected, found,
   previousImportId } ] }] }`.
6. **AC-6 — Non-régression et factorisation.** Tests de 38-11b (PayPal) et des imports CAMT, CSV, manuel inchangés
   en assertions ; le Change Log nomme chaque fonction de 38-11b réutilisée ou généralisée, et confirme qu'aucune
   n'est recopiée. Aucun outil MCP (comme 38-11b).

## Tâches

- [ ] T0 — Reprendre `main` et `epic/38-paypal` ; lire 38-11b telle que développée (contrainte, codes, route) ;
  relever les sites `paypal` en dur.
- [ ] T1 — Gardes prestataire paramétrées, objet `wise`, refus aux autres chemins pour `wise` (AC-2).
- [ ] T2 — Route d'import, rangement par fichier, doublons, continuité, compte rendu ; registre des routes et phrase du
  manuel d'administration (AC-3 à AC-5).
- [ ] T3 — i18n, non-régression (AC-6).

## Stratégie de tests

- **HTTP** (`crates/kesh-api/tests/wise_import_e2e.rs`, `#[sqlx::test]` sur le squash) avec les fixtures de 39-11a :
  société multi-devises, trois comptes bancaires du libellé fictif ; import des trois fichiers → trois entrées,
  nombres et soldes attendus ; réimport → 422 `BANK_IMPORT_DUPLICATE_FILE` ; fichier chevauchant → seules les lignes
  neuves insérées, aucun `gap` ; fichier suivant discontinu → `gap` ; import rattrapé après coup dont la fin chevauche un relevé plus récent et
  dont le début ne prolonge pas le précédent → `gap` ; devise sans compte → 422, rien d'écrit ; deux
  fichiers de même devise → 422 ; compte clos → 409 ; clé d'API `read` → 403 ; numéro de compte non IBAN → `None` et
  détails. Réglages : chaque refus de l'AC-2 pour `wise` ; `paypal` et `wise` ensemble → 400 ; import CAMT, relevé
  saisi, règlement de facture et paiement fournisseur sur un compte Wise → 409 `BANK_ACCOUNT_IS_PROVIDER` ; PayPal
  inchangé.
- Gate complet au push.

## Modules touchés (4)

`kesh-api` (routes `bank_accounts`, `wise_imports`, erreurs), `kesh-reconciliation` (appel de `with_account_locks`,
sans changement), `kesh-import` (lecture de 39-11a), `kesh-i18n`. Plus une phrase du manuel d'administration (`.tex`).

## Note de version

*Ajouté* — Un compte bancaire peut être le solde, dans une devise, d'un compte Wise : sans IBAN, lié à un compte du
plan en devise, avec son compte de frais. Les relevés CSV de Wise s'importent d'un geste, plusieurs devises à la
fois ; les doublons sont ignorés et un trou entre deux relevés est signalé. Un compte Wise ne reçoit ni relevé
bancaire ni règlement de facture.
*Migration* — aucune.

## Notes de développement

- Ancienne 39-1 (client Wise, connexion et coffre, `superseded`) : remplacée par les réglages de 38-11b.
- ⚠️ 38-11b est `drafted` (validée P1) au 2026-11-20 : ses noms (objet `paypal`, codes `PAYPAL_*`) peuvent changer
  avant son développement ; relire au T0.

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
- 2026-11-20 — validation P1 (Claude Opus 5.5, contexte frais) : 0 CRITICAL, 2 HIGH, 4 MEDIUM, 3 LOW, remédiés dans la
  fiche (rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-11b-compte-wise-et-import-P1.md`). HIGH : continuité
  lue sur « le solde courant de la dernière transaction », qui n'est stocké nulle part → `previous_statement_closing`
  existant, sauté quand le fichier chevauche ; migration de repli retirée (Q-39-10 tranchée : 38-11b admet `'wise'`).
  MEDIUM : rôle Administrateur → Comptable et plus (38-11b validée) ; registre `LIB_ROUTES` et phrase du manuel
  d'administration absents ; refus `BANK_ACCOUNT_IS_PROVIDER` et `PATCH` non étendus à `wise` ; verrou
  `with_account_lock` au lieu de `with_account_locks` de 38-11b. LOW : limite du corps multi-fichiers, place du refus
  de devise en double (après la lecture), troncature UTF-8 des détails.
- 2026-11-20 — validation P2 (Claude Sonnet 5.5, contexte frais) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 1 LOW, remédiés (rapport
  `kesh-gate-logs/epic-39-40-kiss/validation/39-11b-compte-wise-et-import-P2.md`). Remédiation P1 relue au code d'`origin/main`
  et à 38-11b (validée P2) : rien à reprendre. MEDIUM : la règle de continuité « aucune ligne du fichier n'est un doublon »
  n'était pas relisible par les écarts de 39-11c (les doublons ne sont pas stockés) et, relue à la revue, signalait à tort
  l'import qui en chevauchait un autre (`previous_statement_closing` rend alors un relevé plus ancien) → chevauchement par
  `overlapping_statements`, une seule définition. LOW : codes nommés `WISE_*` à l'AC-3 contre la règle de l'AC-2, clé du
  registre des routes absente.
- 2026-11-20 — validation P3 ciblée (Claude Opus 5.5, contexte frais, commit `02ed58c6` seul) : 0 CRITICAL, 0 HIGH, 1 MEDIUM,
  0 LOW, remédié (rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-11-P3-ciblee.md`). MEDIUM : le chevauchement jugé
  sur toute la période (`first_date..last_date`) ne donnait pas la même réponse à l'import et à la revue — un relevé
  postérieur chevauchant la fin d'un import masquait à la revue l'écart du début, et un import rattrapé après coup n'était
  jamais jugé → chevauchement du **premier jour** seul (`overlapping_statements(…, first_date, first_date)`, l'import jugé
  exclu à la revue), complémentaire de `previous_statement_closing` (`period_to < first_date`) ; test ajouté.
