# Story 46-4 : outils MCP par groupes

Status: draft

*Revue de Beat (2026-11-19) : approuvée (Q5 : le MCP en a besoin ; tout pilotable par l'API et le MCP) ; approbation de Guy attendue.*

*Esquisse du 2026-11-18. Epic 46, D8. Suit 46-1. Sous réserve de Q5 (le MCP en a-t-il besoin ?). Aucune migration.*

## Story

En tant qu'utilisateur d'un assistant branché sur Kesh, je veux demander le bilan ou le compte de résultat par groupes,
afin d'obtenir une synthèse au bon niveau sans tout le détail.

## Critères d'acceptation (esquisse)

1. **AC-1 — Argument.** `kesh_bilan` et `kesh_compte_de_resultat` acceptent `detailLevel` facultatif (schéma d'entrée,
   `deny_unknown_fields` conservé) ; absent → réponse identique.
2. **AC-2 — Réponse.** Lignes groupées en `TexteNonFiable` pour numéro et nom, montants `Montant`, devise `Code`.
3. **AC-3 — Position des sous-totaux (Beat).** Argument facultatif pour la position des sous-totaux (nom commun avec
   l'API, 46-1 AC-8) ; tout ce que l'écran et le PDF permettent se pilote par le MCP. La balance par groupes (46-6)
   est exposée de la même façon.
4. **AC-4 — Gardes.** `mcp_contrat`, `mcp_gardes_lexicales`, `mcp_matrice_manuel` verts ; tailles re-mesurées.
5. **AC-5 — Doc.** Manuel d'administration (outils MCP) et `docs/api-external.md` si concerné.

## Tâches principales

- [ ] Arguments, conversion des lignes groupées, description de l'outil
- [ ] Tests MCP ; documentation

## Fichiers

`crates/kesh-api/src/mcp/outils/rapports.rs` ; `crates/kesh-api/tests/mcp_*.rs` ; manuel d'administration FR (`.tex`).

## Note de version

*(à écrire au développement)*
