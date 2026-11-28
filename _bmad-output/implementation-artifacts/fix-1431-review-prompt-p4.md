# Correctif #1431 — passe de revue P4, ciblée

Passe ciblée, une lentille, contexte frais, sur le **seul commit `6faee0819`** (`git show 6faee0819`), dans le worktree
du correctif. Ce commit ajoute, à la demande du mainteneur, une section du manuel utilisateur « Réorganiser le plan
comptable » (`docs/manual/fr/user-manual.tex`, `\label{sec:reorganiser-plan}`) et un renvoi à cette section dans les six
messages des refus `ACCOUNT_PARENT_HAS_ENTRIES` et `ACCOUNT_PARENT_DESIGNATED`, dans les quatre locales
(`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`) et dans les textes de repli de
`crates/kesh-api/src/errors.rs` ; une phrase au guide de l'API (`docs/api-external.md`) ; la fiche du correctif.

Demande du mainteneur : « créer les nouveaux comptes, reporter le solde par une écriture de reclassement, puis archiver
l'ancien compte » ; « Un compte de regroupement ne porte pas d'écritures : on ne peut pas désigner comme parent un compte
qui en a, même si son solde est nul » ; pour un compte désigné : « On change d'abord la désignation, puis on
restructure. » Le manuel n'existe qu'en français (`docs/manual/{de,en,it}/` ne portent qu'un README).

## Axes

1. **Le manuel contre le code.** Chaque affirmation de la nouvelle section, et des deux renvois ajoutés plus haut, se
   vérifie dans le code : menus et noms d'écrans (`crates/kesh-i18n/locales/fr-CH/messages.ftl`, `nav-*`, et
   `frontend/src/routes/(app)/…`) ; les chemins qui rendent les deux refus (`refuse_unfit_parent_in_tx`,
   `crates/kesh-db/src/repositories/accounts.rs`, et la route groupée) ; l'archivage (`archive_guarded`, même fichier :
   un compte qui porte des écritures ou un solde s'archive-t-il ? quels refus ?) ; l'unicité des rôles
   (`singleton_role`, migration `20260722000001_accounts_role_postable.sql`) ; la saisie d'une écriture (exercice ouvert,
   verrou de période) ; l'article (« compte de produit » d'un produit du Catalogue). Le PDF compilé, aplati, est dans
   le répertoire de journaux du correctif (`manuel/user-manual.txt`) : contrôler que la section y figure et que les
   renvois (`§`) se résolvent.
2. **Les messages, dans les quatre langues** : exactitude, ton, cohérence entre locales et avec les messages voisins du
   même fichier ; les textes de repli de `errors.rs` égaux au français de `fr-CH` ; les tests qui lisent ces messages
   (`grep -rn` sur des fragments) toujours justes.
3. **Le guide de l'API et la fiche** : la phrase ajoutée est-elle juste ; la fiche désigne la release par sa date
   (« la release du 4 décembre »), non par son numéro.

## Interdits

Lecture seule. Aucun fichier écrit, aucune commande qui écrit (pas de `git commit`, `git stash`, `scripts/prepare-release.sh`,
`make`, `npm`, `cargo`), aucun `gh`, aucun accès réseau, aucune base.

## Rapport

Findings classés CRITICAL / HIGH / MEDIUM / LOW, chacun avec fichier:ligne et la ligne de code lue qui l'établit ; puis
**la liste des axes réellement exercés et de ceux qui ne l'ont pas été**. Un « 0 finding » sans cette liste ne compte pas.
