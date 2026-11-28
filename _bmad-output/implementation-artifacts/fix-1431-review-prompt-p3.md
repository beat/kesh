# Prompt — revue de code P3 du correctif #1431, passe ciblée sur le lot des LOW de la P2

Une lentille, contexte frais, rapport en français. `<worktree>` et `<journaux>` sont donnés par l'orchestrateur. Diff à
relire : `git -C <worktree> show <commit du lot>` (indiqué au Change Log de la fiche). Contexte : `<journaux>/review-p2.md`.

Le lot dit faire : message `error-account-parent-has-entries` neutre (« un compte de groupe n'est pas imputable, et ce solde
ne pourrait plus être corrigé »), aux quatre locales et au repli Rust ; test `groupe_existant_mouvemente_ne_recoit_pas_de_sous_compte`
renommé et son commentaire précisé.

Axes (déclare exercés et non exercés) : le repli Rust suit-il mot pour mot le fr-CH et la variable `{ $number }` ? Les
quatre locales ? Un test, un guide (`docs/api-external.md`), le manuel ou le frontend citent-ils encore l'ancien texte ou
l'ancien nom du test (greper la valeur) ? Le message convient-il aux trois chemins (création, changement de parent,
route groupée) ?

Sévérités CRITICAL, HIGH, MEDIUM, LOW ; fichier, ligne lue, extrait, correctif.

Interdits : lecture seule ; seule écriture, ton rapport `<journaux>/review-p3.md` ; aucun fichier temporaire, aucune
redirection ; aucun `rm`, `kill` ; aucun `git` qui écrit ; aucun `gh` ; aucun interpréteur ni rien qui compile ou exécute
(`python3`, `node`, `npm`, `bash -c`, `jq`, `cargo`, `make`, `docker`, `sed -i`) ; ni `scripts/prepare-release.sh` ni aucun
script.
