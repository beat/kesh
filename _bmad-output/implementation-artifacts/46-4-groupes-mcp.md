# Story 46-4 : outils MCP du bilan et du compte de résultat par groupes

Status: drafted

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
   `subtotalPosition` (`"header"` | `"footer"`), `open`, `closed` (tableaux d'entiers ≥ 1, au plus 500).
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
   - *Refus* : les bornes (1–20, ≥ 1, ≤ 500), « `subtotalPosition`, `open` ou `closed` sans `detailLevel` » et « un
     identifiant dans `open` et `closed` » sont celles de la 46-1 ; un refus de service (`AppError::Validation`) rend,
     par `depuis_app_error`, le texte `(demande)` invalide, **sans nommer l'argument** (`mcp/textes.rs:315`) : la
     description de l'outil énonce donc ces contraintes.
   - *Schéma* : `SCHEMA_PERIODE` ne sert qu'à ces deux outils (la balance a son schéma en ligne,
     `rapports.rs:208-209`) ; il est remplacé par `SCHEMA_PERIODE_GROUPES`, `additionalProperties: false` gardé :
     `detailLevel` = `{"oneOf":[{"type":"integer","minimum":1,"maximum":20},{"type":"string","enum":["all"]}]}`,
     `subtotalPosition` = enum, `open` / `closed` = `{"type":"array","items":{"type":"integer","minimum":1},"maxItems":500}`.
2. **AC-2 — Sans argument neuf.** La réponse est **identique** à celle d'aujourd'hui (même contenu, même texte rendu) :
   un test compare le texte rendu à un **littéral écrit au commit de base** (avant T1) sur la société d'essai, pour
   chacun des deux outils — non à la sortie de la fonction qui le produit (un « avant et après » ne se rejoue pas dans
   un seul commit).
3. **AC-3 — Réponse groupée.** Avec `detailLevel`, la sortie gagne un champ `groupes` (`null` sinon, convention
   C-37-9) : `{ niveau, tousLesNiveaux, positionSousTotaux, profondeurMax, ouverts, fermes, <sections> }` — l'écho
   `grouping` de la 46-1 (`niveau` à `null` et `tousLesNiveaux` à `true` pour `"all"` : un champ de `sortie!` ne porte
   qu'un type) — et, par section, les lignes groupées (`actifs`, `passifs`, `fondsPropres` ; `produits`, `charges`).
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
   la clé `open`, qui dirait une liste). `Resume` n'a que `entier`, `date`, `booleen`, `code`.
5. **AC-5 — Tailles mesurées, et seuil écrit.** *Le plafond est celui de la réponse (24 576 o, refus pour taille) ;
   le bilan plat tient 57 comptes en 20 156 o (`rapports.rs:7-13`), soit ≈ 350 o la ligne, et une ligne groupée est du
   même ordre : **le plan PME complet au niveau `all` en `footer` (86 comptes, une quinzaine de groupes, autant de
   sous-totaux) dépasse le plafond** — un critère « sous le plafond » à ce niveau est inatteignable.* À T3, des
   mesures sur une fixture qui pose des `parent_id` par SQL (les fixtures des tailles créent des comptes sans parent,
   donc des racines) : (a) plan PME complet, comptes en devise, **niveau 2**, `header` puis `footer` : réponse **sous
   23 552 o** (marge des autres outils) ; (b) niveau `all`, `footer` : la taille de l'élément et le **nombre de
   lignes groupées** au-delà duquel la réponse est refusée pour taille (même refus que `plafond_au_maximum`) ; ce
   nombre s'écrit dans la doc du module (`rapports.rs:1-13`, avec sa date) et dans les descriptions des deux outils
   (« au-delà d'environ N lignes, choisissez un niveau plus bas ») ; (c) un test affirme le refus au-delà du seuil.
   Le seuil « environ 60 comptes » du bilan plat est conservé.
6. **AC-6 — Gardes réelles.** Verts : `mcp_contrat` (forme de `inputSchema`, K7), `mcp_gardes_lexicales`
   (`champ_sur_en_un_lieu` : aucun `ChampSur` ni `scelle::` dans `rapports.rs` ; `sorties_sans_serialisation_parallele` :
   le miroir ne sérialise que par `donnees` ; `donnees_construites_par_la_politique_seule`), `mcp_matrice_manuel`
   (n'apparie que **portée × outil** : la phrase du manuel n'est contrôlée par aucune garde, la revue la lit),
   `mcp_outils_e2e` (`bornes_des_schemas` inchangé : le bilan et le compte de résultat ne sont pas paginés) ; un cas
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
`demande`. Les erreurs de service passent par `depuis_app_error` (`mcp/textes.rs:311`), dont `Validation` rend un
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
