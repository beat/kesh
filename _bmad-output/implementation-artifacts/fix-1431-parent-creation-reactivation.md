# Correctif #1431 : la création et la réactivation d'un sous-compte contrôlent ce que porte le parent

Status: review

*Fiche rédigée le 2026-11-28, branche `fix/1431-parent-creation-reactivation`, partie de `origin/main` à `0c44f7d1a`.
Correctif hors epic, pour la **release du 4 décembre**. **Aucune migration.** Le mainteneur a proposé le refus et
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
- **C-1431-4 — Rôles, règles de rapprochement et brouillons hors périmètre.** Un compte qui porte un rôle
  (`DefaultRevenue`, `Receivable`…), qui sert de contrepartie à une règle de rapprochement, ou qu'emploie une ligne d'un
  brouillon (facture, avoir, facture fournisseur, compte de règlement d'une facture fournisseur) n'est pas refusé comme
  parent : le brief borne le correctif aux réglages de facturation et aux articles, désignations durables ; un brouillon se
  corrige à sa validation, qui nomme le compte. Une issue est demandée à l'orchestrateur (rapport final).
- **C-1431-5 — Un parent déjà compte de groupe reste soumis à la garde** (revue P1, M-1). Un groupe qui porte déjà des
  écritures ou une désignation (l'état que laissait l'écart #1431) ne reçoit plus de sous-compte, par création ni par
  réactivation, bien que le geste ne change pas son imputabilité. Retenu : la règle que `PUT /accounts/{id}` (`parentId`,
  depuis la v0.19.1) et le rattachement d'un groupe (`POST /accounts/groups`, `attach`, depuis la v0.20.0) appliquaient
  déjà à un groupe existant ; une garde unique, sans exception ; un tel groupe est une anomalie à corriger (déplacer ses
  écritures, désigner un autre compte), non un état à étendre. Écarté : sauter les deux refus quand le parent est déjà
  non imputable — plus permissif, mais il ferait diverger la création du `PUT` et du rattachement, et changerait la
  règle de ces deux routes. Les messages ne disent plus « redevenu compte de groupe », vrai seulement pour un parent sans
  autre sous-compte. ~~À confirmer par le mainteneur~~ **Confirmé par le mainteneur** le 2026-11-28, par le principe
  qu'il a posé : « Un compte de regroupement ne porte pas d'écritures : on ne peut pas désigner comme parent un compte qui
  en a, même si son solde est nul. » *(Annoté le 2026-11-28 ; la fiche disait « à confirmer ».)*
- **C-1431-6 — Refus, non confirmation ; marche à suivre au manuel** (décision du mainteneur, 2026-11-28). Les deux refus
  restent des refus sans confirmation possible. Le numéro d'un compte ne se modifiant pas, une réorganisation se fait en
  créant les nouveaux comptes, en reportant le solde par une écriture de reclassement, puis en archivant l'ancien compte ;
  pour un compte désigné, « on change d'abord la désignation, puis on restructure ». Le manuel utilisateur porte cette
  marche (§ « Réorganiser le plan comptable », `sec:reorganiser-plan`), et les six messages des deux refus (création ou
  changement de parent, réactivation ; réglage ou article) y renvoient, dans les quatre locales. Le manuel n'existe qu'en
  français (`docs/manual/{de,en,it}/` ne portent qu'un README) : les messages allemand, italien et anglais citent le titre
  français de la section, en le disant. Si un manuel allemand, italien ou anglais voit le jour, ces dix-huit messages (six
  clés, trois locales) seront à reprendre. Les avertissements de l'aperçu de *Créer des groupes*
  (`frontend/src/routes/(app)/accounts/groups/messages.ts`) et le message d'`ACCOUNT_FOREIGN_PARENT_HAS_ENTRIES` ne
  renvoient pas au manuel : hors de la demande (revue P4, LOW-1).

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

*À reprendre au CHANGELOG de la release du 4 décembre, section « Changed » (changements de contrat de l'API).*

- **Un compte qui porte des écritures, désigné dans les réglages de facturation ou compte de produit d'un article, ne
  reçoit plus de sous-compte par la création ou la réactivation d'un sous-compte** ([#1431](https://github.com/guycorbaz/kesh/issues/1431)). Créer un sous-compte sous un
  compte mouvementé, ou réactiver un sous-compte archivé dont le parent a reçu des écritures, est refusé comme l'était
  déjà le changement de parent : `409 ACCOUNT_PARENT_HAS_ENTRIES`. Un compte désigné dans *Administration → Facturation*
  (créance, produit, TVA, créanciers, arrondi, escompte, frais bancaires, pertes) ou compte de produit d'un article ne
  reçoit plus de sous-compte, par aucun chemin — création, réactivation, *Modifier le compte*, *Créer des groupes* :
  `409 ACCOUNT_PARENT_DESIGNATED`, code neuf, dont le message nomme le réglage ou l'article. Jusqu'ici, le compte passait
  non imputable, et la validation suivante d'une facture ou d'un règlement qui l'employait était refusée sans
  avertissement. Aucune écriture n'était faussée. La règle vaut aussi pour un compte déjà compte de groupe, comme au
  changement de parent.
- ⚠️ **Changement de contrat de l'API** : `POST /accounts` (`parentId`) et `PUT /accounts/{id}/reactivate` peuvent
  répondre `409 ACCOUNT_PARENT_HAS_ENTRIES` ou `409 ACCOUNT_PARENT_DESIGNATED` là où ils répondaient `201`/`200` ;
  `PUT /accounts/{id}` (`parentId`) et `POST /accounts/groups` peuvent répondre `409 ACCOUNT_PARENT_DESIGNATED`. Détails
  au guide de l'API, § 10. L'API ne change pas de version.
- Le champ *Compte parent* de la création d'un compte ne propose plus les comptes qui portent des écritures.
- **Réorganiser le plan comptable** : le manuel utilisateur dit la marche à suivre — créer les nouveaux comptes, reporter
  le solde par une écriture de reclassement, archiver l'ancien compte ; pour un compte désigné par la facturation, changer
  d'abord la désignation. Un compte qui porte des écritures ne devient pas compte de groupe, même à solde nul. Les messages
  des deux refus renvoient à cette section (le manuel n'existe qu'en français).

## Dev Agent Record

Journaux hors dépôt (répertoire de journaux du correctif, donné par l'orchestrateur).

**Mutations** (`mutations.py`, au commit `01708501f`, binaires `accounts_parent_guard_e2e`, `account_parent_change`,
`account_groups`, `fx_revaluation` — 71 tests ; source restaurée par `git checkout` après chacune) :

| Mutation | Rouges |
|---|---|
| M1 — garde des écritures retirée | 9 (dont création et réactivation mouvementées, groupes) |
| M2 — garde des désignations retirée | 6 (création, réactivation, `PUT`, route groupée) |
| M3 — articles jamais lus | 3 (création, réactivation, `PUT`) |
| M4 — création revenue à l'ancienne garde | 6 |
| M5 — réactivation revenue à l'ancienne garde | 4 |
| M6 — `PUT` sans désignation | 2 |
| M7 — message de réactivation sans le sous-compte | 5 |

Le verrou des réglages ajouté à la réactivation (C-1431-3) n'est couvert par aucune mutation : une course ne se prouve
pas par un test déterministe ; il est établi à la lecture de l'ordre des verrous.

**Gates réellement exécutés**

- `cargo fmt --check` et `cargo clippy --workspace --all-targets -D warnings` : verts (dans `scripts/test-fast.sh --ci`).
- Nextest complet (`scripts/test-fast.sh --ci`) au commit `76cbc615c` : 6999 exécutés, 6998 passés, 19 ignorés, **1 rouge** —
  `textes_coherents::aucun_chemin_de_station_dans_les_fichiers_versionnes`, sur le prompt de revue P1, qui citait des
  chemins de station ; corrigé en `69f5ba31b`.
- Nextest complet (`scripts/test-fast.sh --ci`, fmt et clippy compris) au **dernier commit de code `670a7b2f6`** :
  **7002 exécutés, 7002 passés**, 19 ignorés (`test-fast-ci-final.log`).
- Frontend au commit `01708501f` (dernier changement du frontend) : `npm run check` 0 erreur (30 avertissements
  préexistants), `lint-i18n-ownership` vert, `test:unit` 5765 tests / 329 fichiers verts.
- Manuel utilisateur : `make user` sur une copie, vert (3 passes) ; PDF aplati relu.
- `gate-ci` et E2E complet : **non exécutés** (réservés à l'orchestrateur avant la PR).

## Change Log

- 2026-11-28 — fiche et implémentation (`01708501f`).
- 2026-11-28 — revue de code, boucle : P1 (Opus, complète) 0 CRITICAL / 0 HIGH / 1 MEDIUM / 8 LOW → remédiation
  `69f5ba31b` (M-1 tranché par le choix C-1431-5, à confirmer par le mainteneur ; L-1 à L-8 corrigés dans la story) ;
  P2 (Sonnet, ciblée sur `69f5ba31b`) 0 au-dessus de LOW, 2 LOW → lot `670a7b2f6` (message d'un parent mouvementé
  neutre, test renommé ; les messages du lien bancaire, antérieurs à la story, laissés) ; P3 (Sonnet, ciblée sur
  `670a7b2f6`) 0 finding. Boucle close. Prompts versionnés : `fix-1431-review-prompt-p1.md`, `-p2.md`, `-p3.md`.
- 2026-11-28 — décisions du mainteneur : refus, non confirmation ; C-1431-5 confirmé par le principe « un compte de
  regroupement ne porte pas d'écritures » ; marche à suivre au manuel (C-1431-6). Commit `6faee0819` : section
  « Réorganiser le plan comptable » du manuel utilisateur (§ 6.5), renvoi dans les six messages des deux refus (quatre
  locales et textes de repli), phrase au guide de l'API. Revue P4 (Sonnet, ciblée sur `6faee0819`, prompt
  `fix-1431-review-prompt-p4.md`) : 0 au-dessus de LOW, 3 LOW, tous de texte — corrigés en un lot sans passe de plus
  (manuel : refus en devise et compte bancaire nommés ; fiche : limite des renvois en français), vérifiés par `grep`.
