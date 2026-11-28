# Prompt — revue de code P2 du correctif #1431, passe ciblée sur la remédiation de la P1

Tu relis **une seule remédiation** du correctif #1431 de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit 5) :
le commit de remédiation de la revue P1 (indiqué au Change Log de la fiche). Contexte frais, une lentille
(« chasseur de régressions »), rapport en français. `<worktree>` et `<journaux>` sont donnés par l'orchestrateur.

Diff à relire : `git -C <worktree> show <commit de remédiation>`. Pour le contexte seulement : la fiche
`_bmad-output/implementation-artifacts/fix-1431-parent-creation-reactivation.md` (Contrat retenu, C-1431-4 et C-1431-5)
et le rapport de la P1 `<journaux>/review-p1.md` (M-1, L-1 à L-8).

## Ce que la remédiation dit faire

- M-1 : garde maintenue pour un parent déjà compte de groupe (choix C-1431-5), messages neutres (« ce qu'un compte de
  groupe n'est pas ») dans les quatre locales et les replis Rust ;
- L-7 : `first_product_designating_in_tx` partagée par `refuse_designated_parent_in_tx` et `currency_use_site_in_tx` ;
- L-1, L-2, L-3 : commentaires et guide ; L-4, L-5, L-6 : tests ; L-8 : note de version et C-1431-4.

## Axes (déclare ceux exercés et non exercés)

1. **Régression introduite par le patch** : `currency_use_site_in_tx` rend-il toujours exactement `CurrencyUseSite::Product(id)`
   avec le même verrou et le même ordre qu'avant ? Les replis Rust suivent-ils mot pour mot le fr-CH (variables `{child}`,
   `{parent_number}`, `{product_name}`) ? Les quatre locales gardent-elles les mêmes variables ?
2. **Tests neufs** : passent-ils à vide (montage qui ne crée pas la situation) ? Le test d'ordre prouve-t-il l'ordre ?
   Le test du groupe existant crée-t-il bien un parent non imputable avec un sous-compte actif et une écriture ?
3. **Docs** : le guide de l'API et la fiche disent-ils ce que fait le code (parent déjà groupe compris) ?
4. **C-1431-5** : le raisonnement tient-il au code (le `PUT` et `attach` refusent-ils vraiment un groupe existant
   mouvementé — lis `guard_parent_change_in_tx` et les tests `rattacher_et_ses_refus`) ?

Sévérités CRITICAL, HIGH, MEDIUM, LOW ; pour chaque finding : fichier, ligne lue, extrait, correctif proposé.

## Interdits absolus

- Lecture seule ; seule écriture permise : ton rapport `<journaux>/review-p2.md`. Aucun fichier temporaire, aucune
  redirection.
- Aucun `rm`, `kill`, `pkill` ; aucun `git` qui écrit ; aucun `gh`.
- Aucun interpréteur ni rien qui compile ou exécute (`python3`, `node`, `npm`, `bash -c`, `jq`, `cargo`, `make`, `docker`,
  `mariadb`, `sqlx`, `curl`, `sed -i`) ; ni `scripts/prepare-release.sh` ni aucun script du dépôt ou des journaux.
