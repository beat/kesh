# Story 46-6 : balance par groupes de comptes

Status: draft

*Ajoutée le 2026-11-19, sur la réponse de Beat à Q6 (« balance : maintenant ; grand livre : pas utile »). Epic 46, D11
(modifiée). Suit 46-1 ; peut se faire en parallèle de 46-2 à 46-4. Aucune migration. Approbation de Guy attendue.*

## Story

En tant que comptable, je veux la balance par groupes de comptes, avec un sous-total par groupe et un niveau de détail
choisi, afin de contrôler mes comptes au niveau des groupes comme dans le bilan.

## Critères d'acceptation (esquisse)

1. **AC-1 — Réutilisation.** La balance emploie la fonction pure `regrouper` de 46-1 (sous-totaux de débit, de crédit
   et de solde par groupe) ; mêmes règles de groupes non affichés (Q2) et de devise (Q3).
2. **AC-2 — Écran.** Niveau, hiérarchie visible (accordéons ouverts, contrôles tout ouvrir / fermer, hiérarchie
   masquée / montrée, Q10) et position des sous-totaux, comme 46-2.
3. **AC-3 — PDF et CSV.** Fidèles à l'écran, comme 46-3 ; sans paramètre, identiques à l'actuel (NFR-46-1).
4. **AC-4 — API et MCP.** `detailLevel` et position des sous-totaux sur la route de la balance et son outil MCP ;
   entièrement pilotable sans écran.
5. **AC-5 — Totaux.** Total débit = total crédit, inchangé à tout niveau ; somme des sous-totaux de premier niveau =
   total.
6. **AC-6 — Grand livre.** Hors périmètre (Beat : pas utile) ; inchangé.
7. **AC-7 — Tests et manuel.** Unitaires, E2E, captures, manuel FR (`.tex`), `docs/api-external.md`.

## Tâches principales

- [ ] Relevé du code de la balance (`kesh-report`, route, écran, MCP) à la validation de la fiche
- [ ] Branchement de `regrouper`, rendus, route, MCP
- [ ] Tests, captures, manuel

## Fichiers

À relever à la validation (balance : `kesh-report`, `routes/reports.rs`, `frontend/src/lib/features/reports/`,
`mcp/outils/rapports.rs`).

## Note de version

*(à écrire au développement)*
