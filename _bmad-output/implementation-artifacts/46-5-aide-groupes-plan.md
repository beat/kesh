# Story 46-5 : aide à la création des groupes d'un plan existant (conditionnelle)

Status: draft

*Esquisse du 2026-11-18. Epic 46, D12. **Conditionnelle** (Q9). Part de `main` après la fusion de
`fix/api-parent-compte`. Aucune migration attendue.*

## Story

En tant que comptable qui range un plan existant, je veux que Kesh me propose des comptes de groupe d'après les numéros
de mes comptes, afin de structurer mon plan (une banque = un groupe, puis les liquidités) sans tout créer à la main.

## Critères d'acceptation (esquisse)

1. **AC-1 — Proposition.** Pour une sélection de comptes, Kesh propose un groupe = « numéro sans son dernier chiffre »
   (règle à confirmer, Q9), avec nom à saisir et parent éventuel ; aperçu de l'arbre résultant.
2. **AC-2 — Rien d'office.** Aucun compte créé ni déplacé sans confirmation ; un numéro déjà pris, un compte qui porte
   des écritures ou un cycle sont signalés dans l'aperçu et exclus.
3. **AC-3 — Écriture.** À la confirmation : création des comptes de groupe puis changement de parent, par les mêmes
   contrôles que `POST /accounts` et `PUT /accounts/{id}`, dans une transaction ; audit.
4. **AC-4 — Tests et manuel.** Tests d'intégration ; E2E ; captures ; manuel FR (`.tex`).

## Tâches principales

- [ ] Calcul de la proposition (pur, testé) ; aperçu à l'écran du plan
- [ ] Route de création groupée (ou appels existants enchaînés, à décider à la validation)
- [ ] Tests, captures, manuel

## Fichiers

`crates/kesh-api/src/routes/accounts.rs` ; `crates/kesh-db/src/repositories/accounts.rs` ;
`frontend/src/routes/(app)/accounts/+page.svelte`, `frontend/src/lib/features/accounts/` ; locales i18n ;
`docs/manual/fr/user-manual.tex`.

## Note de version

*(à écrire au développement)*
