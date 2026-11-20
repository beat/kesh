# Story 39-12b : bouton « Télécharger depuis Wise », manuels, E2E

Status: drafted

*Fiche rédigée le 2026-11-20 (reprend l'écran de l'ancienne 39-12b ; le parallélisme est passé à 39-12a2). Branche
`epic/39-wise`. Epic 39, **phase 2** ; **socle S7 et S8** (fichier d'epic). **Dépend de** : 39-12a2, 39-11d, du composant
d'écran générique de 38-12b (socle S7).*

## Story

En tant que **Comptable**, je veux lancer depuis mon compte bancaire Wise le téléchargement d'une période, en suivre la
progression et lire son compte rendu ; en tant qu'**Administrateur**, je veux savoir au manuel comment créer le jeton en
lecture seule, générer la paire de clés de signature, déposer la clé publique chez Wise et poser les variables.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-12b-1` | Un bouton « Télécharger depuis Wise » n'apparaît que si le compte Wise a sa série (jeton et clé de signature) dans l'environnement. | AC-1 |
| `FR-39-12b-2` | Période, progression, pauses et compte rendu par le dialogue générique du socle. | AC-2 |
| `FR-39-12b-3` | Le manuel d'administration dit jeton, variables, paire de clés, dépôt de la clé publique, limites. | AC-3 |
| `NFR-39-12b-1` | E2E contre le serveur de prestataire simulé du socle ; captures ; `.tex` seulement. | AC-4 |

## Critères d'acceptation

1. **AC-1 — Indicateur et bouton** (socle S7). La réponse des comptes bancaires porte **`providerApi: { configured,
   mode? }`**, champ générique de 38-12b (vrai si une série existe au libellé du compte, selon le prestataire du compte ;
   jamais un secret) ; pour Wise, rien de plus : la clé de signature est **exigée** dans la série (39-12a1 AC-1), si bien que
   `configured` vaut « jeton et clé posés » — le champ `scaKeyConfigured?` que porte 38-12b pour Wise n'est pas rempli
   (validation P2, simplification ; écart signalé à l'Epic 38).
   Le bouton se montre à Comptable et plus quand `configured`. Non configuré : ligne d'aide
   (Administrateur seulement) ; « Importer un CSV » de 39-11d reste seul.
2. **AC-2 — Dialogue** : le composant générique de 38-12b, paramétré par le prestataire (route `wise-downloads`, libellés) :
   période (défaut : exercice ouvert), progression « relevé `done`/`total` », pause « Wise demande de patienter », compte rendu
   de 39-11b plus `unlinkedCurrencies` (« soldes sans compte bancaire, non téléchargés »), refus par code `PROVIDER_*`,
   `WISE_PROFILE_AMBIGUOUS` et `WISE_SCA_REJECTED` en clair aux quatre locales (409 : reprise du suivi par
   `details.jobId`), « rien n'a été enregistré, relancez » sur échec. **Aucun composant propre à Wise.**
3. **AC-3 — Manuels** (`.tex` seul, aucun PDF commité ; PDF contrôlé à la régénération, `pdftotext … | tr '\n' ' '`).
   *Administration* : créer un jeton personnel **lecture seule** (profil d'entreprise) ; variables `KESH_WISE_<n>_ACCOUNT`
   (le libellé **identique** à celui du compte Wise de Kesh — il est **propre à l'installation** : deux sociétés qui portent le
   même libellé atteignent le même compte Wise), `_TOKEN`, `_PROFILE_ID`, **`_SCA_KEY_FILE`** (exigée : sans elle, Kesh refuse de démarrer), `KESH_WISE_BASE_URL`,
   `KESH_WISE_PARALLEL`, dans **un tableau neuf du § `sec:env-vars`** (modèle `:840-862`) **et** le tableau de la section Wise, jamais le tableau SMTP (`:1155`) ; comment aller **au-delà de trois comptes** (lignes à
   ajouter aux compose) ; la ligne `volumes:` du fichier de clé (`:ro`) ; **générer la paire de clés** (`openssl genrsa -out wise-sca.pem 2048`, puis `openssl rsa -in
   wise-sca.pem -pubout -out wise-sca-pub.pem`), **déposer la clé publique** dans les réglages API de Wise (chemin d'interface
   **à vérifier**), monter le fichier privé en **lecture seule** dans le conteneur, droits `600`, **hors** des dossiers
   sauvegardés ; rien en base ni dans `.keshbackup` (à reposer après une restauration sur un autre serveur) ; rotation et
   perte de la clé (`WISE_SCA_REJECTED` : l'import CSV continue) ; refus de démarrage et leurs messages ; limites de Wise (SCA,
   profondeur d'historique, 469 jours par requête, découpés en fenêtres de 468 jours de Zurich). Points **non confirmés** de 39-12a1 écrits comme à vérifier.
   *Utilisateur* : le bouton, la période, la progression, ce qu'un échec veut dire, le mélange sans doublon avec les CSV.
   `docs/api-external.md` : la route de 39-12a2 (sessions seules, 403 `API_KEY_OPERATION_FORBIDDEN`), avec ses codes. La
   phrase du registre des routes (« N des M routes ») est déjà relevée par 39-12a2 : ne pas la retoucher ici.
4. **AC-4 — E2E** `frontend/tests/e2e/wise-download.spec.ts`, **serveur de prestataire simulé de 38-12b (socle S8)**, port lu
   de `KESH_E2E_PROVIDER_MOCK_PORT`, backend d'E2E démarré avec `KESH_WISE_1_*` (jeton, clé de test) et `KESH_WISE_BASE_URL`
   vers lui (ajout au montage de `docs/testing.md` ; le bloc de `CLAUDE.md` n'est pas touché) : sans variables, bouton absent ;
   période → défi SCA relevé (le serveur simulé vérifie la **présence** de `x-2fa-approval` et `X-Signature` au rejeu ; la
   validité de la signature est prouvée par les tests Rust de 39-12a1, sans clé publique dans le serveur Playwright) →
   progression → compte rendu ; second téléchargement → « 0 insérée » ; 401 simulé affiché. Sans la
   variable du port, ignoré avec son motif écrit (jamais compté vert). Vitest : indicateur `providerApi` (configuré, non
   configuré). Captures hors du dépôt.

## Tâches

- [ ] T1 — `providerApi` pour un compte Wise (AC-1).
- [ ] T2 — Paramétrage du dialogue générique, i18n quatre locales, `sitesTotal` recompté (AC-1, AC-2).
- [ ] T3 — Manuels et `docs/api-external.md` (AC-3).
- [ ] T4 — E2E (AC-4).

## Stratégie de tests

Vitest ; `npm run check`, `lint-i18n-ownership`, `build` ; E2E **complet au dernier commit de code** ; gate backend complet
(la réponse des comptes bancaires change). Captures de chaque état (thème clair).

## Modules touchés (3)

`frontend`, `kesh-i18n` (+ `kesh-api` : le seul champ `providerApi` pour Wise), manuels et `docs/`.

## Fichiers communs

`frontend/src/routes/(app)/bank-accounts/` (38-11d, 38-12b), catalogues i18n, `i18n-keys.test.ts`, manuels `.tex`,
`docs/api-external.md`, `docs/testing.md`.

## Note de version

*Ajouté* — Bouton « Télécharger depuis Wise » avec sa progression ; le manuel d'administration décrit le jeton en lecture
seule et la clé de signature de l'authentification forte.

## Change Log

- 2026-11-20 — fiche rédigée (phase 2), non validée.
- 2026-11-20 — validation P1 (Claude Sonnet 5.5, contexte frais) : voir 39-12a2 ; écran et E2E alignés sur le composant
  et le serveur simulé génériques de 38-12b, `providerApi`, manuel de la paire de clés SCA.
- 2026-11-20 — validation P2 (Claude Opus 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-P2.md`) :
  ici, `scaKeyConfigured` et son avertissement retirés (clé SCA exigée), E2E du rejeu borné à la présence des en-têtes,
  reprise du suivi sur 409 par `details.jobId`, phrase du registre laissée à 39-12a2.
- 2026-11-20 — validation P3 ciblée (Claude Sonnet 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-40-phase2-P3-ciblee.md`) : tableaux de variables précisés (§ `sec:env-vars` neuf + section Wise, jamais SMTP).
- 2026-11-20 — validation P2, complément (Claude Opus 5.5)
 : manuel — deux tableaux de variables, au-delà de trois comptes,
  volume du fichier de clé.
