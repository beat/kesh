-- Story 47-1 (Epic 47, comptes miroirs) — le COMPTE MIROIR d'un compte.
--
-- Un compte (la source) peut désigner un compte miroir : un compte de
-- présentation, sans écriture, de l'autre côté du même rapport (actif <->
-- passif, produit <-> charge), où les rapports présentent le solde de sens
-- inverse de la source, sans écriture de reclassement (CO 959a). Le lien
-- s'écrit sur la source seulement ; la cible n'est jamais écrite.
--
-- (a) NON BREAKING au sens de P1 (`CLAUDE.md` § Migration breaking policy) :
--     « sans relever `kesh_version_min_required` et sans corrompre le schéma ».
--     Une colonne nullable et un index ajoutés (P3 : aucune opération de la
--     liste) ; un binaire antérieur liste ses colonnes et ignore la neuve, et
--     `check_schema_compat` n'exige d'une sauvegarde que les colonnes NOT NULL
--     sans défaut : une sauvegarde 0.19.0 ou 0.20.0 s'importe, miroirs vides.
--     `kesh_version_min_required` reste 0.17.6.
-- (b) SANS CLÉ ÉTRANGÈRE vers accounts, comme `fx_gain_account_id`
--     (20261114000001) : une auto-référence ajouterait un ordre d'insertion à
--     l'import d'une sauvegarde (qui réinsère les tables sans ordre imposé) et
--     à la remise à zéro d'une société (qui vide accounts). L'intégrité (même
--     société, active, compte de présentation, de l'autre côté, sans chaîne)
--     est tenue par le dépôt sous verrou (`accounts::check_mirror`), et un lien
--     hors règle écrit par un import ou un SQL est lu comme absent
--     (`accounts::retain_valid_links`).
-- (c) PAS DE CHECK « pas soi-même » (D10) : écart constaté au T0 le 2026-11-20
--     (MariaDB 10.11 : ERROR 1901, AUTO_INCREMENT interdit dans un CHECK) ;
--     contrainte retirée, refus tenu par le code (raison `self`).
-- (d) P7 : DDL seul, aucune ligne écrite — rien à trier (ni registre de rejeu
--     ni exemption).
-- (e) LE SQL QUI DÉFAIT CETTE MIGRATION (P1) — perd les miroirs posés :
--       ALTER TABLE accounts DROP INDEX idx_accounts_mirror, DROP COLUMN mirror_account_id;
--       DELETE FROM _sqlx_migrations WHERE version = 20261119000001;
--     Le retour ordinaire reste la sauvegarde d'avant la mise à jour.
--
-- Une seule instruction ALTER TABLE : atomique sous MariaDB (le DDL n'étant pas
-- transactionnel, deux instructions laisseraient la première posée sur un
-- échec de la seconde).
ALTER TABLE accounts
    ADD COLUMN mirror_account_id BIGINT NULL
        COMMENT 'Compte miroir de présentation (accounts.id, sans FK), NULL = aucun',
    ADD INDEX idx_accounts_mirror (company_id, mirror_account_id);
