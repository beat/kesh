//! Rapports par groupes de comptes (Story 46-1, Epic 46).
//!
//! Le bilan et le compte de résultat se présentent, sur demande, **par groupes**
//! de l'arbre du plan comptable (`accounts.parent_id`, D1 — jamais le numéro) :
//! un sous-total par groupe, un niveau de détail, des groupes ouverts ou fermés.
//!
//! Ce module est **pur** (aucune I/O) :
//!
//! - [`ArbreComptes`] construit l'arbre depuis les comptes de la société, en
//!   traitant en racine tout compte dont le parent est inconnu, d'une autre
//!   société ou pris dans un cycle (AC-7) ;
//! - [`Dissolution`] repère les groupes **à cheval** sur plusieurs partitions
//!   (sections du bilan, nature débitrice ou créditrice de la balance) : ils ne
//!   sont affichés nulle part et leurs enfants remontent (AC-4, art. 959a CO) ;
//! - [`regrouper`] produit, pour une liste de lignes (une section), les lignes
//!   groupées dans l'ordre d'affichage ([`LigneGroupee`]) ;
//! - [`AvecGroupes`] enveloppe un rapport sans le modifier : sans demande de
//!   groupes, sa sérialisation est celle du rapport seul, à l'octet (NFR-46-1).
//!
//! Les montants restent des `Decimal` exacts : ce module ne formate ni n'arrondit
//! rien (AC-13 ; la garde de la Story 48-11 interdit tout formatage neuf ici).

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;

use kesh_db::entities::{Account, AccountRole};
use rust_decimal::Decimal;
use serde::{Serialize, Serializer};

use crate::balance_sheet::{AccountBalance, BalanceSheet};
use crate::income_statement::IncomeStatement;

// ===========================================================================
// Options
// ===========================================================================

/// Plus grand niveau de détail numérique accepté (`detailLevel`, 1 à 20).
pub const NIVEAU_MAX: u8 = 20;

/// Niveau de détail : jusqu'à quelle profondeur les groupes sont ouverts
/// d'office.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NiveauDetail {
    /// Les groupes de profondeur strictement inférieure au niveau sont ouverts.
    Niveau(u8),
    /// Tous les groupes sont ouverts (`all`).
    Tous,
}

impl Serialize for NiveauDetail {
    /// Un entier, ou la chaîne `"all"` (contrat de l'écho `grouping`).
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Niveau(n) => s.serialize_u8(*n),
            Self::Tous => s.serialize_str("all"),
        }
    }
}

/// Où se place le sous-total d'un groupe ouvert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PositionSousTotal {
    /// Sur la ligne du titre du groupe (`header`, défaut).
    #[serde(rename = "header")]
    Titre,
    /// Sur une ligne `subtotal` sous la descendance du groupe (`footer`).
    #[serde(rename = "footer")]
    Pied,
}

/// Les choix de présentation d'un rapport groupé.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionsGroupes {
    /// Niveau de détail (état d'ouverture de départ).
    pub niveau: NiveauDetail,
    /// Position des sous-totaux.
    pub position: PositionSousTotal,
    /// Groupes ouverts à la main (priment sur le niveau).
    pub ouverts: BTreeSet<i64>,
    /// Groupes fermés à la main (priment sur le niveau).
    pub fermes: BTreeSet<i64>,
}

impl OptionsGroupes {
    /// Règle d'ouverture du contrat : un groupe affiché de profondeur `p` est
    /// ouvert si et seulement si `id ∈ ouverts`, ou bien `id ∉ fermes` et
    /// (`niveau = Tous` ou `p < niveau`).
    pub fn est_ouvert(&self, id: i64, profondeur: u32) -> bool {
        if self.ouverts.contains(&id) {
            return true;
        }
        if self.fermes.contains(&id) {
            return false;
        }
        match self.niveau {
            NiveauDetail::Tous => true,
            NiveauDetail::Niveau(n) => profondeur < u32::from(n),
        }
    }
}

// ===========================================================================
// L'arbre
// ===========================================================================

/// Ce que l'arbre retient d'un compte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoeudCompte {
    /// Identifiant du compte.
    pub id: i64,
    /// Société du compte (un parent d'une autre société est refusé).
    pub company_id: i64,
    /// Numéro du compte (tri des frères, D10).
    pub numero: String,
    /// Nom du compte.
    pub nom: String,
    /// Parent déclaré (`accounts.parent_id`).
    pub parent_id: Option<i64>,
}

impl From<&Account> for NoeudCompte {
    fn from(a: &Account) -> Self {
        Self {
            id: a.id,
            company_id: a.company_id,
            numero: a.number.clone(),
            nom: a.name.clone(),
            parent_id: a.parent_id,
        }
    }
}

/// L'arbre des comptes d'une société, depuis `parent_id` seul (D1).
///
/// Défensif (AC-7, NFR-46-4) : un parent inconnu, d'une autre société, ou un
/// cycle — possibles par SQL seulement — font du compte une **racine**, avec
/// un `tracing::warn!` par compte ; aucune boucle, aucune panique.
#[derive(Debug, Clone, Default)]
pub struct ArbreComptes {
    noeuds: HashMap<i64, NoeudCompte>,
    /// Parent **retenu** (validé) de chaque compte.
    parents: HashMap<i64, i64>,
}

impl ArbreComptes {
    /// Construit l'arbre depuis les comptes de la société (archivés compris).
    pub fn depuis(comptes: &[Account]) -> Self {
        Self::depuis_noeuds(comptes.iter().map(NoeudCompte::from).collect())
    }

    /// Construit l'arbre depuis des nœuds déjà extraits (tests, et
    /// [`ArbreComptes::depuis`]).
    pub fn depuis_noeuds(noeuds: Vec<NoeudCompte>) -> Self {
        let noeuds: HashMap<i64, NoeudCompte> = noeuds.into_iter().map(|n| (n.id, n)).collect();
        let mut parents: HashMap<i64, i64> = HashMap::new();
        // Ordre déterministe (numéro, puis identifiant) : le compte coupé d'un
        // cycle est toujours le même d'un appel à l'autre.
        let mut ordre: Vec<&NoeudCompte> = noeuds.values().collect();
        ordre.sort_by(|a, b| a.numero.cmp(&b.numero).then(a.id.cmp(&b.id)));
        for n in &ordre {
            let Some(p) = n.parent_id else { continue };
            match noeuds.get(&p) {
                None => tracing::warn!(
                    compte = %n.numero,
                    "arbre des comptes : parent inconnu, compte traité en racine"
                ),
                Some(parent) if parent.company_id != n.company_id => tracing::warn!(
                    compte = %n.numero,
                    "arbre des comptes : parent d'une autre société, compte traité en racine"
                ),
                Some(_) if p == n.id => tracing::warn!(
                    compte = %n.numero,
                    "arbre des comptes : compte parent de lui-même, traité en racine"
                ),
                Some(_) => {
                    parents.insert(n.id, p);
                }
            }
        }
        // Coupe des cycles : la remontée depuis un compte qui revient à lui-même
        // en moins de `noeuds.len()` pas le fait racine. Bornée à chaque fois.
        let borne = noeuds.len();
        for n in &ordre {
            let mut courant = parents.get(&n.id).copied();
            let mut pas = 0usize;
            while let Some(c) = courant {
                if c == n.id {
                    tracing::warn!(
                        compte = %n.numero,
                        "arbre des comptes : cycle de parents, compte traité en racine"
                    );
                    parents.remove(&n.id);
                    break;
                }
                pas += 1;
                if pas > borne {
                    break;
                }
                courant = parents.get(&c).copied();
            }
        }
        Self { noeuds, parents }
    }

    /// Parent retenu d'un compte (`None` : racine, ou compte inconnu).
    pub fn parent(&self, id: i64) -> Option<i64> {
        self.parents.get(&id).copied()
    }

    /// Le nœud d'un compte, s'il est connu de l'arbre.
    pub fn noeud(&self, id: i64) -> Option<&NoeudCompte> {
        self.noeuds.get(&id)
    }

    /// Ancêtres stricts d'un compte, du parent vers la racine (bornés par le
    /// nombre de comptes : l'arbre retenu est sans cycle, la borne est une
    /// ceinture de plus).
    fn ancetres(&self, id: i64) -> impl Iterator<Item = i64> + '_ {
        let borne = self.noeuds.len();
        let mut courant = self.parent(id);
        let mut pas = 0usize;
        std::iter::from_fn(move || {
            let c = courant?;
            pas += 1;
            if pas > borne {
                return None;
            }
            courant = self.parent(c);
            Some(c)
        })
    }
}

// ===========================================================================
// Les lignes
// ===========================================================================

/// Une ligne de rapport que le regroupement sait additionner : le bilan et le
/// compte de résultat l'implémentent pour [`AccountBalance`] (un montant), la
/// balance (46-6a) pour ses lignes à cinq colonnes.
pub trait LigneAgregeable {
    /// Identifiant du compte.
    fn compte_id(&self) -> i64;
    /// Numéro du compte.
    fn numero(&self) -> &str;
    /// Nom du compte.
    fn nom(&self) -> &str;
    /// Devise étrangère du compte, `None` en devise principale.
    fn devise(&self) -> Option<&str>;
    /// Les montants en devise principale (même longueur pour toutes les lignes).
    fn montants(&self) -> Vec<Decimal>;
    /// Les montants en devise (même longueur que [`Self::montants`]), `None`
    /// pour un compte sans devise.
    fn montants_devise(&self) -> Option<Vec<Decimal>>;
}

impl LigneAgregeable for AccountBalance {
    fn compte_id(&self) -> i64 {
        self.account_id
    }
    fn numero(&self) -> &str {
        &self.account_number
    }
    fn nom(&self) -> &str {
        &self.account_name
    }
    fn devise(&self) -> Option<&str> {
        self.currency.as_deref()
    }
    fn montants(&self) -> Vec<Decimal> {
        vec![self.balance]
    }
    fn montants_devise(&self) -> Option<Vec<Decimal>> {
        self.balance_currency.map(|b| vec![b])
    }
}

/// Genre d'une ligne groupée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum GenreLigne {
    /// Un compte de groupe affiché ; ses montants sont son sous-total.
    #[serde(rename = "group")]
    Groupe,
    /// Un compte feuille de la section (ou un groupe dissous, ou dont aucun
    /// sous-compte n'est présent).
    #[serde(rename = "account")]
    Compte,
    /// Le solde propre d'un compte de groupe qui porte des écritures (D5).
    #[serde(rename = "own")]
    Propre,
    /// Le sous-total d'un groupe ouvert, en position `footer`.
    #[serde(rename = "subtotal")]
    SousTotal,
}

/// Une ligne groupée, neutre (un vecteur de montants) : chaque rapport la
/// convertit en sa ligne JSON à champs nommés.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LigneGroupee {
    /// Genre de la ligne.
    pub genre: GenreLigne,
    /// Profondeur dans l'arbre affiché de la section (racines : 1).
    pub profondeur: u32,
    /// Identifiant du compte.
    pub compte_id: i64,
    /// Numéro du compte.
    pub numero: String,
    /// Nom du compte.
    pub nom: String,
    /// Montants (sous-total pour un groupe ou un sous-total).
    pub montants: Vec<Decimal>,
    /// Devise étrangère, si toute la descendance la partage (AC-6).
    pub devise: Option<String>,
    /// Montants en devise, présents avec [`Self::devise`].
    pub montants_devise: Option<Vec<Decimal>>,
    /// État d'un groupe (`Groupe` seulement).
    pub ouvert: Option<bool>,
    /// Indice de la ligne d'entrée (`Compte` et `Propre` seulement) : de quoi
    /// relire ce que la ligne neutre ne porte pas (actif, rôle, miroir).
    pub source: Option<usize>,
}

/// Une section regroupée, avec ce qu'en lit l'écho `grouping`.
#[derive(Debug, Clone, Default)]
pub struct SectionGroupee {
    /// Les lignes, dans l'ordre d'affichage.
    pub lignes: Vec<LigneGroupee>,
    /// Plus grande profondeur d'un groupe ou d'un compte de l'arbre affiché,
    /// indépendamment de l'ouverture ; 0 pour une section vide.
    pub profondeur_max: u32,
    /// Les groupes de l'arbre affiché (visibles ou non).
    pub groupes: BTreeSet<i64>,
}

// ===========================================================================
// Dissolution des groupes à cheval
// ===========================================================================

/// Les groupes **dissous** : leur descendance présente (solde propre compris)
/// tombe dans plusieurs partitions (AC-4).
///
/// La partition est une clé fournie par l'appelant, distincte de la liste de
/// sortie : la section pour le bilan et le compte de résultat, la nature pour la
/// balance (46-6a). La dissolution dépend des **lignes présentes**, non de la
/// structure du plan.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Dissolution(HashSet<i64>);

impl Dissolution {
    /// Calcule les groupes dissous depuis les lignes présentes, chacune avec sa
    /// clé de partition. Un groupe est dissous s'il a au moins un descendant
    /// présent et que ses lignes (la sienne comprise) portent plus d'une clé.
    pub fn calculer<K: Eq + Hash + Clone>(
        arbre: &ArbreComptes,
        lignes: impl IntoIterator<Item = (i64, K)>,
    ) -> Self {
        let mut parts: HashMap<i64, HashSet<K>> = HashMap::new();
        let mut avec_descendance: HashSet<i64> = HashSet::new();
        for (id, cle) in lignes {
            for a in arbre.ancetres(id) {
                avec_descendance.insert(a);
                parts.entry(a).or_default().insert(cle.clone());
            }
            parts.entry(id).or_default().insert(cle);
        }
        Self(
            avec_descendance
                .into_iter()
                .filter(|a| parts.get(a).is_some_and(|p| p.len() > 1))
                .collect(),
        )
    }

    /// `true` si le groupe est dissous.
    pub fn contient(&self, id: i64) -> bool {
        self.0.contains(&id)
    }
}

// ===========================================================================
// Le regroupement d'une liste
// ===========================================================================

/// Agrégat d'un nœud affiché : montants exacts en devise principale, et en
/// devise étrangère par devise (pour décider d'une devise unique, AC-6, et d'un
/// sous-total nul « en CHF et en devise », AC-5).
#[derive(Debug, Clone)]
struct Agregat {
    montants: Vec<Decimal>,
    par_devise: BTreeMap<String, Vec<Decimal>>,
    /// Au moins une ligne d'entrée en devise principale.
    ligne_principale: bool,
}

impl Agregat {
    fn vide(n: usize) -> Self {
        Self {
            montants: vec![Decimal::ZERO; n],
            par_devise: BTreeMap::new(),
            ligne_principale: false,
        }
    }

    fn ajouter_ligne<L: LigneAgregeable>(&mut self, l: &L) {
        additionner(&mut self.montants, &l.montants());
        match (l.devise(), l.montants_devise()) {
            (Some(d), Some(m)) => {
                let cible = self
                    .par_devise
                    .entry(d.to_string())
                    .or_insert_with(|| vec![Decimal::ZERO; m.len()]);
                additionner(cible, &m);
            }
            _ => self.ligne_principale = true,
        }
    }

    fn ajouter(&mut self, autre: &Agregat) {
        additionner(&mut self.montants, &autre.montants);
        for (d, m) in &autre.par_devise {
            let cible = self
                .par_devise
                .entry(d.clone())
                .or_insert_with(|| vec![Decimal::ZERO; m.len()]);
            additionner(cible, m);
        }
        self.ligne_principale |= autre.ligne_principale;
    }

    /// La devise unique de la descendance, et ses montants (AC-6).
    fn devise_unique(&self) -> (Option<String>, Option<Vec<Decimal>>) {
        if self.ligne_principale || self.par_devise.len() != 1 {
            return (None, None);
        }
        let (d, m) = self.par_devise.iter().next().expect("une devise");
        (Some(d.clone()), Some(m.clone()))
    }

    /// Nul en devise principale **et** dans chaque devise (AC-5).
    fn est_nul(&self) -> bool {
        self.montants.iter().all(Decimal::is_zero)
            && self
                .par_devise
                .values()
                .all(|m| m.iter().all(Decimal::is_zero))
    }
}

/// Addition exacte, colonne à colonne (une colonne absente vaut zéro).
fn additionner(cible: &mut Vec<Decimal>, ajout: &[Decimal]) {
    if cible.len() < ajout.len() {
        cible.resize(ajout.len(), Decimal::ZERO);
    }
    for (c, a) in cible.iter_mut().zip(ajout) {
        *c += *a;
    }
}

/// Un nœud de l'arbre affiché d'une liste.
#[derive(Debug, Default)]
struct NoeudAffiche {
    /// La ligne d'entrée du compte lui-même, s'il en a une.
    ligne: Option<usize>,
    enfants: Vec<i64>,
}

/// L'arbre affiché d'une liste : les comptes présents et leurs ancêtres non
/// dissous ; un compte remonte au premier ancêtre non dissous (AC-4).
struct ArbreAffiche<'a> {
    arbre: &'a ArbreComptes,
    noeuds: HashMap<i64, NoeudAffiche>,
    racines: Vec<i64>,
}

impl<'a> ArbreAffiche<'a> {
    fn construire<L: LigneAgregeable>(
        arbre: &'a ArbreComptes,
        dissous: &Dissolution,
        lignes: &[L],
    ) -> Self {
        let mut noeuds: HashMap<i64, NoeudAffiche> = HashMap::new();
        let mut racines: Vec<i64> = Vec::new();
        for (i, l) in lignes.iter().enumerate() {
            let id = l.compte_id();
            let deja = noeuds.contains_key(&id);
            noeuds.entry(id).or_default().ligne = Some(i);
            if deja {
                continue;
            }
            // Accroche le compte, puis ses ancêtres affichés tant qu'ils sont
            // neufs : chaque arête n'est posée qu'une fois.
            let mut enfant = id;
            loop {
                match arbre.ancetres(enfant).find(|a| !dissous.contient(*a)) {
                    None => {
                        racines.push(enfant);
                        break;
                    }
                    Some(p) => {
                        let neuf = !noeuds.contains_key(&p);
                        noeuds.entry(p).or_default().enfants.push(enfant);
                        if !neuf {
                            break;
                        }
                        enfant = p;
                    }
                }
            }
        }
        let mut a = Self {
            arbre,
            noeuds,
            racines,
        };
        a.trier(lignes);
        a
    }

    /// Numéro d'affichage d'un compte : celui de l'arbre, sinon celui de sa
    /// ligne (compte absent de l'arbre).
    fn numero<'l, L: LigneAgregeable>(&'l self, id: i64, lignes: &'l [L]) -> &'l str {
        if let Some(n) = self.arbre.noeud(id) {
            return &n.numero;
        }
        self.noeuds
            .get(&id)
            .and_then(|n| n.ligne)
            .map(|i| lignes[i].numero())
            .unwrap_or("")
    }

    /// Frères triés par numéro, `String::cmp` (D10).
    fn trier<L: LigneAgregeable>(&mut self, lignes: &[L]) {
        let cle = |s: &Self, id: i64| (s.numero(id, lignes).to_string(), id);
        let mut racines = std::mem::take(&mut self.racines);
        racines.sort_by_key(|id| cle(self, *id));
        self.racines = racines;
        let ids: Vec<i64> = self.noeuds.keys().copied().collect();
        for id in ids {
            let mut enfants = std::mem::take(&mut self.noeuds.get_mut(&id).unwrap().enfants);
            enfants.sort_by_key(|e| cle(self, *e));
            self.noeuds.get_mut(&id).unwrap().enfants = enfants;
        }
    }

    fn est_groupe(&self, id: i64) -> bool {
        self.noeuds.get(&id).is_some_and(|n| !n.enfants.is_empty())
    }
}

/// Le regroupement en cours d'une liste.
struct Regroupement<'a, L> {
    affiche: &'a ArbreAffiche<'a>,
    lignes: &'a [L],
    options: &'a OptionsGroupes,
    largeur: usize,
    agregats: HashMap<i64, Agregat>,
    sortie: Vec<LigneGroupee>,
    profondeur_max: u32,
    groupes: BTreeSet<i64>,
}

impl<L: LigneAgregeable> Regroupement<'_, L> {
    /// Parcours postfixe : agrégat de chaque nœud (solde propre compris, D5),
    /// profondeur maximale et ensemble des groupes.
    fn agreger(&mut self, id: i64, profondeur: u32) -> Agregat {
        self.profondeur_max = self.profondeur_max.max(profondeur);
        let affiche = self.affiche;
        let noeud = &affiche.noeuds[&id];
        let mut agg = Agregat::vide(self.largeur);
        if let Some(i) = noeud.ligne {
            agg.ajouter_ligne(&self.lignes[i]);
        }
        if !noeud.enfants.is_empty() {
            self.groupes.insert(id);
        }
        for &e in &noeud.enfants {
            let a = self.agreger(e, profondeur + 1);
            agg.ajouter(&a);
        }
        self.agregats.insert(id, agg.clone());
        agg
    }

    fn nom(&self, id: i64) -> (String, String) {
        let ligne = self.affiche.noeuds[&id].ligne.map(|i| &self.lignes[i]);
        match (self.affiche.arbre.noeud(id), ligne) {
            (_, Some(l)) => (l.numero().to_string(), l.nom().to_string()),
            (Some(n), None) => (n.numero.clone(), n.nom.clone()),
            (None, None) => (String::new(), String::new()),
        }
    }

    fn ligne_entree(&self, genre: GenreLigne, i: usize, profondeur: u32) -> LigneGroupee {
        let l = &self.lignes[i];
        let devise = l.devise().map(str::to_string);
        let montants_devise = devise.as_ref().and(l.montants_devise());
        LigneGroupee {
            genre,
            profondeur,
            compte_id: l.compte_id(),
            numero: l.numero().to_string(),
            nom: l.nom().to_string(),
            montants: l.montants(),
            devise: if montants_devise.is_some() {
                devise
            } else {
                None
            },
            montants_devise,
            ouvert: None,
            source: Some(i),
        }
    }

    fn ligne_groupe(
        &self,
        genre: GenreLigne,
        id: i64,
        profondeur: u32,
        ouvert: Option<bool>,
    ) -> LigneGroupee {
        let agg = &self.agregats[&id];
        let (devise, montants_devise) = agg.devise_unique();
        let (numero, nom) = self.nom(id);
        LigneGroupee {
            genre,
            profondeur,
            compte_id: id,
            numero,
            nom,
            montants: agg.montants.clone(),
            devise,
            montants_devise,
            ouvert,
            source: None,
        }
    }

    /// Parcours préfixe : émission des lignes selon la règle d'ouverture.
    fn emettre(&mut self, id: i64, profondeur: u32) {
        let affiche = self.affiche;
        let noeud = &affiche.noeuds[&id];
        if !affiche.est_groupe(id) {
            if let Some(i) = noeud.ligne {
                let l = self.ligne_entree(GenreLigne::Compte, i, profondeur);
                self.sortie.push(l);
            }
            return;
        }
        let ouvert = self.options.est_ouvert(id, profondeur);
        if !ouvert {
            // AC-5 : un groupe fermé **par le niveau** à sous-total nul n'est pas
            // affiché (son zéro reste dans le sous-total du parent) ; fermé à la
            // main, il reste, pour pouvoir être rouvert.
            let ferme_a_la_main = self.options.fermes.contains(&id);
            if !ferme_a_la_main && self.agregats[&id].est_nul() {
                return;
            }
            let l = self.ligne_groupe(GenreLigne::Groupe, id, profondeur, Some(false));
            self.sortie.push(l);
            return;
        }
        let l = self.ligne_groupe(GenreLigne::Groupe, id, profondeur, Some(true));
        self.sortie.push(l);
        if let Some(i) = noeud.ligne {
            let l = self.ligne_entree(GenreLigne::Propre, i, profondeur + 1);
            self.sortie.push(l);
        }
        for &e in &noeud.enfants {
            self.emettre(e, profondeur + 1);
        }
        if self.options.position == PositionSousTotal::Pied {
            let l = self.ligne_groupe(GenreLigne::SousTotal, id, profondeur, None);
            self.sortie.push(l);
        }
    }
}

/// Regroupe une liste de lignes (une section) selon l'arbre, les groupes
/// dissous et les options. Pur, O(n × profondeur).
///
/// Invariants (AC-2) : la somme des montants des lignes de profondeur 1 —
/// lignes `SousTotal` exclues — égale la somme des lignes d'entrée, à tout
/// niveau et pour tout état d'ouverture ; chaque ligne d'entrée est comptée une
/// fois et une seule.
pub fn regrouper<L: LigneAgregeable>(
    lignes: &[L],
    arbre: &ArbreComptes,
    dissous: &Dissolution,
    options: &OptionsGroupes,
) -> SectionGroupee {
    let affiche = ArbreAffiche::construire(arbre, dissous, lignes);
    let largeur = lignes.first().map_or(0, |l| l.montants().len());
    let mut r = Regroupement {
        affiche: &affiche,
        lignes,
        options,
        largeur,
        agregats: HashMap::new(),
        sortie: Vec::with_capacity(lignes.len()),
        profondeur_max: 0,
        groupes: BTreeSet::new(),
    };
    let racines = affiche.racines.clone();
    for &id in &racines {
        r.agreger(id, 1);
    }
    for &id in &racines {
        r.emettre(id, 1);
    }
    SectionGroupee {
        lignes: r.sortie,
        profondeur_max: r.profondeur_max,
        groupes: r.groupes,
    }
}

/// Regroupe des sections dont la **section** est la clé de partition (bilan,
/// compte de résultat) : dissolution calculée sur toutes les sections, puis
/// [`regrouper`] par section.
pub fn regrouper_sections<L: LigneAgregeable>(
    sections: &[&[L]],
    arbre: &ArbreComptes,
    options: &OptionsGroupes,
) -> Vec<SectionGroupee> {
    let dissous = Dissolution::calculer(
        arbre,
        sections
            .iter()
            .enumerate()
            .flat_map(|(k, s)| s.iter().map(move |l| (l.compte_id(), k))),
    );
    sections
        .iter()
        .map(|s| regrouper(s, arbre, &dissous, options))
        .collect()
}

// ===========================================================================
// Le contrat JSON du bilan et du compte de résultat
// ===========================================================================

/// L'écho `grouping` : les choix **retenus** et la profondeur maximale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Grouping {
    /// Niveau de détail demandé (`1`…`20` ou `"all"`).
    pub detail_level: NiveauDetail,
    /// Position des sous-totaux.
    pub subtotal_position: PositionSousTotal,
    /// Plus grande profondeur de l'arbre affiché, toutes sections, quel que
    /// soit l'état d'ouverture ; 0 pour un rapport vide.
    pub max_depth: u32,
    /// Groupes ouverts à la main retenus (groupes de l'arbre affiché), triés.
    pub open: Vec<i64>,
    /// Groupes fermés à la main retenus, triés.
    pub closed: Vec<i64>,
}

impl Grouping {
    /// L'écho des options, filtré sur les groupes de l'arbre affiché des
    /// sections : un identifiant inconnu, d'une autre société ou qui n'est pas
    /// un groupe affiché est ignoré, et retiré de l'écho.
    fn depuis(options: &OptionsGroupes, sections: &[SectionGroupee]) -> Self {
        let groupes: BTreeSet<i64> = sections
            .iter()
            .flat_map(|s| s.groupes.iter().copied())
            .collect();
        let retenus = |e: &BTreeSet<i64>| e.intersection(&groupes).copied().collect();
        Self {
            detail_level: options.niveau,
            subtotal_position: options.position,
            max_depth: sections.iter().map(|s| s.profondeur_max).max().unwrap_or(0),
            open: retenus(&options.ouverts),
            closed: retenus(&options.fermes),
        }
    }
}

/// Une ligne groupée du bilan ou du compte de résultat, telle que sérialisée
/// dans `groupedRows`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LigneRapport {
    /// `group`, `account`, `own` ou `subtotal`.
    pub kind: GenreLigne,
    /// Profondeur dans l'arbre affiché de la section.
    pub depth: u32,
    /// Identifiant du compte.
    pub account_id: i64,
    /// Numéro du compte.
    pub account_number: String,
    /// Nom du compte.
    pub account_name: String,
    /// Solde (sous-total d'un groupe), exact.
    pub balance: Decimal,
    /// Devise étrangère (AC-6).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// Solde en devise (AC-6).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_currency: Option<Decimal>,
    /// État d'un groupe (`group` seulement).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open: Option<bool>,
    /// Compte actif (`account` et `own`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Rôle du compte (`account` et `own`), lu tel quel de [`AccountBalance`] ;
    /// `null` pour un compte sans rôle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Option<AccountRole>>,
}

impl LigneRapport {
    fn depuis(l: LigneGroupee, entree: &[AccountBalance]) -> Self {
        let source = l.source.map(|i| &entree[i]);
        Self {
            kind: l.genre,
            depth: l.profondeur,
            account_id: l.compte_id,
            account_number: l.numero,
            account_name: l.nom,
            balance: l.montants.first().copied().unwrap_or(Decimal::ZERO),
            currency: l.devise,
            balance_currency: l.montants_devise.and_then(|m| m.first().copied()),
            open: l.ouvert,
            active: source.map(|a| a.active),
            role: source.map(|a| a.role),
        }
    }

    fn section(s: SectionGroupee, entree: &[AccountBalance]) -> Vec<Self> {
        s.lignes
            .into_iter()
            .map(|l| Self::depuis(l, entree))
            .collect()
    }
}

/// Les sections groupées du bilan (`groupedRows`). Les lignes calculées
/// (`retainedEarnings`, `equityResult`) restent hors des groupes (D4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionsBilan {
    /// Actifs.
    pub assets: Vec<LigneRapport>,
    /// Dettes.
    pub liabilities: Vec<LigneRapport>,
    /// Fonds propres (ordre de l'arbre, non par rôle).
    pub equity: Vec<LigneRapport>,
}

/// Les sections groupées du compte de résultat (`groupedRows`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionsResultat {
    /// Produits.
    pub revenues: Vec<LigneRapport>,
    /// Charges.
    pub expenses: Vec<LigneRapport>,
}

/// Un rapport, et, sur demande, ses groupes (C-46-1-4).
///
/// Le rapport est sérialisé à plat (`flatten`), puis `grouping` et
/// `groupedRows`, **absents** (non `null`) sans demande : la réponse est alors
/// celle du rapport seul, à l'octet (NFR-46-1).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvecGroupes<T, S> {
    /// Le rapport, inchangé.
    #[serde(flatten)]
    pub rapport: T,
    /// L'écho des choix retenus.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grouping: Option<Grouping>,
    /// Les sections groupées.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grouped_rows: Option<S>,
}

impl<T, S> AvecGroupes<T, S> {
    /// Le rapport seul, sans groupes.
    pub fn sans_groupes(rapport: T) -> Self {
        Self {
            rapport,
            grouping: None,
            grouped_rows: None,
        }
    }
}

impl AvecGroupes<BalanceSheet, SectionsBilan> {
    /// Le bilan groupé : Actifs, Dettes, Fonds propres sont les partitions
    /// (AC-4).
    pub fn bilan(rapport: BalanceSheet, arbre: &ArbreComptes, options: &OptionsGroupes) -> Self {
        let mut s = regrouper_sections(
            &[&rapport.assets, &rapport.liabilities, &rapport.equity],
            arbre,
            options,
        );
        let grouping = Grouping::depuis(options, &s);
        let equity = LigneRapport::section(s.pop().unwrap_or_default(), &rapport.equity);
        let liabilities = LigneRapport::section(s.pop().unwrap_or_default(), &rapport.liabilities);
        let assets = LigneRapport::section(s.pop().unwrap_or_default(), &rapport.assets);
        Self {
            rapport,
            grouping: Some(grouping),
            grouped_rows: Some(SectionsBilan {
                assets,
                liabilities,
                equity,
            }),
        }
    }
}

impl AvecGroupes<IncomeStatement, SectionsResultat> {
    /// Le compte de résultat groupé : Produits, Charges sont les partitions.
    pub fn compte_de_resultat(
        rapport: IncomeStatement,
        arbre: &ArbreComptes,
        options: &OptionsGroupes,
    ) -> Self {
        let mut s = regrouper_sections(&[&rapport.revenues, &rapport.expenses], arbre, options);
        let grouping = Grouping::depuis(options, &s);
        let expenses = LigneRapport::section(s.pop().unwrap_or_default(), &rapport.expenses);
        let revenues = LigneRapport::section(s.pop().unwrap_or_default(), &rapport.revenues);
        Self {
            rapport,
            grouping: Some(grouping),
            grouped_rows: Some(SectionsResultat { revenues, expenses }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    // ---- Montage -------------------------------------------------------

    /// Ligne d'entrée de test : deux colonnes pour le test de propriété, une
    /// seule ailleurs.
    #[derive(Debug, Clone)]
    struct L {
        id: i64,
        numero: String,
        devise: Option<String>,
        m: Vec<Decimal>,
        md: Option<Vec<Decimal>>,
    }

    impl LigneAgregeable for L {
        fn compte_id(&self) -> i64 {
            self.id
        }
        fn numero(&self) -> &str {
            &self.numero
        }
        fn nom(&self) -> &str {
            "nom"
        }
        fn devise(&self) -> Option<&str> {
            self.devise.as_deref()
        }
        fn montants(&self) -> Vec<Decimal> {
            self.m.clone()
        }
        fn montants_devise(&self) -> Option<Vec<Decimal>> {
            self.md.clone()
        }
    }

    /// Un plan de test : `(numéro, numéro du parent)` ; l'identifiant d'un
    /// compte est son numéro lu comme entier, préfixé de `1` (pour garder les
    /// zéros de tête distincts).
    struct Plan {
        arbre: ArbreComptes,
    }

    fn ident(numero: &str) -> i64 {
        format!("1{numero}").parse().unwrap()
    }

    fn plan(comptes: &[(&str, Option<&str>)]) -> Plan {
        let noeuds = comptes
            .iter()
            .map(|(n, p)| NoeudCompte {
                id: ident(n),
                company_id: 1,
                numero: n.to_string(),
                nom: format!("Compte {n}"),
                parent_id: p.map(ident),
            })
            .collect();
        Plan {
            arbre: ArbreComptes::depuis_noeuds(noeuds),
        }
    }

    fn l(numero: &str, montant: Decimal) -> L {
        L {
            id: ident(numero),
            numero: numero.to_string(),
            devise: None,
            m: vec![montant],
            md: None,
        }
    }

    fn l_devise(numero: &str, montant: Decimal, devise: &str, md: Decimal) -> L {
        L {
            devise: Some(devise.to_string()),
            md: Some(vec![md]),
            ..l(numero, montant)
        }
    }

    fn opts(niveau: NiveauDetail) -> OptionsGroupes {
        OptionsGroupes {
            niveau,
            position: PositionSousTotal::Titre,
            ouverts: BTreeSet::new(),
            fermes: BTreeSet::new(),
        }
    }

    fn un_seul(lignes: &[L], p: &Plan, o: &OptionsGroupes) -> SectionGroupee {
        regrouper(lignes, &p.arbre, &Dissolution::default(), o)
    }

    /// Rendu compact d'une section : `genre:numéro@profondeur=montant`.
    fn rendu(s: &SectionGroupee) -> Vec<String> {
        s.lignes
            .iter()
            .map(|g| {
                let genre = match g.genre {
                    GenreLigne::Groupe if g.ouvert == Some(true) => "G+",
                    GenreLigne::Groupe => "G-",
                    GenreLigne::Compte => "C",
                    GenreLigne::Propre => "P",
                    GenreLigne::SousTotal => "S",
                };
                format!("{genre}:{}@{}={}", g.numero, g.profondeur, g.montants[0])
            })
            .collect()
    }

    /// Somme des lignes de profondeur 1, lignes `SousTotal` exclues (AC-2).
    fn somme_racines(s: &SectionGroupee, col: usize) -> Decimal {
        s.lignes
            .iter()
            .filter(|g| g.profondeur == 1 && g.genre != GenreLigne::SousTotal)
            .map(|g| g.montants[col])
            .sum()
    }

    /// Un petit plan : 1 > 10 > {1000, 1020}, 1 > 14 > 1500, 9 seul.
    fn petit_plan() -> Plan {
        plan(&[
            ("1", None),
            ("10", Some("1")),
            ("1000", Some("10")),
            ("1020", Some("10")),
            ("14", Some("1")),
            ("1500", Some("14")),
            ("9", None),
        ])
    }

    fn petites_lignes() -> Vec<L> {
        vec![
            l("1000", dec!(10)),
            l("1020", dec!(20)),
            l("1500", dec!(5)),
            l("9", dec!(1)),
        ]
    }

    // ---- Niveaux (AC-3) ------------------------------------------------

    #[test]
    fn niveaux_un_deux_trois_et_tous() {
        let p = petit_plan();
        let ls = petites_lignes();
        let n1 = un_seul(&ls, &p, &opts(NiveauDetail::Niveau(1)));
        assert_eq!(rendu(&n1), ["G-:1@1=35", "C:9@1=1"]);
        let n2 = un_seul(&ls, &p, &opts(NiveauDetail::Niveau(2)));
        assert_eq!(
            rendu(&n2),
            ["G+:1@1=35", "G-:10@2=30", "G-:14@2=5", "C:9@1=1"]
        );
        let n3 = un_seul(&ls, &p, &opts(NiveauDetail::Niveau(3)));
        let tous = un_seul(&ls, &p, &opts(NiveauDetail::Tous));
        let attendu = [
            "G+:1@1=35",
            "G+:10@2=30",
            "C:1000@3=10",
            "C:1020@3=20",
            "G+:14@2=5",
            "C:1500@3=5",
            "C:9@1=1",
        ];
        assert_eq!(rendu(&n3), attendu);
        assert_eq!(rendu(&tous), attendu);
        // Un niveau supérieur à la profondeur réelle se comporte comme `all`.
        assert_eq!(
            rendu(&un_seul(&ls, &p, &opts(NiveauDetail::Niveau(20)))),
            attendu
        );
        for s in [&n1, &n2, &n3, &tous] {
            assert_eq!(somme_racines(s, 0), dec!(36));
        }
    }

    #[test]
    fn profondeur_max_independante_de_l_ouverture() {
        let p = petit_plan();
        let ls = petites_lignes();
        let n1 = un_seul(&ls, &p, &opts(NiveauDetail::Niveau(1)));
        let tous = un_seul(&ls, &p, &opts(NiveauDetail::Tous));
        assert_eq!(n1.profondeur_max, 3);
        assert_eq!(tous.profondeur_max, 3);
        assert_eq!(n1.groupes, tous.groupes);
        // Section vide : 0 ; plan sans groupe : 1.
        assert_eq!(
            un_seul(&[], &p, &opts(NiveauDetail::Tous)).profondeur_max,
            0
        );
        let plat = plan(&[("1000", None), ("2000", None)]);
        let s = un_seul(
            &[l("1000", dec!(1)), l("2000", dec!(2))],
            &plat,
            &opts(NiveauDetail::Niveau(1)),
        );
        assert_eq!(s.profondeur_max, 1);
        assert_eq!(rendu(&s), ["C:1000@1=1", "C:2000@1=2"]);
    }

    // ---- Ouverture manuelle ------------------------------------------------

    #[test]
    fn ouvert_et_ferme_a_la_main() {
        let p = petit_plan();
        let ls = petites_lignes();
        // Niveau 2 : `10` fermé par le niveau ; `open=10` l'ouvre.
        let mut o = opts(NiveauDetail::Niveau(2));
        o.ouverts.insert(ident("10"));
        assert_eq!(
            rendu(&un_seul(&ls, &p, &o)),
            [
                "G+:1@1=35",
                "G+:10@2=30",
                "C:1000@3=10",
                "C:1020@3=20",
                "G-:14@2=5",
                "C:9@1=1"
            ]
        );
        // `closed=1` ferme `1` et cache `10`, ouvert ou non.
        o.fermes.insert(ident("1"));
        let s = un_seul(&ls, &p, &o);
        assert_eq!(rendu(&s), ["G-:1@1=35", "C:9@1=1"]);
        assert_eq!(somme_racines(&s, 0), dec!(36));
        // Tous ouverts, sauf `14` fermé à la main.
        let mut o = opts(NiveauDetail::Tous);
        o.fermes.insert(ident("14"));
        assert_eq!(
            rendu(&un_seul(&ls, &p, &o)),
            [
                "G+:1@1=35",
                "G+:10@2=30",
                "C:1000@3=10",
                "C:1020@3=20",
                "G-:14@2=5",
                "C:9@1=1"
            ]
        );
    }

    // ---- Position des sous-totaux --------------------------------------

    #[test]
    fn sous_total_en_pied_seulement_pour_un_groupe_ouvert() {
        let p = petit_plan();
        let ls = petites_lignes();
        let mut o = opts(NiveauDetail::Niveau(2));
        o.position = PositionSousTotal::Pied;
        let s = un_seul(&ls, &p, &o);
        assert_eq!(
            rendu(&s),
            [
                "G+:1@1=35",
                "G-:10@2=30",
                "G-:14@2=5",
                "S:1@1=35",
                "C:9@1=1"
            ]
        );
        // AC-2 : la ligne `subtotal`, de même profondeur que son groupe, n'est
        // pas comptée deux fois.
        assert_eq!(somme_racines(&s, 0), dec!(36));
        let st = s
            .lignes
            .iter()
            .find(|g| g.genre == GenreLigne::SousTotal)
            .unwrap();
        assert_eq!((st.compte_id, st.ouvert), (ident("1"), None));
    }

    // ---- Solde propre (D5) ---------------------------------------------

    #[test]
    fn groupe_a_solde_propre_en_premier_enfant() {
        let p = petit_plan();
        let ls = vec![l("1020", dec!(20)), l("10", dec!(7)), l("1000", dec!(10))];
        let s = un_seul(&ls, &p, &opts(NiveauDetail::Tous));
        assert_eq!(
            rendu(&s),
            [
                "G+:1@1=37",
                "G+:10@2=37",
                "P:10@3=7",
                "C:1000@3=10",
                "C:1020@3=20"
            ]
        );
        let propre = &s.lignes[2];
        assert_eq!(propre.source, Some(1));
        // Fermé : le solde propre est dans le sous-total.
        assert_eq!(
            rendu(&un_seul(&ls, &p, &opts(NiveauDetail::Niveau(1)))),
            ["G-:1@1=37"]
        );
        // Un groupe dont seul le solde propre est présent est une ligne `account`.
        let s = un_seul(&[l("10", dec!(7))], &p, &opts(NiveauDetail::Tous));
        assert_eq!(rendu(&s), ["G+:1@1=7", "C:10@2=7"]);
    }

    // ---- Groupes nuls (AC-5) -------------------------------------------

    #[test]
    fn groupe_nul_masque_s_il_est_ferme_par_le_niveau_seulement() {
        let p = petit_plan();
        let ls = vec![
            l("1000", dec!(100)),
            l("1020", dec!(-100)),
            l("1500", dec!(5)),
        ];
        // Fermé par le niveau, nul : masqué, son zéro reste dans `1`.
        let s = un_seul(&ls, &p, &opts(NiveauDetail::Niveau(2)));
        assert_eq!(rendu(&s), ["G+:1@1=5", "G-:14@2=5"]);
        // Fermé à la main, nul : affiché.
        let mut o = opts(NiveauDetail::Niveau(2));
        o.fermes.insert(ident("10"));
        assert_eq!(
            rendu(&un_seul(&ls, &p, &o)),
            ["G+:1@1=5", "G-:10@2=0", "G-:14@2=5"]
        );
        // Ouvert, sous-total nul, descendance non nulle : affiché avec elle.
        let s = un_seul(&ls, &p, &opts(NiveauDetail::Tous));
        assert_eq!(
            rendu(&s),
            [
                "G+:1@1=5",
                "G+:10@2=0",
                "C:1000@3=100",
                "C:1020@3=-100",
                "G+:14@2=5",
                "C:1500@3=5"
            ]
        );
        // Nul en CHF mais non en devise : affiché, fermé par le niveau.
        let ls = vec![
            l_devise("1000", dec!(0), "USD", dec!(3)),
            l("1500", dec!(5)),
        ];
        assert_eq!(
            rendu(&un_seul(&ls, &p, &opts(NiveauDetail::Niveau(2)))),
            ["G+:1@1=5", "G-:10@2=0", "G-:14@2=5"]
        );
    }

    // ---- Devise (AC-6) -------------------------------------------------

    #[test]
    fn devise_unique_melee_ou_avec_la_devise_principale() {
        let p = petit_plan();
        let o = opts(NiveauDetail::Niveau(2));
        let groupe =
            |s: &SectionGroupee, n: &str| s.lignes.iter().find(|g| g.numero == n).cloned().unwrap();
        // Devise unique : présente, somme exacte.
        let ls = vec![
            l_devise("1000", dec!(9.12), "USD", dec!(10)),
            l_devise("1020", dec!(4.56), "USD", dec!(5)),
        ];
        let g = groupe(&un_seul(&ls, &p, &o), "10");
        assert_eq!(g.devise.as_deref(), Some("USD"));
        assert_eq!(g.montants_devise, Some(vec![dec!(15)]));
        assert_eq!(g.montants, vec![dec!(13.68)]);
        // Devises mêlées : CHF seul, aucune ligne supplémentaire.
        let ls = vec![
            l_devise("1000", dec!(9.12), "USD", dec!(10)),
            l_devise("1020", dec!(4.56), "EUR", dec!(5)),
        ];
        let s = un_seul(&ls, &p, &o);
        let g = groupe(&s, "10");
        assert_eq!((g.devise, g.montants_devise), (None, None));
        assert_eq!(s.lignes.len(), 2);
        // Devise principale mêlée à une devise : CHF seul.
        let ls = vec![
            l_devise("1000", dec!(9.12), "USD", dec!(10)),
            l("1020", dec!(1)),
        ];
        let g = groupe(&un_seul(&ls, &p, &o), "10");
        assert_eq!((g.devise, g.montants_devise), (None, None));
        // Ligne `account` en devise : la sienne, telle quelle.
        let ls = vec![l_devise("1000", dec!(9.12), "USD", dec!(10))];
        let s = un_seul(&ls, &p, &opts(NiveauDetail::Tous));
        let c = groupe(&s, "1000");
        assert_eq!(
            (c.devise.as_deref(), c.montants_devise),
            (Some("USD"), Some(vec![dec!(10)]))
        );
    }

    // ---- Arbre défensif (AC-7) -----------------------------------------

    #[test]
    fn cycle_parent_inconnu_et_parent_etranger_traites_en_racine() {
        let mut noeuds = vec![
            NoeudCompte {
                id: 1,
                company_id: 1,
                numero: "1".into(),
                nom: "A".into(),
                parent_id: Some(2),
            },
            NoeudCompte {
                id: 2,
                company_id: 1,
                numero: "2".into(),
                nom: "B".into(),
                parent_id: Some(1),
            },
            NoeudCompte {
                id: 3,
                company_id: 1,
                numero: "3".into(),
                nom: "C".into(),
                parent_id: Some(99),
            },
            NoeudCompte {
                id: 4,
                company_id: 1,
                numero: "4".into(),
                nom: "D".into(),
                parent_id: Some(5),
            },
            NoeudCompte {
                id: 5,
                company_id: 2,
                numero: "5".into(),
                nom: "E".into(),
                parent_id: None,
            },
            NoeudCompte {
                id: 6,
                company_id: 1,
                numero: "6".into(),
                nom: "F".into(),
                parent_id: Some(6),
            },
        ];
        // Cycle plus long : 7 → 8 → 9 → 7.
        for (id, p) in [(7, 8), (8, 9), (9, 7)] {
            noeuds.push(NoeudCompte {
                id,
                company_id: 1,
                numero: id.to_string(),
                nom: "cycle".into(),
                parent_id: Some(p),
            });
        }
        let arbre = ArbreComptes::depuis_noeuds(noeuds);
        // Le premier compte (par numéro) de chaque cycle devient racine.
        assert_eq!(arbre.parent(1), None);
        assert_eq!(arbre.parent(2), Some(1));
        assert_eq!(arbre.parent(3), None);
        assert_eq!(arbre.parent(4), None);
        assert_eq!(arbre.parent(6), None);
        assert_eq!(arbre.parent(7), None);
        assert_eq!((arbre.parent(8), arbre.parent(9)), (Some(9), Some(7)));
        // Aucune boucle : le regroupement termine.
        let ls: Vec<L> = [1, 2, 3, 4, 6, 7, 8, 9]
            .iter()
            .map(|&i| L {
                id: i,
                numero: i.to_string(),
                devise: None,
                m: vec![dec!(1)],
                md: None,
            })
            .collect();
        let s = regrouper(
            &ls,
            &arbre,
            &Dissolution::default(),
            &opts(NiveauDetail::Tous),
        );
        assert_eq!(somme_racines(&s, 0), dec!(8));
    }

    // ---- Dissolution (AC-4) --------------------------------------------

    #[test]
    fn groupe_a_cheval_dissous_et_son_solde_propre() {
        // 2 > {20 > 2000, 28 > 2800}, et `2` porte un solde propre.
        let p = plan(&[
            ("2", None),
            ("20", Some("2")),
            ("2000", Some("20")),
            ("28", Some("2")),
            ("2800", Some("28")),
        ]);
        let dettes = [l("2000", dec!(10)), l("2", dec!(3))];
        let fonds = [l("2800", dec!(50))];
        let s = regrouper_sections(
            &[&dettes[..], &fonds[..]],
            &p.arbre,
            &opts(NiveauDetail::Tous),
        );
        // `2` dissous : `20` racine des dettes, le solde propre de `2` devient
        // une ligne `account` au rang de son numéro parmi les enfants remontés.
        assert_eq!(rendu(&s[0]), ["C:2@1=3", "G+:20@1=10", "C:2000@2=10"]);
        assert_eq!(rendu(&s[1]), ["G+:28@1=50", "C:2800@2=50"]);
        // Sans fonds propres présents, `2` n'est pas dissous.
        let s = regrouper_sections(&[&dettes[..], &[][..]], &p.arbre, &opts(NiveauDetail::Tous));
        assert_eq!(
            rendu(&s[0]),
            ["G+:2@1=13", "P:2@2=3", "G+:20@2=10", "C:2000@3=10"]
        );
    }

    /// Le plan PME (`pme.json`) : sections du bilan par rôle, du compte de
    /// résultat par type ; dissolutions relevées à la validation P2.
    #[test]
    fn plan_pme_complet_et_ses_dissolutions() {
        use kesh_core::chart_of_accounts::{AccountRole as Role, AccountType, load_chart};
        let plan_pme = load_chart("pme").unwrap();
        let p = plan(
            &plan_pme
                .iter()
                .map(|e| (e.number.as_str(), e.parent_number.as_deref()))
                .collect::<Vec<_>>(),
        );
        let type_de = |n: &str| {
            plan_pme
                .iter()
                .find(|e| e.number == n)
                .unwrap()
                .account_type
        };
        let est_fonds_propres = |n: &str| {
            plan_pme
                .iter()
                .find(|e| e.number == n)
                .unwrap()
                .role
                .is_some_and(|r| {
                    matches!(
                        r,
                        Role::EquityCapital
                            | Role::EquityOther
                            | Role::RetainedEarnings
                            | Role::CurrentYearResult
                    )
                })
        };
        let feuilles: Vec<&str> = plan_pme
            .iter()
            .filter(|e| {
                !plan_pme
                    .iter()
                    .any(|f| f.parent_number.as_deref() == Some(&e.number))
            })
            .map(|e| e.number.as_str())
            .collect();
        // Toutes les feuilles portent un solde distinct (non nul).
        let mut actifs = vec![];
        let mut dettes = vec![];
        let mut fonds = vec![];
        let mut produits = vec![];
        let mut charges = vec![];
        for (i, n) in feuilles.iter().enumerate() {
            let ligne = l(n, Decimal::from(i as i64 + 1));
            match type_de(n) {
                AccountType::Asset => actifs.push(ligne),
                AccountType::Liability if est_fonds_propres(n) => fonds.push(ligne),
                AccountType::Liability => dettes.push(ligne),
                AccountType::Revenue => produits.push(ligne),
                AccountType::Expense => charges.push(ligne),
            }
        }
        let numeros_groupes = |s: &SectionGroupee| -> BTreeSet<String> {
            s.lignes
                .iter()
                .filter(|g| g.genre == GenreLigne::Groupe)
                .map(|g| g.numero.clone())
                .collect()
        };
        let niveaux: Vec<NiveauDetail> = (1..=NIVEAU_MAX)
            .map(NiveauDetail::Niveau)
            .chain([NiveauDetail::Tous])
            .collect();
        for niveau in niveaux {
            let o = opts(niveau);
            let bilan = regrouper_sections(&[&actifs[..], &dettes[..], &fonds[..]], &p.arbre, &o);
            let resultat = regrouper_sections(&[&produits[..], &charges[..]], &p.arbre, &o);
            for sections in [&bilan, &resultat] {
                // Aucun groupe dans deux sections.
                let mut vus = BTreeSet::new();
                for s in sections.iter() {
                    for g in numeros_groupes(s) {
                        assert!(vus.insert(g.clone()), "{g} dans deux sections ({niveau:?})");
                    }
                }
            }
            // Totaux de section conservés.
            for (s, entree) in bilan.iter().zip([&actifs, &dettes, &fonds]) {
                let total: Decimal = entree.iter().map(|x| x.m[0]).sum();
                assert_eq!(somme_racines(s, 0), total, "{niveau:?}");
            }
            for (s, entree) in resultat.iter().zip([&produits, &charges]) {
                let total: Decimal = entree.iter().map(|x| x.m[0]).sum();
                assert_eq!(somme_racines(s, 0), total, "{niveau:?}");
            }
            // Profondeur 1 : `1` aux actifs, `20` et `24` aux dettes, `28` aux
            // fonds propres (`2 Passifs` dissous).
            let racines = |s: &SectionGroupee| -> Vec<String> {
                s.lignes
                    .iter()
                    .filter(|g| g.profondeur == 1)
                    .map(|g| g.numero.clone())
                    .collect()
            };
            assert_eq!(racines(&bilan[0]), ["1"]);
            assert_eq!(racines(&bilan[1]), ["20", "24"]);
            assert_eq!(racines(&bilan[2]), ["28"]);
            // Compte de résultat : `6`, `7`, `70`, `8`, `80` dissous.
            if niveau == NiveauDetail::Tous {
                let tous: BTreeSet<String> = resultat.iter().flat_map(numeros_groupes).collect();
                for dissous in ["2", "6", "7", "70", "8", "80"] {
                    assert!(!tous.contains(dissous), "{dissous} devrait être dissous");
                }
                for garde in ["3", "30", "4", "40", "5", "50", "60", "9", "90"] {
                    assert!(tous.contains(garde), "{garde} devrait être affiché");
                }
                // `6950 Produits financiers` remonte en racine des produits ;
                // `60` en racine des charges.
                assert!(racines(&resultat[0]).contains(&"6950".to_string()));
                assert!(racines(&resultat[1]).contains(&"60".to_string()));
            }
        }
    }

    // ---- Propriété (AC-2) ----------------------------------------------

    /// Générateur congruentiel, graine fixe : aucune dépendance neuve.
    struct Lcg(u64);
    impl Lcg {
        fn suivant(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0 >> 33
        }
        fn sous(&mut self, n: u64) -> u64 {
            self.suivant() % n
        }
    }

    /// Arbres et soldes tirés : à tout niveau et pour des états d'ouverture
    /// tirés, la somme des lignes de profondeur 1 (sous-totaux exclus) égale le
    /// total de chaque section ; chaque ligne d'entrée est émise au plus une
    /// fois. Une fois sur deux, une seconde colonne de poids distincts (jamais
    /// nulle) rend tout double comptage ou oubli visible dans la somme ; une
    /// colonne seule, souvent compensée, exerce les groupes nuls masqués.
    #[test]
    fn propriete_sous_totaux_exacts_sur_arbres_tires() {
        let mut r = Lcg(0x46_01);
        for _ in 0..300 {
            let n = 2 + r.sous(40) as i64;
            let noeuds: Vec<NoeudCompte> = (1..=n)
                .map(|id| NoeudCompte {
                    id,
                    company_id: 1,
                    numero: format!("{:04}", r.sous(10_000)),
                    nom: String::new(),
                    // Parent tiré parmi les précédents, ou racine ; parfois un
                    // cycle ou un parent inconnu.
                    parent_id: match r.sous(10) {
                        0 => None,
                        1 => Some(n + 5),
                        2 => Some(1 + r.sous(n as u64) as i64),
                        _ if id > 1 => Some(1 + r.sous(id as u64 - 1) as i64),
                        _ => None,
                    },
                })
                .collect();
            let arbre = ArbreComptes::depuis_noeuds(noeuds);
            let deux_colonnes = r.sous(2) == 0;
            let largeur = if deux_colonnes { 2 } else { 1 };
            let mut sections: Vec<Vec<L>> = vec![vec![], vec![]];
            for id in 1..=n {
                if r.sous(3) == 0 {
                    continue;
                }
                // Colonne 0 : petites valeurs, souvent compensées ; colonne 1 :
                // poids distincts, pour détecter un double comptage.
                let m0 = Decimal::from(r.sous(7) as i64 - 3);
                let m1 = Decimal::from(1i64 << (id % 40)) + Decimal::from(id);
                let devise = if r.sous(4) == 0 {
                    Some("USD".to_string())
                } else {
                    None
                };
                let m = if deux_colonnes {
                    vec![m0, m1]
                } else {
                    vec![m0]
                };
                let md = devise.as_ref().map(|_| m.clone());
                sections[r.sous(2) as usize].push(L {
                    id,
                    numero: arbre.noeud(id).unwrap().numero.clone(),
                    devise,
                    m,
                    md,
                });
            }
            let refs: Vec<&[L]> = sections.iter().map(|s| &s[..]).collect();
            let dissous = Dissolution::calculer(
                &arbre,
                refs.iter()
                    .enumerate()
                    .flat_map(|(k, s)| s.iter().map(move |x| (x.id, k))),
            );
            for _ in 0..6 {
                let niveau = match r.sous(5) {
                    0 => NiveauDetail::Tous,
                    v => NiveauDetail::Niveau(v as u8),
                };
                let mut o = opts(niveau);
                o.position = if r.sous(2) == 0 {
                    PositionSousTotal::Titre
                } else {
                    PositionSousTotal::Pied
                };
                for id in 1..=n {
                    match r.sous(6) {
                        0 => {
                            o.ouverts.insert(id);
                        }
                        1 => {
                            o.fermes.insert(id);
                        }
                        _ => {}
                    }
                }
                for s in &refs {
                    let g = regrouper(s, &arbre, &dissous, &o);
                    for col in 0..largeur {
                        let total: Decimal = s.iter().map(|x| x.m[col]).sum();
                        assert_eq!(somme_racines(&g, col), total, "colonne {col}");
                    }
                    let mut emises = BTreeSet::new();
                    for x in g.lignes.iter().filter_map(|x| x.source) {
                        assert!(emises.insert(x), "ligne d'entrée émise deux fois");
                    }
                    // Chaque sous-total est la somme exacte de sa descendance.
                    for x in g.lignes.iter().filter(|x| x.genre == GenreLigne::SousTotal) {
                        let titre = g
                            .lignes
                            .iter()
                            .find(|y| y.genre == GenreLigne::Groupe && y.compte_id == x.compte_id)
                            .unwrap();
                        assert_eq!(titre.montants, x.montants);
                        assert_eq!(titre.ouvert, Some(true));
                    }
                }
            }
        }
    }

    // ---- Enveloppe -----------------------------------------------------

    #[test]
    fn enveloppe_sans_groupes_serialise_le_rapport_seul() {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct R {
            a_b: u8,
            total: Decimal,
        }
        let seul = serde_json::to_string(&R {
            a_b: 1,
            total: dec!(1.50),
        })
        .unwrap();
        let env: AvecGroupes<R, SectionsResultat> = AvecGroupes::sans_groupes(R {
            a_b: 1,
            total: dec!(1.50),
        });
        assert_eq!(serde_json::to_string(&env).unwrap(), seul);
    }

    #[test]
    fn ligne_rapport_et_grouping_au_contrat_json() {
        let grouping = Grouping {
            detail_level: NiveauDetail::Tous,
            subtotal_position: PositionSousTotal::Pied,
            max_depth: 3,
            open: vec![1],
            closed: vec![],
        };
        assert_eq!(
            serde_json::to_value(&grouping).unwrap(),
            serde_json::json!({"detailLevel": "all", "subtotalPosition": "footer",
                "maxDepth": 3, "open": [1], "closed": []})
        );
        let g2 = Grouping {
            detail_level: NiveauDetail::Niveau(2),
            subtotal_position: PositionSousTotal::Titre,
            ..grouping
        };
        assert_eq!(serde_json::to_value(&g2).unwrap()["detailLevel"], 2);
        assert_eq!(
            serde_json::to_value(&g2).unwrap()["subtotalPosition"],
            "header"
        );
        let ligne = LigneRapport {
            kind: GenreLigne::Groupe,
            depth: 1,
            account_id: 7,
            account_number: "1".into(),
            account_name: "Actifs".into(),
            balance: dec!(12.30),
            currency: None,
            balance_currency: None,
            open: Some(false),
            active: None,
            role: None,
        };
        assert_eq!(
            serde_json::to_value(&ligne).unwrap(),
            serde_json::json!({"kind": "group", "depth": 1, "accountId": 7,
                "accountNumber": "1", "accountName": "Actifs", "balance": "12.30", "open": false})
        );
        let compte = LigneRapport {
            kind: GenreLigne::Compte,
            open: None,
            active: Some(true),
            role: Some(None),
            ..ligne
        };
        let v = serde_json::to_value(&compte).unwrap();
        assert_eq!(
            (v["kind"].as_str(), v["active"].as_bool()),
            (Some("account"), Some(true))
        );
        assert!(v["role"].is_null() && v.as_object().unwrap().contains_key("role"));
    }
}
