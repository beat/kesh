# Story 46-7 : solde de la dette de l'epic

Status: in-progress

*Esquisse du 2026-11-18 (renumérotée 46-7 le 2026-11-19 : la balance prend le 46-6). Revue de Beat : approuvée. Story de solde (`CLAUDE.md` § *Priorités des défauts*, « Tarir la source, puis solder »).
Plafond : **2 issues**, fixé au fichier d'epic (§ *Stories*). **Dernière story de l'epic**, après 46-1 à 46-6b (et
46-5 si décidée), avant le tag de la release (0.21.0, prévision).*

## Amendement du 2026-11-22 (au développement) — la dette réelle de l'epic

La fiche date d'avant les stories 46-2 à 46-6b. Aucune issue de l'epic n'est encore **ouverte** sur GitHub : les
revues ont versé leur dette à la liste des issues à ouvrir de l'epic (`kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md`,
hors dépôt), où elle attend l'accord du responsable de l'epic. C'est donc cette liste, recoupée avec les Change Logs
des fiches 46-1 à 46-6b (AC-1), qui fait l'inventaire. La 46-5 est hors périmètre (décision en attente).

**Inventaire (dix points, ligne lue entre parenthèses)**

| # | Origine | Point | Label, priorité | Suite |
|---|---|---|---|---|
| D1 | 46-1 P1 (fiche 46-1, « Interaction avec la 47-3 », n° 20 de la liste) | un miroir de passif rangé sous un groupe d'actif dissout ce groupe à l'affichage groupé | enhancement ou documentation, P3 | **reste à ouvrir** : décision d'interface du responsable de l'epic, non un correctif |
| D2 | 46-2 P3 L2 (Change Log 46-2, « amélioration tracée ») | changer la vue du bilan pendant le chargement d'un autre rapport l'interrompt | enhancement, P4 | reste à ouvrir (enchaînement des chargements, risque de régression de l'écran) |
| D3 | 46-2 P3 L3 (Change Log 46-2, « L3 : noté, **non remédié** ») | la redemande du bilan efface l'erreur de l'autre onglet | enhancement, P4 | **n'était pas tracé** : versé à la liste par cette story (« on trace toujours ») ; reste à ouvrir |
| D4 | 46-3 P2 L3 (Change Log 46-3, « L3 (suffixe « (solde propre) » tronqué, production) ») | PDF groupé : la troncature d'une ligne « solde propre » coupe son suffixe | bug, P4 | **retenu, corrigé** |
| D5 | 46-3 P1 L2 (même entrée) | PDF groupé : limites en caractères, non en largeur réelle | bug, P4 | reste à ouvrir (toucherait les limites de tous les rapports groupés) |
| D6 | 46-3 P1 L4 (même entrée) | `texte_pdf::lignes` sans notion de page | technical-debt, P4 | reste à ouvrir |
| D7 | 46-3 P1 L5 (même entrée) | fidélité JSON/PDF de l'export groupé : un seul jeu de paramètres | technical-debt, P4 | reste à ouvrir |
| D8 | 46-4 P2 F2 (Change Log 46-4, « tracé dans `issues-a-ouvrir.md` ») | MCP : le test de fidélité saute les lignes de groupe | technical-debt, P4 | reste à ouvrir |
| D9 | 46-6b P2 LOW-3 (Change Log 46-6b, « LOW-3 … à ouvrir en issue ») | balance groupée en PDF : le premier compte d'un groupe peut passer seul en haut de page | bug, P4 | reste à ouvrir (plafond atteint) |
| D10 | revue d'intégration de l'Epic 46, L3 (liste, « Relevé par la revue d'intégration de l'Epic 46 (L3) ») | fermer puis rouvrir un groupe laisse « détail ajusté à la main » au PDF | bug, P4 | **retenu, corrigé** |

**Choix dans le plafond (2), AC-1.** Tous les points sont P4 sauf D1 (P3, mais une décision, non un correctif). Par
effet visible : les quatre `bug` (D4, D5, D9, D10). D10 met au PDF remis à une fiduciaire une mention fausse ; D4 rend
une ligne « solde propre » indiscernable d'une ligne de compte ; D9 n'est qu'une coupure de page, D5 demanderait de
recalculer les limites de tous les libellés groupés (risque de régression sur les trois rapports). Retenus : **D10 et
D4**, les plus visibles et les plus sûrs (un module chacun, AC-4).

**Hors plafond : LOW de revue jamais ouverts** (AC-4 : ils n'ont pas été ouverts ; ce sont des rédactions et des
commentaires, sans production) :
- revue P2 de la fusion de la 46-6b (`kesh-gate-logs/integration/epic-46-fusion-46-6b-p2.md`) : LOW-1 (« le titre et
  le sous-total d'un groupe répètent » : un seul des deux porte les montants dans un export donné), LOW-2 (redite et
  parenthèse mal placée de `docs/api-external.md`), LOW-3 (commentaires de l'E2E d'avant l'extraction de l'aide) —
  **corrigés** ;
- revue P4 d'intégration (`epic-46-revue-p4.md`), P4-1 et P4-2 : **déjà corrigés** par la reprise de la phrase à la
  fusion de la 46-6b (le texte actuel des quatre sites dit que titres et sous-totaux répètent les montants, et que
  l'export à plat porte lui aussi ses lignes de total, à filtrer par `Section`) ; constaté, rien à faire.

## Story

En tant que mainteneur, je veux solder, avant le tag, la dette ouverte par les stories de l'Epic 46, afin que l'epic
ne laisse pas de LOW sans suite.

## Critères d'acceptation (esquisse)

1. **AC-1** — Ne prend que des issues ouvertes par les stories 46-*, au plus le plafond fixé au kickoff. **La liste
   se tire des Change Logs et Dev Agent Records des fiches 46-1 à 46-6b** (et 46-5 si décidée), où chaque revue nomme
   les issues qu'elle ouvre (« on trace toujours ») ; chaque issue retenue y est citée avec la ligne lue. La mention
   « Epic 46, story 46-N » dans le corps d'une issue, à son ouverture, aide à la recherche mais n'est pas la source : rien
   ne l'impose aux autres fiches. Au-delà du plafond, le choix se fait par priorité (de `P1` à `P4`), puis par effet
   visible pour l'utilisateur ; chaque choix s'écrit au Dev Agent Record avec son motif.
2. **AC-2** — Chaque correctif suit la boucle de revue ; gates complets et E2E au dernier commit de code.
3. **AC-3** — Les issues 46-* au-delà du plafond restent ouvertes et sont nommées au bilan de l'epic (« on trace
   toujours ») ; aucune n'est fermée pour tenir le chiffre.
4. **AC-4** — Un correctif retenu qui toucherait plus de cinq modules n'est pas pris : il reste ouvert et se nomme
   (règle de splitting préventif). Les LOW des stories 46-1 à 46-6b que leur boucle de revue a corrigés sur place
   (§ *Tarir la source*) ne comptent pas dans le plafond : ils n'ont pas été ouverts.

## Tâches principales

- [ ] Liste des issues 46-* ; choix dans le plafond ; correctifs

## Fichiers

Selon les issues retenues.

## Note de version

*(à écrire au développement)*

## Change Log

- 2026-11-20 — Plafond fixé (2), position en dernier, AC-3 ; statut `drafted`.
- 2026-11-20 — validation P1 (Sonnet 5.5) : 0 C, 0 H, 0 M, 1 L, remédié. L : l'origine « ouverte par une story 46-* »
  n'avait aucune marque vérifiable, le choix au-delà du plafond aucun critère, le périmètre aucune borne : AC-1 et AC-4.
  Plafond 2 confronté au fichier d'epic (§ *Stories*) et à `CLAUDE.md` § *Priorités des défauts* : conforme. Axes
  exercés : fiche, fichier d'epic, `CLAUDE.md` ; non exercés : issues réelles (aucun accès à `gh`).
- 2026-11-20 — validation P2 (Claude Opus 5.5) : 0 C, 0 H, 1 M, 1 L, remédiés. M : la marque « Epic 46, story 46-N »
  posée par la P1 comme seule source de la liste n'est imposée par aucune autre fiche — AC-1 inapplicable → liste tirée
  des Change Logs et Dev Agent Records des fiches 46-1 à 46-6b, la mention restant une aide. L : « `P1` puis `P4` » →
  « de `P1` à `P4` ». Plafond (2) et borne de cinq modules confrontés au fichier d'epic : conformes. Rapport :
  `kesh-gate-logs/epic-46/validation/46-7-P2.md`.
