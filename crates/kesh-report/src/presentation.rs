//! Présentation **miroir** du bilan et du compte de résultat (Story 47-3).
//!
//! Un compte peut désigner un *compte miroir*, de l'autre côté du même rapport
//! (`accounts.mirror_account_id`, Story 47-1). Quand le solde du compte est **de
//! sens inverse** à sa nature (banque à découvert, dette à solde débiteur…), la
//! présentation l'en retire et le montre, positif, sous son miroir (CO art. 959a) :
//! aucune écriture, aucune compensation — la balance garde les soldes réels.
//!
//! Les fonctions sont **pures** (sans I/O) et se branchent **après** les
//! générateurs, qui restent bruts : `balance_sheet::generate` et
//! `income_statement::generate` ne les appellent pas. Elles ne lisent ni
//! n'écrivent jamais `equity` ni les lignes calculées du bilan.

use std::collections::{BTreeMap, HashMap};

use kesh_db::entities::{AccountType, MirrorLink};
use rust_decimal::Decimal;

use crate::balance_sheet::{AccountBalance, BalanceSheet, MirrorSource};
use crate::income_statement::IncomeStatement;

/// Une section modifiable : le type de ses comptes et ses lignes.
type Section = (AccountType, Vec<AccountBalance>);

/// Les deux sections d'un rapport, après reclassement.
type Sections = (Vec<AccountBalance>, Vec<AccountBalance>);

/// Ce qu'un miroir reçoit : sa somme et ses sources.
struct Reçu<'a> {
    link: &'a MirrorLink,
    somme: Decimal,
    sources: Vec<MirrorSource>,
}

/// Reclasse les soldes de sens inverse des deux sections. `None` si aucune
/// source n'est de sens inverse : le rapport reste alors **tel quel**.
fn reclasser(a: Section, b: Section, links: &[MirrorLink]) -> Option<Sections> {
    let par_source: HashMap<i64, &MirrorLink> = links.iter().map(|l| (l.source_id, l)).collect();
    let (type_a, type_b) = (a.0, b.0);
    let mut recus: BTreeMap<i64, Reçu> = BTreeMap::new();
    let mut restes = [Vec::new(), Vec::new()];
    for (i, (section_type, lignes)) in [a, b].into_iter().enumerate() {
        for ligne in lignes {
            let lien = par_source
                .get(&ligne.account_id)
                .filter(|l| ligne.balance < Decimal::ZERO && l.mirror_account_type != section_type);
            match lien {
                // Un miroir d'un type étranger aux deux sections reste sans effet.
                Some(l) if [type_a, type_b].contains(&l.mirror_account_type) => {
                    let reçu = recus.entry(l.mirror_id).or_insert_with(|| Reçu {
                        link: l,
                        somme: Decimal::ZERO,
                        sources: Vec::new(),
                    });
                    reçu.somme -= ligne.balance;
                    reçu.sources.push(MirrorSource {
                        account_id: ligne.account_id,
                        account_number: ligne.account_number,
                        account_name: ligne.account_name,
                        balance: ligne.balance,
                    });
                }
                _ => restes[i].push(ligne),
            }
        }
    }
    let [mut reste_a, mut reste_b] = restes;
    if recus.is_empty() {
        return None;
    }
    for (mirror_id, mut reçu) in recus {
        let l = reçu.link;
        let cible = if l.mirror_account_type == type_a {
            &mut reste_a
        } else {
            &mut reste_b
        };
        // État forgé seulement : le miroir a une ligne brute, son solde s'ajoute.
        if let Some(pos) = cible.iter().position(|x| x.account_id == mirror_id) {
            reçu.somme += cible.remove(pos).balance;
        }
        if reçu.somme.is_zero() {
            continue;
        }
        reçu
            .sources
            .sort_by(|x, y| x.account_number.cmp(&y.account_number));
        cible.push(AccountBalance {
            account_id: mirror_id,
            account_number: l.mirror_number.clone(),
            account_name: l.mirror_name.clone(),
            account_type: l.mirror_account_type,
            active: l.mirror_active,
            balance: reçu.somme,
            role: None,
            currency: None,
            balance_currency: None,
            mirror_of: reçu.sources,
        });
    }
    for s in [&mut reste_a, &mut reste_b] {
        s.sort_by(|x, y| x.account_number.cmp(&y.account_number));
    }
    Some((reste_a, reste_b))
}

fn somme(lignes: &[AccountBalance]) -> Decimal {
    lignes.iter().map(|l| l.balance).sum()
}

/// L'invariant du bilan que la présentation aurait rompu — défaut de code, que
/// `present_balance_sheet` trace avant de rendre le bilan brut.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DefautBilan {
    /// L'écart actif − passif diffère du brut (une source retirée d'un côté n'a
    /// pas été rendue, du même montant, de l'autre).
    Ecart,
    /// L'écart est conservé, mais l'équation présentée ne dit pas la même chose que
    /// celle du brut.
    Equation,
}

impl DefautBilan {
    /// La phrase du journal, propre à l'invariant rompu.
    fn message(self) -> &'static str {
        match self {
            Self::Ecart => {
                "présentation miroir : l'écart actif − passif diffère du brut, bilan brut rendu"
            }
            Self::Equation => {
                "présentation miroir : l'équation du bilan diffère du brut, bilan brut rendu"
            }
        }
    }
}

/// Contrôle les totaux présentés contre le bilan brut : rend l'équation présentée
/// et, s'il y en a un, l'invariant rompu. L'écart actif − passif est invariant par
/// construction (une source −x retirée d'un côté, +x ajoutée de l'autre), même
/// quand l'équation brute est fausse ; il se contrôle le premier.
fn controler_bilan(
    raw: &BalanceSheet,
    total_assets: Decimal,
    total_liabilities: Decimal,
) -> (bool, Option<DefautBilan>) {
    let equation_holds = total_assets
        == total_liabilities + raw.total_equity + raw.retained_earnings + raw.equity_result;
    let defaut = if total_assets - total_liabilities != raw.total_assets - raw.total_liabilities {
        Some(DefautBilan::Ecart)
    } else if equation_holds != raw.equation_holds {
        Some(DefautBilan::Equation)
    } else {
        None
    };
    (equation_holds, defaut)
}

/// Le bilan présenté : les soldes de sens inverse de l'actif et du passif
/// paraissent sous leur miroir. Rendu **brut** si aucun lien ne s'applique, ou
/// si les invariants (équation, résultat) ne tenaient plus — défaut de code,
/// tracé, jamais une panique.
pub fn present_balance_sheet(raw: BalanceSheet, links: &[MirrorLink]) -> BalanceSheet {
    if links.is_empty() {
        return raw;
    }
    let (assets, liabilities) = match reclasser(
        (AccountType::Asset, raw.assets.clone()),
        (AccountType::Liability, raw.liabilities.clone()),
        links,
    ) {
        Some(sections) => sections,
        None => return raw,
    };
    let total_assets = somme(&assets);
    let total_liabilities = somme(&liabilities);
    let (equation_holds, defaut) = controler_bilan(&raw, total_assets, total_liabilities);
    if let Some(defaut) = defaut {
        tracing::error!(
            equation_brute = raw.equation_holds,
            equation_presentee = equation_holds,
            ecart_brut = %(raw.total_assets - raw.total_liabilities),
            ecart_presente = %(total_assets - total_liabilities),
            "{}",
            defaut.message()
        );
        return raw;
    }
    BalanceSheet {
        assets,
        liabilities,
        total_assets,
        total_liabilities,
        equation_holds,
        ..raw
    }
}

/// Le compte de résultat présenté : même règle sur les produits et les charges.
/// Rendu brut si le résultat net recalculé diffère du brut (défaut de code).
pub fn present_income_statement(raw: IncomeStatement, links: &[MirrorLink]) -> IncomeStatement {
    if links.is_empty() {
        return raw;
    }
    let (revenues, expenses) = match reclasser(
        (AccountType::Revenue, raw.revenues.clone()),
        (AccountType::Expense, raw.expenses.clone()),
        links,
    ) {
        Some(sections) => sections,
        None => return raw,
    };
    let total_revenues = somme(&revenues);
    let total_expenses = somme(&expenses);
    let net_result = total_revenues - total_expenses;
    if net_result != raw.net_result {
        tracing::error!(
            net_brut = %raw.net_result,
            net_presente = %net_result,
            "présentation miroir : le résultat net diffère du brut, compte de résultat brut rendu"
        );
        return raw;
    }
    IncomeStatement {
        revenues,
        expenses,
        total_revenues,
        total_expenses,
        net_result,
        ..raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::period::ReportPeriod;
    use AccountType::{Asset, Expense, Liability, Revenue};
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    fn period() -> ReportPeriod {
        ReportPeriod {
            fiscal_year_id: 1,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        }
    }

    fn ligne(id: i64, number: &str, t: AccountType, balance: Decimal) -> AccountBalance {
        AccountBalance {
            account_id: id,
            account_number: number.into(),
            account_name: format!("Compte {number}"),
            account_type: t,
            active: true,
            balance,
            role: None,
            currency: Some("EUR".into()),
            balance_currency: Some(dec!(1)),
            mirror_of: Vec::new(),
        }
    }

    fn lien(source: i64, mirror: i64, number: &str, t: AccountType) -> MirrorLink {
        MirrorLink {
            source_id: source,
            mirror_id: mirror,
            mirror_number: number.into(),
            mirror_name: format!("Miroir {number}"),
            mirror_account_type: t,
            mirror_active: true,
        }
    }

    fn bilan(assets: Vec<AccountBalance>, liabilities: Vec<AccountBalance>) -> BalanceSheet {
        let total_assets = somme(&assets);
        let total_liabilities = somme(&liabilities);
        BalanceSheet {
            period: period(),
            assets,
            liabilities,
            equity: vec![],
            total_assets,
            total_liabilities,
            total_equity: Decimal::ZERO,
            retained_earnings: Decimal::ZERO,
            // Le résultat de l'exercice porte l'équation (fonds propres vides).
            equity_result: total_assets - total_liabilities,
            equation_holds: true,
        }
    }

    fn resultat(rev: Vec<AccountBalance>, exp: Vec<AccountBalance>) -> IncomeStatement {
        let (tr, te) = (somme(&rev), somme(&exp));
        IncomeStatement {
            period: period(),
            revenues: rev,
            expenses: exp,
            total_revenues: tr,
            total_expenses: te,
            net_result: tr - te,
        }
    }

    /// Le cas de l'exemple : une banque à découvert paraît au passif.
    #[test]
    fn banque_a_decouvert_passe_au_passif() {
        let raw = bilan(
            vec![
                ligne(1, "1020", Asset, dec!(-300)),
                ligne(2, "1100", Asset, dec!(1000)),
            ],
            vec![ligne(3, "2000", Liability, dec!(200))],
        );
        let p = present_balance_sheet(raw.clone(), &[lien(1, 9, "1020.1", Liability)]);
        assert_eq!(p.assets.len(), 1);
        assert_eq!(p.liabilities.len(), 2);
        let m = p.liabilities.iter().find(|l| l.account_id == 9).unwrap();
        assert_eq!(m.balance, dec!(300));
        assert_eq!(m.account_number, "1020.1");
        assert_eq!(m.account_type, Liability);
        assert!(m.role.is_none() && m.currency.is_none() && m.balance_currency.is_none());
        assert_eq!(m.mirror_of.len(), 1);
        assert_eq!(m.mirror_of[0].balance, dec!(-300));
        assert_eq!(p.total_assets, dec!(1000));
        assert_eq!(p.total_liabilities, dec!(500));
        assert!(p.equation_holds);
        assert_eq!(p.equity_result, raw.equity_result);
        // Triées par numéro.
        assert!(
            p.liabilities
                .windows(2)
                .all(|w| w[0].account_number <= w[1].account_number)
        );
    }

    /// Plusieurs sources vers un miroir, dont une de sens normal (jamais compensée).
    #[test]
    fn plusieurs_sources_dont_une_de_sens_normal() {
        let raw = bilan(
            vec![
                ligne(1, "1020", Asset, dec!(-300)),
                ligne(2, "1021", Asset, dec!(-50)),
                ligne(4, "1022", Asset, dec!(700)),
            ],
            vec![],
        );
        let links = [
            lien(1, 9, "2100.1", Liability),
            lien(2, 9, "2100.1", Liability),
            lien(4, 9, "2100.1", Liability),
        ];
        let p = present_balance_sheet(raw, &links);
        assert_eq!(p.assets.len(), 1);
        assert_eq!(p.assets[0].account_id, 4);
        assert_eq!(p.liabilities[0].balance, dec!(350));
        let nums: Vec<_> = p.liabilities[0]
            .mirror_of
            .iter()
            .map(|s| s.account_number.as_str())
            .collect();
        assert_eq!(nums, ["1020", "1021"]);
    }

    /// Source de sens normal, nulle en CHF (devise seulement), ou sans lien : inchangée ;
    /// le rapport est égal octet pour octet en JSON.
    #[test]
    fn rien_a_presenter_rend_le_rapport_tel_quel() {
        let mut fx = ligne(1, "1020", Asset, Decimal::ZERO);
        fx.balance_currency = Some(dec!(-5));
        let raw = bilan(vec![fx, ligne(2, "1100", Asset, dec!(50))], vec![]);
        let links = [
            lien(1, 9, "2100.1", Liability),
            lien(2, 9, "2100.1", Liability),
        ];
        let p = present_balance_sheet(raw.clone(), &links);
        let j = |b: &BalanceSheet| serde_json::to_string(b).unwrap();
        assert_eq!(j(&p), j(&raw));
        assert_eq!(j(&present_balance_sheet(raw.clone(), &[])), j(&raw));
        assert!(!j(&p).contains("mirrorOf"));
    }

    /// Compte de résultat : un produit à solde débiteur paraît en charge.
    #[test]
    fn resultat_net_inchange() {
        let raw = resultat(
            vec![
                ligne(1, "6950", Revenue, dec!(-250)),
                ligne(2, "3000", Revenue, dec!(2000)),
            ],
            vec![ligne(3, "6000", Expense, dec!(50))],
        );
        let p = present_income_statement(raw.clone(), &[lien(1, 8, "6900.1", Expense)]);
        assert_eq!(p.total_revenues, dec!(2000));
        assert_eq!(p.total_expenses, dec!(300));
        assert_eq!(p.net_result, raw.net_result);
        assert_eq!(p.expenses.len(), 2);
    }

    /// État forgé : le miroir a une ligne brute, son solde s'ajoute.
    #[test]
    fn miroir_a_une_ligne_brute() {
        let raw = bilan(
            vec![ligne(1, "1020", Asset, dec!(-300))],
            vec![ligne(9, "2100.1", Liability, dec!(40))],
        );
        let p = present_balance_sheet(raw, &[lien(1, 9, "2100.1", Liability)]);
        assert_eq!(p.liabilities.len(), 1);
        assert_eq!(p.liabilities[0].balance, dec!(340));
        assert!(p.equation_holds);
    }

    /// Une équation fausse en brut reste fausse, et égale, en présenté.
    #[test]
    fn equation_fausse_du_brut_conservee() {
        let mut raw = bilan(vec![ligne(1, "1020", Asset, dec!(-300))], vec![]);
        raw.equity_result += dec!(1);
        raw.equation_holds = false;
        let p = present_balance_sheet(raw, &[lien(1, 9, "2100.1", Liability)]);
        assert!(!p.equation_holds);
        assert_eq!(p.liabilities.len(), 1);
    }

    /// Branches de `reclasser` : ligne de miroir de somme nulle omise (état forgé),
    /// miroir d'un type étranger aux deux sections sans effet, sources triées par numéro.
    #[test]
    fn branches_somme_nulle_type_etranger_et_tri() {
        // Somme nulle : la ligne brute du miroir (−300) annule la source (−300 → +300).
        let raw = bilan(
            vec![ligne(1, "1020", Asset, dec!(-300))],
            vec![ligne(9, "2100.1", Liability, dec!(-300))],
        );
        let p = present_balance_sheet(raw, &[lien(1, 9, "2100.1", Liability)]);
        assert!(p.assets.is_empty() && p.liabilities.is_empty());
        assert!(p.equation_holds);

        // Type étranger : le rapport reste tel quel.
        let raw = bilan(vec![ligne(1, "1020", Asset, dec!(-300))], vec![]);
        let p = present_balance_sheet(raw.clone(), &[lien(1, 9, "3000.1", Revenue)]);
        assert_eq!(
            serde_json::to_string(&p).unwrap(),
            serde_json::to_string(&raw).unwrap()
        );

        // Tri : les sources arrivent dans l'ordre inverse des numéros.
        let raw = bilan(
            vec![
                ligne(1, "1021", Asset, dec!(-50)),
                ligne(2, "1020", Asset, dec!(-300)),
            ],
            vec![],
        );
        let links = [
            lien(1, 9, "2100.1", Liability),
            lien(2, 9, "2100.1", Liability),
        ];
        let p = reclasser((Asset, raw.assets), (Liability, raw.liabilities), &links).unwrap();
        let nums: Vec<_> = p.1[0]
            .mirror_of
            .iter()
            .map(|s| s.account_number.as_str())
            .collect();
        assert_eq!(nums, ["1020", "1021"]);
    }

    /// Le contrôle des totaux présentés (`controler_bilan`) : l'écart rompu se
    /// signale le premier et sous son nom ; l'équation, quand seul son verdict
    /// diffère ; rien quand tout tient. Les deux défauts ont chacun leur phrase.
    #[test]
    fn controle_du_bilan_nomme_l_invariant_rompu() {
        // Brut : actif 1000, passif 500, résultat 500, équation vraie.
        let raw = bilan(
            vec![ligne(1, "1100", Asset, dec!(1000))],
            vec![ligne(2, "2000", Liability, dec!(500))],
        );
        // Présenté juste : +300 des deux côtés.
        assert_eq!(controler_bilan(&raw, dec!(1300), dec!(800)), (true, None));
        // Écart rompu : 300 ajoutés à l'actif seulement.
        assert_eq!(
            controler_bilan(&raw, dec!(1300), dec!(500)),
            (false, Some(DefautBilan::Ecart))
        );
        // Écart rompu dans l'autre sens : présenté inférieur au brut (400 pour 500).
        assert_eq!(
            controler_bilan(&raw, dec!(1200), dec!(800)),
            (false, Some(DefautBilan::Ecart))
        );
        // Écart conservé, mais l'équation brute déclarée fausse ne l'est plus en présenté.
        let mut faux = raw.clone();
        faux.equation_holds = false;
        assert_eq!(
            controler_bilan(&faux, dec!(1300), dec!(800)),
            (true, Some(DefautBilan::Equation))
        );
        // Équation brute réellement fausse : conservée fausse, sans défaut.
        let mut desequilibre = raw.clone();
        desequilibre.equity_result += dec!(1);
        desequilibre.equation_holds = false;
        assert_eq!(
            controler_bilan(&desequilibre, dec!(1300), dec!(800)),
            (false, None)
        );
        assert!(
            DefautBilan::Ecart
                .message()
                .contains("l'écart actif − passif")
        );
        assert!(
            DefautBilan::Equation
                .message()
                .contains("l'équation du bilan")
        );
    }

    /// Revue P1 de la 47-5, L4 : le repli de `present_balance_sheet` lui-même. Un
    /// bilan brut dont un invariant est forgé (le reclassement, lui, est juste)
    /// est rendu **tel quel**, sections brutes comprises, quel que soit le sens de
    /// l'écart, plutôt que présenté.
    #[test]
    fn invariant_rompu_rend_le_bilan_brut() {
        let brut = bilan(
            vec![
                ligne(1, "1020", Asset, dec!(-300)),
                ligne(2, "1100", Asset, dec!(1000)),
            ],
            vec![ligne(3, "2000", Liability, dec!(200))],
        );
        let links = [lien(1, 9, "1020.1", Liability)];
        let j = |b: &BalanceSheet| serde_json::to_string(b).unwrap();
        // Sanité : sans défaut, le même bilan se présente (le miroir paraît).
        assert_eq!(
            present_balance_sheet(brut.clone(), &links)
                .liabilities
                .len(),
            2
        );
        // Équation brute déclarée fausse alors qu'elle tient en présenté.
        let mut equation = brut.clone();
        equation.equation_holds = false;
        // Écart brut forgé, supérieur puis inférieur à l'écart présenté.
        let mut grand = brut.clone();
        grand.total_assets += dec!(5);
        let mut petit = brut.clone();
        petit.total_assets -= dec!(5);
        for forge in [equation, grand, petit] {
            let p = present_balance_sheet(forge.clone(), &links);
            assert_eq!(j(&p), j(&forge));
            assert_eq!(p.liabilities.len(), 1);
            assert!(p.liabilities.iter().all(|l| l.mirror_of.is_empty()));
        }
    }

    /// Un lien vers un miroir **du même type** que sa source (état que l'API refuse,
    /// forgé ici) reste sans effet : la source garde sa place, aucun reclassement.
    #[test]
    fn miroir_du_meme_type_que_la_source_sans_effet() {
        let raw = bilan(
            vec![
                ligne(1, "1020", Asset, dec!(-300)),
                ligne(2, "1100", Asset, dec!(1000)),
            ],
            vec![],
        );
        let links = [lien(1, 9, "1020.1", Asset)];
        assert!(
            reclasser(
                (Asset, raw.assets.clone()),
                (Liability, raw.liabilities.clone()),
                &links
            )
            .is_none()
        );
        let p = present_balance_sheet(raw.clone(), &links);
        assert_eq!(
            serde_json::to_string(&p).unwrap(),
            serde_json::to_string(&raw).unwrap()
        );
    }

    /// Générateur déterministe (graine fixe), sans dépendance.
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self, n: u64) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (self.0 >> 33) % n
        }
        fn montant(&mut self) -> Decimal {
            Decimal::new(self.next(200_001) as i64 - 100_000, 2)
        }
    }

    /// Propriété : équation du bilan et résultat net inchangés, fonds propres et
    /// lignes calculées identiques, chemin court sans lien applicable.
    #[test]
    fn propriete_equation_et_resultat_net_inchanges() {
        let mut g = Lcg(0x4B45_5348);
        for tour in 0..500 {
            // Comptes 1..=n ; n_a actifs, n_p passifs ; liens aléatoires valides.
            let (n_a, n_p) = (1 + g.next(5) as i64, 1 + g.next(5) as i64);
            let mut assets = Vec::new();
            let mut liabilities = Vec::new();
            let mut links = Vec::new();
            let miroir_passif = 900;
            let miroir_actif = 901;
            for i in 0..n_a {
                let b = g.montant();
                if !b.is_zero() {
                    assets.push(ligne(i + 1, &format!("1{:03}", i), Asset, b));
                }
                if g.next(3) > 0 {
                    links.push(lien(i + 1, miroir_passif, "2999", Liability));
                }
            }
            for i in 0..n_p {
                let b = g.montant();
                if !b.is_zero() {
                    liabilities.push(ligne(100 + i, &format!("2{:03}", i), Liability, b));
                }
                if g.next(3) > 0 {
                    links.push(lien(100 + i, miroir_actif, "1999", Asset));
                }
            }
            let raw = bilan(assets, liabilities);
            // Les invariants se jugent sur la sortie de `reclasser` elle-même : le repli
            // sur le brut de `present_*` les satisferait toujours.
            let reclasse = reclasser(
                (Asset, raw.assets.clone()),
                (Liability, raw.liabilities.clone()),
                &links,
            );
            let p = present_balance_sheet(raw.clone(), &links);
            if let Some((a, l)) = &reclasse {
                assert_eq!(
                    somme(a) - somme(l),
                    raw.total_assets - raw.total_liabilities,
                    "reclasser : écart actif − passif, tour {tour}"
                );
                assert_eq!(
                    somme(a),
                    somme(l) + raw.total_equity + raw.retained_earnings + raw.equity_result,
                    "reclasser : équation, tour {tour}"
                );
                // La présentation a bien été appliquée, non remplacée par le brut.
                assert_eq!(
                    serde_json::to_string(&p.assets).unwrap(),
                    serde_json::to_string(a).unwrap()
                );
                assert_eq!(
                    serde_json::to_string(&p.liabilities).unwrap(),
                    serde_json::to_string(l).unwrap()
                );
                assert!(
                    p.assets
                        .iter()
                        .chain(&p.liabilities)
                        .any(|x| !x.mirror_of.is_empty()),
                    "tour {tour}"
                );
            }
            assert!(p.equation_holds, "tour {tour}");
            assert_eq!(p.equation_holds, raw.equation_holds);
            assert_eq!(p.equity_result, raw.equity_result);
            assert_eq!(p.retained_earnings, raw.retained_earnings);
            assert_eq!(p.total_equity, raw.total_equity);
            assert_eq!(
                p.total_assets - p.total_liabilities,
                raw.total_assets - raw.total_liabilities,
                "tour {tour}"
            );
            assert_eq!(p.total_assets, somme(&p.assets));
            assert_eq!(p.total_liabilities, somme(&p.liabilities));
            // Sans ligne de miroir, la présentation égale le brut, octet pour octet.
            if !p
                .assets
                .iter()
                .chain(&p.liabilities)
                .any(|l| !l.mirror_of.is_empty())
            {
                assert_eq!(
                    serde_json::to_string(&p).unwrap(),
                    serde_json::to_string(&raw).unwrap()
                );
            }

            let rev: Vec<_> = (0..n_a)
                .map(|i| ligne(i + 1, &format!("3{:03}", i), Revenue, g.montant()))
                .filter(|l| !l.balance.is_zero())
                .collect();
            let exp: Vec<_> = (0..n_p)
                .map(|i| ligne(100 + i, &format!("6{:03}", i), Expense, g.montant()))
                .filter(|l| !l.balance.is_zero())
                .collect();
            let mut liens_r = Vec::new();
            for l in &rev {
                if g.next(2) == 0 {
                    liens_r.push(lien(l.account_id, 800, "6999", Expense));
                }
            }
            for l in &exp {
                if g.next(2) == 0 {
                    liens_r.push(lien(l.account_id, 801, "3999", Revenue));
                }
            }
            let raw = resultat(rev, exp);
            let reclasse = reclasser(
                (Revenue, raw.revenues.clone()),
                (Expense, raw.expenses.clone()),
                &liens_r,
            );
            let p = present_income_statement(raw.clone(), &liens_r);
            if let Some((r, e)) = &reclasse {
                assert_eq!(
                    somme(r) - somme(e),
                    raw.net_result,
                    "reclasser : net, tour {tour}"
                );
                assert_eq!(
                    serde_json::to_string(&p.revenues).unwrap(),
                    serde_json::to_string(r).unwrap()
                );
                assert_eq!(
                    serde_json::to_string(&p.expenses).unwrap(),
                    serde_json::to_string(e).unwrap()
                );
            }
            assert_eq!(p.net_result, raw.net_result, "tour {tour}");
            assert_eq!(p.total_revenues, somme(&p.revenues));
            assert_eq!(p.total_expenses, somme(&p.expenses));
        }
    }
}
