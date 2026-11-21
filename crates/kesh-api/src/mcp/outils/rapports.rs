//! `kesh_balance`, `kesh_bilan`, `kesh_compte_de_resultat` — les rapports à
//! période d'un exercice (Story 32-9a, AC-a10) ; chaque consultation s'inscrit
//! au journal d'audit (`ContexteMcp::afficher`). Portée `books:read`.
//!
//! La balance rend tout compte actif : l'outil la **tranche** (défaut 23,
//! maximum 42 lignes, re-mesurés avec les colonnes en devise à la Story 37-4)
//! et rend les totaux du rapport entier. Le bilan et le
//! compte de résultat ne sont pas paginés : au-delà d'environ 60 comptes à
//! solde non nul, la réponse est refusée pour sa taille (C-32-9-8). Re-mesuré
//! avec les champs en devise (Story 37-4, `tailles_mesurees_en_devise`) : un
//! bilan de 57 comptes en devise et des comptes de la société d'essai tient en
//! 20 156 o (plafond 24 576) — le seuil d'« environ 60 » reste vrai, en
//! borne basse, et n'est pas relevé.
//!
//! **Par groupes** (Story 46-4) : le bilan et le compte de résultat acceptent
//! les arguments de groupement de l'API (`detailLevel`, `subtotalPosition`,
//! `open`, `closed`) ; la sortie porte alors un champ `groupes` (lignes
//! groupées par section, écho des choix retenus) et des listes plates vides.
//! Une seule conversion ([`groupes_demandes`]) mène les arguments à la demande
//! typée de la Story 46-1, qui seule les valide. Pas de pagination
//! (C-46-4-3) : le niveau borne la taille. Mesuré le 2026-11-21
//! (`tailles_groupees_plan_pme`, `tailles_groupees_seuil`) : le plan PME
//! complet — ses 52 comptes imputables sans rôle ni écriture passés en USD et
//! mouvementés, les autres tels que la société d'essai les laisse — tient sous
//! 23 552 o aux niveaux 2 et `all`, `header` et `footer` ; sur une fixture synthétique (comptes en devise, noms de
//! 34 caractères, `all`, `footer`), la réponse est refusée au-delà d'environ
//! [`SEUIL_LIGNES_GROUPEES`] lignes groupées. Relevés : plan PME, au plus 46
//! lignes et 16 343 o (compte de résultat, `all`,
//! `footer`) ; fixture synthétique, élément de 356 o, 67 lignes servies au
//! plus (24 389 o), refus à la suivante — le seuil de 60 est une borne basse,
//! comme celui du bilan plat.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::mcp::outil::ErreurOutil;
use crate::mcp::outil::{ContexteMcp, FuturOutil, McpTool, arguments};
use crate::mcp::outils::commun::{decalage, limite, nombre, tranche};
use crate::mcp::sortie::{
    Code, Date, Devise, Montant, Resume, TexteNonFiable, champ_enum, donnees, sortie,
};
use crate::mcp::textes::depuis_app_error;
use crate::services::acces::Portee;
use crate::services::rapports::{self, Affichage, DemandeGroupes, DemandePeriode, NiveauDemande};

/// Lignes de balance par défaut. Story 37-4 : re-mesurée avec les colonnes en
/// devise remplies (élément de 542 o, `tailles_mesurees_en_devise`), 30 → 23.
pub const LIMITE_DEFAUT_BALANCE: i64 = 23;
/// Lignes de balance au plus (aucun plafond de service). Story 37-4 : 55 → 42.
pub const LIMITE_MAX_MCP_BALANCE: i64 = 42;

/// Lignes groupées (toutes sections) au-delà desquelles le bilan ou le compte
/// de résultat par groupes est refusé pour sa taille — borne basse mesurée
/// (`tailles_groupees_seuil`), écrite dans les descriptions des deux outils.
pub const SEUIL_LIGNES_GROUPEES: usize = 60;

/// La phrase des arguments de groupement, commune aux descriptions du bilan et
/// du compte de résultat (Story 46-4, AC-1, AC-3, AC-5). Le nombre est
/// [`SEUIL_LIGNES_GROUPEES`] (test `descriptions_des_groupes`).
macro_rules! description_groupes {
    ($plates:literal) => {
        concat!(
            "Par groupes de comptes, comme à l'écran : detailLevel (entier de 1 à 20, ou \"all\") \
             ouvre les groupes de profondeur inférieure au niveau ; open et closed (tableaux \
             d'identifiants de comptes de groupe, 500 au plus, jamais le même dans les deux) \
             ouvrent ou ferment des groupes à la main ; subtotalPosition (\"header\", par défaut, \
             ou \"footer\") place le sous-total d'un groupe ouvert sur son titre ou sur une ligne \
             subtotal sous sa descendance — un groupe fermé porte son montant sur son titre. \
             subtotalPosition, open et closed exigent detailLevel ; un argument hors de ces \
             règles est refusé (Arguments invalides). Avec detailLevel, groupes porte l'écho des \
             choix retenus et les lignes groupées de chaque section (genre group, account, own \
             ou subtotal ; un identifiant inconnu est ignoré) et les listes plates (",
            $plates,
            ") sont vides ; sans, groupes vaut null. Au-delà d'environ 60 lignes groupées, la \
             réponse est refusée : choisissez un niveau plus bas."
        )
    };
}

/// La balance.
pub struct KeshBalance;
/// Le bilan.
pub struct KeshBilan;
/// Le compte de résultat.
pub struct KeshCompteDeResultat;

/// Les arguments d'un rapport à période : la période, et les arguments de
/// groupement (Story 46-4), aux noms de l'API. Les quatre derniers ne servent
/// que le bilan et le compte de résultat ; `kesh_balance` les pose à `None`
/// jusqu'à la Story 46-6a.
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ArgumentsPeriode {
    fiscal_year_id: i64,
    #[serde(default, deserialize_with = "crate::dates_recues::date_opt")]
    period_start: Option<NaiveDate>,
    #[serde(default, deserialize_with = "crate::dates_recues::date_opt")]
    period_end: Option<NaiveDate>,
    detail_level: Option<NiveauArgument>,
    subtotal_position: Option<PositionSousTotaux>,
    open: Option<Vec<i64>>,
    closed: Option<Vec<i64>>,
}

/// `detailLevel` : un entier JSON, ou la chaîne `"all"` — tout autre JSON
/// (`2.5`, `"2"`, `"ALL"`, `true`) est refusé à la lecture. La plage (1 à 20)
/// est celle de la Story 46-1, validée par [`DemandeGroupes::depuis_valeurs`].
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(untagged)]
enum NiveauArgument {
    /// Un niveau.
    Niveau(i64),
    /// `"all"`.
    Tous(Tous),
}

/// La seule chaîne admise par `detailLevel`.
#[derive(Debug, Clone, Copy, Deserialize)]
enum Tous {
    /// `"all"`.
    #[serde(rename = "all")]
    All,
}

/// Où se place le sous-total d'un groupe ouvert — miroir de
/// [`kesh_report::PositionSousTotal`], aux valeurs de l'API (`header`,
/// `footer`) : lu en argument (`subtotalPosition`) et rendu dans l'écho
/// (`positionSousTotaux`).
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum PositionSousTotaux {
    Header,
    Footer,
}

/// Genre d'une ligne groupée — miroir de [`kesh_report::groupes::GenreLigne`],
/// aux valeurs de l'API (`group`, `account`, `own`, `subtotal`).
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
enum GenreLigne {
    Group,
    Account,
    Own,
    Subtotal,
}

champ_enum!(PositionSousTotaux, GenreLigne);

impl PositionSousTotaux {
    /// La valeur de l'API, pour le résumé d'appel.
    fn code(self) -> &'static str {
        match self {
            Self::Header => "header",
            Self::Footer => "footer",
        }
    }

    fn vers_rapport(self) -> kesh_report::PositionSousTotal {
        match self {
            Self::Header => kesh_report::PositionSousTotal::Titre,
            Self::Footer => kesh_report::PositionSousTotal::Pied,
        }
    }

    fn depuis_rapport(p: kesh_report::PositionSousTotal) -> Self {
        match p {
            kesh_report::PositionSousTotal::Titre => Self::Header,
            kesh_report::PositionSousTotal::Pied => Self::Footer,
        }
    }
}

fn genre(g: kesh_report::groupes::GenreLigne) -> GenreLigne {
    use kesh_report::groupes::GenreLigne as G;
    match g {
        G::Groupe => GenreLigne::Group,
        G::Compte => GenreLigne::Account,
        G::Propre => GenreLigne::Own,
        G::SousTotal => GenreLigne::Subtotal,
    }
}

/// La **seule** conversion des arguments de groupement en la demande typée de
/// la Story 46-1 (réutilisée par la balance à la Story 46-6a) : aucune
/// validation locale, celle de [`DemandeGroupes::depuis_valeurs`] seule ; un
/// refus y devient « Arguments invalides. » (`depuis_app_error`).
fn groupes_demandes(a: &ArgumentsPeriode) -> Result<Option<DemandeGroupes>, ErreurOutil> {
    DemandeGroupes::depuis_valeurs(
        a.detail_level.map(|n| match n {
            NiveauArgument::Niveau(n) => NiveauDemande::Niveau(n),
            NiveauArgument::Tous(Tous::All) => NiveauDemande::Tous,
        }),
        a.subtotal_position.map(PositionSousTotaux::vers_rapport),
        a.open.clone(),
        a.closed.clone(),
    )
    .map_err(depuis_app_error)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ArgumentsBalance {
    fiscal_year_id: i64,
    #[serde(default, deserialize_with = "crate::dates_recues::date_opt")]
    period_start: Option<NaiveDate>,
    #[serde(default, deserialize_with = "crate::dates_recues::date_opt")]
    period_end: Option<NaiveDate>,
    limit: Option<i64>,
    offset: Option<i64>,
}

/// Le schéma du bilan et du compte de résultat : la période et les arguments
/// de groupement (Story 46-4). Indicatif : il n'est pas appliqué à l'exécution.
const SCHEMA_PERIODE_GROUPES: &str = r#"{"type":"object","properties":{"fiscalYearId":{"type":"integer","minimum":1},"periodStart":{"type":"string","format":"date"},"periodEnd":{"type":"string","format":"date"},"detailLevel":{"oneOf":[{"type":"integer","minimum":1,"maximum":20},{"type":"string","enum":["all"]}]},"subtotalPosition":{"type":"string","enum":["header","footer"]},"open":{"type":"array","items":{"type":"integer","minimum":1},"maxItems":500},"closed":{"type":"array","items":{"type":"integer","minimum":1},"maxItems":500}},"required":["fiscalYearId"],"additionalProperties":false}"#;

sortie! {
    /// La période résolue.
    struct Periode { exercice_id: i64, debut: Date, fin: Date }
}

sortie! {
    /// Un compte et son solde (bilan, compte de résultat).
    struct LigneDeSolde {
        compte_id: i64,
        numero: TexteNonFiable,
        nom: TexteNonFiable,
        type_de_compte: kesh_db::entities::AccountType,
        actif: bool,
        solde: Montant,
        role: Option<kesh_db::entities::AccountRole>,
        // Story 37-4 (AC-6) : la devise du compte et son solde en devise,
        // présents à `null` pour un compte en devise principale (C-37-9). Les
        // totaux restent en devise principale.
        devise: Option<Code<Devise>>,
        solde_devise: Option<Montant>,
    }
}

sortie! {
    /// Une ligne groupée (bilan, compte de résultat) : un groupe et son
    /// sous-total, un compte, le solde propre d'un groupe, ou le sous-total en
    /// pied d'un groupe ouvert.
    struct LigneGroupee {
        genre: GenreLigne,
        profondeur: u32,
        compte_id: i64,
        numero: TexteNonFiable,
        nom: TexteNonFiable,
        solde: Montant,
        devise: Option<Code<Devise>>,
        solde_devise: Option<Montant>,
        ouvert: Option<bool>,
    }
}

sortie! {
    /// Le bilan par groupes : l'écho des choix retenus et les sections.
    struct GroupesBilan {
        niveau: Option<i64>,
        tous_les_niveaux: bool,
        position_sous_totaux: PositionSousTotaux,
        profondeur_max: u32,
        ouverts: Vec<i64>,
        fermes: Vec<i64>,
        actifs: Vec<LigneGroupee>,
        passifs: Vec<LigneGroupee>,
        fonds_propres: Vec<LigneGroupee>,
    }
}

sortie! {
    /// Le compte de résultat par groupes : l'écho des choix retenus et les
    /// sections.
    struct GroupesResultat {
        niveau: Option<i64>,
        tous_les_niveaux: bool,
        position_sous_totaux: PositionSousTotaux,
        profondeur_max: u32,
        ouverts: Vec<i64>,
        fermes: Vec<i64>,
        produits: Vec<LigneGroupee>,
        charges: Vec<LigneGroupee>,
    }
}

sortie! {
    /// Le bilan.
    struct Bilan {
        periode: Periode,
        actifs: Vec<LigneDeSolde>,
        passifs: Vec<LigneDeSolde>,
        fonds_propres: Vec<LigneDeSolde>,
        total_actifs: Montant,
        total_passifs: Montant,
        total_fonds_propres: Montant,
        report: Montant,
        resultat: Montant,
        equation_tenue: bool,
        // Story 46-4 : `null` sans `detailLevel` (C-37-9, C-46-4-5).
        groupes: Option<GroupesBilan>,
    }
}

sortie! {
    /// Le compte de résultat.
    struct CompteDeResultat {
        periode: Periode,
        produits: Vec<LigneDeSolde>,
        charges: Vec<LigneDeSolde>,
        total_produits: Montant,
        total_charges: Montant,
        resultat: Montant,
        // Story 46-4 : `null` sans `detailLevel` (C-37-9, C-46-4-5).
        groupes: Option<GroupesResultat>,
    }
}

sortie! {
    /// Une ligne de balance.
    struct LigneBalance {
        compte_id: i64,
        numero: TexteNonFiable,
        nom: TexteNonFiable,
        type_de_compte: kesh_db::entities::AccountType,
        actif: bool,
        ouverture: Montant,
        debit: Montant,
        credit: Montant,
        solde: Montant,
        cloture: Montant,
        // Story 37-4 (AC-6) : les mêmes colonnes en devise du compte, `null`
        // pour un compte en devise principale ; aucun total en devise.
        devise: Option<Code<Devise>>,
        ouverture_devise: Option<Montant>,
        debit_devise: Option<Montant>,
        credit_devise: Option<Montant>,
        solde_devise: Option<Montant>,
        cloture_devise: Option<Montant>,
    }
}

sortie! {
    /// La balance : une tranche, et les totaux du rapport entier.
    struct Balance {
        periode: Periode,
        lignes: Vec<LigneBalance>,
        total: i64,
        limit: i64,
        offset: i64,
        total_debit: Montant,
        total_credit: Montant,
        equilibree: bool,
        report_benefice: Montant,
        ouverture_equilibree: bool,
    }
}

fn periode(p: kesh_report::ReportPeriod) -> Periode {
    Periode {
        exercice_id: p.fiscal_year_id,
        debut: Date(p.start_date),
        fin: Date(p.end_date),
    }
}

fn ligne(a: kesh_report::AccountBalance) -> LigneDeSolde {
    LigneDeSolde {
        compte_id: a.account_id,
        numero: TexteNonFiable::de(a.account_number),
        nom: TexteNonFiable::de(a.account_name),
        type_de_compte: a.account_type,
        actif: a.active,
        solde: Montant(a.balance),
        role: a.role,
        devise: a.currency.map(Code::depuis_str),
        solde_devise: a.balance_currency.map(Montant),
    }
}

fn lignes(v: Vec<kesh_report::AccountBalance>) -> Vec<LigneDeSolde> {
    v.into_iter().map(ligne).collect()
}

fn lignes_groupees(v: Vec<kesh_report::LigneRapport>) -> Vec<LigneGroupee> {
    v.into_iter()
        .map(|l| LigneGroupee {
            genre: genre(l.kind),
            profondeur: l.depth,
            compte_id: l.account_id,
            numero: TexteNonFiable::de(l.account_number),
            nom: TexteNonFiable::de(l.account_name),
            solde: Montant(l.balance),
            devise: l.currency.map(Code::depuis_str),
            solde_devise: l.balance_currency.map(Montant),
            ouvert: l.open,
        })
        .collect()
}

/// L'écho `grouping` de la Story 46-1, champ par champ : `niveau` à `null` et
/// `tousLesNiveaux` à `true` pour `"all"`.
struct Echo {
    niveau: Option<i64>,
    tous_les_niveaux: bool,
    position_sous_totaux: PositionSousTotaux,
    profondeur_max: u32,
    ouverts: Vec<i64>,
    fermes: Vec<i64>,
}

fn echo(g: kesh_report::Grouping) -> Echo {
    let (niveau, tous_les_niveaux) = match g.detail_level {
        kesh_report::NiveauDetail::Niveau(n) => (Some(i64::from(n)), false),
        kesh_report::NiveauDetail::Tous => (None, true),
    };
    Echo {
        niveau,
        tous_les_niveaux,
        position_sous_totaux: PositionSousTotaux::depuis_rapport(g.subtotal_position),
        profondeur_max: g.max_depth,
        ouverts: g.open,
        fermes: g.closed,
    }
}

fn resume_periode(nom: &'static str, a: &ArgumentsPeriode) -> Resume {
    let resume = Resume::outil(nom)
        .entier("fiscalYearId", Some(a.fiscal_year_id))
        .date("periodStart", a.period_start)
        .date("periodEnd", a.period_end);
    let resume = match a.detail_level {
        Some(NiveauArgument::Niveau(n)) => resume.entier("detailLevel", Some(n)),
        Some(NiveauArgument::Tous(Tous::All)) => resume.code("detailLevel", Some("all")),
        None => resume,
    };
    resume
        .code(
            "subtotalPosition",
            a.subtotal_position.map(PositionSousTotaux::code),
        )
        .entier("openCount", a.open.as_ref().map(|v| nombre(v.len())))
        .entier("closedCount", a.closed.as_ref().map(|v| nombre(v.len())))
}

fn demande(a: &ArgumentsPeriode) -> DemandePeriode {
    DemandePeriode {
        fiscal_year_id: a.fiscal_year_id,
        period_start: a.period_start,
        period_end: a.period_end,
    }
}

impl McpTool for KeshBalance {
    fn nom(&self) -> &'static str {
        "kesh_balance"
    }

    fn description(&self) -> &'static str {
        "Lit la balance des comptes d'un exercice (fiscalYearId, requis ; kesh_exercices), \
         sur une période facultative (periodStart, periodEnd) : soldes d'ouverture, \
         mouvements et soldes par compte, par tranches (limit, offset), avec les totaux du \
         rapport entier. Si la réponse est trop grosse, réduisez limit. Un compte en devise \
         (devise, code ISO 4217) porte aussi ses colonnes dans sa devise (champs suffixés \
         « devise », null sur un compte en CHF) ; les totaux restent en CHF."
    }

    fn schema_entree(&self) -> &'static str {
        r#"{"type":"object","properties":{"fiscalYearId":{"type":"integer","minimum":1},"periodStart":{"type":"string","format":"date"},"periodEnd":{"type":"string","format":"date"},"limit":{"type":"integer","minimum":1,"maximum":42,"default":23},"offset":{"type":"integer","minimum":0}},"required":["fiscalYearId"],"additionalProperties":false}"#
    }

    fn portee(&self) -> Portee {
        Portee::BooksRead
    }

    fn executer<'a>(
        &'a self,
        ctx: &'a ContexteMcp<'a>,
        valeur: serde_json::Value,
    ) -> FuturOutil<'a> {
        Box::pin(async move {
            let b: ArgumentsBalance = arguments(valeur)?;
            let limit = limite(b.limit, LIMITE_DEFAUT_BALANCE, LIMITE_MAX_MCP_BALANCE)?;
            let offset = decalage(b.offset)?;
            let a = ArgumentsPeriode {
                fiscal_year_id: b.fiscal_year_id,
                period_start: b.period_start,
                period_end: b.period_end,
                // Story 46-6a : la balance par groupes.
                detail_level: None,
                subtotal_position: None,
                open: None,
                closed: None,
            };
            let resume = resume_periode("kesh_balance", &a)
                .entier("limit", Some(limit))
                .entier("offset", Some(offset));
            let rapport = rapports::balance(ctx.acces(), ctx.lecture(), demande(&a), Affichage)
                .await
                .map_err(depuis_app_error)?;
            let r = ctx.afficher(rapport).await;
            let total = nombre(r.rows.len());
            let sortie = Balance {
                periode: periode(r.period),
                lignes: tranche(r.rows, offset, limit)
                    .into_iter()
                    .map(|row| LigneBalance {
                        compte_id: row.account_id,
                        numero: TexteNonFiable::de(row.account_number),
                        nom: TexteNonFiable::de(row.account_name),
                        type_de_compte: row.account_type,
                        actif: row.active,
                        ouverture: Montant(row.opening_balance),
                        debit: Montant(row.total_debit),
                        credit: Montant(row.total_credit),
                        solde: Montant(row.balance),
                        cloture: Montant(row.closing_balance),
                        devise: row.currency.map(Code::depuis_str),
                        ouverture_devise: row.opening_currency.map(Montant),
                        debit_devise: row.total_debit_currency.map(Montant),
                        credit_devise: row.total_credit_currency.map(Montant),
                        solde_devise: row.balance_currency.map(Montant),
                        cloture_devise: row.closing_currency.map(Montant),
                    })
                    .collect(),
                total,
                limit,
                offset,
                total_debit: Montant(r.total_debit),
                total_credit: Montant(r.total_credit),
                equilibree: r.balanced,
                report_benefice: Montant(r.retained_earnings),
                ouverture_equilibree: r.opening_balanced,
            };
            donnees(resume, &sortie)
        })
    }
}

impl McpTool for KeshBilan {
    fn nom(&self) -> &'static str {
        "kesh_bilan"
    }

    fn description(&self) -> &'static str {
        concat!(
            "Lit le bilan d'un exercice (fiscalYearId, requis ; kesh_exercices) à la fin d'une \
         période facultative : actifs, passifs, fonds propres et leurs totaux, comptes à \
         solde non nul (en CHF ou dans leur devise). Un compte en devise porte sa devise \
         (code ISO 4217) et son solde dans cette devise (soldeDevise), null sur un compte en \
         CHF ; les totaux restent en CHF. Non paginé : au-delà d'environ 60 comptes à solde \
         non nul, la réponse est refusée. ",
            description_groupes!("actifs, passifs, fondsPropres")
        )
    }

    fn schema_entree(&self) -> &'static str {
        SCHEMA_PERIODE_GROUPES
    }

    fn portee(&self) -> Portee {
        Portee::BooksRead
    }

    fn executer<'a>(
        &'a self,
        ctx: &'a ContexteMcp<'a>,
        valeur: serde_json::Value,
    ) -> FuturOutil<'a> {
        Box::pin(async move {
            let a: ArgumentsPeriode = arguments(valeur)?;
            let resume = resume_periode("kesh_bilan", &a);
            let g = groupes_demandes(&a)?;
            let rapport =
                rapports::bilan_par_groupes(ctx.acces(), ctx.lecture(), demande(&a), g, Affichage)
                    .await
                    .map_err(depuis_app_error)?;
            let enveloppe = ctx.afficher(rapport).await;
            let r = enveloppe.rapport;
            // Par groupes : les listes plates vides (C-46-4-1, taille).
            let groupes = enveloppe
                .grouping
                .zip(enveloppe.grouped_rows)
                .map(|(g, s)| {
                    let e = echo(g);
                    GroupesBilan {
                        niveau: e.niveau,
                        tous_les_niveaux: e.tous_les_niveaux,
                        position_sous_totaux: e.position_sous_totaux,
                        profondeur_max: e.profondeur_max,
                        ouverts: e.ouverts,
                        fermes: e.fermes,
                        actifs: lignes_groupees(s.assets),
                        passifs: lignes_groupees(s.liabilities),
                        fonds_propres: lignes_groupees(s.equity),
                    }
                });
            let plates = |v| {
                if groupes.is_some() {
                    Vec::new()
                } else {
                    lignes(v)
                }
            };
            let sortie = Bilan {
                periode: periode(r.period),
                actifs: plates(r.assets),
                passifs: plates(r.liabilities),
                fonds_propres: plates(r.equity),
                total_actifs: Montant(r.total_assets),
                total_passifs: Montant(r.total_liabilities),
                total_fonds_propres: Montant(r.total_equity),
                report: Montant(r.retained_earnings),
                resultat: Montant(r.equity_result),
                equation_tenue: r.equation_holds,
                groupes,
            };
            donnees(resume, &sortie)
        })
    }
}

impl McpTool for KeshCompteDeResultat {
    fn nom(&self) -> &'static str {
        "kesh_compte_de_resultat"
    }

    fn description(&self) -> &'static str {
        concat!(
            "Lit le compte de résultat d'un exercice (fiscalYearId, requis ; kesh_exercices) sur \
         une période facultative : produits, charges, leurs totaux et le résultat. Un \
         compte en devise porte sa devise (code ISO 4217) et son solde dans cette devise \
         (soldeDevise), null sur un compte en CHF ; les totaux restent en CHF. Non paginé : \
         au-delà d'environ 60 comptes à solde non nul, la réponse est refusée ; choisissez \
         une période plus courte. ",
            description_groupes!("produits, charges")
        )
    }

    fn schema_entree(&self) -> &'static str {
        SCHEMA_PERIODE_GROUPES
    }

    fn portee(&self) -> Portee {
        Portee::BooksRead
    }

    fn executer<'a>(
        &'a self,
        ctx: &'a ContexteMcp<'a>,
        valeur: serde_json::Value,
    ) -> FuturOutil<'a> {
        Box::pin(async move {
            let a: ArgumentsPeriode = arguments(valeur)?;
            let resume = resume_periode("kesh_compte_de_resultat", &a);
            let g = groupes_demandes(&a)?;
            let rapport = rapports::compte_de_resultat_par_groupes(
                ctx.acces(),
                ctx.lecture(),
                demande(&a),
                g,
                Affichage,
            )
            .await
            .map_err(depuis_app_error)?;
            let enveloppe = ctx.afficher(rapport).await;
            let r = enveloppe.rapport;
            // Par groupes : les listes plates vides (C-46-4-1, taille).
            let groupes = enveloppe
                .grouping
                .zip(enveloppe.grouped_rows)
                .map(|(g, s)| {
                    let e = echo(g);
                    GroupesResultat {
                        niveau: e.niveau,
                        tous_les_niveaux: e.tous_les_niveaux,
                        position_sous_totaux: e.position_sous_totaux,
                        profondeur_max: e.profondeur_max,
                        ouverts: e.ouverts,
                        fermes: e.fermes,
                        produits: lignes_groupees(s.revenues),
                        charges: lignes_groupees(s.expenses),
                    }
                });
            let plates = |v| {
                if groupes.is_some() {
                    Vec::new()
                } else {
                    lignes(v)
                }
            };
            let sortie = CompteDeResultat {
                periode: periode(r.period),
                produits: plates(r.revenues),
                charges: plates(r.expenses),
                total_produits: Montant(r.total_revenues),
                total_charges: Montant(r.total_expenses),
                resultat: Montant(r.net_result),
                groupes,
            };
            donnees(resume, &sortie)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lire(json: &str) -> Result<ArgumentsPeriode, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// AC-1 — `detailLevel` : un entier ou `"all"`, tout autre JSON refusé à
    /// la lecture ; la plage est celle de la Story 46-1 (conversion).
    #[test]
    fn lecture_de_detail_level() {
        let a = lire(r#"{"fiscalYearId":1,"detailLevel":3}"#).unwrap();
        assert!(matches!(a.detail_level, Some(NiveauArgument::Niveau(3))));
        let a = lire(r#"{"fiscalYearId":1,"detailLevel":"all"}"#).unwrap();
        assert!(matches!(
            a.detail_level,
            Some(NiveauArgument::Tous(Tous::All))
        ));
        let a = lire(r#"{"fiscalYearId":1,"detailLevel":null}"#).unwrap();
        assert!(a.detail_level.is_none());
        for v in [r#"2.5"#, r#""2""#, r#""ALL""#, r#""""#, "true", "[2]", "{}"] {
            let j = format!(r#"{{"fiscalYearId":1,"detailLevel":{v}}}"#);
            assert!(lire(&j).is_err(), "{j}");
        }
        for v in [r#""middle""#, r#""Header""#, "1"] {
            let j = format!(r#"{{"fiscalYearId":1,"detailLevel":1,"subtotalPosition":{v}}}"#);
            assert!(lire(&j).is_err(), "{j}");
        }
        for v in [r#""1,2""#, r#"["1"]"#, "[1.5]", "1"] {
            let j = format!(r#"{{"fiscalYearId":1,"detailLevel":1,"open":{v}}}"#);
            assert!(lire(&j).is_err(), "{j}");
        }
    }

    /// AC-1 — la conversion unique : la demande typée de la Story 46-1, et ses
    /// refus ; un identifiant nul ou négatif n'est pas un refus.
    #[test]
    fn conversion_unique() {
        let a = lire(r#"{"fiscalYearId":1}"#).unwrap();
        assert!(groupes_demandes(&a).unwrap().is_none());
        let a = lire(
            r#"{"fiscalYearId":1,"detailLevel":"all","subtotalPosition":"footer","open":[3,0,-4],"closed":[5]}"#,
        )
        .unwrap();
        let d = groupes_demandes(&a).unwrap().unwrap();
        assert_eq!(d.options.niveau, kesh_report::NiveauDetail::Tous);
        assert_eq!(d.options.position, kesh_report::PositionSousTotal::Pied);
        assert_eq!(
            d.options.ouverts.iter().copied().collect::<Vec<_>>(),
            [-4, 0, 3]
        );
        assert_eq!(d.options.fermes.iter().copied().collect::<Vec<_>>(), [5]);
        let a = lire(r#"{"fiscalYearId":1,"detailLevel":2}"#).unwrap();
        let d = groupes_demandes(&a).unwrap().unwrap();
        assert_eq!(d.options.niveau, kesh_report::NiveauDetail::Niveau(2));
        assert_eq!(d.options.position, kesh_report::PositionSousTotal::Titre);
        for j in [
            r#"{"fiscalYearId":1,"detailLevel":0}"#,
            r#"{"fiscalYearId":1,"detailLevel":21}"#,
            r#"{"fiscalYearId":1,"subtotalPosition":"header"}"#,
            r#"{"fiscalYearId":1,"open":[]}"#,
            r#"{"fiscalYearId":1,"closed":[1]}"#,
            r#"{"fiscalYearId":1,"detailLevel":1,"open":[2],"closed":[2]}"#,
        ] {
            assert!(
                matches!(
                    groupes_demandes(&lire(j).unwrap()),
                    Err(ErreurOutil::ArgumentsInvalides { .. })
                ),
                "{j}"
            );
        }
    }

    /// AC-4 — le résumé d'appel : le niveau, la position, et le **nombre**
    /// d'identifiants des listes.
    #[test]
    fn resume_des_groupes() {
        let r = |j: &str| resume_periode("kesh_bilan", &lire(j).unwrap()).texte();
        assert_eq!(
            r(r#"{"fiscalYearId":7}"#),
            "Outil kesh_bilan : fiscalYearId=7"
        );
        assert_eq!(
            r(
                r#"{"fiscalYearId":7,"detailLevel":2,"subtotalPosition":"footer","open":[1,2,3],"closed":[]}"#
            ),
            "Outil kesh_bilan : fiscalYearId=7, detailLevel=2, subtotalPosition=footer, \
             openCount=3, closedCount=0"
        );
        assert_eq!(
            r(r#"{"fiscalYearId":7,"detailLevel":"all"}"#),
            "Outil kesh_bilan : fiscalYearId=7, detailLevel=all"
        );
    }

    /// AC-5 — les deux descriptions énoncent le seuil mesuré et les
    /// contraintes des arguments.
    #[test]
    fn descriptions_des_groupes() {
        for d in [KeshBilan.description(), KeshCompteDeResultat.description()] {
            assert!(
                d.contains(&format!("environ {SEUIL_LIGNES_GROUPEES} lignes groupées")),
                "{d}"
            );
            for mot in [
                "detailLevel",
                "subtotalPosition",
                "open",
                "closed",
                "500",
                "null",
            ] {
                assert!(d.contains(mot), "{mot} : {d}");
            }
        }
        assert!(
            KeshBilan
                .description()
                .contains("(actifs, passifs, fondsPropres)")
        );
        assert!(
            KeshCompteDeResultat
                .description()
                .contains("(produits, charges)")
        );
    }
}
