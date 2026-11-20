# Story 39-11b : réglages du compte Wise et import des relevés, ventilé par devise

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/39-wise`, numéros de ligne relevés au code de `origin/main` (`fd04099f`).
Epic 39, **phase 1 KISS** (G2, G3, G4, G10, G11). **Dépend de** : 39-11a (lecteur), **38-11b** (réglages prestataire
d'un compte bancaire, cœur de l'import ventilé ; fiche `drafted` lue dans le worktree de l'Epic 38 le 2026-11-20).
**Suivie par** : 39-11c (rapprochement), 39-11d (écrans).*

## Story

En tant qu'**Administrateur**, je veux faire d'un compte bancaire Kesh le solde, dans une devise, de mon compte Wise
— sans IBAN obligatoire, lié à un compte du plan en devise, avec le compte de frais — puis, en tant que
**Comptable**, importer d'un geste les relevés CSV de ce compte Wise (une devise par fichier, plusieurs fichiers à
la fois), sans doublon, afin que chaque ligne attende son rapprochement (39-11c).

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-11b-1` | Un compte bancaire porte des réglages de prestataire `wise` : **libellé du compte Wise** et **compte de frais** — ceux de 38-11b, sans champ propre à Wise. | AC-1, AC-2 |
| `FR-39-11b-2` | Un compte Wise a au plus un compte bancaire par devise ; sans IBAN, lié à un compte du plan en devise (levée de C-37-9 étendue à `wise`). | AC-2 |
| `FR-39-11b-3` | Import à la demande (route et bouton) d'un ou plusieurs relevés d'un compte Wise : chaque fichier va au compte bancaire de sa devise ; une devise sans compte refuse tout l'import en la nommant. | AC-3, AC-4 |
| `FR-39-11b-4` | Une ligne = une transaction ; doublons (même identifiant Wise sur le même compte) ignorés et comptés ; soldes d'ouverture et de clôture posés ; un import qui ne prolonge pas le précédent est signalé. | AC-4, AC-5 |
| `NFR-39-11b-1` | Aucune migration si 38-11b admet `'wise'` ; sinon une seule, non breaking, DDL seul (repli G11). | AC-1 |
| `NFR-39-11b-2` | Code commun avec 38-11b factorisé, non recopié ; PayPal inchangé (ses tests verts sans retouche de leurs attentes). | AC-6 |

## Critères d'acceptation

1. **AC-1 — Valeur `wise`, sans migration ou par le repli.** Si la contrainte `chk_bank_accounts_provider` de 38-11b
   admet déjà `'wise'` (demande G11, Q-39-10) : **aucune migration**, `kesh_version_min_required` et versions Cargo
   inchangés. **Sinon (repli)** : migration `2026112X000001_bank_accounts_provider_wise.sql`, horodatage **après** la
   migration de 38-11b et après toute migration de `main` à la reprise ; elle fait `ALTER TABLE bank_accounts DROP
   CONSTRAINT chk_bank_accounts_provider` puis la repose **sous le même nom**, identique à celle de 38-11b sauf
   `BINARY provider IN ('paypal', 'wise', 'stripe')` ; non breaking au sens du ⚠️ de P1 (en tête, l'argument de
   38-11b) ; aucun `UPDATE` (P7 sans objet) ; P5 (ligne à `docs/migrations-idempotence-audit.md`, total et partitions
   recomptés depuis la source), P6 (`grep -rn "migrations.len()\|apply_migrations_up_to" crates/`, chaque site
   inspecté), P8 ; squash de test régénéré (`test_schema_guard.rs` vert) ; test : `provider = 'wise'` admis,
   `'Wise'` refusé (octet). Gate **complet** (exception `kesh-db`).
2. **AC-2 — Réglages du compte Wise.** Routes existantes `POST` et `PUT /api/v1/bank-accounts` (Administrateur).
   Mêmes gardes que 38-11b AC-2 et AC-3 (libellé 1 à 120 caractères, compte lié obligatoire et dont la devise **est**
   celle du solde, un compte bancaire par devise et par libellé, compte de frais de charge en CHF ou dans la devise du
   compte lié, réglages figés dès qu'une transaction existe, IBAN facultatif mais contrôlé s'il est donné, QR-IBAN
   refusé, jamais principal, levée de `ACCOUNT_IN_FOREIGN_CURRENCY` — `crates/kesh-api/src/routes/bank_accounts.rs:435-458`
   et le pré-vol — pour un compte prestataire). **Factorisation (G10)** : ces gardes deviennent **une** fonction
   paramétrée par le prestataire ; le corps de l'API est l'objet générique `provider: { kind: "paypal" | "wise",
   account, feeAccountId }` si 38-11b l'a adopté. Si 38-11b est développée avec un objet `paypal` **non publié**,
   39-11b le renomme en `provider` (frontend de 38-11d compris) ; **publié**, 39-11b ajoute un objet `wise` de même
   forme, servi par la même fonction, et le changement de contrat est écrit à la note de version. Codes d'erreur de
   38-11b réutilisés, renommés en `PROVIDER_*` dans le premier cas seulement (quatre locales, `crates/kesh-i18n/locales/*`).
3. **AC-3 — Route d'import** `POST /api/v1/wise-imports` (multipart : un ou plusieurs `file`, `account` = libellé),
   Comptable et plus, clés d'API `read-write` admises (comme `POST /api/v1/bank-imports`, `crates/kesh-api/src/lib.rs:946`),
   inscrite au registre des routes auditées (`crates/kesh-api/tests/audit_route_registry.rs`) et aux compteurs de
   routes recomptés depuis la source. Taille maximale par fichier : celle de l'import bancaire ; au plus un fichier
   par devise (deux fichiers de même devise → 400 `WISE_DUPLICATE_CURRENCY_FILE`). Ordre des refus, rien d'écrit : 400 `VALIDATION_ERROR` → 404
   `PROVIDER_ACCOUNT_NOT_FOUND` (aucun compte bancaire non archivé `provider = 'wise'` de ce libellé) → 422
   `WISE_CSV_INVALID` (erreur de 39-11a, `details` : nom du fichier, puis colonnes manquantes ou erreurs de ligne, forme
   de `CsvLineErrorPayload`, `crates/kesh-api/src/errors.rs:1541`) → 422 `PROVIDER_CURRENCY_ACCOUNT_MISSING`
   (`details: { currencies }`, jamais de solde créé d'office) → 409 `BANK_ACCOUNT_CLOSED` → 422
   `BANK_IMPORT_DUPLICATE_FILE` (empreinte de l'AC-5). Si 38-11b expose une route générique d'import prestataire, Wise
   s'y branche au lieu d'une route neuve (G10) ; le choix s'écrit au Change Log.
4. **AC-4 — Rangement.** Par fichier, **une** entrée `bank_imports` (`create_with_transactions`,
   `crates/kesh-db/src/repositories/bank_imports.rs:72`) au compte bancaire de sa devise, par le cœur commun de 38-11b
   (verrou de **chaque** compte visé pris par identifiant croissant, `with_account_lock`,
   `crates/kesh-reconciliation/src/mutex.rs:75` ; une transaction de base, tout ou rien) : `source_format` = la valeur
   que 38-11b emploie (`'CSV'`, aucune valeur neuve), période = `first_date`..`last_date`, `opening_balance` et
   `closing_balance` de 39-11a AC-5. Chaque ligne donne une `NewBankTransaction`
   (`crates/kesh-db/src/entities/bank_transaction.rs:113`, champs de 38-11b compris) : `amount` = `Amount` (signé),
   `currency`, `booking_date` = `Date`, `value_date = None`, `transaction_id` = identifiant Wise, `reference` =
   `Payment Reference` (sinon `None`), `end_to_end_id = None`, `counterparty_name` = `Merchant`, sinon `Payee Name`
   (débit) ou `Payer Name` (crédit), `counterparty_iban` = `Payee Account Number` normalisé **s'il a la forme d'un IBAN**
   (lettres, chiffres de contrôle, ≤ 34, colonne `VARCHAR(34)`, `crates/kesh-db/migrations/20260504000001_bank_imports.sql:68`),
   sinon `None` et le numéro va aux détails, `counterparty_email = None`, `provider_type` = `id_prefix` (critère de
   règle de 38-11b, G5), `details` = `Description`, `Transaction Details Type`, numéro de compte non IBAN, `Exchange
   From`→`Exchange To` au cours `Exchange Rate`, `Note`, non vides, joints par « · », tronqués comme les autres imports.
   `Total fees` n'est **jamais** une transaction (les frais sont leurs propres lignes, 39-11a). Somme des transactions
   d'une devise = clôture − ouverture (test). Trace d'audit : celle de l'import existant, par fichier, plus
   `wise_import.created` (`details` : libellé, fichiers, compte rendu).
5. **AC-5 — Doublons, continuité, compte rendu.** Empreinte : SHA-256 du fichier (une par `bank_imports`), refus de
   l'AC-3 si `find_by_company_and_hash` (`bank_imports.rs:308`) la trouve. Doublon de transaction : même
   `transaction_id` déjà sur ce compte bancaire → ligne ignorée (la fenêtre date-montant-référence des imports
   bancaires, `find_in_dedup_window`, `crates/kesh-db/src/repositories/bank_transactions.rs:83`, ne s'applique pas,
   comme pour PayPal) ; un fichier tout en doublons ne crée pas d'entrée. **Continuité avec l'import précédent** du
   même compte (dernière transaction Wise antérieure à `first_date` : son solde courant, lu de la clôture de son
   import) : différent de `opening_balance` du fichier → avertissement `gap` au compte rendu (l'import passe).
   Réponse `201` : `{ account, files: [{ filename, currency, bankAccountId, importId | null, inserted, duplicates,
   openingBalance, closingBalance, warnings: [ { code: "gap", expected, found } ] }] }`.
6. **AC-6 — Non-régression et factorisation.** Tests de 38-11b (PayPal) et des imports CAMT, CSV, manuel inchangés
   en assertions ; le Change Log nomme chaque fonction de 38-11b réutilisée ou généralisée, et confirme qu'aucune
   n'est recopiée ; sauvegarde et export CSV : rien de neuf hors repli (les colonnes sont celles de 38-11b ; au repli,
   seule une contrainte change, sans colonne).

## Tâches

- [ ] T0 — Reprendre `main` et `epic/38-paypal` ; lire 38-11b telle que développée ; choisir sans migration ou repli.
- [ ] T1 — Repli G11 seulement : migration, squash, audit, test (AC-1).
- [ ] T2 — Gardes prestataire en une fonction, `wise` admis (AC-2).
- [ ] T3 — Route d'import, rangement par fichier, doublons, continuité, compte rendu (AC-3 à AC-5).
- [ ] T4 — i18n, non-régression (AC-6).

## Stratégie de tests

- **HTTP** (`crates/kesh-api/tests/wise_import_e2e.rs`, `#[sqlx::test]` sur le squash) avec les fixtures de 39-11a :
  société multi-devises, trois comptes bancaires du libellé fictif ; import des trois fichiers → trois entrées,
  nombres et soldes attendus ; réimport → 422 `BANK_IMPORT_DUPLICATE_FILE` ; fichier chevauchant → seules les lignes
  neuves insérées ; devise sans compte → 422, rien d'écrit ; deux fichiers de même devise → 400 ; fichier discontinu
  avec un import précédent → `gap` ; clé d'API `read` → 403. Réglages : chaque refus de l'AC-2 pour `wise`, PayPal
  inchangé.
- Gate complet au push ; gate complet en cours de boucle au repli G11 (exception `kesh-db`).

## Modules touchés (4)

`kesh-api` (routes `bank_accounts`, `wise_imports`, erreurs), `kesh-import` (lecture de 39-11a), `kesh-i18n`,
`kesh-db` (au repli G11 seulement : migration et squash).

## Note de version

*Ajouté* — Un compte bancaire peut être le solde, dans une devise, d'un compte Wise : sans IBAN, lié à un compte du
plan en devise, avec son compte de frais. Les relevés CSV de Wise s'importent d'un geste, plusieurs devises à la
fois ; les doublons sont ignorés et un trou entre deux relevés est signalé.
*Migration* — aucune (ou, au repli G11, une migration non breaking : la sauvegarde d'avant la mise à jour reste
exigée).

## Notes de développement

- Ancienne 39-1 (client Wise, connexion et coffre, `superseded`) : remplacée par les réglages de 38-11b.
- ⚠️ 38-11b est `drafted` au 2026-11-20 : ses noms (`paypal`, codes `PAYPAL_*`) peuvent changer avant son
  développement ; relire au T0.

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
