//! Jeu d'essai des rendus **par groupes** (Story 46-3), partagé par les tests
//! du PDF et du CSV — compilé sous `cfg(test)` seulement.

use std::collections::BTreeSet;

use chrono::NaiveDate;
use kesh_db::entities::AccountType;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::balance_sheet::{AccountBalance, BalanceSheet};
use crate::groupes::{
    ArbreComptes, AvecGroupes, NiveauDetail, NoeudCompte, OptionsGroupes, PositionSousTotal,
    SectionsBilan,
};
use crate::income_statement::IncomeStatement;
use crate::period::ReportPeriod;

/// Un compte : numéro = identifiant.
pub fn ab(id: i64, nom: &str, balance: Decimal, ty: AccountType) -> AccountBalance {
    AccountBalance {
        account_id: id,
        account_number: id.to_string(),
        account_name: nom.into(),
        account_type: ty,
        active: true,
        balance,
        role: None,
        currency: None,
        balance_currency: None,
    }
}

/// Un compte en dollars : `chf` en devise principale, `usd` en devise.
pub fn usd(id: i64, nom: &str, chf: Decimal, usd: Decimal) -> AccountBalance {
    AccountBalance {
        currency: Some("USD".into()),
        balance_currency: Some(usd),
        ..ab(id, nom, chf, AccountType::Asset)
    }
}

/// Un nœud de l'arbre : numéro = identifiant.
pub fn noeud(id: i64, nom: &str, parent: Option<i64>) -> NoeudCompte {
    NoeudCompte {
        id,
        company_id: 1,
        numero: id.to_string(),
        nom: nom.into(),
        parent_id: parent,
    }
}

/// Un bilan de 2026 : `actifs`, puis une dette du même total, sans fonds
/// propres ni résultat.
pub fn bilan(actifs: Vec<AccountBalance>) -> BalanceSheet {
    let total: Decimal = actifs.iter().map(|a| a.balance).sum();
    BalanceSheet {
        period: ReportPeriod {
            fiscal_year_id: 1,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        },
        assets: actifs,
        liabilities: vec![ab(2000, "Fournisseurs", total, AccountType::Liability)],
        equity: vec![],
        total_assets: total,
        total_liabilities: total,
        total_equity: Decimal::ZERO,
        retained_earnings: Decimal::ZERO,
        equity_result: Decimal::ZERO,
        equation_holds: true,
    }
}

/// L'arbre d'essai : `10 Actifs circulants` ⊃ `100 Liquidités` (qui porte un
/// solde propre) ⊃ `1000 Caisse`, `1020 Banque` ; `14 Immobilisations` ⊃
/// `1500 Machines` ; `2000 Fournisseurs` en racine.
pub fn arbre() -> ArbreComptes {
    ArbreComptes::depuis_noeuds(vec![
        noeud(10, "Actifs circulants", None),
        noeud(100, "Liquidités", Some(10)),
        noeud(1000, "Caisse", Some(100)),
        noeud(1020, "Banque", Some(100)),
        noeud(14, "Immobilisations", None),
        noeud(1500, "Machines", Some(14)),
        noeud(2000, "Fournisseurs", None),
    ])
}

/// Le bilan d'essai : `1000` à 10.005 et `1020` à 5.006 (sous-total exact de
/// `100` : 16.011, AC-6), `100` avec un solde propre de 1, `1500` à 300.
pub fn bilan_essai() -> BalanceSheet {
    bilan(vec![
        ab(100, "Liquidités", dec!(1), AccountType::Asset),
        ab(1000, "Caisse", dec!(10.005), AccountType::Asset),
        ab(1020, "Banque", dec!(5.006), AccountType::Asset),
        ab(1500, "Machines", dec!(300), AccountType::Asset),
    ])
}

/// Des options de groupes.
pub fn options(
    niveau: NiveauDetail,
    position: PositionSousTotal,
    ouverts: &[i64],
    fermes: &[i64],
) -> OptionsGroupes {
    OptionsGroupes {
        niveau,
        position,
        ouverts: ouverts.iter().copied().collect::<BTreeSet<_>>(),
        fermes: fermes.iter().copied().collect::<BTreeSet<_>>(),
    }
}

/// Le bilan d'essai groupé selon `o`.
pub fn bilan_essai_groupe(o: &OptionsGroupes) -> AvecGroupes<BalanceSheet, SectionsBilan> {
    AvecGroupes::bilan(bilan_essai(), &arbre(), o)
}

/// La suite des lignes entre `debut` (exclu) et la première ligne qui commence
/// par `fin` (exclue).
pub fn entre(lignes: &[String], debut: &str, fin: &str) -> Vec<String> {
    let a = lignes
        .iter()
        .position(|l| l == debut)
        .unwrap_or_else(|| panic!("« {debut} » absent de {lignes:?}"))
        + 1;
    let b = a + lignes[a..]
        .iter()
        .position(|l| l.starts_with(fin))
        .unwrap_or_else(|| panic!("« {fin} » absent de {lignes:?}"));
    lignes[a..b].to_vec()
}

/// Un compte de résultat de 2026 : `produits`, `charges`, totaux et résultat
/// calculés.
pub fn resultat(produits: Vec<AccountBalance>, charges: Vec<AccountBalance>) -> IncomeStatement {
    let total_revenues: Decimal = produits.iter().map(|a| a.balance).sum();
    let total_expenses: Decimal = charges.iter().map(|a| a.balance).sum();
    IncomeStatement {
        period: bilan(vec![]).period,
        revenues: produits,
        expenses: charges,
        total_revenues,
        total_expenses,
        net_result: total_revenues - total_expenses,
    }
}

/// Le compte de résultat d'essai : `3000 Ventes` et `3200 Services` sous
/// `3 Produits d'exploitation`, `4000 Achats` en racine ([`arbre_resultat`]).
pub fn resultat_essai() -> IncomeStatement {
    resultat(
        vec![
            ab(3000, "Ventes", dec!(100), AccountType::Revenue),
            ab(3200, "Services", dec!(50), AccountType::Revenue),
        ],
        vec![ab(4000, "Achats", dec!(40), AccountType::Expense)],
    )
}

/// L'arbre du compte de résultat d'essai.
pub fn arbre_resultat() -> ArbreComptes {
    ArbreComptes::depuis_noeuds(vec![
        noeud(3, "Produits d'exploitation", None),
        noeud(3000, "Ventes", Some(3)),
        noeud(3200, "Services", Some(3)),
        noeud(4000, "Achats", None),
    ])
}
