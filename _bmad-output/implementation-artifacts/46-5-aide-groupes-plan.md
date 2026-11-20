# Story 46-5 : aide à la création des groupes d'un plan existant (conditionnelle)

Status: drafted

*Revue de Beat (2026-11-19) : approuvée ; **proposition d'interface (maquette) avant de décider** de la story (Q9) ; approuvée par Guy le 2026-11-19 (revue 1).*

*Esquisse du 2026-11-18, reprise le 2026-11-20. Epic 46, D12. **Conditionnelle** (Q9) : ni validée en boucle ni
développée avant la maquette de l'AC-0 et la décision de Beat et de Guy. `fix/api-parent-compte` est fusionné dans
`main` (#1351, `d29aa557`) : `PUT /accounts/{id}` accepte `parentId`, refuse un cycle et un parent qui porte des
écritures. Release visée 0.21.0 si décidée ; sinon la story sort de l'epic, qui compte alors six stories. Aucune
migration attendue.*

**Modules touchés** (règle de splitting préventif) : `kesh-api/routes/accounts` (ou appels existants enchaînés),
`kesh-db/repositories/accounts` (transaction), `frontend` (écran du plan), `kesh-i18n`, manuel — cinq ; à recompter
à la validation selon la maquette.

## Story

En tant que comptable qui range un plan existant, je veux que Kesh me propose des comptes de groupe d'après les numéros
de mes comptes, afin de structurer mon plan (une banque = un groupe, puis les liquidités) sans tout créer à la main.

## Critères d'acceptation (esquisse)

0. **AC-0 — Maquette avant décision (Q9, Beat).** Une maquette de l'interface (sélection, proposition, aperçu de
   l'arbre) est présentée à Beat et à Guy ; **la story n'est décidée, ni développée, qu'après cette maquette**. La règle
   « numéro sans son dernier chiffre » y est confirmée ou changée.

1. **AC-1 — Proposition.** Pour une sélection de comptes, Kesh propose un groupe = « numéro sans son dernier chiffre »
   (règle à confirmer, Q9), avec nom à saisir et parent éventuel ; aperçu de l'arbre résultant.
2. **AC-2 — Rien d'office.** Aucun compte créé ni déplacé sans confirmation ; un numéro déjà pris, un compte qui porte
   des écritures (refus de #1351 : un compte déjà mouvementé ne devient pas un groupe — le groupe d'une banque est un
   compte neuf) ou un cycle sont signalés dans l'aperçu et exclus.
3. **AC-3 — Écriture.** À la confirmation : création des comptes de groupe puis changement de parent, par les mêmes
   contrôles que `POST /accounts` et `PUT /accounts/{id}`, dans une transaction ; audit.
4. **AC-4 — Tests et manuel.** Tests d'intégration ; E2E ; captures ; manuel FR (`.tex`).

## Tâches principales

- [ ] Calcul de la proposition (pur, testé) ; aperçu à l'écran du plan
- [ ] Route de création groupée (ou appels existants enchaînés, à décider à la validation) ; réutiliser les contrôles
      de `accounts::create` et du changement de parent de #1351 (`crates/kesh-db/src/repositories/accounts.rs:189`,
      `:1079`), sans les recopier
- [ ] Tests, captures, manuel

## Fichiers

`crates/kesh-api/src/routes/accounts.rs` ; `crates/kesh-db/src/repositories/accounts.rs` ;
`frontend/src/routes/(app)/accounts/+page.svelte`, `frontend/src/lib/features/accounts/` ; locales i18n ;
`docs/manual/fr/user-manual.tex`.

## Note de version

*(à écrire au développement)*

## Change Log

- 2026-11-20 — Reprise : #1351 fusionnée, refus d'un parent mouvementé rappelé, modules déclarés, statut `drafted`
  (conditionnelle, maquette avant décision).
