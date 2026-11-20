# Story 39-12b : parallélisme en option, écran du téléchargement Wise, manuel et E2E

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/39-wise`. Epic 39, **phase 2** (H3). **Dépend de** : 39-12a (passée sa
recette), 39-11d.*

## Story

En tant que **Comptable**, je veux lancer le téléchargement Wise d'une période depuis l'écran et en suivre la
progression ; en tant qu'**Administrateur**, je veux pouvoir accélérer les téléchargements longs par une variable
d'environnement.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-12b-1` | `KESH_WISE_PARALLEL` : nombre de requêtes simultanées, défaut 1, maximum 6 ; hors bornes → refus au démarrage. | AC-1 |
| `FR-39-12b-2` | Bouton « Télécharger depuis Wise » (libellé, période), progression, compte rendu. | AC-2 |
| `FR-39-12b-3` | Manuel : jeton lecture seule, variables, limites (relevés au jeton personnel selon le pays, SCA), délais. | AC-3 |
| `NFR-39-12b-1` | Même résultat en parallèle qu'en séquentiel (test) ; reprise sur `429` inchangée. | AC-1 |
| `NFR-39-12b-2` | E2E sur le transport factice ; captures ; `.tex` seulement. | AC-4 |

## Critères d'acceptation

1. **AC-1 — Parallélisme.** `KESH_WISE_PARALLEL` (1 à 6, défaut 1 ; lecteur de configuration de 39-12a) borne les
   requêtes simultanées (une par devise et par période) ; un `429` met en pause **toutes** les requêtes du
   téléchargement le temps de l'attente ; l'import reste par devise, tout ou rien, dans l'ordre des devises (résultat
   identique au séquentiel, test qui compare les deux). Même règle que `KESH_PAYPAL_PARALLEL` (phase 2 de l'Epic 38),
   par la même aide si elle existe (G10).
2. **AC-2 — Écran.** À côté du bouton d'import CSV de 39-11d : choix du libellé (seulement ceux qui ont un jeton :
   lecture d'un indicateur sans secret), période (défaut : exercice ouvert), lancement, progression lue de la route
   d'état, compte rendu de 39-11b ; erreurs `PROVIDER_*` en clair (quatre locales).
3. **AC-3 — Manuels** (`.tex`) : administration — créer un jeton personnel **lecture seule** dans Wise (profil
   d'entreprise), variables `KESH_WISE_<n>_ACCOUNT`, `KESH_WISE_<n>_TOKEN`, `KESH_WISE_<n>_PROFILE_ID`,
   `KESH_WISE_PARALLEL`, aucune copie en base ni en sauvegarde (à reposer après une restauration sur un autre
   serveur), limites de Wise ; utilisateur — le bouton, la période, le mélange sans doublon avec les CSV.
4. **AC-4 — E2E** `frontend/tests/e2e/wise-download.spec.ts` contre le transport factice : téléchargement d'une
   période → compte rendu ; second téléchargement → tout en doublons ; erreur 401 factice affichée. Captures hors du
   dépôt.

## Modules touchés (4)

`kesh-api` (parallélisme, route d'état), `frontend`, `kesh-i18n`, `docs/manual` (`.tex`).

## Note de version

*Ajouté* — Bouton « Télécharger depuis Wise » avec sa progression ; `KESH_WISE_PARALLEL` accélère les téléchargements
(défaut 1, maximum 6).

## Change Log

- 2026-11-20 — fiche rédigée (phase 2), non validée.
