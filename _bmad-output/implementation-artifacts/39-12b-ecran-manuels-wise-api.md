# Story 39-12b : bouton « Télécharger depuis Wise », manuels, E2E

Status: ready-for-dev

*Fiche réécrite le 2026-11-20 sur la **voie activités** (recherche du 2026-11-20, `data-not-committed/wise-recherche.md`,
métadonnées seulement) : plus de clé de signature SCA. Branche `epic/39-wise`. Epic 39, **phase 2** ; **socle S7 et S8**
(fichier d'epic). **Dépend de** : 39-12a3, 39-11d, du composant d'écran générique de 38-12b (socle S7).*

## Story

En tant que **Comptable**, je veux lancer depuis mon compte bancaire Wise le téléchargement d'une période, en suivre la
progression et lire son compte rendu ; en tant qu'**Administrateur**, je veux savoir au manuel comment créer le jeton en
lecture seule, poser les variables et ce que le téléchargement ne sait pas faire.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-12b-1` | Un bouton « Télécharger depuis Wise » n'apparaît que si le compte Wise a sa série (libellé et jeton) dans l'environnement. | AC-1 |
| `FR-39-12b-2` | Période, progression, pauses et compte rendu par le dialogue générique du socle. | AC-2 |
| `FR-39-12b-3` | Le manuel d'administration dit jeton, variables et limites ; le manuel utilisateur dit le bouton, le contrôle de solde et le mélange avec les CSV. | AC-3 |
| `NFR-39-12b-1` | E2E contre le serveur de prestataire simulé du socle ; captures ; `.tex` seulement. | AC-4 |

## Critères d'acceptation

1. **AC-1 — Indicateur et bouton** (socle S7). La réponse des comptes bancaires porte **`providerApi: { configured,
   mode? }`**, champ générique de 38-12b (vrai si une série existe au libellé du compte, selon le prestataire du compte ;
   jamais un secret) ; pour Wise, rien de plus (`configured` = libellé et jeton posés ; aucun indicateur de clé). Le bouton
   se montre à Comptable et plus quand `configured`. Non configuré : ligne d'aide (Administrateur seulement) ; « Importer
   un CSV » de 39-11d reste seul.
2. **AC-2 — Dialogue** : le composant générique de 38-12b, paramétré par le prestataire (route `wise-downloads`,
   libellés) : période (défaut : exercice ouvert), progression « étape `done`/`total` » (total inconnu jusqu'à la fin des
   transferts, 39-12a3 AC-2), pause « Wise demande de patienter », compte rendu de 39-11b plus, propres à Wise :
   `unlinkedCurrencies` (« soldes sans compte bancaire, non importés »), `feesIncluded` (« frais de conversion d'un relevé
   CSV déjà compris dans le montant téléchargé »), `balanceUnchecked` (« solde non contrôlé : aucun relevé antérieur dans
   Kesh pour cette devise ; comparez au site de Wise, surtout si des mouvements y sont en cours »), `ignored.notCompleted` (« mouvements en cours chez Wise, repris au prochain
   téléchargement »), avertissement `conversion_ambiguous` ; refus par code `PROVIDER_*`, `WISE_PROFILE_AMBIGUOUS`,
   `WISE_ACTIVITY_UNSUPPORTED` (« un mouvement de type … du … n'est pas pris en charge ; le téléchargement ne peut pas remonter avant cette date : importez le relevé CSV », la date pouvant être **hors de la période demandée**) et les raisons de `PROVIDER_RESPONSE_INVALID`
   (`WISE_API_BALANCE_MISMATCH` : « le solde reconstruit ne concorde pas avec votre relevé du … ; rien n'a été enregistré ; si un mouvement vient d'avoir lieu ou est en cours chez Wise, relancez quand il sera terminé ; si le refus persiste ensuite, voir le manuel », `amount_mismatch`) en clair aux quatre locales (409 : reprise du suivi par `details.jobId`), « rien n'a été
   enregistré, relancez » sur échec. **Aucun composant propre à Wise** : les libellés propres sont des paramètres. Le compte rendu de l'**import CSV** d'un compte Wise (39-11d) affiche aussi `feesIncluded` quand la clé est présente (39-12a3 AC-4, R1), avec le même libellé.
3. **AC-3 — Manuels** (`.tex` seul, aucun PDF commité ; PDF contrôlé à la régénération, `pdftotext … | tr '\n' ' '`).
   *Administration* : créer un jeton personnel **lecture seule** (profil d'entreprise) ; aucune authentification forte ni
   clé à déposer ; variables `KESH_WISE_<n>_ACCOUNT` (le libellé **identique** à celui du compte Wise de Kesh — **propre à
   l'installation** : deux sociétés qui portent le même libellé atteignent le même compte Wise), `_TOKEN`, `_PROFILE_ID`,
   `KESH_WISE_BASE_URL`, `KESH_WISE_PARALLEL`, dans **un tableau neuf du § `sec:env-vars`** (modèle `:840-862`) **et** le
   tableau de la section Wise, jamais le tableau SMTP (`:1155`) ; comment aller **au-delà de trois comptes** (lignes à
   ajouter aux compose) ; rien en base ni dans `.keshbackup` (à reposer après une restauration sur un autre serveur) ;
   rotation du jeton (redémarrage) ; refus de démarrage et leurs messages. **Ce que Kesh lit et ne lit pas** : les
   mouvements terminés des soldes (activités), les frais des virements sortants (par leur devis) ; **pas** le relevé de
   Wise (refusé par Wise aux jetons personnels hors de six pays) ; les frais de conversion sont **compris** dans le montant
   débité (le relevé CSV les met sur une ligne à part : les deux se mélangent sans doublon, 39-12a3 AC-4) ; un type de
   mouvement inconnu de Kesh refuse le téléchargement (importer le CSV de la période) ; le solde reconstruit est contrôlé
   contre le dernier relevé de Kesh, d'où le conseil : **un premier relevé importé en CSV**, puis des téléchargements ;
   Kesh relit le dernier jour du relevé précédent (`importFrom`, 39-12a3 AC-3) et accepte un relevé arrêté en cours de
   journée, si bien que télécharger « jusqu'à aujourd'hui » plusieurs fois ne perd ni ne double rien ; si la période demandée commence bien après ce relevé, Kesh comble le trou. *Refus qui dure* : un `WISE_API_BALANCE_MISMATCH` qui persiste après relance a deux causes, que le manuel distingue. (i) Un **mouvement en cours** chez Wise déjà porté au solde décale tous les soldes reconstruits : le relevé de Kesh n'est pas en cause, il faut attendre que le mouvement soit terminé (ou annulé ; le site de Wise le montre) puis relancer — **ne supprimer aucun relevé**. (ii) Sinon, le relevé nommé (sa date de fin) ne concorde pas avec l'activité de Wise (mouvement terminé ou mis à jour après coup, ou relevé importé sans contrôle pendant qu'un mouvement était en cours) : comparer son solde de clôture au site de Wise, puis, **seulement s'il en diffère** et qu'aucune de ses transactions n'est rapprochée, le supprimer (suppression d'un import, Story 27-1, `DELETE /api/v1/bank-imports/{id}`) et relancer ; sinon importer le CSV de la période.
   *Éprouvé sur 2024 seulement* : le manuel dit que la voie a été comparée au relevé CSV de 2024 et que 2025 et la suite se vérifient sur le premier téléchargement réel (39-12a1 AC-0 c) ; conseil : comparer le solde de clôture au site de Wise. *Volume* : un devis par virement sortant lu, de `readFrom` à aujourd'hui, donc un téléchargement ancien est long (progression par étape, `KESH_WISE_PARALLEL`). *Comptabilité des conversions* : les frais de conversion étant compris dans le montant débité, l'écriture d'une conversion téléchargée ne les passe pas à part en charge ; importés du CSV d'abord, ils se proposent au compte de frais — le traitement dépend de l'ordre. *Virement remboursé* : il se télécharge comme deux lignes (le débit, puis le crédit de remboursement, ce dernier à l'identifiant suffixé tant que 39-12a1 AC-0 (a)(8) n'a pas relevé celui du CSV) ; ne pas importer ce même virement une fois par le CSV et une fois par le téléchargement, les deux identifiants pouvant différer. *Type inconnu* : un mouvement d'un type non pris en charge, **même postérieur à la période demandée**, refuse tout téléchargement qui remonte avant sa date ; importer alors le CSV de la période. *Utilisateur* : le bouton, la période, la progression, ce qu'un échec veut dire (rien n'est enregistré), le contrôle de
   solde, le mélange sans doublon avec les CSV. `docs/api-external.md` : la route de 39-12a3 (sessions seules, 403
   `API_KEY_OPERATION_FORBIDDEN`), avec ses codes. Les deux phrases du registre des routes sont déjà relevées par 39-12a3 :
   ne pas les retoucher ici.
4. **AC-4 — E2E** `frontend/tests/e2e/wise-download.spec.ts`, **serveur de prestataire simulé de 38-12b (socle S8)**, port lu
   de `KESH_E2E_PROVIDER_MOCK_PORT`, **le serveur simulé commun gagne les routes de Wise** (`frontend/tests/e2e/support/provider-mock.ts`, que 38-12b ne fait jouer qu'à PayPal, en CHF : profils, soldes, activités paginées, transferts, devis ; un seul solde CHF, avec le compte lié créé par la spec ; un 401 et un solde décalé commandés par la spec), backend d'E2E démarré avec `KESH_WISE_1_*` (jeton fictif) et `KESH_WISE_BASE_URL` vers
   lui (ajout au montage de `docs/testing.md` ; le bloc de `CLAUDE.md` n'est pas touché) : sans variables, bouton absent ;
   période → progression → compte rendu (activités fictives de 39-12a2, une page de soldes, de transferts et un devis) ;
   second téléchargement → « 0 insérée » ; solde qui ne concorde pas (la spec a d'abord posé un relevé à solde par l'import CSV de 39-11d, dont la clôture diffère de ce que reconstruit le serveur simulé) → message de `WISE_API_BALANCE_MISMATCH` ; 401 simulé
   affiché. Sans la variable du port, ignoré avec son motif écrit (jamais compté vert). Vitest : indicateur `providerApi`
   (configuré, non configuré), libellés propres à Wise. Captures hors du dépôt.

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

*Ajouté* — Bouton « Télécharger depuis Wise » avec sa progression et son contrôle de solde ; le manuel d'administration
décrit le jeton en lecture seule et ce que le téléchargement lit.

## Change Log

- 2026-11-20 — validation P5 ciblée sur le commit `06a8f336` (Claude Sonnet 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P5-ciblee.md`) : 0 C, 0 H, 0 M, 1 L, remédié. Ici : le manuel dit le virement remboursé (deux lignes, ne pas mélanger CSV et téléchargement) et où voir qu'un mouvement est en cours ; message de `WISE_API_BALANCE_MISMATCH` et manuel à deux cas relus, cohérents.
- 2026-11-20 — validation P4 ciblée sur le commit `a98da37a` (Claude Opus 5.5, contexte frais ; commune à 39-12a1, 39-12a2, 39-12a3, 39-12b ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P4-ciblee.md`) : 0 C, 0 H, 1 M, 2 L, remédiés. Ici : **M** — le message de `WISE_API_BALANCE_MISMATCH` et le manuel menaient à supprimer un relevé juste quand un mouvement est en cours chez Wise ; ils distinguent désormais les deux causes (attendre la fin du mouvement ; supprimer seulement un relevé dont la clôture diffère du site de Wise).
- 2026-11-20 — validation P3 de la voie activités (Claude Sonnet 5.5, contexte frais ; commune à 39-12a1, 39-12a2, 39-12a3 ;
  rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P3.md`) : 0 C, 0 H, 3 M, 4 L, remédiés. Ici : **M2** — sortie
  d'un refus de solde qui dure (comparer, supprimer le relevé fautif non rapproché, relancer), message qui le dit ; L —
  libellé de `balanceUnchecked` (mouvements en cours), trou comblé par `importFrom`.
- 2026-11-20 — validation P2 de la voie activités (Claude Opus 5.5, contexte frais ; commune à 39-12a1, 39-12a2, 39-12a3 ;
  rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P2.md`) : 0 C, 1 H, 2 M, 9 L, remédiés. Ici : manuel —
  relevé arrêté en cours de journée et dernier jour relu (H1 de 39-12a2/39-12a3) ; chemin du serveur simulé commun.
- 2026-11-20 — validation P1 de la voie activités (Claude Sonnet 5.5, contexte frais ; commune à 39-12a1, 39-12a2, 39-12a3 ;
  rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P1.md`) : routes de Wise ajoutées au serveur simulé commun (E2E
  sans elles ne tournait pas), relevé préalable pour le cas du solde qui ne concorde pas, manuel — éprouvé sur 2024 seulement,
  volume d'appels, frais de conversion et ordre des imports, portée du refus d'un type inconnu —, message de solde (relancer),
  `feesIncluded` à l'écran d'import CSV.
- 2026-11-20 — **réécrite sur la voie activités (2026-11-20)** : clé de signature SCA, sa paire de clés, son dépôt chez
  Wise, son volume et `WISE_SCA_REJECTED` retirés ; libellés propres (`feesIncluded`, `balanceUnchecked`,
  `ignored.notCompleted`, `conversion_ambiguous`, `WISE_ACTIVITY_UNSUPPORTED`, `WISE_API_BALANCE_MISMATCH`) ; manuel : ce que
  Kesh lit et ne lit pas, premier relevé en CSV conseillé ; dépend de 39-12a3 (ex-39-12a2). Statut `deferred` → `drafted`,
  à revalider.

*Historique — entrées précédentes, laissées telles quelles :*


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

- 2026-11-20 — **reportée** : recettes réelles 1 et 2 (11 appels) : le relevé de solde par jeton personnel est refusé
  pour le compte suisse (403 `x-2fa-approval` à chaque requête ; rejeu signé par clé publique, signature vérifiée en
  local, toujours `REJECTED` ; endpoint de statut du jeton à usage unique 404). La doc publique des jetons personnels le
  dit : relevés par API non pris en charge hors US, CA, AU, NZ, SG, MY. Reprise seulement si Wise ouvre l'accès (support
  ou accord Wise Platform). Voie retenue : import CSV de la phase 1. Détails : `data-not-committed/wise-recette-ac0*.md`.
