# Story 47-4 : contrôle de clôture — solde de sens inverse

Status: draft

Epic : `epic-47-comptes-miroirs.md` (FR-47-5, Q6). Dépend de 47-1.

## Intention

Avant de clore, voir les comptes qui paraîtraient négatifs faute de miroir ou de paire.

## Critères d'acceptation

1. `closing_checks` gagne un contrôle **non bloquant** : comptes de bilan ou de résultat (hors fonds propres) à
   solde de sens inverse à la date de clôture, sans miroir ni paire ; détails : compte, solde.
2. Écran de clôture : avertissement listé ; API et test de dépôt.
3. Aucun effet sur la possibilité de clore.

## Fichiers

`kesh-db/src/repositories/closing_checks.rs`, `kesh-api/src/routes/fiscal_years.rs`, écran de clôture, i18n.

## Note de version

Contrôle de clôture : comptes à solde de sens inverse sans présentation.
