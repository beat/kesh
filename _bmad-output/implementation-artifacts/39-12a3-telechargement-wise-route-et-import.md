# Story 39-12a3 : téléchargement des mouvements Wise — route, tâche en fond, import, doublons avec le CSV, audit

Status: ready-for-dev

*Fiche réécrite le 2026-11-20 sur la **voie activités** (recherche du 2026-11-20, `data-not-committed/wise-recherche.md`,
métadonnées seulement) ; elle portait le numéro **39-12a2** (« téléchargement des relevés Wise — route, tâche en fond,
import et audit »), renumérotée 39-12a3 parce que la conversion des activités en lignes prend la 39-12a2. Branche
`epic/39-wise`. Epic 39, **phase 2** ; **socle commun S1 à S9** (fichier d'epic). **Dépend de** : 39-12a1 (client),
39-12a2 (conversion), 39-11b (import CSV), 39-11c (rapprochement), du **cœur commun `ProviderLine`**
(`provider_imports.rs`) extrait par 38-12a — ou par 39-11b s'il passe avant (socle S6) —, du **socle S2, S3, S4, S6**
écrit par 38-12a (ou, à défaut, par la première fiche de phase 2 développée : C-40-K6). **Suivie par** : 39-12b.*

## Story

En tant que **Comptable**, je veux que Kesh télécharge, pour une période que je choisis, les mouvements de chaque solde
de mon compte Wise et les importe comme un relevé CSV, sans doublon avec ce que j'ai déjà importé à la main, et
seulement si le solde reconstruit concorde avec mes relevés.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-12a3-1` | Une route lance le téléchargement d'une période en tâche de fond ; une autre en donne la progression et le résultat. | AC-1, AC-2 |
| `FR-39-12a3-2` | Tout est lu **avant** toute écriture (soldes, activités, transferts, devis), converti par 39-12a2 et contrôlé, puis importé par le cœur de 39-11b, une seule transaction de base. | AC-3 |
| `FR-39-12a3-3` | CSV et téléchargement se mélangent dans les deux ordres sans doublon, frais de conversion compris. | AC-4 |
| `FR-39-12a3-4` | Parallélisme réglé par `KESH_WISE_PARALLEL` (socle S2). | AC-5 |
| `NFR-39-12a3-1` | Aucun secret dans une réponse, un journal, une erreur ou une trace d'audit ; aucun appel réel dans les tests. | AC-6 |

## Critères d'acceptation

1. **AC-1 — Route** `POST /api/v1/wise-downloads` `{ account, from, to }` (socle S3 : **Comptable et plus, sessions seules —
   clés d'API non admises**, la route fait sortir un appel authentifié par un secret de l'Administrateur ; une clé d'API est
   refusée sur le `POST` **et** le `GET`, dans le gestionnaire, avant tout, en 403 `API_KEY_OPERATION_FORBIDDEN`, patron de
   `bank_imports::delete`, `routes/bank_imports.rs:1688`, comme 38-12a AC-8). Le `POST` est inscrit à `LIB_ROUTES`
   (`crates/kesh-api/tests/audit_route_registry.rs:440`) en `Traced, SansEcritureAuJournal` ; **dans cette story**, le compte
   de `the_registry_partition_is_what_the_story_declares` (`:1082`) est relevé de un avec sa raison, et **les deux phrases**
   du manuel d'administration lues par `le_manuel_dit_le_compte_du_registre` (`:1194`) — « N des M routes à verbe mutant » et
   « N + E + R = M », même paragraphe de `docs/manual/fr/admin-manual.tex` — ainsi que les assertions à valeur littérale qui
   bougent (`LIB_ROUTES.len()`, `traced`, `LIB_ROUTES.len() + TEST_ENDPOINT_ROUTES.len()` : relevées de un ; `exempt` et
   `no_matter` inchangés) — `.tex` seul, comptes **recomptés** après ceux des stories fusionnées avant (sans cela le gate de
   cette story rougit, le manuel n'étant touché qu'en 39-12b). Refus dans cet ordre, rien lancé : 403
   `API_KEY_OPERATION_FORBIDDEN` → 400 `VALIDATION_ERROR` → 404 `PROVIDER_ACCOUNT_NOT_FOUND` (38-11b) → 422
   `PROVIDER_API_NOT_CONFIGURED` (aucun jeton pour ce libellé) → 400 `PROVIDER_PERIOD_INVALID` (`check_period` du socle, `details.reason` = `from_after_to` ou `future` — `from` après le jour de Zurich du jour ; **une `to` future n'est pas refusée, elle est ramenée à l'instant**, comme pour PayPal —, **sans** `too_old` : l'API des activités n'a pas de borne d'ancienneté, le paramètre `years` reçoit **100** (une valeur qui ne limite rien ; jamais `u32::MAX` : `check_period` calcule
   `Months::new(12 * years)`, qui déborderait) ; aucun plafond de durée) → 409 `PROVIDER_DOWNLOAD_RUNNING` (une tâche tourne déjà pour ce libellé ;
   `details: { provider: "wise", jobId }`) ; sinon **202** `{ jobId }`.
2. **AC-2 — État** `GET /api/v1/wise-downloads/{jobId}` (socle S3, société de l'appelant seulement, sinon 404) :
   `{ state, steps: { done, total }, pausedUntilSeconds, result, error: { code, message, details? } }`. **Étapes** : profil,
   soldes, activités (toutes les fenêtres et pages : une étape), transferts (une étape), puis **une par devis** ; `total`
   est nul jusqu'à la fin de l'étape « transferts », puis vaut 4 + le nombre de devis. `result` = le compte rendu de 39-11b
   AC-5 (par devise : `inserted`, `duplicates`, `openingBalance`, `closingBalance`, `warnings`) plus `feesIncluded` (lignes
   de frais de conversion d'un CSV reconnues incluses, AC-4), `ignored: { notCompleted, cardChecks }` (39-12a2),
   `balanceUnchecked: [devise]` (aucun relevé de Kesh n'a permis le contrôle, AC-3), `unlinkedCurrencies: [..]` (soldes
   Wise sans compte bancaire de la devise dans Kesh : **listés, non importés**, jamais créés d'office) et
   `downloaded: { from, importFrom, to, readFrom }` (`importFrom`, AC-3 (4)). ⚠️ Écart assumé avec PayPal (38-12a, `PROVIDER_CURRENCY_ACCOUNT_MISSING`) : Wise
   liste d'avance ses soldes. **Une activité dans une devise non liée** n'empêche pas l'import des autres : ses lignes ne
   sont simplement pas importées, et la devise figure dans `unlinkedCurrencies`. La tâche vit en mémoire (socle S2),
   perdue au redémarrage ; relancer est sans risque (AC-4).
3. **AC-3 — Lire, convertir, contrôler, puis importer.** Ordre : (1) profil ; (2) **soldes**, à l'instant `read_until`
   (`now`) ; (3) **points de contrôle** dans Kesh, pour chaque devise liée : la clôture du relevé à solde antérieur à `from`
   (`previous_statement_closing`, `crates/kesh-db/src/repositories/bank_imports.rs:358`) et celle de chaque relevé à solde
   du compte qui se termine entre `from` et aujourd'hui (`overlapping_statements`, `:386`) — relevés importés (CSV ou
   téléchargement) ou saisis ; (4) **début d'import** `importFrom` = le plus tôt de `from` et du dernier jour (`period_to`) du relevé
   de `previous_statement_closing` — un relevé arrêté en cours de journée (téléchargement jusqu'à l'instant, CSV exporté
   dans la journée) laisse en dehors de Kesh les mouvements de la fin de ce jour ; les relire, doublons ignorés par
   identifiant, les rattrape, alors qu'une période commencée au lendemain les perdrait sans rien dire —, et **début de
   lecture** `readFrom` = `importFrom` (le jour du plus ancien point de contrôle, relevé précédent ou relevé qui
   chevauche, n'est jamais antérieur à `importFrom` : une clause de plus serait sans effet ; le contrôle de 39-12a2 AC-5 lit
   les instants de ce jour, que la lecture depuis le début de ce jour atteint). Quand `from` est bien postérieur au
   relevé précédent, `importFrom` comble le trou (les mouvements entre les deux sont importés et le compte rendu le dit
   par `downloaded.importFrom`) ; (5) **activités** de `readFrom` à
   `read_until` (fenêtres de 366 jours, 39-12a1) ; (6) **transferts** (`readFrom` − 31 jours à `read_until`) et **devis**
   de chaque virement sortant `COMPLETED` lu ; (7) `rows_from_activities` (39-12a2) avec ces données et `period = (importFrom, to)`, **les horodatages des activités ramenés avant l'appel** au fuseau que la recette 39-12a1 AC-0 (a)(1) a retenu (`kesh_core::horloge::heure_suisse`, ajoutée à `kesh-core/src/horloge.rs` par 38-12a, si c'est Zurich ; `kesh-import` ne connaît aucun fuseau) ; un horodatage illisible (texte rendu tel quel par 39-12a1) → `PROVIDER_RESPONSE_INVALID`, `details.reason = "activity_date"`. Les soldes sont lus à l'instant `read_until` = `now` et les activités jusqu'à lui : un mouvement survenu entre les deux lectures fait échouer le contrôle (`BalanceMismatch`), le message invite à relancer.
   **Une erreur à n'importe quelle étape — reprises épuisées, conversion refusée, `BalanceMismatch` — fait échouer la tâche
   entière : rien n'est écrit**, pour aucune devise. Puis les `WiseCsv` rendus, **ceux des devises non liées retirés d'abord** (le cœur refuse sinon toute
   l'opération en 422 `PROVIDER_CURRENCY_ACCOUNT_MISSING` ; elles vont à `unlinkedCurrencies`, AC-2), passent, comme les
   fichiers d'un import CSV, par le **cœur commun de 39-11b** (socle S6 ; une entrée `bank_imports` par devise, `source_format` = la valeur de 39-11b,
   `filename` = « Wise API <devise> <importFrom>…<to> » (forme du « PayPal API <from>…<to> » de 38-12a, devise en plus :
   un nom **par devise**, que le cœur accepte depuis 39-11b — un nom et une empreinte par fichier ; le cœur de 38-12a
   n'en prend qu'un pour toutes les devises), verrous, doublons par identifiant Wise et règles de l'AC-4, **une seule
   transaction de base**, tout ou rien). **Empreinte** (`file_hash`) = SHA-256 des identifiants des lignes de la devise, triés, ‖ `":"` ‖ devise (la forme de 38-12a AC-7 et du socle S6) :
   le refus `BANK_IMPORT_DUPLICATE_FILE` **ne s'applique pas** à un téléchargement (relancer la même période rend
   `inserted: 0`). La continuité avec l'import précédent (39-11b AC-5) se juge comme pour le CSV ; elle est ici garantie par
   le contrôle de 39-12a2 AC-5 quand un point de contrôle existe.
4. **AC-4 — Doublons avec le CSV, dans les deux ordres.** Règle de base (39-11b AC-5) : même `transaction_id` sur le même
   compte bancaire → ligne ignorée. S'y ajoutent trois règles **propres à Wise**, qui ne s'écrivent **pas** dans le chemin
   commun tel quel : le cœur `provider_imports.rs` reçoit un paramètre `rules: Option<&dyn DuplicateRules>` (rien par défaut),
   appelé **sous les verrous et dans la transaction d'écriture**, après le test d'identifiant et **avant** le calcul de la période et de la clôture de la devise et le cas « aucune ligne gardée » (qui jugent les lignes restantes) ; **PayPal (38-11b, 38-12a) et
   Stripe (40-7b) passent `None` et se comportent comme avant, octet pour octet** — un doublon de montant différent y reste
   ignoré, test de non-régression sur une entrée PayPal ; Wise (la route CSV de 39-11b, `wise_imports::create`, **et** cette
   story) passe `WiseDuplicateRules` (module neuf `crates/kesh-api/src/wise_duplicates.rs`, préfixes d'identifiant Wise
   seuls). **Contrat du point d'accroche** (sans dépendance neuve) : le paramètre est un champ de la demande au cœur
   (`ImportRequest` de 38-12a), et chaque prestataire qui la construit le pose — **PayPal (`paypal_imports.rs`,
   `paypal_downloads.rs`) et Stripe (`stripe_imports.rs`) sont donc touchés d'une ligne**, `None` ; le trait est
   **objet-sûr sans `async_trait`**, sa méthode rendant une future en boîte (patron de `mail::MailFuture`,
   `crates/kesh-api/src/mail/mod.rs:12`, et de `exchange_rates/source.rs:23`), et reçoit, par devise : la connexion de la
   transaction d'écriture, le compte bancaire, la **source** (`ImportSource::File` ou `Download` : R2 ne vaut qu'au
   téléchargement), les lignes **gardées** et les lignes **écartées comme doublons** par le test d'identifiant ; elle rend
   pour chaque ligne gardée « garder » ou « ignorer » (R1, R3), un refus (R2, la tâche échoue), un compte `fees_included`
   et des avertissements. Le cœur reporte ces deux derniers au compte rendu de la devise (`CurrencyReport`) en champs
   **omis quand ils sont vides** (`skip_serializing_if`) : la réponse de PayPal et de Stripe garde sa forme, octet pour
   octet. Une ligne ignorée par R1 ou R3 compte dans `duplicates` (elle n'est pas insérée) **et**, pour R1, dans
   `feesIncluded`. *(Relevé au code des worktrees de 38-12a et de 40-7b le 2026-11-20, P3 : le cœur de 40-7b **reprend** celui de 38-12a —
   `ProviderLine`, `ImportRequest`, `import_lines` — et n'y ajoute que `ProviderLine.value_date` (`None` pour Wise), plus des
   aides de formulaire ; il n'y a plus qu'une forme, au contraire de ce que la P2 avait relevé. `dedupe_lines` garde sa
   signature et ses tests : les lignes écartées pour le point d'accroche se déduisent des lignes gardées. Les copies
   privées de PayPal que 40-7b dit « à replier » ne concernent pas Wise.)* Le dépôt des transactions bancaires gagne **deux lectures** (`existing_transaction_ids` ne rend que des
   identifiants) : `existing_transactions_by_ids(company, compte, ids)` et `transactions_by_prefix_window(company, compte,
   préfixe, du, au)`, rendant `(transaction_id, amount, booking_date)`. Les tests de 39-11b restent verts sans changement
   d'assertion (un ré-import à l'identique ne remplit aucune condition ci-dessous).
   - **(R1) Frais de conversion inclus.** Une ligne de frais `FEE-BALANCE-x` ou `FEE-CONVERSION_ORDER-x` d'un CSV dont la
     ligne d'origine `x` est un doublon, et dont l'origine en base a pour montant `montant de x dans le fichier + montant
     des frais` (débit téléchargé, frais inclus, 39-12a2 AC-4), est **ignorée** et comptée dans `feesIncluded` — sans quoi
     les frais compteraient deux fois. « L'origine en base » est la ligne de même identifiant, ou celle que (R3) a
     reconnue pour un `CONVERSION_ORDER-x` du CSV (une `AUTO_CONVERSION-y` téléchargée). Le compte est rendu par fichier à la réponse de l'import CSV sous la clé
     `feesIncluded`, **absente quand elle vaut 0** (les réponses de 39-11b gardent leur forme) ; l'écran d'import CSV
     (39-11d) l'affiche avec le libellé de 39-12b.
   - **(R2) Montant différent.** Hors (R1), un doublon de **téléchargement** dont le montant diffère de la ligne en base →
     la tâche échoue, `PROVIDER_RESPONSE_INVALID`, `details.reason = "amount_mismatch"` (`currency`, `id`) ; rien n'est
     écrit. **Exemption — CSV d'abord, puis téléchargement** : le débit téléchargé d'une conversion porte les frais, la
     ligne du CSV en base non ; si la ligne en base est un débit `BALANCE-x` ou `CONVERSION_ORDER-x`, que le compte porte sa
     ligne `FEE-BALANCE-x` ou `FEE-CONVERSION_ORDER-x` de montant `f` et que `montant téléchargé = montant en base + f`, le
     doublon est **ignoré** (sans cette exemption, le mélange CSV puis téléchargement des conversions échouerait à chaque
     fois, en contradiction avec cette AC) ; sans ligne de frais en base ou avec un autre écart, `amount_mismatch`. L'import
     CSV, lui, garde le comportement de 39-11b (doublon ignoré).
   - **(R3) Conversion automatique, tant que son identifiant est provisoire** (39-12a2 AC-3 ; règle retirée si 39-12a1
     AC-0 (a)(2) trouve le numéro du CSV) : une ligne `AUTO_CONVERSION-y` est un doublon si le compte porte une ligne
     `CONVERSION_ORDER-n` de date à un jour près et de même montant (crédit), ou dont le montant plus celui de
     `FEE-CONVERSION_ORDER-n` égale le sien (débit) ; symétriquement, une ligne CSV `CONVERSION_ORDER-n` (et sa ligne de
     frais, par R1) est un doublon d'une ligne `AUTO_CONVERSION-y` en base qui remplit la même condition. Plusieurs
     candidates → aucune n'est retenue et la ligne est importée, avec l'avertissement `conversion_ambiguous` au compte
     rendu, **qui nomme les identifiants des candidates** (jamais un rattachement deviné : une conversion peut alors figurer
     deux fois, visible et à rapprocher à la main). La recherche se fait par `transactions_by_prefix_window`.
   - **Rapprochement** (39-11c) : le préfixe `AUTO_CONVERSION` s'ajoute aux préfixes de conversion reconnus (même
     identifiant sur deux soldes du même compte Wise) ; l'écart `fee_without_origin` ne vise que les lignes `FEE-x`, une
     conversion téléchargée sans ligne de frais n'en est donc pas un (aucun écart « origine sans frais » n'existe).
5. **AC-5 — Parallélisme.** `KESH_WISE_PARALLEL` (socle S2, lue par 39-12a1) borne les requêtes de devis faites en même
   temps pour l'ensemble du serveur (les activités et transferts se lisent dans l'ordre, par curseur et par `offset`) ; la
   reprise sur 429 (pause commune, `Retry-After`, six essais) est celle du socle, la même qu'à PayPal et Stripe ; **aucune
   pause fixe** entre deux requêtes. Résultat **identique** à 1 et à 3 en parallèle (test qui compare les lignes importées).
6. **AC-6 — Audit, secrets, i18n.** Trace `wise_download.created` (`details` : libellé, période, `readFrom`, étapes, compte rendu) sous l'acteur (`NewAuditLogEntry::for_actor`), **écrite dans la même transaction que l'import** (comme `paypal_download.created`, 38-12a AC-7), plus `bank_import.created` par devise comme à l'import CSV, jamais `wise_import.created` ; l'action est **déclarée à `ACTIONS` de `crates/kesh-api/src/audit_labels.rs` avec son libellé dans les quatre catalogues** (la garde `audit_label_registry.rs` rougit sinon) ; **aucun secret** (jeton) ni `KESH_WISE_BASE_URL`
   dans une réponse ou une trace. Codes `PROVIDER_*` du socle S4, `WISE_PROFILE_AMBIGUOUS` et **`WISE_ACTIVITY_UNSUPPORTED`** (`details: { activityType, date }` : un mouvement de ce type, daté de `date`, se trouve entre le début de lecture et maintenant, même hors de la période demandée, et le solde ne peut plus se reconstruire : le message le dit, et que le téléchargement ne remontera pas avant cette date ; le manuel dit d'importer le CSV) ; les autres refus de 39-12a2
   deviennent `PROVIDER_RESPONSE_INVALID` avec `details.reason` = leur code (`WISE_API_BALANCE_MISMATCH` porte aussi
   `currency`, `date`, `expected`, `found`) ; **la panique de la tâche** → `INTERNAL_ERROR` (socle S4). `errors.rs` et
   messages aux quatre locales (`crates/kesh-i18n/locales/*`) — les `PROVIDER_*` du socle sont posés par 38-12a, seuls les
   `WISE_*` et les raisons le sont ici ; garde d'audit `audit_acteur_garde.rs` verte.
7. **AC-7 — Tests** (`crates/kesh-api/tests/wise_download_e2e.rs`, serveur Wise simulé de 39-12a1, base `#[sqlx::test]`,
   activités fictives de 39-12a2 qui redonnent les fixtures CSV de 39-11a) : période de 400 jours (deux fenêtres), trois
   soldes dont un sans compte bancaire (`unlinkedCurrencies`) → import par devise, soldes d'ouverture et de clôture
   exacts ; **CSV puis téléchargement** des mêmes mouvements → rien de doublé ; **téléchargement puis CSV** → rien de
   doublé, `FEE-BALANCE-…` comptée dans `feesIncluded`, solde du compte égal dans les deux ordres ; conversion automatique dans les deux ordres (R3), et deux candidates → `conversion_ambiguous` ; **CSV d'abord puis téléchargement d'une conversion à frais** → doublon ignoré par l'exemption de R2, non `amount_mismatch` ;
   **relevé arrêté en cours de journée** (un téléchargement jusqu'à l'instant, puis un mouvement le même jour, puis un
   téléchargement dont `from` est le lendemain) → contrôle accepté, le mouvement de la fin du jour importé (`importFrom`),
   les autres lignes de ce jour en doublon ; une devise non liée → aucune 422, devise listée ; **non-régression** : le cœur appelé avec `None` (PayPal) ignore un doublon de montant différent comme avant ; doublon de montant différent → `amount_mismatch`,
   rien d'écrit ; relevé antérieur dont la clôture ne concorde pas → `WISE_API_BALANCE_MISMATCH`, rien d'écrit ; aucun relevé
   antérieur → `balanceUnchecked` ; relance de la même période → `inserted: 0` ; activité d'un type non pris en charge →
   `WISE_ACTIVITY_UNSUPPORTED` ; échec au troisième devis → rien d'écrit ; 429 puis reprise ; parallélisme 3 =
   parallélisme 1 ; deux tâches du même libellé → 409 avec `details.jobId` ; clé d'API → 403 sur le `POST` et le `GET` ;
   registre et phrases du manuel au compte ; jeton absent des journaux (test au marqueur de 39-12a1). Tests de 39-11b et 39-11c verts sans changement d'assertion (la clé `feesIncluded` n'apparaît pas dans leurs réponses).

## Tâches

- [ ] T1 — Route, état, tâche (socle S2/S3) (AC-1, AC-2).
- [ ] T2 — Ordre de lecture, points de contrôle, appel de 39-12a2, appel du cœur d'import, empreinte (AC-3).
- [ ] T3 — Point d'accroche `rules` du cœur commun, règles R1 à R3 côté Wise (`wise_duplicates.rs`), deux lectures du dépôt, préfixe `AUTO_CONVERSION` de 39-11c (AC-4).
- [ ] T4 — Parallélisme, audit, erreurs, i18n (AC-5, AC-6).
- [ ] T5 — Tests (AC-7).

## Stratégie de tests

Gate ciblé : `binary(wise_download_e2e)`, `binary(audit_route_registry)` et les binaires de 39-11b et 39-11c ;
**gate complet** à chaque patch qui touche un dépôt de `kesh-db` (règle d'exception de `CLAUDE.md`) et au dernier commit.
Aucune migration : P5 à P8 sans objet.

## Modules touchés (4)

`kesh-api` (route, tâche, cœur d'import, `wise_duplicates.rs`, `audit_labels.rs`), `kesh-db` (deux lectures du dépôt des transactions bancaires, R1 à R3),
`kesh-reconciliation` (un préfixe de conversion), `kesh-i18n` ; plus les deux phrases du registre dans
`docs/manual/fr/admin-manual.tex`.

## Fichiers communs

`crates/kesh-api/src/lib.rs` (routes, état), `errors.rs`, `provider_imports.rs` (cœur commun : un paramètre `rules` facultatif y entre, PayPal et Stripe le passent à `None`),
`routes/paypal_imports.rs` et `routes/paypal_downloads.rs` (38-11b, 38-12a) et `routes/stripe_imports.rs` (40-7b) : une
ligne chacun, `None` ; `routes/wise_imports.rs` (39-11b : la route CSV passe `WiseDuplicateRules`), `audit_route_registry.rs` et les phrases du registre de
`admin-manual.tex` (comptes recomptés), catalogues i18n (leur bloc), dépôt des transactions bancaires : à relire à la
reprise de `main`.

## Note de version

*Ajouté* — Kesh télécharge les mouvements Wise d'une période par l'API de Wise (jeton en lecture seule, sans
authentification forte), contrôle le solde reconstruit contre les relevés déjà importés et les importe comme un relevé
CSV, sans doublon dans un sens comme dans l'autre ; tout ou rien en cas d'échec.

## Notes de développement

- Aucun appel réel en test ni en CI ; la première exécution réelle est celle de Beat, après la recette finale de 39-12a1
  (AC-0 b).
- Un écart de la recette (règle de sens, date, identifiant) se corrige dans 39-12a2, jamais dans le cœur de 39-11b.
- Les règles R1 à R3 ne touchent que des identifiants Wise et vivent côté Wise ; le cœur commun ne gagne qu'un point d'accroche, que PayPal et Stripe laissent à `None`.
- Volume d'appels : un devis par virement sortant lu de `readFrom` à maintenant, donc quelques appels pour un mois courant, plusieurs centaines pour un an lu depuis un relevé ancien (progression par étape, `KESH_WISE_PARALLEL`) ; le manuel (39-12b) le dit.
- Aucun outil MCP (comme 39-11b).

## Change Log

- 2026-11-20 — validation P5 ciblée sur le commit `06a8f336` (Claude Sonnet 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P5-ciblee.md`) : rien à changer ; point d'accroche et `readFrom` non touchés par le commit.
- 2026-11-20 — validation P4 ciblée sur le commit `a98da37a` (Claude Opus 5.5, contexte frais ; commune à 39-12a1, 39-12a2, 39-12a3, 39-12b ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P4-ciblee.md`) : 0 C, 0 H, 1 M, 2 L, remédiés. Ici : rien à changer (point d'accroche avant `kept.is_empty()`, la période et la clôture, relu au cœur de 38-12a ; cœur de 40-7b = celui de 38-12a plus `value_date` et des aides de formulaire ; `readFrom` = `importFrom` vérifié contre `previous_statement_closing` et `overlapping_statements`).
- 2026-11-20 — validation P3 de la voie activités (Claude Sonnet 5.5, contexte frais ; commune à 39-12a1, 39-12a2, 39-12b ;
  rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P3.md`) : 0 C, 0 H, 3 M, 4 L, remédiés. Ici : **M3** — le
  point d'accroche `rules` s'exécute avant la période, la clôture et le cas « aucune ligne gardée » ; le renvoi « deux
  extractions divergent » est périmé (40-7b reprend le cœur de 38-12a) ; **L** — `readFrom` = `importFrom` (la clause du
  point de contrôle était sans effet), trou comblé par `importFrom` écrit.
- 2026-11-20 — validation P2 de la voie activités (Claude Opus 5.5, contexte frais ; commune à 39-12a1, 39-12a2, 39-12b ;
  rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P2.md`) : 0 C, 1 H, 2 M, 9 L, remédiés. Ici : **H1** —
  `importFrom` (dernier jour du relevé précédent, relu) et `readFrom` au jour du plus ancien point de contrôle, avec la
  tolérance « à un instant du jour » de 39-12a2 AC-5 (sinon refus durable après un téléchargement jusqu'à l'instant, et
  mouvements de la fin du jour perdus) ; **M1** — contrat du point d'accroche `rules` : champ de `ImportRequest`, future en
  boîte (patron `MailFuture`, sans `async_trait`), source connue (R2 au seul téléchargement), lignes gardées et écartées,
  `fees_included` et avertissements omis à vide, appelants PayPal et Stripe aux fichiers communs, deux extractions du cœur
  à réunir (38-12a, 40-7b) ; devises non liées retirées avant le cœur (422 sinon) ; `years` = 100 ; horodatage illisible ;
  nom par devise ; origine reconnue par R3 pour R1.
- 2026-11-20 — validation P1 de la voie activités (Claude Sonnet 5.5, contexte frais ; commune à 39-12a1, 39-12a2, 39-12b ;
  rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P1.md`) : **R1 à R3 sortis du cœur commun** (R2 aurait changé
  PayPal et Stripe : point d'accroche `rules`, `None` pour eux) ; **exemption de R2** pour le CSV d'abord (le débit téléchargé
  à frais inclus n'est pas le débit du CSV : la fiche échouait dans l'ordre qu'elle déclarait sans effet) ; deux lectures du
  dépôt (montants par identifiant, fenêtre par préfixe) ; `feesIncluded` aussi à l'import CSV, absent à 0 ;
  `PROVIDER_PERIOD_INVALID` aligné sur `check_period` (`to` future ramenée, pas de `too_old`) ; empreinte de 38-12a ; audit dans
  la transaction, `bank_import.created`, `audit_labels.rs` ; horodatages ramenés ici ; portée du refus `WISE_ACTIVITY_UNSUPPORTED` ;
  « origine sans frais » (écart inexistant) retiré.
- 2026-11-20 — **réécrite sur la voie activités (2026-11-20)** : renumérotée 39-12a2 → 39-12a3 ; relevés et rejeu SCA
  remplacés par soldes, activités, transferts et devis (39-12a1) convertis par 39-12a2 ; ordre de lecture, points de
  contrôle et début de lecture ; règles de doublon R1 (frais de conversion inclus), R2 (montant différent), R3
  (conversion automatique provisoire) ; préfixe `AUTO_CONVERSION` au rapprochement ; codes `WISE_SCA_REJECTED` retiré,
  `WISE_ACTIVITY_UNSUPPORTED` ajouté ; étapes redéfinies. Statut `deferred` → `drafted`, à revalider.

*Historique — entrées de la fiche « téléchargement des relevés Wise — route, tâche en fond, import et audit » (39-12a2),
laissées telles quelles :*


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

- 2026-11-20 — **reportée** : recettes réelles 1 et 2 (11 appels) : le relevé de solde par jeton personnel est refusé
  pour le compte suisse (403 `x-2fa-approval` à chaque requête ; rejeu signé par clé publique, signature vérifiée en
  local, toujours `REJECTED` ; endpoint de statut du jeton à usage unique 404). La doc publique des jetons personnels le
  dit : relevés par API non pris en charge hors US, CA, AU, NZ, SG, MY. Reprise seulement si Wise ouvre l'accès (support
  ou accord Wise Platform). Voie retenue : import CSV de la phase 1. Détails : `data-not-committed/wise-recette-ac0*.md`.
