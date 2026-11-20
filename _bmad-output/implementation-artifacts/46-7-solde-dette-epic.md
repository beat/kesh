# Story 46-7 : solde de la dette de l'epic

Status: drafted

*Esquisse du 2026-11-18 (renumérotée 46-7 le 2026-11-19 : la balance prend le 46-6). Revue de Beat : approuvée. Story de solde (`CLAUDE.md` § *Priorités des défauts*, « Tarir la source, puis solder »).
Plafond : **2 issues**, fixé au fichier d'epic (§ *Stories*). **Dernière story de l'epic**, après 46-1 à 46-6b (et
46-5 si décidée), avant le tag de la release (0.21.0, prévision).*

## Story

En tant que mainteneur, je veux solder, avant le tag, la dette ouverte par les stories de l'Epic 46, afin que l'epic
ne laisse pas de LOW sans suite.

## Critères d'acceptation (esquisse)

1. **AC-1** — Ne prend que des issues ouvertes par les stories 46-*, au plus le plafond fixé au kickoff. Une issue
   ouverte par une story de l'epic porte dans son corps « Epic 46, story 46-N » : la liste se tire de cette mention
   (aucune autre marque ne dit d'où elle vient). Au-delà du plafond, le choix se fait par priorité (`P1` puis `P4`),
   puis par effet visible pour l'utilisateur ; chaque choix s'écrit au Dev Agent Record avec son motif.
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
