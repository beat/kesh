# Story 46-1 : arbre des groupes — modèle et API

Status: draft

*Esquisse du 2026-11-18, branche `epic/46-rapports-groupes`. Epic 46, D1–D8, D10. Aucune migration attendue. À
valider en boucle après la revue de Beat. Part de `main` après la fusion de `fix/bilan-resultat-reporte`.*

## Story

En tant que comptable, je veux que le bilan et le compte de résultat puissent se calculer par groupes de comptes, avec
un sous-total par groupe et un niveau de détail choisi, afin de présenter mes comptes comme dans mon plan.

## Critères d'acceptation (esquisse)

1. **AC-1 — Arbre.** Une fonction pure `kesh_report::groupes::regrouper(comptes, soldes, niveau)` construit l'arbre
   depuis `parent_id` (D1), ordonne les frères par numéro (D10), et rend une liste ordonnée de lignes
   `{ kind: group|account|own, depth, … }`.
2. **AC-2 — Sous-totaux.** Sous-total d'un groupe = Σ de ses descendants à solde non nul, solde propre compris (D5) ;
   somme des lignes de premier niveau = total de la section (NFR-46-2).
3. **AC-3 — Niveau.** `1`…`n` replie les comptes plus profonds dans leur ancêtre ; `all` donne chaque compte (D3).
4. **AC-4 — Sections.** Le bilan garde Actifs, Dettes, Fonds propres et les deux lignes calculées ; un groupe à cheval
   sur deux sections n'est pas affiché, ses enfants remontent (D4, sous réserve de Q4). Même règle au compte de
   résultat (produits / charges).
5. **AC-5 — Vides et devise.** Groupe nul masqué (D6) ; solde en devise d'un groupe seulement si une devise étrangère
   unique (D7).
6. **AC-6 — API.** `detailLevel` (`1`…`n` | `all`) sur `GET /reports/balance-sheet` et `/income-statement` ; valeur
   invalide → `400 VALIDATION_ERROR` ; la réponse gagne `groupedRows` par section. Sans paramètre : JSON identique
   (NFR-46-1).
7. **AC-7 — Arbre défensif.** Cycle ou parent d'une autre société posé par SQL : compte traité en racine,
   `tracing::warn!`, ni boucle ni panique (NFR-46-4).
8. **AC-8 — Tests.** Unitaires de `regrouper` (plan PME, groupe à solde propre, types mixtes, cycle, devise unique et
   mêlée, niveaux 1 à `all`) ; E2E API sur deux exercices ; `docs/api-external.md` à jour.

## Tâches principales

- [ ] `groupes.rs` : types, `regrouper`, tests unitaires (AC-1 à AC-5, AC-7)
- [ ] Lecture de l'arbre des comptes de la société (une requête) ; branchement dans `balance_sheet::generate` et
      `income_statement::generate` derrière un paramètre optionnel
- [ ] `ReportQuery` / `ExportQuery` : `detail_level` ; validation (AC-6)
- [ ] Tests E2E `reports_e2e.rs` ; documentation API

## Fichiers

`crates/kesh-report/src/{groupes.rs (neuf), lib.rs, balance_sheet.rs, income_statement.rs}` ;
`crates/kesh-api/src/routes/reports.rs` ; `crates/kesh-api/tests/reports_e2e.rs` ; `docs/api-external.md`.

## Note de version

*(à écrire au développement)*
