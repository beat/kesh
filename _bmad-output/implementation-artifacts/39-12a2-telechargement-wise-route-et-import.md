# Story 39-12a2 : téléchargement des relevés Wise — route, tâche en fond, import et audit

Status: drafted

*Fiche rédigée le 2026-11-20 (seconde moitié de l'ancienne 39-12a, voir 39-12a1). Branche `epic/39-wise`. Epic 39,
**phase 2** ; **socle commun S1 à S9** (fichier d'epic). **Dépend de** : 39-12a1 (client, signature SCA), 39-11b (import CSV), du
**cœur commun `ProviderLine`** (`provider_imports.rs`) extrait par 38-12a — ou par 39-11b s'il passe avant (socle S6) ; aucune
fiche de phase 1 ne le factorise aujourd'hui, et 39-12a2 ne l'extrait pas : elle l'appelle, du **socle S2, S3, S4, S6** écrit par 38-12a (ou, à défaut, par la première fiche de phase 2 développée :
C-40-K6). **Suivie par** : 39-12b.*

## Story

En tant que **Comptable**, je veux que Kesh télécharge, pour une période que je choisis, le relevé de chaque solde de mon
compte Wise et l'importe comme un relevé CSV, sans doublon avec ce que j'ai déjà importé.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-12a2-1` | Une route lance le téléchargement d'une période en tâche de fond ; une autre en donne la progression et le résultat. | AC-1, AC-2 |
| `FR-39-12a2-2` | Tous les relevés sont lus **avant** toute écriture, puis importés par le cœur de 39-11b, une seule transaction de base. | AC-3 |
| `FR-39-12a2-3` | CSV et téléchargement se mélangent : doublons par identifiant Wise. | AC-3 |
| `FR-39-12a2-4` | Parallélisme réglé par `KESH_WISE_PARALLEL` (socle S2). | AC-4 |
| `NFR-39-12a2-1` | Aucun secret dans une réponse, un journal, une erreur ou une trace d'audit ; aucun appel réel dans les tests. | AC-5 |

## Critères d'acceptation

1. **AC-1 — Route** `POST /api/v1/wise-downloads` `{ account, from, to }` (socle S3 : **Comptable et plus, sessions seules —
   clés d'API non admises**, la route fait sortir un appel authentifié par un secret de l'Administrateur ; une clé d'API est refusée sur le `POST` **et** le `GET`, dans le gestionnaire, avant tout, en 403
   `API_KEY_OPERATION_FORBIDDEN`, patron de `bank_imports::delete`, `routes/bank_imports.rs:1688`, comme
   38-12a AC-8). Le `POST` est inscrit à `LIB_ROUTES` (`crates/kesh-api/tests/audit_route_registry.rs:440`) en
   `Traced, SansEcritureAuJournal` ; **dans cette story**, le compte de `the_registry_partition_is_what_the_story_declares`
   (`:1082`) est relevé de un avec sa raison et la phrase « N des M routes à verbe mutant » du manuel d'administration
   (`docs/manual/fr/admin-manual.tex`, lue par `le_manuel_dit_le_compte_du_registre`, `:1194`) mise à jour, **ainsi que la phrase « N + E + R = M »** du même paragraphe (lue par le même test) et les assertions à valeur littérale qui bougent (`LIB_ROUTES.len()` 157, `traced` 149, `LIB_ROUTES.len() + TEST_ENDPOINT_ROUTES.len()` 161 : relevées de un ; `exempt` et `no_matter` inchangés) — `.tex` seul, comptes **recomptés** après ceux des stories fusionnées avant (validation P2 : sans cela, le gate de cette story rougit,
   le manuel n'étant touché qu'en 39-12b). Refus dans cet ordre, rien lancé : 403 `API_KEY_OPERATION_FORBIDDEN` → 400
   `VALIDATION_ERROR` → 404 `PROVIDER_ACCOUNT_NOT_FOUND` (38-11b) → 422
   `PROVIDER_API_NOT_CONFIGURED` (aucun jeton pour ce libellé) → 400 `PROVIDER_PERIOD_INVALID` (`from` > `to`, `to` dans le
   futur à Zurich ; **aucun plafond de durée** : le socle découpe en fenêtres de 468 jours, 39-12a1) → 409
   `PROVIDER_DOWNLOAD_RUNNING` (une tâche tourne déjà pour ce libellé ; `details: { provider: "wise", jobId }`, pour que
   l'écran reprenne le suivi) ; sinon **202** `{ jobId }`.
2. **AC-2 — État** `GET /api/v1/wise-downloads/{jobId}` (socle S3, société de l'appelant seulement, sinon 404) :
   `{ state, steps: { done, total }, pausedUntilSeconds, result, error: { code, message, details? } }` (un seul nom, `details`). Une **étape** = un relevé demandé (un solde sur une
   fenêtre) ; `total` est connu après la liste des soldes (nul avant). `result` = le compte rendu de 39-11b AC-5 plus
   `unlinkedCurrencies: [..]` (soldes Wise sans compte bancaire de la devise : **listés, non téléchargés**, jamais créés
   d'office) et `downloaded: [{ currency, from, to }]`. ⚠️ Écart assumé avec PayPal (38-12a, qui échoue en
   `PROVIDER_CURRENCY_ACCOUNT_MISSING`) : Wise **liste d'avance** ses soldes, si bien qu'un solde sans compte se saute sans
   rien lire de lui ; PayPal ne connaît ses devises qu'après lecture. La tâche vit en mémoire (socle S2), perdue au redémarrage ;
   relancer est sans risque (AC-3).
3. **AC-3 — Lire d'abord, importer ensuite.** Toutes les étapes sont lues en mémoire (client de 39-12a1, rejeu SCA compris),
   **puis** passent, comme les fichiers d'un import CSV, par le **cœur commun de 39-11b** (socle S6 ; une entrée
   `bank_imports` par devise, `filename` = « API Wise <devise> <from>–<to> », verrous, doublons par identifiant de
   transaction Wise, **une seule transaction de base**, tout ou rien). **Une étape qui échoue (après les reprises) fait
   échouer la tâche entière : rien n'est écrit** — y compris pour les devises déjà lues (la version initiale de cette fiche
   laissait « les devises déjà importées » : abandonné, par cohérence avec PayPal et Stripe, relancer est sans risque).
   **Empreinte** (`file_hash`) = SHA-256 de `« wise-api:<devise> » ‖ identifiants triés` : le refus
   `BANK_IMPORT_DUPLICATE_FILE` **ne s'applique pas** à un téléchargement (relancer la même période rend `inserted: 0` ; un
   CSV puis un téléchargement, ou l'inverse, ne créent rien en double). La continuité avec l'import précédent (39-11b AC-5)
   se juge comme pour le CSV, sur le premier jour de la période.
4. **AC-4 — Parallélisme.** `KESH_WISE_PARALLEL` (socle S2, lue par 39-12a1) borne les relevés demandés en même temps pour
   l'ensemble du serveur ; la reprise sur 429 (pause commune, `Retry-After`, six essais) est celle du socle, **la même qu'à
   PayPal et Stripe** ; **aucune pause fixe entre deux requêtes** (la reprise sur 429 suffit : aucune limite de débit n'a été
   vue à la recette 1). Résultat **identique** à 1 et à 3 en parallèle (test qui compare les lignes importées).
5. **AC-5 — Audit, secrets, i18n.** Trace `wise_download.created` (`details` : libellé, période, étapes, compte rendu) sous
   l'acteur (`NewAuditLogEntry::for_actor`), à la fin réussie ; **aucun secret** (jeton, clé, défi, signature) ni
   `KESH_WISE_BASE_URL` dans une réponse ou une trace. Codes `PROVIDER_*` du socle S4 et `WISE_PROFILE_AMBIGUOUS`,
   `WISE_SCA_REJECTED` ; **la panique de la tâche** → `INTERNAL_ERROR`, non `PROVIDER_API_ERROR` (socle S4) (la clé SCA étant exigée au démarrage, 39-12a1 AC-1, aucun code « clé non posée ») : `errors.rs` et messages aux quatre locales
   (`crates/kesh-i18n/locales/*`) — les `PROVIDER_*` du socle sont posés par 38-12a, seuls les `WISE_*` le sont ici ; garde
   d'audit `audit_acteur_garde.rs` verte.
6. **AC-6 — Tests** (`crates/kesh-api/tests/wise_download_e2e.rs`, serveur Wise simulé de 39-12a1, base `#[sqlx::test]`) :
   période de 500 jours (deux fenêtres), trois soldes dont un sans compte bancaire (`unlinkedCurrencies`) → import par
   devise, soldes d'ouverture et de clôture exacts ; CSV (fixture de 39-11a) puis téléchargement des mêmes mouvements → rien de
   doublé, et l'inverse ; relance de la même période → `inserted: 0` ; défi SCA puis rejeu, et relevé servi sans défi ;
   échec à la deuxième étape → rien d'écrit ; 429 puis reprise ; parallélisme 3 = parallélisme 1 ; deux tâches du même
   libellé → 409 avec `details.jobId` ; clé d'API → 403 `API_KEY_OPERATION_FORBIDDEN` sur le `POST` et le `GET` ; registre et phrase
   du manuel au compte ; secrets absents des journaux (test au marqueur de 39-12a1).

## Tâches

- [ ] T1 — Route, état, tâche (socle S2/S3) (AC-1, AC-2).
- [ ] T2 — Lecture de toutes les étapes, appel du cœur d'import, empreinte (AC-3).
- [ ] T3 — Parallélisme, audit, erreurs, i18n (AC-4, AC-5).
- [ ] T4 — Tests (AC-6).

## Stratégie de tests

Gate ciblé : `binary(wise_download_e2e)` et `binary(audit_route_registry)` ; **gate complet** au dernier commit (le cœur d'import de 39-11b est appelé : ses
tests et `audit_route_registry` restent verts sans changement d'assertion). Aucune migration : P5 à P8 sans objet.

## Modules touchés (3)

`kesh-api` (route, tâche, import), `kesh-i18n`, `kesh-db` (aucune écriture neuve : appel du cœur de 39-11b ; compté par
prudence) ; plus la seule phrase du registre dans `docs/manual/fr/admin-manual.tex`.

## Fichiers communs

`crates/kesh-api/src/lib.rs` (routes, état), `errors.rs`, `audit_route_registry.rs` et la phrase du registre de
`admin-manual.tex` (comptes recomptés), catalogues i18n (leur bloc), cœur d'import de 39-11b : à relire à
la reprise de `main`.

## Note de version

*Ajouté* — Kesh télécharge les relevés Wise d'une période par l'API de Wise (signature d'authentification forte comprise) et
les importe comme un relevé CSV, sans doublon ; tout ou rien en cas d'échec.

## Notes de développement

- Aucun appel réel en test ni en CI ; la première exécution réelle est celle de Beat, avec sa clé publique déposée.
- Si la recette 2 (39-12a1 AC-0) contredit une hypothèse (colonnes du CSV de l'API), la correction va à la conversion de
  39-12a1, jamais au cœur de 39-11b.

## Change Log

- 2026-11-20 — fiche rédigée (seconde moitié de l'ancienne 39-12a), non validée.
- 2026-11-20 — validation P1 (Claude Sonnet 5.5, contexte frais ; commune à 39-12a1, 39-12a2, 39-12b et à l'Epic 40) :
  **3 HIGH, 11 MEDIUM, 8 LOW, remédiés** (rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-P1.md`). Pour 39-12 :
  HIGH — l'arrêt de l'AC-0 sur un défi SCA, que la recette 1 a constaté : voie SCA intégrée (39-12a1) ; MEDIUM — codes
  `WISE_*` redondants avec les `PROVIDER_*` génériques, route d'état sans identifiant de tâche, politique de reprise (30 s,
  cinq essais) et pause fixe différentes de PayPal, importation devise par devise (écriture partielle), refus de l'empreinte
  appliqué à un téléchargement identique, clés d'API admises sur une route qui emploie un secret, `reqwest` sans `json`
  (`Cargo.toml:112`) et transport factice non HTTP, raccord de plusieurs fenêtres d'une devise non défini, libellé de compte
  commun à toutes les sociétés non dit ; LOW — plafond de 469 jours (devenu fenêtres), budget d'appels réels non écrit,
  indicateur de configuration vague, parallélisme sans effet sur trois requêtes (gardé : décision de l'epic).
- 2026-11-20 — validation P2 (Claude Opus 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-P2.md`) :
  ici, compte du registre et phrase du manuel relevés **dans cette story** (son gate les lit), clé d'API refusée en 403
  `API_KEY_OPERATION_FORBIDDEN` sur les deux routes (comme 38-12a), 409 avec `details: { provider, jobId }`, `details` seul
  nom, `WISE_SCA_KEY_NOT_CONFIGURED` retiré, écart assumé avec PayPal sur les soldes sans compte écrit, fenêtres de 468 jours.
- 2026-11-20 — validation P3 ciblée (Claude Sonnet 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-40-phase2-P3-ciblee.md`) : registre : les **deux** phrases du manuel et les six `assert_eq!` du test relevées ; panique de tâche → `INTERNAL_ERROR`.
- 2026-11-20 — validation P4 ciblée (Claude Haiku 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-40-phase2-P4-ciblee.md`) : registre : six `assert_eq!` dont cinq littéraux, les deux phrases du manuel ; garde : noms sans numéro `KESH_WISE_BASE_URL` et `KESH_WISE_PARALLEL` inscrits ; `payout_total_mismatch` (code exact de 40-7c).
- 2026-11-20 — validation P5 ciblée (Claude Sonnet 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-40-phase2-P5-ciblee.md`) : 0 C, 0 H, 0 M, 1 L, remédié : entrée P4 limitée à ce qui concerne ce fichier (la recette 2 relève de 39-12a1), casse des noms.
- 2026-11-20 — validation P2, complément (Claude Opus 5.5)
 : cœur `ProviderLine` extrait par 38-12a (ou 39-11b), appelé ici.
