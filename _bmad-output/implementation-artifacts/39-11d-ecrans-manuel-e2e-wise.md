# Story 39-11d : écrans, manuel et E2E de l'import Wise

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/39-wise`. Epic 39, **phase 1 KISS**. **Dépend de** : 39-11a à 39-11c,
**38-11d** (écrans PayPal : réglages prestataire d'un compte bancaire, bouton d'import, critères de règle, liste des
écarts), dont les composants sont réutilisés.*

## Story

En tant que **Comptable**, je veux régler mes comptes Wise, importer mes relevés et traiter les propositions et les
écarts depuis l'écran, et trouver au manuel comment exporter le relevé de Wise et ce que Kesh en fait.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-11d-1` | Fiche d'un compte bancaire : prestataire « Wise », libellé, compte de frais (composant de 38-11d, paramétré). | AC-1 |
| `FR-39-11d-2` | Bouton « Importer des relevés Wise » : choix du libellé, un à trois fichiers, compte rendu par fichier (insérées, doublons, soldes, trou). | AC-2 |
| `FR-39-11d-3` | Propositions Wise (frais, conversion, virement interne, versement Stripe) et liste des écarts lisibles, quatre locales. | AC-3 |
| `FR-39-11d-4` | Critère « type ou préfixe d'identifiant » : aide qui liste les préfixes Wise. | AC-3 |
| `NFR-39-11d-1` | Manuels d'administration et utilisateur à jour (`.tex` seulement, PDF à la release) ; *Note de version*. | AC-4 |
| `NFR-39-11d-2` | E2E sur les fixtures fictives ; captures d'écran de chaque état touché (*Recette visuelle*). | AC-5 |

## Critères d'acceptation

1. **AC-1 — Réglages.** Le composant de réglages prestataire de 38-11d gagne « Wise » (aucune copie : un prestataire
   en paramètre) ; libellé, compte de frais filtré comme au serveur ; refus du serveur affichés par leur code.
2. **AC-2 — Import.** Bouton sur la liste des comptes bancaires (ou à l'endroit choisi par 38-11d pour PayPal, même
   composant) : libellé parmi ceux des comptes Wise, sélection multiple de fichiers `.csv`, envoi à
   `POST /api/v1/wise-imports` ; compte rendu par fichier ; erreurs de 39-11a (colonnes manquantes, ligne et champ)
   lisibles ; avertissement `gap` expliqué (« le relevé ne commence pas où finissait le précédent »).
3. **AC-3 — Propositions et écarts.** Libellés des genres `provider_fee`, `provider_conversion` (« Conversion au cours
   de Wise : 100.00 EUR → 95.00 CHF », exemple fictif), `own_transfer`, `stripe_payout` ; motifs des écarts ; aide du
   critère `provider_type` (préfixes `TRANSFER`, `CARD`, `BALANCE`, `CONVERSION_ORDER`, `FEE-…`). Textes « proposer »,
   jamais « comptabiliser automatiquement ».
4. **AC-4 — Manuels** (`docs/manual/fr/*.tex` ; DE, IT, EN selon la règle du dépôt) : utilisateur — exporter de Wise le
   relevé CSV de chaque solde (période, une devise par fichier), l'importer, ce que Kesh propose ligne par ligne, les
   conversions au cours de Wise, les versements Stripe rattachés (Epic 40), les écarts, la limite de fuseau de la
   date ; administration — un compte bancaire par devise, compte de frais, compte de conversion requis pour EUR↔USD.
   Contrôle local du PDF permis (aplati, `pdftotext … | tr '\n' ' '`), PDF non commité.
5. **AC-5 — E2E** `frontend/tests/e2e/wise-import.spec.ts` : réglage de trois comptes, import des trois fixtures de
   39-11a, compte rendu, acceptation d'un frais et d'une conversion, écart Stripe affiché ; réimport → refus doublon.
   Captures (thème clair) hors du dépôt (`kesh-gate-logs/`), jointes au Dev Agent Record.

## Modules touchés (4)

`frontend`, `kesh-i18n` (s'il porte des libellés serveur), `docs/manual` (`.tex`), E2E.

## Note de version

*Ajouté* — Écrans de l'import Wise : réglage des comptes, import des relevés, propositions et écarts ; chapitre Wise
aux manuels.

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
