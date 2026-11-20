# Story 46-4 : outils MCP du bilan et du compte de résultat par groupes

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-46-rapports-groupes.md` — D8, Q5 ; FR-46-4, FR-46-11 ; NFR-46-6. Release
visée **0.21.0** (prévision). Approuvée par Beat et par Guy (revue 1, 2026-11-19) ; fiche écrite le 2026-11-20, à
valider en boucle. **Suit la 46-1.** Aucune migration. La balance par groupes au MCP est la 46-6a.

## Story

En tant qu'utilisateur d'un assistant branché sur Kesh,
je veux demander le bilan ou le compte de résultat par groupes, au niveau, avec les groupes ouverts ou fermés et la
place des sous-totaux de mon choix,
afin d'obtenir une synthèse au bon niveau, comme à l'écran, sans tout le détail.

## Modules touchés (règle de splitting préventif)

Trois : `kesh-api/mcp/outils/rapports.rs`, les tests MCP (`crates/kesh-api/tests/mcp_*.rs`), le manuel
d'administration (`docs/manual/fr/admin-manual.tex`, tableau des outils MCP).

## Critères d'acceptation

1. **AC-1 — Arguments.** `kesh_bilan` et `kesh_compte_de_resultat` (`ArgumentsPeriode`, `deny_unknown_fields` conservé)
   gagnent quatre arguments facultatifs aux **noms de l'API** : `detailLevel` (entier 1–20 ou `"all"`),
   `subtotalPosition` (`"header"` | `"footer"`), `open`, `closed` (tableaux d'entiers ≥ 1, au plus 500). Le schéma
   d'entrée (`SCHEMA_PERIODE`, ou un schéma propre aux deux outils si la balance doit garder le sien) les décrit,
   `additionalProperties: false` gardé. Validation par **la fonction du service de la 46-1** ; un refus rend le
   message d'erreur MCP habituel (`depuis_app_error`).
2. **AC-2 — Sans argument neuf.** La réponse est **identique** à celle d'aujourd'hui (même contenu, même texte rendu) :
   test qui compare la sortie avant et après sur la société d'essai.
3. **AC-3 — Réponse groupée.** Avec `detailLevel`, la sortie gagne un champ `groupes` (`null` sinon, convention
   C-37-9) : l'écho `grouping` de la 46-1 et, par section, les lignes groupées — `genre` (`groupe`, `compte`,
   `propre`, `soustotal`), `profondeur`, `compteId`, `numero` et `nom` en `TexteNonFiable`, `solde` en `Montant`,
   `devise` en `Code<Devise>` et `soldeDevise` (`null` hors devise unique), `ouvert` pour un groupe. Les listes plates
   (`actifs`, `passifs`, `fondsPropres`, `produits`, `charges`) sont alors **vides** : la réponse reste sous le plafond
   de taille (C-32-9-8) ; la description de l'outil le dit, et dit que la règle de rendu (montant sur le titre ou sur
   le sous-total) est celle de l'écran. Totaux, report, résultat et équation : inchangés.
4. **AC-4 — Résumé d'appel.** Le résumé (`resume_periode`, `Resume::outil`) porte `detailLevel` et
   `subtotalPosition` quand ils sont donnés, et le **nombre** d'identifiants de `open` / `closed` (pas la liste).
5. **AC-5 — Tailles re-mesurées.** Un test mesure la réponse groupée la plus grosse attendue (plan PME complet, niveau
   `all`, `footer`, comptes en devise) sous le plafond de 24 576 o ; le chiffre mesuré s'écrit dans la doc du module
   (`rapports.rs:1-13`) avec sa date.
6. **AC-6 — Gardes.** `mcp_contrat`, `mcp_gardes_lexicales` (longueur des descriptions comprise),
   `mcp_matrice_manuel`, `mcp_outils_e2e` verts ; un cas groupé ajouté à `mcp_outils_e2e.rs`.
7. **AC-7 — Documentation.** Manuel d'administration FR, tableau des outils MCP (troisième colonne de `kesh_bilan` et
   `kesh_compte_de_resultat`) : une phrase sur les arguments de groupement ; `.tex` seul.

## Tâches / sous-tâches

- [ ] **T1 — arguments et schéma** (AC 1, 4).
- [ ] **T2 — sortie** (AC 2, 3) : `sortie!` d'une ligne groupée et de `groupes` ; conversion depuis `LigneGroupee`.
- [ ] **T3 — tests** (AC 2, 5, 6) ; **T4 — manuel** (AC 7).
- [ ] **T5 — gate complet** ; E2E complet au dernier commit de code.

## Dev Notes

### État du code (relevé le 2026-11-20 à `29b9ec27`)

`crates/kesh-api/src/mcp/outils/rapports.rs` (372 lignes) : `ArgumentsPeriode` (`:38`, partagé par bilan et compte de
résultat), `ArgumentsBalance` (`:48`, propre à la balance, paginée), `SCHEMA_PERIODE` (`:59`), `LigneDeSolde`
(`:66`), `Bilan` (`:82`), `CompteDeResultat` (`:97`), `ligne`/`lignes` (`:160-178`), `resume_periode`, `demande`.
Plafond de taille et seuil d'« environ 60 comptes » dans la doc du module. La 47-3, si fusionnée, ne change que les
descriptions de ces deux outils (une phrase sur le miroir) : garder les deux phrases, longueur contrôlée par
`mcp_gardes_lexicales`.

### Choix et écarts

- **C-46-4-1** : listes plates vidées en mode groupé, pour la taille ; *écarté* : les deux à la fois (risque de
  dépasser le plafond sur un plan complet).
- **C-46-4-2** : `open`/`closed` en tableaux JSON au MCP (listes séparées par des virgules à l'API REST) : même
  sémantique, forme idiomatique de chaque canal.

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
