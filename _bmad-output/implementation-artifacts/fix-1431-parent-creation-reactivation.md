# Correctif #1431 : la création et la réactivation d'un sous-compte contrôlent ce que porte le parent

Status: review

*Fiche rédigée le 2026-11-28, branche `fix/1431-parent-creation-reactivation`, partie de `origin/main` à `0c44f7d1a`.
Correctif hors epic, pour la release **v0.21.0** (4 décembre). **Aucune migration.** Le mainteneur a proposé le refus et
laissé le contrat de l'API à notre lecture du code ; les détails ci-dessous sont les nôtres.*

## Intention

`POST /accounts` avec `parentId` et `PUT /accounts/{id}/reactivate` d'un sous-compte archivé rendent le parent **non
imputable** (règle 14-3a D2 : un compte qui a un sous-compte actif n'est pas imputable) **sans regarder ce qu'il porte**,
hors trois refus : lien bancaire (`ACCOUNT_PARENT_BANK_LINKED`), compte de différence de change
(`ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`), compte de bilan en devise mouvementé (`ACCOUNT_FOREIGN_PARENT_HAS_ENTRIES`). Trois cas
passaient :

1. un parent CHF qui porte des écritures — son solde ne se corrige plus (le `PUT` avec `parentId` le refusait déjà,
   `ACCOUNT_PARENT_HAS_ENTRIES`) ;
2. un parent désigné dans les réglages de facturation (créance, produit, TVA…) ;
3. un parent compte de produit par défaut d'un article.

En 2 et 3, la validation suivante d'une facture ou d'un règlement échoue en `400 ACCOUNT_NOT_POSTABLE`, sans avertissement
préalable. Le `PUT` avec `parentId` et la route groupée laissaient eux aussi passer 2 et 3.

**Prémisse vérifiée au code** (`0c44f7d1a`) : `crates/kesh-db/src/repositories/accounts.rs`, `create_in_tx` (« La règle des
parents mouvementés, elle, ne change pas ici (création plus large que le `PUT`, #1431) ») ; `reactivate` (« Un parent
mouvementé n'est pas refusé (même écart que la création, #1431) ») ; `guard_parent_change_in_tx` (désignations de change,
lien bancaire, devise, écritures — aucune lecture des réglages de facturation ni des articles).

## Contrat retenu

- **Cas 1** — `POST /accounts` et `PUT /accounts/{id}/reactivate` : `409 ACCOUNT_PARENT_HAS_ENTRIES`, `details {
  parentAccountId, parentNumber }`, le code et les `details` du changement de parent (aucun code neuf).
- **Cas 2 et 3** — code neuf `409 ACCOUNT_PARENT_DESIGNATED` sur `POST /accounts`, `PUT /accounts/{id}` (`parentId`),
  `PUT /accounts/{id}/reactivate` et `POST /accounts/groups` (groupe réutilisé, rattaché, ou parent d'un groupe créé ; avec
  `details.groupStep`). `details { parentAccountId, parentNumber, designation, … }` : `designation: "invoiceSettings"` avec
  `setting` (codes de `DESIGNATED_ACCOUNT_COLUMNS`, ceux d'`ACCOUNT_DESIGNATED_IN_SETTINGS`), ou `designation: "product"`
  avec `productId` et `productName`. La première désignation trouvée : réglages d'abord (ordre des colonnes), puis articles
  (le premier par identifiant), **actif ou archivé** (un article archivé se réactive).
- **Ordre des refus du parent**, sur les trois chemins : désignations de change → lien bancaire → désignation de facturation
  → bilan en devise mouvementé → mouvementé en toute devise.
- **Membres** : un compte désigné ou mouvementé rangé **sous** un groupe reste accepté et imputable ; seul le parent est
  contrôlé.
- **Réactivation** : le sous-compte reste archivé ; les messages des trois refus (lien bancaire, désignation, écritures)
  nomment le sous-compte (`reactivated_number`), les `details` restent ceux du changement de parent.
- **Écran** : le champ *Compte parent* de la création ne propose plus les comptes qui portent des écritures
  (`currencyLocked`), comme la modification. Les désignations ne sont pas filtrées d'avance (la liste des comptes ne les dit
  pas) : le message du serveur s'affiche tel quel (création, réactivation, modification, page des groupes).

### Choix consignés

- **C-1431-1 — Forme de `details`.** Le brief proposait `designation: "invoice_settings.<champ>" |
  "product.default_revenue_account"`. Retenu : la convention voisine d'`ACCOUNT_CURRENCY_IN_USE` (`site: invoiceSettings`
  avec `setting`, `product` avec `productId`) et d'`ACCOUNT_DESIGNATED_IN_SETTINGS` (mêmes codes de réglage), en camelCase,
  avec `productName` pour le message (comme `bankName` d'`ACCOUNT_PARENT_BANK_LINKED`). Réversible avant la release.
- **C-1431-2 — Une seule garde.** `refuse_unfit_parent_in_tx` (lien bancaire, désignation, devise, écritures) est appelée
  par `create_in_tx`, `guard_parent_change_in_tx` et `reactivate`. Les désignations de change restent jugées par chaque
  appelant, parce que le changement de parent exclut celles du compte déplacé (revue P1-4 de `fix-api-parent-compte`).
- **C-1431-3 — Verrous.** `reactivate` prend désormais les réglages de facturation en partage après `companies` et avant
  le `FOR UPDATE` du parent, l'ordre de `create` et d'`update` (une modification des réglages les tient `FOR UPDATE` puis
  lit les comptes). Les articles se lisent en `LOCK IN SHARE MODE` par l'index `idx_products_default_revenue_account`,
  après le parent : une modification d'article prend le compte en partage avant sa ligne d'article, pas de cycle. La
  réactivation n'est pas rejouée sur interblocage (inchangé, documenté sur `refuse_bank_linked_parent_in_tx`).
- **C-1431-4 — Rôles et règles de rapprochement hors périmètre.** Un compte qui porte un rôle (`DefaultRevenue`,
  `Receivable`…) ou qui sert de contrepartie à une règle de rapprochement n'est pas refusé comme parent : le brief borne le
  correctif aux réglages de facturation et aux articles. Une issue est demandée à l'orchestrateur (rapport final).

## Tests

`crates/kesh-api/tests/accounts_parent_guard_e2e.rs` (dix tests, à travers HTTP) : chaque refus sur chaque route, le cas
qui passe (parent CHF vide, non désigné : `201`/`200`, parent non imputable sans hausse de `version`), et, sur chaque refus,
le parent qui garde `postable` et `version`, rien de créé, le sous-compte réactivé qui reste archivé. Quatre tests existants
qui décrivaient l'écart (« #1431 inchangée ») sont retournés : `account_groups::creation_d_un_sous_compte_sous_un_compte_lie_refusee`,
`account_groups::parent_mouvemente_d_un_groupe_cree_est_refuse`,
`account_parent_change::reactivation_sous_un_parent_en_devise_mouvemente_refusee`,
`fx_revaluation::sous_compte_sous_un_compte_de_bilan_en_devise`.

Mutations : voir le Dev Agent Record.

## Notes de version

*À reprendre au CHANGELOG de la v0.21.0, section « Changed » (changements de contrat de l'API).*

- **Un compte qui porte des écritures, ou que la facturation emploie, ne devient plus compte de groupe par la création ou
  la réactivation d'un sous-compte** ([#1431](https://github.com/guycorbaz/kesh/issues/1431)). Créer un sous-compte sous un
  compte mouvementé, ou réactiver un sous-compte archivé dont le parent a reçu des écritures, est refusé comme l'était
  déjà le changement de parent : `409 ACCOUNT_PARENT_HAS_ENTRIES`. Un compte désigné dans *Administration → Facturation*
  (créance, produit, TVA, créanciers, arrondi, escompte, frais bancaires, pertes) ou compte de produit d'un article ne
  reçoit plus de sous-compte, par aucun chemin — création, réactivation, *Modifier le compte*, *Créer des groupes* :
  `409 ACCOUNT_PARENT_DESIGNATED`, code neuf, dont le message nomme le réglage ou l'article. Jusqu'ici, le compte passait
  non imputable, et la validation suivante d'une facture ou d'un règlement qui l'employait était refusée sans
  avertissement. Aucune écriture n'était faussée.
- ⚠️ **Changement de contrat de l'API** : `POST /accounts` (`parentId`) et `PUT /accounts/{id}/reactivate` peuvent
  répondre `409 ACCOUNT_PARENT_HAS_ENTRIES` ou `409 ACCOUNT_PARENT_DESIGNATED` là où ils répondaient `201`/`200` ;
  `PUT /accounts/{id}` (`parentId`) et `POST /accounts/groups` peuvent répondre `409 ACCOUNT_PARENT_DESIGNATED`. Détails
  au guide de l'API, § 10. L'API ne change pas de version.
- Le champ *Compte parent* de la création d'un compte ne propose plus les comptes qui portent des écritures.

## Dev Agent Record

*(Complété après les gates.)*

## Change Log

- 2026-11-28 — fiche et implémentation (`01708501f`).
