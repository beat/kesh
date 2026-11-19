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
   note ; quand une paire s'applique, le PDF porte en clair **« présentation compensée »** (Guy, D9 et Q9) ;
   `presentation=brute` disponible pour l'écran, le PDF, le CSV et l'API (D6) ; MCP `kesh_bilan` et `kesh_compte_de_resultat` au même contrat.
6. Devise : solde en devise de la ligne présentée seulement si ses sources ont la même devise (Q3).
7. L'exemple chiffré du fichier d'epic est rejoué par un test d'intégration.
8. Libellé de la ligne : celui du compte qui la porte (Q7) ; l'accueil reste sur les totaux bruts (Q11).
9. Documentation (§ ci-dessous) : `docs/api-external.md` (`presentation=brute`, `sources`) ; manuel d'administration
   (paramètre à l'API et aux exports).

## Tâches

- [ ] Fonction et tests unitaires ; branchement bilan et compte de résultat.
- [ ] Route (`presentation`), PDF, CSV, MCP ; tests d'API.
- [ ] `docs/api-external.md`, manuel d'administration.

## Documentation (exigence de Guy, 2026-11-19)

- `docs/api-external.md` : paramètre `presentation=brute`, champ `sources`, mention « présentation compensée ».
- Manuel d'administration (`docs/manual/fr/admin-manual.tex`) : présentation brute à l'API et dans les exports.

Manuels FR (`docs/manual/fr/`, `.tex` seulement, PDF régénérés à la release) ; s'ils sont traduits d'ici la livraison, le mécanisme y figure aussi. **Le prompt de chaque passe de revue de cette story nomme ces manuels** et exige le contrôle du PDF (`CLAUDE.md` § *Le prompt d'une passe doit NOMMER le manuel*).

## Fichiers

`kesh-report/src/{presentation,balance_sheet,income_statement,pdf,csv}.rs`, `kesh-api/src/routes/reports.rs`,
`services/rapports.rs`, MCP, i18n. Conflits attendus : Epic 46, `fix/pdf-montants-alignes`, `fix/bilan-resultat-reporte`.

## Note de version

Bilan et compte de résultat présentent les comptes miroirs et les paires compensées ; présentation brute sur demande.
