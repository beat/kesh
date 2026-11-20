# Story 46-6a : balance par groupes — modèle, API et MCP

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-46-rapports-groupes.md` — D11 modifiée (Q6 : la balance dans le
périmètre), D1 à D7, D10 ; FR-46-6, FR-46-11 ; NFR-46-1 à NFR-46-4, NFR-46-6. Release visée **0.21.0** (prévision).
Fiche écrite le 2026-11-20 sur la remarque 2 de Guy (« décrire la story de la balance, laisser le solde en dernier »),
à valider en boucle. **Suit la 46-1** ; en parallèle des 46-2 à 46-4. Aucune migration.

**Découpage** : la balance par groupes (la « 46-6 » de la revue 1) touche plus de cinq modules (`kesh-report` modèle
et rendus, service et route, MCP, écran, i18n, manuel) : règle de splitting préventif, d'où **46-6a** (modèle, API,
MCP) et **46-6b** (écran, PDF, CSV). Le grand livre reste hors périmètre (Beat : « pas utile »).

## Story

En tant que comptable,
je veux la balance des comptes par groupes, avec pour chaque groupe ses sous-totaux d'ouverture, de mouvements et de
clôture, au niveau et avec les groupes ouverts ou fermés de mon choix,
afin de contrôler mes comptes au niveau des groupes, comme au bilan.

## Modules touchés (règle de splitting préventif)

Cinq : `kesh-report/trial_balance` (implémentation de `LigneAgregeable`, champs de structure), `kesh-report/groupes`
(clé de partition « nature », déjà prévue par la 46-1), `kesh-api/services/rapports` et `routes/reports`
(`GET /reports/trial-balance`), `kesh-api/mcp/outils/rapports.rs` (`kesh_balance`), la documentation
(`docs/api-external.md`, manuel d'administration, tableau MCP).

## Critères d'acceptation

1. **AC-1 — Réutilisation.** La balance emploie `kesh_report::groupes::regrouper` de la 46-1, sans copie :
   `TrialBalanceRow` implémente `LigneAgregeable` avec **cinq montants** (ouverture, débit, crédit, solde de la période,
   clôture) et leurs cinq montants en devise ; mêmes règles de groupes non affichés (Q2), de devise (Q3), de niveau, de
   règle d'ouverture, de position des sous-totaux et d'arbre défensif que la 46-1.
2. **AC-2 — Nature, au lieu de section.** La balance n'a pas de sections ; ses soldes sont signés du côté naturel de
   chaque compte (`trial_balance.rs:91`). La clé de partition est la **nature** (`opening::is_debit_natured`) : un
   groupe dont la descendance mêle comptes de nature débitrice et créditrice (`7 Résultats annexes` du plan PME :
   produits et charges) est **dissous**, ses enfants remontent ; tout sous-total additionne donc des soldes de même
   signe. Débit et crédit d'un groupe sont la somme exacte des mouvements de sa descendance.
3. **AC-3 — Inclusion.** La balance rend aussi des comptes actifs **à zéro** (règle d'inclusion,
   `trial_balance.rs:13-18`) : une ligne d'entrée à cinq montants nuls est permise ; un groupe fermé dont les cinq
   montants (et leurs montants en devise) sont nuls n'est pas affiché (AC-5 de la 46-1). La ligne calculée du résultat
   reporté reste **hors** des groupes, à sa place d'aujourd'hui.
4. **AC-4 — Totaux et contrôles inchangés.** `total_debit`, `total_credit`, `balanced`, `retained_earnings`,
   `opening_balanced` identiques à tout niveau ; la somme des débits (crédits) des lignes de profondeur 1 égale
   `total_debit` (`total_credit`) — test de propriété de la 46-1 étendu à la balance.
5. **AC-5 — API.** `GET /reports/trial-balance` (`routes/reports.rs:348`) accepte `detailLevel`, `subtotalPosition`,
   `open`, `closed`, mêmes refus et même validation (fonction du service de la 46-1) ; la réponse gagne `grouping` et
   `groupedRows` (une seule liste, `rows`), lignes `{ kind, depth, accountId, accountNumber, accountName, openingBalance,
   totalDebit, totalCredit, balance, closingBalance, currency?, …Currency?, open?, active? }`. Sans paramètre : JSON
   identique à l'octet (NFR-46-1). Une seule requête de plus (l'arbre), aucune sans paramètre.
6. **AC-6 — MCP.** `kesh_balance` (`ArgumentsBalance`, `deny_unknown_fields`) gagne les quatre arguments de la 46-4 ;
   en mode groupé, la **tranche** (`limit`, `offset`, défaut 23, maximum 42) porte sur les lignes groupées, `total` en
   compte le nombre, et les lignes plates sont vides ; totaux du rapport entier inchangés ; tailles re-mesurées avec les
   colonnes en devise et `LIMITE_DEFAUT_BALANCE` / `LIMITE_MAX_MCP_BALANCE` (`mcp/outils/rapports.rs:24-29`) revues
   ou confirmées, chiffre daté dans la doc du module. Sans argument neuf : sortie identique.
7. **AC-7 — Trace d'audit inchangée** (comme 46-1 AC-11) ; `services_motif.rs`, `audit_route_registry.rs` verts.
8. **AC-8 — Arrondi.** Aucun formatage neuf ; aucun `{:.2}` dans `kesh-report` (garde de la 48-11).
9. **AC-9 — Tests et documentation.** Unitaires (`LigneAgregeable` de la balance, groupe mixte dissous, compte à zéro,
   groupe fermé à zéro, devise) ; intégration (`reports_e2e.rs`) : refus, niveau 2 et `all`, égalité à l'octet sans
   paramètre, sonde négative (sans dissolution par nature, le test d'un groupe mixte rougit) ; `mcp_outils_e2e.rs`,
   `mcp_contrat`, `mcp_gardes_lexicales`, `mcp_matrice_manuel` verts ; `docs/api-external.md` (balance groupée) ;
   manuel d'administration (tableau MCP, `kesh_balance`), `.tex` seul.

## Tâches / sous-tâches

- [ ] **T0** — Reprise de la branche d'epic avec la 46-1 fusionnée.
- [ ] **T1** — `LigneAgregeable` pour `TrialBalanceRow` ; clé « nature » ; champs `grouping`, `grouped_rows` de
      `TrialBalance` (`Option`, `skip_serializing_if`) ; tests (AC 1 à 4, 8).
- [ ] **T2** — service `balance` (`services/rapports.rs:640`) et route (AC 5, 7).
- [ ] **T3** — MCP `kesh_balance` (AC 6).
- [ ] **T4** — tests d'intégration, documentation (AC 9) ; **T5** — gate complet, E2E complet au dernier commit de code.

## Dev Notes

- `crates/kesh-report/src/trial_balance.rs` (446 lignes) : `TrialBalance` (`:66`), `TrialBalanceRow` (`:91`, soldes du
  côté naturel), `RawRow`, `trial_balance_sql` (une requête, `ORDER BY a.number`), ligne calculée `retained_earnings`
  (`fetch_retained_earnings`, partagée avec le bilan), `is_opening_balanced`. `crate::opening` : `is_debit_natured`,
  `debit_sense`.
- Le grand livre (`general_ledger.rs`) n'est pas touché.
- **C-46-6a-1** : partition par nature (AC-2) plutôt que par type de compte : actif et charge (débiteurs) peuvent
  partager un groupe sans fausser une somme ; *écarté* : sommer en sens débit et marquer « D/C », illisible.
- **C-46-6a-2** : pagination MCP sur les lignes groupées (AC-6) : un groupe et son sous-total en pied peuvent tomber
  dans deux tranches — la description de l'outil le dit.

### References

Fiches 46-1 (§ *Le contrat*), 46-4 ; `CLAUDE.md` § *Le prompt d'une passe doit nommer le manuel*.

## Note de version

*(à écrire au développement)* — La balance des comptes se demande par groupes (API, MCP), avec les sous-totaux
d'ouverture, de mouvements et de clôture de chaque groupe ; sans paramètre, rien ne change.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-20 — Fiche écrite (remarque 2 de Guy) ; découpée de la « 46-6 » de la revue 1 par la règle de splitting
  préventif (46-6a modèle, API, MCP ; 46-6b écran, PDF, CSV). Neuf critères. `drafted`.
