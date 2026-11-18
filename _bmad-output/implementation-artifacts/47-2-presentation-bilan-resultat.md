# Story 47-2 : présentation au bilan et au compte de résultat

Status: draft

Epic : `epic-47-comptes-miroirs.md` (D4–D9). Dépend de 47-1 et de l'Epic 46 fusionné.

## Intention

Le bilan et le compte de résultat n'affichent plus de montant négatif dû à un compte de sens inverse ou à une paire.

## Critères d'acceptation

1. Une fonction pure `kesh-report::presentation` reçoit les lignes brutes et les liens, rend les lignes présentées
   avec leurs `sources` ; appliquée avant les totaux et avant les groupes de l'Epic 46.
2. Miroir : solde de sens inverse → ligne du compte miroir, de l'autre côté, montant positif ; solde de sens normal :
   inchangé. Paire : net sur le compte du sens du net ; net nul : aucune ligne.
3. Équation du bilan vraie et résultat net identique en brut et en présenté (test de propriété) ; fonds propres et
   lignes calculées inchangés.
4. À toute date d'arrêté (D4) ; `presentation=brute` rend le rapport actuel (D6).
5. JSON : chaque ligne présentée porte `sources: [{ accountId, accountNumber, balance }]` ; PDF et CSV : marque et
   note ; MCP `kesh_bilan` et `kesh_compte_de_resultat` au même contrat.
6. Devise : solde en devise de la ligne présentée seulement si ses sources ont la même devise (Q3).
7. L'exemple chiffré du fichier d'epic est rejoué par un test d'intégration.

## Tâches

- [ ] Fonction et tests unitaires ; branchement bilan et compte de résultat.
- [ ] Route (`presentation`), PDF, CSV, MCP ; tests d'API.
- [ ] `docs/api-external.md`.

## Fichiers

`kesh-report/src/{presentation,balance_sheet,income_statement,pdf,csv}.rs`, `kesh-api/src/routes/reports.rs`,
`services/rapports.rs`, MCP, i18n. Conflits attendus : Epic 46, `fix/pdf-montants-alignes`, `fix/bilan-resultat-reporte`.

## Note de version

Bilan et compte de résultat présentent les comptes miroirs et les paires compensées ; présentation brute sur demande.
