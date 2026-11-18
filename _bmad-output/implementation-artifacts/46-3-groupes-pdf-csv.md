# Story 46-3 : PDF et CSV par groupes

Status: draft

*Esquisse du 2026-11-18. Epic 46, D8. Suit 46-1 ; commence après la fusion de `fix/pdf-montants-alignes`. Aucune
migration.*

## Story

En tant que comptable, je veux imprimer et exporter le bilan et le compte de résultat au niveau de détail choisi, afin
de remettre un bilan par groupes à ma fiduciaire ou de le retravailler dans un tableur.

## Critères d'acceptation (esquisse)

1. **AC-1 — Export.** `detailLevel` accepté par `/reports/{balance-sheet,income-statement}/export` (PDF et CSV).
2. **AC-2 — PDF.** Groupes en gras avec sous-total, retrait par profondeur, solde en devise d'un groupe à la place prévue
   par 37-4 ; garder-avec-la-suite pour un titre de groupe ; totaux de section inchangés.
3. **AC-3 — CSV.** Colonnes `Niveau` et `TypeLigne` (groupe, compte, propre) ajoutées seulement en mode groupé ; montants
   `format_amount_iso`.
4. **AC-4 — Sans paramètre.** CSV identique octet pour octet, PDF aux mêmes lignes de texte (NFR-46-1), comparés à des
   fixtures du commit de base.
5. **AC-5 — Tests et manuel.** Tests de rendu (texte extrait du PDF), E2E d'export ; manuel FR (`.tex`).

## Tâches principales

- [ ] Rendu PDF des `groupedRows` ; libellés dans `SectionLabels` et les quatre locales
- [ ] Rendu CSV ; fixtures de non-régression
- [ ] Route d'export ; tests ; manuel

## Fichiers

`crates/kesh-report/src/{pdf.rs, csv.rs}` ; `crates/kesh-api/src/routes/reports.rs` ; locales i18n ;
`crates/kesh-api/tests/reports_e2e.rs` ; `docs/manual/fr/user-manual.tex`.

## Note de version

*(à écrire au développement)*
