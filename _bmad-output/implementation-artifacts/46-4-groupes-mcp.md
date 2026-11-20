# Story 46-4 : outils MCP du bilan et du compte de résultat par groupes

Status: ready-for-dev

Epic : `_bmad-output/planning-artifacts/epic-46-rapports-groupes.md` — D8, Q5 ; FR-46-4, FR-46-11 ; NFR-46-6. Release
visée **0.21.0** (prévision). Approuvée par Beat et par Guy (revue 1, 2026-11-19) ; fiche écrite le 2026-11-20, à
valider en boucle. **Suit la 46-1 ; précède la 46-6a.** Aucune migration. La balance par groupes au MCP est la 46-6a.

## Story

En tant qu'utilisateur d'un assistant branché sur Kesh,
je veux demander le bilan ou le compte de résultat par groupes, au niveau, avec les groupes ouverts ou fermés et la
place des sous-totaux de mon choix,
afin d'obtenir une synthèse au bon niveau, comme à l'écran, sans tout le détail.

## Modules touchés (règle de splitting préventif)

Trois : `kesh-api/mcp/outils/rapports.rs`, les tests MCP (`crates/kesh-api/tests/mcp_outils_e2e.rs`, `mcp_contrat.rs`),
le manuel d'administration (`docs/manual/fr/admin-manual.tex`, tableau des outils MCP). `mcp/sortie.rs` n'est **pas**
touché : l'énumération de genre se déclare dans `rapports.rs` par `champ_enum!`, comme `postes_ouverts.rs:75`.

**Ordre** : suit la 46-1 ; **précède la 46-6a**, dont le MCP de la balance reprend, sans copie, les arguments, le
schéma et la fonction de conversion posés ici (validation P1).

## Critères d'acceptation

1. **AC-1 — Arguments.** `kesh_bilan` et `kesh_compte_de_resultat` (`ArgumentsPeriode`, `deny_unknown_fields`
   conservé) gagnent quatre arguments facultatifs aux **noms de l'API** : `detailLevel` (entier 1–20 ou `"all"`),
   `subtotalPosition` (`"header"` | `"footer"`), `open`, `closed` (tableaux d'entiers, au plus 500).
   - *Lecture des arguments* : `arguments()` n'est que `serde_json::from_value` et rend `(arguments)` invalide sans
     jamais recopier le message de `serde` ; le schéma JSON n'est **pas** appliqué à l'exécution. `detailLevel` se lit
     donc par un type écrit à la main (entier ou `"all"`, tout autre JSON refusé), `open` et `closed` par
     `Option<Vec<i64>>`. **Aucun schéma MCP existant ne porte de tableau ni de `oneOf`** (relevé du 2026-11-20,
     `git grep` sur `src/mcp`) : c'est une première, que `mcp_contrat` (clause K7, forme de `inputSchema`) doit
     continuer d'accepter — relevé à T1.
   - *Pas de `#[serde(flatten)]`* : il est incompatible avec `deny_unknown_fields`. Les quatre champs se déclarent dans
     `ArgumentsPeriode` (et, à la 46-6a, dans `ArgumentsBalance`) ; **une seule fonction** les convertit en la demande
     typée de la 46-1 (`DemandeGroupes`) et en appelle la validation — **sans** passer par les chaînes séparées par des
     virgules de l'API REST (la 46-1 expose une entrée typée, à côté de l'analyse des chaînes de la route).
   - *Refus* : les bornes (`detailLevel` 1–20, au plus 500 identifiants), « `subtotalPosition`, `open` ou `closed`
     sans `detailLevel` » et « un identifiant dans `open` et `closed` » sont celles de la 46-1 (entrée typée, T3 de la
     46-1). Un identifiant nul ou négatif n'est **pas** un refus de la 46-1 : il est ignoré comme un identifiant inconnu
     (le `minimum: 1` du schéma n'est qu'une indication, le schéma n'étant pas appliqué). Tout refus — `serde`
     (`(arguments)`) ou service (`AppError::Validation` → `(demande)`, `mcp/textes.rs:315`) — rend à l'agent le même
     texte fixe « Arguments invalides. » (`textes.rs:127`, `:249-252`), qui ne nomme jamais l'argument : la description
     de l'outil énonce donc ces contraintes ; aucune validation locale de plus n'est écrite dans l'outil (une seule
     validation, celle de la 46-1).
   - *Schéma* : `SCHEMA_PERIODE` ne sert qu'à ces deux outils (la balance a son schéma en ligne,
     `rapports.rs:208-209`) ; il est remplacé par `SCHEMA_PERIODE_GROUPES`, `additionalProperties: false` gardé :
     `detailLevel` = `{"oneOf":[{"type":"integer","minimum":1,"maximum":20},{"type":"string","enum":["all"]}]}`,
     `subtotalPosition` = enum, `open` / `closed` = `{"type":"array","items":{"type":"integer","minimum":1},"maxItems":500}`.
2. **AC-2 — Sans argument neuf.** La réponse est **identique** à celle d'aujourd'hui (même contenu, même texte rendu) :
   un test compare le texte rendu à un **littéral écrit au commit de base** (avant T1) sur la société d'essai, pour
   chacun des deux outils — non à la sortie de la fonction qui le produit (un « avant et après » ne se rejoue pas dans
   un seul commit). ⚠️ La société d'essai (`societe_piegee`) est datée du jour de l'horloge de test
   (`tests/common/jour.rs`, `aujourd_hui_test`), que la recette fait varier (`KESH_TEST_INSTANT`) : les dates de la
   période (et tout identifiant qui ne serait pas stable) s'écrivent dans le littéral en **jetons** (`{debut}`,
   `{fin}`), remplacés par les valeurs de `a.p` avant la comparaison ; tout le reste est littéral.
3. **AC-3 — Réponse groupée.** Avec `detailLevel`, la sortie gagne un champ `groupes` (`null` sinon, convention
   C-37-9) : `{ niveau, tousLesNiveaux, positionSousTotaux, profondeurMax, ouverts, fermes, <sections> }` — l'écho
   `grouping` de la 46-1 (`niveau` à `null` et `tousLesNiveaux` à `true` pour `"all"` : un champ de `sortie!` ne porte
   qu'un type ; `positionSousTotaux` est, comme `genre`, un miroir admis par `champ_enum!`, valeurs `header`/`footer`) —
   et, par section, les lignes groupées (`actifs`, `passifs`, `fondsPropres` ; `produits`, `charges`).
   Une ligne : `genre`, `profondeur`, `compteId`, `numero` et `nom` en `TexteNonFiable`, `solde` en `Montant`, `devise`
   en `Code<Devise>` et `soldeDevise` (`null` hors devise unique), `ouvert` (`null` hors groupe).
   - **`genre` suit la convention des outils** (`postes_ouverts.rs:75-110`) : un **miroir** local, énumération sans
     champ qui dérive `Serialize` en `camelCase`, construite par un `match` exhaustif depuis le genre de la 46-1, admise
     par `champ_enum!` — valeurs `group`, `account`, `own`, `subtotal`, **celles de l'API** (un seul vocabulaire ; le
     français `groupe|compte|propre|soustotal` est celui du CSV, 46-3). `sortie!` n'admet aucun attribut de champ.
   - La sortie lit l'enveloppe `AvecGroupes<…>` de la 46-1 (C-46-1-4) : `grouping` et `groupedRows` d'elle, les
     listes et totaux plats de `.rapport`.
   - Les lignes de groupée ne portent ni `typeDeCompte`, ni `actif`, ni `role` : le détail d'un compte se lit sans
     `detailLevel` (listes plates). *Si la 47-3 est fusionnée*, une ligne de compte miroir porte le champ de miroir de
     `LigneDeSolde` sous le même nom (à relever à T0).
   - Les listes plates (`actifs`, `passifs`, `fondsPropres`, `produits`, `charges`) sont alors **vides** (`[]`) : la
     réponse reste sous le plafond de taille (C-32-9-8) ; la description de l'outil le dit, dit que la règle de rendu
     (montant sur le titre ou sur le sous-total) est celle de l'écran, et énonce les contraintes des arguments (AC-1).
     Totaux, report, résultat et équation : inchangés.
4. **AC-4 — Résumé d'appel.** Le résumé (`resume_periode`, `Resume::outil`) porte `detailLevel` (`entier`, ou
   `code("detailLevel", "all")`) et `subtotalPosition` (`code`, chaînes statiques) quand ils sont donnés, et le
   **nombre** d'identifiants de `open` / `closed` sous les clés `openCount` / `closedCount` (pas la liste, et pas sous
   la clé `open`, qui dirait une liste). Les méthodes de `Resume` (`sortie.rs:404-458`) sont `entier`, `date`,
   `booleen`, `code` et `recherche_fournie` ; seules les quatre premières servent ici.
5. **AC-5 — Tailles mesurées, et seuil écrit.** *Le plafond est celui de la réponse (24 576 o, refus pour taille,
   `MCP_ENVOI_OCTETS_MAX`) ; le bilan plat tient 57 comptes en devise en 20 156 o (`rapports.rs:7-13`), soit ≈ 350 o
   la ligne, et une ligne groupée est du même ordre. Le plan PME (`pme.json`, 86 comptes) ne se rend pas en entier
   dans un seul rapport : le bilan n'en porte que les 31 comptes de bilan **feuilles** (18 actifs, 13 passifs), le compte de résultat
   les 34 de produits et de charges feuilles (9 et 25) — les 21 autres sont des groupes, qui ne portent pas d'écriture
   propre ici (relevé du 2026-11-20 sur `pme.json`) —, et seulement ceux à solde non nul. Calcul du 2026-11-20 (tous les comptes à solde non nul, `footer`,
   `2 Passifs` dissous au bilan, `6`, `7`, `70`, `8`, `80` au compte de résultat) : **43 lignes groupées au bilan et 52
   au compte de résultat à `all`**, soit ≈ 15 et ≈ 18 Ko — sous le plafond attendu, à mesurer. (La remédiation P1
   affirmait le contraire en comptant les 86 comptes dans un seul rapport : corrigé en P2.)* À T3, des mesures sur
   une fixture qui pose des `parent_id` par SQL (les fixtures des tailles créent des comptes sans parent, donc des
   racines) : (a) plan PME complet, tous ses comptes de bilan, de produits et de charges à solde non nul **en
   devise**, **niveaux 2 et `all`**, `header` puis `footer` : réponse **sous 23 552 o** (marge des autres outils) ; si
   `all` en `footer` dépasse, le chiffre mesuré s'écrit et la description le dit ; (b) sur une fixture synthétique
   (comptes en devise, noms de 34 caractères, groupes posés par SQL), niveau `all`, `footer` : la taille de l'élément
   et le **nombre de lignes groupées** au-delà duquel la réponse est refusée pour taille (même refus que
   `plafond_au_maximum`) ; ce nombre s'écrit dans la doc du module (`rapports.rs:1-13`, avec sa date) et dans les descriptions des deux outils
   (« au-delà d'environ N lignes, choisissez un niveau plus bas » : N n'est connu qu'après la mesure, les descriptions
   se complètent donc à T3) ; (c) un test affirme le refus au-delà du seuil.
   Le seuil « environ 60 comptes » du bilan plat est conservé.
6. **AC-6 — Gardes réelles.** Verts : `mcp_contrat` (forme de `inputSchema`, K7), `mcp_gardes_lexicales`
   (`champ_sur_en_un_lieu` : aucun `ChampSur` ni `scelle::` dans `rapports.rs` ; `sorties_sans_serialisation_parallele` :
   le miroir ne sérialise que par `donnees` ; `donnees_construites_par_la_politique_seule`), `mcp_matrice_manuel`
   (n'apparie que **portée × outil** : la phrase du manuel n'est contrôlée par aucune garde, la revue la lit),
   `mcp_outils_e2e` (`bornes_des_schemas` inchangé : le bilan et le compte de résultat ne sont pas paginés ;
   `proprietes_du_schema_acceptees` exige que l'argument « complet » de `TEMOINS`
   (`tests/common/mcp_pieges.rs:186`) nomme **chaque** propriété du schéma et soit servi : celui de `kesh_bilan` et de
   `kesh_compte_de_resultat` gagne `detailLevel`, `subtotalPosition`, `open` et `closed` — deux listes disjointes —,
   `mcp_enveloppe_e2e` parcourt `TEMOINS` mais ne lit que ses **jeux** (`for (outil, jeux, _)`, `:654`, `:974`, `:1291`),
   jamais l'argument « complet » : il n'est pas touché) ; un cas
   groupé ajouté à `mcp_outils_e2e.rs` (arguments valides, un refus par contrainte de l'AC-1, `groupes` non nul, listes
   plates vides, témoin hors groupe où `groupes` vaut `null`). *(« Longueur des descriptions » : aucune garde ne la
   contrôle — retiré, validation P1.)*
7. **AC-7 — Documentation.** Manuel d'administration FR, tableau des outils MCP (troisième colonne de `kesh_bilan` et
   `kesh_compte_de_resultat`) : une phrase sur les arguments de groupement et le seuil de l'AC-5 ; `.tex` seul.

## Tâches / sous-tâches

- [ ] **T0** — Reprise de la branche d'epic (46-1 fusionnée ; `Rapport<AvecGroupes<…>, U>`, appelants du MCP à `:305`, `:357` selon la 46-1) ; état de la 47-3 (champ de miroir) ; ligne lue citée.
- [ ] **T1 — arguments et schéma** (AC 1, 4) : type de `detailLevel`, tableaux, `SCHEMA_PERIODE_GROUPES`, fonction
      de conversion unique (réutilisée par la 46-6a).
- [ ] **T2 — sortie** (AC 2, 3) : `GenreLigne` (miroir + `champ_enum!`), `sortie!` de la ligne groupée et de `groupes`,
      conversion depuis la ligne de la 46-1 ; littéral de base du témoin (AC-2) **écrit avant T1**.
- [ ] **T3 — tests** (AC 2, 5, 6) ; **T4 — manuel** (AC 7).
- [ ] **T5 — gate complet** ; E2E complet au dernier commit de code.

## Dev Notes

### État du code (relevé le 2026-11-20 à `origin/main` `d29aa557`)

`crates/kesh-api/src/mcp/outils/rapports.rs` (372 lignes) : `ArgumentsPeriode` (`:40`, bilan et compte de résultat
seuls), `ArgumentsBalance` (`:50`, propre à la balance, paginée, schéma en ligne `:208-209`), `SCHEMA_PERIODE` (`:60`),
`LigneDeSolde` (`:69`), `Bilan` (`:87`), `CompteDeResultat` (`:103`), `ligne`/`lignes` (`:161-177`), `resume_periode`,
`demande`. ⚠️ `ArgumentsPeriode` n'est pas propre au bilan et au compte de résultat : `kesh_balance` en **construit**
une (`:225-229`) depuis `ArgumentsBalance` pour réutiliser `resume_periode` et `demande`. Ajouter quatre champs à
`ArgumentsPeriode` oblige donc à les poser à `None` à `:225` dans cette story ; à la 46-6a, la balance y recopie ses
quatre champs, et la fonction de conversion unique (T1), qui lit `&ArgumentsPeriode`, la sert sans autre code (KISS). Les erreurs de service passent par `depuis_app_error` (`mcp/textes.rs:311`), dont `Validation` rend un
texte sans l'argument. La 47-3, si fusionnée, ne change que les descriptions de ces deux outils (une phrase sur le
miroir) et ajoute un champ de miroir : garder les deux phrases.

### Choix et écarts

- **C-46-4-1** : listes plates vidées en mode groupé, pour la taille ; *écarté* : les deux à la fois (le plat seul
  atteint ≈ 60 lignes).
- **C-46-4-2** : `open`/`closed` en tableaux JSON au MCP (listes séparées par des virgules à l'API REST) : même
  sémantique, forme idiomatique de chaque canal ; la validation se fait sur des valeurs typées, une seule fois.
- **C-46-4-3** : **pas de pagination** du bilan ni du compte de résultat groupés (comme à plat) : c'est le niveau
  qui borne la taille, et il est l'objet de la story ; le refus pour taille dit déjà de le baisser. *Écarté* :
  `limit`/`offset` sur les lignes groupées — un groupe et son sous-total en pied tomberaient dans deux tranches, pour
  une réponse qui reste petite aux niveaux utiles. (La balance, dont la ligne pèse 540 o, est tranchée : 46-6a.)
- **C-46-4-4** : le genre de ligne reprend les codes de l'API, non le français du CSV.

### References

Fiche 46-1 (§ *Le contrat*) ; `CLAUDE.md` § *Le prompt d'une passe doit nommer le manuel*.

## Note de version

*(à écrire au développement)* — Les outils MCP `kesh_bilan` et `kesh_compte_de_resultat` acceptent le niveau de
détail, les groupes ouverts ou fermés et la place des sous-totaux.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-20 — Fiche écrite depuis l'esquisse : arguments aux noms de l'API, sortie groupée sous le plafond de taille,
  résumé d'appel ; la balance passe à la 46-6a. Sept critères. `drafted`.
- 2026-11-20 — **validation P1 (Sonnet 5.5) : 0 C, 1 H, 5 M, 1 L, remédiés.** H : l'AC-5 exigeait le plan PME complet au
  niveau `all` sous le plafond de 24 576 o — inatteignable (≈ 350 o la ligne, 57 comptes = 20 156 o) : mesures par
  niveau, seuil écrit et refus affirmé. M : vocabulaire de `genre` aligné sur les miroirs des outils (`champ_enum!`,
  codes de l'API) ; lecture des arguments (aucun tableau ni `oneOf` au MCP, `serde` seul, pas de `flatten` avec
  `deny_unknown_fields`, refus sans nom d'argument) ; garde « longueur des descriptions » inexistante (gardes réelles
  listées, `mcp_matrice_manuel` n'apparie que portée × outil) ; témoin « avant et après » devenu littéral du commit de
  base ; `SCHEMA_PERIODE` ne sert pas la balance (schéma renommé). Propagation : **46-4 avant 46-6a** déclarée (finding de la 46-6a). L :
  clés du résumé d'appel. Axes exercés : code de `origin/main` (rapports.rs, sortie.rs, outil.rs, textes.rs, tests
  MCP), concordance 46-1 à 46-3, 46-6a, manuel d'administration ; non exercés : compilation, mesures de taille réelles.
- 2026-11-20 — **validation P2 (Claude Opus 5.5, contexte frais, au code de `origin/main` `d29aa557`) : 0 C, 0 H, 3 M,
  4 L, remédiés.** M : prémisse fausse de la remédiation P1 à l'AC-5 (« le plan PME complet au niveau `all` dépasse le
  plafond », en comptant les 86 comptes dans un seul rapport) — le bilan n'en porte que 31, le compte de résultat 34 ;
  calcul : 43 et 52 lignes groupées à `all` en `footer`, sous le plafond attendu → mesures aux niveaux 2 et `all` sur
  le plan PME, seuil sur fixture synthétique ; témoin littéral de l'AC-2 dépendant de la date du jour de l'horloge de
  test (`societe_piegee`, `aujourd_hui_test`, `KESH_TEST_INSTANT`) → dates en jetons ; `proprietes_du_schema_acceptees`
  et l'argument « complet » de `TEMOINS` (`tests/common/mcp_pieges.rs:186`) omis des gardes (AC-6). L : `≥ 1` n'est pas
  un refus de la 46-1 (identifiant ignoré) ; tout refus rend « Arguments invalides. » (`textes.rs:127`), sans argument
  nommé ; `positionSousTotaux` miroir `champ_enum!` ; `ArgumentsPeriode` construit aussi par `kesh_balance`
  (`rapports.rs:225-229`) → champs à `None`, et conversion unique sur `&ArgumentsPeriode` réutilisée par la 46-6a ;
  `Resume` a aussi `recherche_fournie`. Rapport : `kesh-gate-logs/epic-46/validation/46-4-P2.md`.
- 2026-11-20 — **validation P3 ciblée (Claude Sonnet 5.5, contexte frais, sur le seul commit `bb673929`, au code de
  `origin/main`) : 0 C, 0 H, 0 M, 3 L, remédiés.** Recompté et confirmé : 31 et 34 feuilles du plan PME (`pme.json`, 86
  comptes dont 21 groupes), 43 et 52 lignes groupées à `all` en `footer` (script d'arbre avec dissolution : 6, 7, 70, 8,
  80 et `2`), `MCP_ENVOI_OCTETS_MAX` 24 576 (`envois_mcp.rs:55`), marge 23 552 (`plan_comptable.rs:7`),
  `aujourd_hui_test` (`tests/common/jour.rs:14`), `proprietes_du_schema_acceptees` (`mcp_outils_e2e.rs:234`),
  `TEMOINS` (`mcp_pieges.rs:186`), `depuis_app_error` (`textes.rs:311`). L : `mcp_enveloppe_e2e` ne lit pas l'argument
  « complet » de `TEMOINS` (seulement ses jeux) — l'affirmation de la P2 retirée ; « 31 comptes de bilan » précisé
  (feuilles) ; descriptions de l'AC-5 (b) complétées à T3. Rapport : `kesh-gate-logs/epic-46/validation/46-4567-P3-ciblee.md`.
