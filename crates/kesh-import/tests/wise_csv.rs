//! Lecteur du CSV « relevé » de Wise sur les fixtures fictives (Story 39-11a,
//! AC-1 à AC-7).
//!
//! `tests/fixtures/wise-releve-{chf,eur,usd}-fictif.csv` sont inventées de
//! toutes pièces (catalogue : `tests/fixtures/README.md`) : 23 colonnes dans
//! l'ordre de Wise, sans BOM, toutes les dates en 2020, la plus récente en tête.

use chrono::NaiveDate;
use kesh_import::wise::{REQUIRED_COLUMNS, parse_wise_csv};
use kesh_import::{WiseCsvError, WiseDirection, WiseLineErrorCode};
use rust_decimal_macros::dec;

const CHF: &[u8] = include_bytes!("fixtures/wise-releve-chf-fictif.csv");
const EUR: &[u8] = include_bytes!("fixtures/wise-releve-eur-fictif.csv");
const USD: &[u8] = include_bytes!("fixtures/wise-releve-usd-fictif.csv");

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Récrit un fichier par le crate `csv` après transformation de chaque
/// enregistrement (en-tête compris).
fn rewrite(src: &[u8], f: impl Fn(usize, Vec<String>) -> Vec<String>) -> Vec<u8> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(src);
    let mut writer = csv::WriterBuilder::new()
        .flexible(true)
        .from_writer(Vec::new());
    for (i, record) in reader.records().enumerate() {
        let cells: Vec<String> = record.unwrap().iter().map(str::to_string).collect();
        writer.write_record(f(i, cells)).unwrap();
    }
    writer.into_inner().unwrap()
}

/// Indice d'une colonne dans l'en-tête de la fixture.
fn col(name: &str) -> usize {
    let header = std::str::from_utf8(CHF).unwrap().lines().next().unwrap();
    header.split(',').position(|c| c == name).unwrap()
}

/// Remplace une cellule de la ligne de données `row` (0 = première ligne de
/// données, soit la plus récente).
fn set(src: &[u8], row: usize, name: &str, value: &str) -> Vec<u8> {
    let c = col(name);
    rewrite(src, |i, mut cells| {
        if i == row + 1 {
            cells[c] = value.to_string();
        }
        cells
    })
}

fn line_codes(err: WiseCsvError) -> Vec<(usize, WiseLineErrorCode)> {
    match err {
        WiseCsvError::Lines(l) => l.into_iter().map(|e| (e.line, e.code)).collect(),
        other => panic!("attendu Lines, reçu {other:?}"),
    }
}

#[test]
fn fixtures_have_the_wise_shape() {
    for f in [CHF, EUR, USD] {
        assert_ne!(&f[..3], b"\xEF\xBB\xBF", "pas de BOM");
        let text = std::str::from_utf8(f).unwrap();
        let header = text.lines().next().unwrap();
        assert_eq!(header.split(',').count(), 23);
        assert!(header.starts_with("TransferWise ID,Date,Date Time,Amount"));
        assert!(header.ends_with("Transaction Type,Transaction Details Type"));
    }
}

#[test]
fn chf_statement_is_read_in_chronological_order() {
    let s = parse_wise_csv(CHF).unwrap();
    assert_eq!(s.currency, "CHF");
    assert_eq!(s.rows.len(), 10);
    assert_eq!(s.opening_balance, dec!(0));
    assert_eq!(s.closing_balance, dec!(2433.55));
    assert_eq!(s.first_date, date(2020, 1, 10));
    assert_eq!(s.last_date, date(2020, 3, 15));
    let ids: Vec<&str> = s.rows.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids[0], "BALANCE-90000001");
    assert_eq!(ids[9], "BALANCE-90000008");
    // Une ligne de frais précède sa ligne d'origine.
    let fee = ids
        .iter()
        .position(|i| *i == "FEE-BALANCE-90000003")
        .unwrap();
    assert_eq!(ids[fee + 1], "BALANCE-90000003");
    // Chaînage du solde courant.
    for w in s.rows.windows(2) {
        assert_eq!(w[1].running_balance, w[0].running_balance + w[1].amount);
    }
}

#[test]
fn conversion_fee_and_prefixes_are_read() {
    let s = parse_wise_csv(CHF).unwrap();
    let get = |id: &str| s.rows.iter().find(|r| r.id == id).unwrap();

    let conv = get("BALANCE-90000003");
    assert_eq!(conv.id_prefix, "BALANCE");
    assert!(!conv.is_fee);
    assert_eq!(conv.fee_of, None);
    assert_eq!(conv.amount, dec!(-500.00));
    assert_eq!(conv.direction, WiseDirection::Debit);
    assert_eq!(conv.exchange_from.as_deref(), Some("CHF"));
    assert_eq!(conv.exchange_to.as_deref(), Some("EUR"));
    assert_eq!(conv.exchange_rate, Some(dec!(0.9246)));
    assert_eq!(conv.exchange_to_amount, Some(dec!(462.30)));
    assert_eq!(conv.total_fees, Some(dec!(0.35)));
    assert_eq!(conv.details_type, "CONVERSION");
    assert_eq!(conv.date_time.unwrap().to_string(), "2020-02-05 10:30:00");

    let fee = get("FEE-BALANCE-90000003");
    assert!(fee.is_fee);
    assert_eq!(fee.id_prefix, "FEE-BALANCE");
    assert_eq!(fee.fee_of.as_deref(), Some("BALANCE-90000003"));
    assert_eq!(fee.amount, dec!(-0.35));
    assert_eq!(fee.total_fees, None);

    let eur = parse_wise_csv(EUR).unwrap();
    let order = eur
        .rows
        .iter()
        .find(|r| r.id == "FEE-CONVERSION_ORDER-90000011")
        .unwrap();
    assert_eq!(order.id_prefix, "FEE-CONVERSION_ORDER");
    assert_eq!(
        eur.rows
            .iter()
            .find(|r| r.id == "CONVERSION_ORDER-90000011")
            .unwrap()
            .id_prefix,
        "CONVERSION_ORDER"
    );
}

#[test]
fn transfers_deposits_and_cards_are_read() {
    let s = parse_wise_csv(CHF).unwrap();
    let get = |id: &str| s.rows.iter().find(|r| r.id == id).unwrap();

    let to_company = get("TRANSFER-90000004");
    assert_eq!(to_company.payee_name.as_deref(), Some("Société Fictive SA"));
    assert_eq!(
        to_company.payee_account.as_deref(),
        Some("CH93 0076 2011 6238 5295 7")
    );
    assert_eq!(
        to_company.payment_reference.as_deref(),
        Some("Virement test")
    );
    assert_eq!(
        get("TRANSFER-90000006").payee_account.as_deref(),
        Some("0123456789")
    );

    let dep = get("BALANCE-90000001");
    assert_eq!(dep.direction, WiseDirection::Credit);
    assert_eq!(dep.payer_name.as_deref(), Some("Stripe Payments Europe"));
    assert_eq!(dep.details_type, "DEPOSIT");
    assert_eq!(dep.description.as_deref(), Some("Argent reçu de Stripe"));

    let card = get("CARD-90000002");
    assert_eq!(card.merchant.as_deref(), Some("Café du Marché Fictif"));
    assert_eq!(card.id_prefix, "CARD");
}

#[test]
fn narrow_no_break_space_is_kept_in_the_description() {
    let s = parse_wise_csv(CHF).unwrap();
    let row = s.rows.iter().find(|r| r.id == "TRANSFER-90000005").unwrap();
    assert_eq!(
        row.description.as_deref(),
        Some("Envoi de 120\u{202f}CHF à Atelier Exemple Sàrl")
    );
}

#[test]
fn card_columns_never_reach_the_result() {
    // Les colonnes de carte et la pièce jointe sont remplies : rien ne les
    // lit, et le résultat ne les porte pas.
    let filled = rewrite(CHF, |i, mut cells| {
        if i > 0 {
            for name in [
                "Card Last Four Digits",
                "Card Holder Full Name",
                "Attachment",
            ] {
                cells[col(name)] = "VALEUR-SECRETE".to_string();
            }
        }
        cells
    });
    let s = parse_wise_csv(&filled).unwrap();
    assert!(!format!("{s:?}").contains("VALEUR-SECRETE"));
    assert_eq!(s, parse_wise_csv(CHF).unwrap());
}

#[test]
fn rows_are_sorted_on_the_read_timestamp_not_on_file_order() {
    // JJ-MM-AAAA ne se trie pas en texte (20-01-2020 précède 05-02-2020) : le
    // tri porte sur l'horodatage lu. Deux lignes de données échangées dans le fichier : l'ordre inverse du
    // fichier ne se suit plus, l'horodatage lu, si.
    let swapped = {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(CHF);
        let mut records: Vec<csv::StringRecord> = reader.records().map(|r| r.unwrap()).collect();
        records.swap(2, 7);
        let mut writer = csv::WriterBuilder::new().from_writer(Vec::new());
        for r in &records {
            writer.write_record(r).unwrap();
        }
        writer.into_inner().unwrap()
    };
    let ids = |s: kesh_import::WiseCsv| s.rows.into_iter().map(|r| r.id).collect::<Vec<_>>();
    assert_eq!(
        ids(parse_wise_csv(&swapped).unwrap()),
        ids(parse_wise_csv(CHF).unwrap())
    );
}

#[test]
fn columns_in_another_order_are_read_the_same() {
    // Colonnes inversées, en-tête compris.
    let reversed = rewrite(CHF, |_, mut cells| {
        cells.reverse();
        cells
    });
    assert_eq!(
        parse_wise_csv(&reversed).unwrap(),
        parse_wise_csv(CHF).unwrap()
    );
}

#[test]
fn other_files_are_read() {
    let eur = parse_wise_csv(EUR).unwrap();
    assert_eq!((eur.rows.len(), eur.currency.as_str()), (5, "EUR"));
    assert_eq!(eur.closing_balance, dec!(160.00));
    let usd = parse_wise_csv(USD).unwrap();
    assert_eq!((usd.rows.len(), usd.currency.as_str()), (3, "USD"));
    assert_eq!(usd.opening_balance, dec!(0));
    assert_eq!(usd.closing_balance, dec!(317.01));
}

#[test]
fn bom_is_accepted() {
    let mut with_bom = b"\xEF\xBB\xBF".to_vec();
    with_bom.extend_from_slice(CHF);
    assert_eq!(
        parse_wise_csv(&with_bom).unwrap(),
        parse_wise_csv(CHF).unwrap()
    );
}

#[test]
fn all_missing_required_columns_are_listed_in_order() {
    let text = std::str::from_utf8(USD).unwrap();
    let header = text.lines().next().unwrap();
    let broken = text.replacen(header, "Foo,Bar", 1);
    // Chaque ligne a maintenant plus de cellules que l'en-tête : le refus de
    // colonnes passe avant celui des lignes.
    match parse_wise_csv(broken.as_bytes()).unwrap_err() {
        WiseCsvError::MissingColumns(names) => {
            assert_eq!(names, REQUIRED_COLUMNS.map(String::from).to_vec());
        }
        other => panic!("attendu MissingColumns, reçu {other:?}"),
    }
    let one = text.replacen("Running Balance", "Solde", 1);
    match parse_wise_csv(one.as_bytes()).unwrap_err() {
        WiseCsvError::MissingColumns(names) => assert_eq!(names, vec!["Running Balance"]),
        other => panic!("{other:?}"),
    }
}

#[test]
fn duplicate_column_is_refused() {
    let text = std::str::from_utf8(USD)
        .unwrap()
        .replacen("Merchant", "Amount", 1);
    assert_eq!(
        parse_wise_csv(text.as_bytes()).unwrap_err(),
        WiseCsvError::DuplicateColumn("Amount".into())
    );
}

#[test]
fn empty_inputs_are_refused() {
    assert_eq!(parse_wise_csv(b"").unwrap_err(), WiseCsvError::Empty);
    let header_only = std::str::from_utf8(USD)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_string();
    assert_eq!(
        parse_wise_csv(header_only.as_bytes()).unwrap_err(),
        WiseCsvError::Empty
    );
}

#[test]
fn two_currencies_are_refused() {
    let mixed = set(CHF, 3, "Currency", "EUR");
    assert_eq!(
        parse_wise_csv(&mixed).unwrap_err(),
        WiseCsvError::MixedCurrencies(vec!["CHF".into(), "EUR".into()])
    );
}

#[test]
fn duplicate_id_is_refused_with_its_lines() {
    let dup = set(CHF, 0, "TransferWise ID", "balance-90000001");
    assert_eq!(
        parse_wise_csv(&dup).unwrap_err(),
        WiseCsvError::DuplicateId {
            id: "BALANCE-90000001".into(),
            lines: vec![2, 11],
        }
    );
}

#[test]
fn running_balance_break_names_the_line() {
    // Ligne 5 du fichier = 4e ligne de données du plus récent au plus ancien.
    let broken = set(CHF, 3, "Running Balance", "1.00");
    match parse_wise_csv(&broken).unwrap_err() {
        WiseCsvError::RunningBalanceBreak { line, found, .. } => {
            assert_eq!(line, 5);
            assert_eq!(found, dec!(1.00));
        }
        other => panic!("attendu RunningBalanceBreak, reçu {other:?}"),
    }
}

#[test]
fn ordering_falls_back_on_reversed_file_order_without_date_time() {
    // Une seule ligne sans `Date Time` : tri sur l'ordre inverse du fichier seul.
    let one_blank = set(CHF, 5, "Date Time", "");
    assert_eq!(parse_wise_csv(&one_blank).unwrap(), {
        let mut s = parse_wise_csv(CHF).unwrap();
        for r in &mut s.rows {
            if r.line == 7 {
                r.date_time = None;
            }
        }
        s
    });
    // Colonne `Date Time` absente : même ordre.
    let idx = col("Date Time");
    let without = rewrite(CHF, |_, mut cells| {
        cells.remove(idx);
        cells
    });
    let s = parse_wise_csv(&without).unwrap();
    assert!(s.rows.iter().all(|r| r.date_time.is_none()));
    assert_eq!(s.rows[0].id, "BALANCE-90000001");
    assert_eq!(s.closing_balance, dec!(2433.55));
}

#[test]
fn sign_and_fee_sign_are_checked() {
    let wrong = set(CHF, 0, "Transaction Type", "DEBIT");
    assert_eq!(
        line_codes(parse_wise_csv(&wrong).unwrap_err()),
        vec![(2, WiseLineErrorCode::SignMismatch)]
    );
    // Crédit à montant négatif (ligne de données 3 : un débit relevé « CREDIT »).
    let credit = set(CHF, 2, "Transaction Type", "CREDIT");
    assert_eq!(
        line_codes(parse_wise_csv(&credit).unwrap_err()),
        vec![(4, WiseLineErrorCode::SignMismatch)]
    );
    // Ligne de frais créditée (ligne de données 5 : FEE-TRANSFER).
    let fee = rewrite(CHF, |i, mut cells| {
        if i == 6 {
            assert!(cells[col("TransferWise ID")].starts_with("FEE-"));
            cells[col("Amount")] = "1.20".into();
            cells[col("Transaction Type")] = "CREDIT".into();
        }
        cells
    });
    assert_eq!(
        line_codes(parse_wise_csv(&fee).unwrap_err()),
        vec![(7, WiseLineErrorCode::FeeSign)]
    );
}

#[test]
fn dates_are_strict() {
    for bad in [
        "01/20/2020",
        "2020/01/20",
        "32-01-2020",
        "20-1-2020",
        "0001-01-01",
    ] {
        let f = set(CHF, 0, "Date", bad);
        assert_eq!(
            line_codes(parse_wise_csv(&f).unwrap_err()),
            vec![(2, WiseLineErrorCode::Date)],
            "{bad}"
        );
    }
    let iso = set(CHF, 0, "Date", "2020-03-15");
    assert!(parse_wise_csv(&iso).is_ok());
    for bad in [
        "15-03-2020",
        "15-03-2020 08:00",
        "15-03-2020 25:00:00",
        "15-03-2020 08:00:00.",
        "15/03/2020 08:00:00",
        // Hors AC-4 : heure sur un chiffre, décimales non chiffres ou en
        // nombre différent de trois, seconde intercalaire, forme ISO.
        "15-03-2020 8:00:00",
        "15-03-2020 08:00:00.abc",
        "15-03-2020 08:00:00.12",
        "15-03-2020 08:00:00.1234",
        "15-03-2020 23:59:60",
        "15-03-2020 23:59:60.000",
        "2020-03-15 08:00:00",
        // Revue P2, L1 : longueur de l'heure et six chiffres ASCII.
        "15-03-2020 08:00:00x",
        "15-03-2020 +8:00:00",
    ] {
        let f = set(CHF, 0, "Date Time", bad);
        assert_eq!(
            line_codes(parse_wise_csv(&f).unwrap_err()),
            vec![(2, WiseLineErrorCode::DateTime)],
            "{bad}"
        );
    }
    assert!(parse_wise_csv(&set(CHF, 0, "Date Time", "15-03-2020 08:00:00")).is_ok());
    assert!(parse_wise_csv(&set(CHF, 0, "Date Time", "15-03-2020 23:59:59.999")).is_ok());
}

#[test]
fn amounts_follow_the_closed_form() {
    for bad in ["1.500,00", "1,50", "1 500.00", "1'500.00", "+5.00", "1.234"] {
        let f = set(CHF, 0, "Amount", bad);
        let codes = line_codes(parse_wise_csv(&f).unwrap_err());
        assert_eq!(codes, vec![(2, WiseLineErrorCode::Amount)], "{bad}");
    }
    // Total fees et Exchange To Amount obéissent à la même forme.
    let f = set(CHF, 0, "Total fees", "0,35");
    assert_eq!(
        line_codes(parse_wise_csv(&f).unwrap_err()),
        vec![(2, WiseLineErrorCode::Amount)]
    );
    let f = set(CHF, 0, "Exchange To Amount", "462,30");
    assert_eq!(
        line_codes(parse_wise_csv(&f).unwrap_err()),
        vec![(2, WiseLineErrorCode::Amount)]
    );
}

#[test]
fn exchange_rate_is_closed_at_ten_decimals() {
    let ten = set(CHF, 0, "Exchange Rate", "0.1234567890");
    assert!(parse_wise_csv(&ten).is_ok());
    for bad in ["0.12345678901", "0,9246", "-1", ".5", "1."] {
        let f = set(CHF, 0, "Exchange Rate", bad);
        assert_eq!(
            line_codes(parse_wise_csv(&f).unwrap_err()),
            vec![(2, WiseLineErrorCode::Rate)],
            "{bad}"
        );
    }
}

#[test]
fn ids_currency_and_direction_are_checked() {
    for (name, bad, code) in [
        ("TransferWise ID", "", WiseLineErrorCode::Id),
        ("TransferWise ID", "TRANSFER 1", WiseLineErrorCode::Id),
        ("Currency", "CH", WiseLineErrorCode::Currency),
        ("Currency", "C1F", WiseLineErrorCode::Currency),
        ("Transaction Type", "NEUTRAL", WiseLineErrorCode::Direction),
        ("Transaction Details Type", "", WiseLineErrorCode::Required),
    ] {
        let f = set(CHF, 0, name, bad);
        let codes = line_codes(parse_wise_csv(&f).unwrap_err());
        assert!(codes.contains(&(2, code)), "{name}={bad:?} : {codes:?}");
    }
    let long = set(CHF, 0, "TransferWise ID", &"A".repeat(65));
    assert_eq!(
        line_codes(parse_wise_csv(&long).unwrap_err()),
        vec![(2, WiseLineErrorCode::Id)]
    );
}

#[test]
fn field_count_is_an_error_with_its_line() {
    let short = rewrite(CHF, |i, mut cells| {
        if i == 3 {
            cells.pop();
        }
        cells
    });
    assert_eq!(
        line_codes(parse_wise_csv(&short).unwrap_err()),
        vec![(4, WiseLineErrorCode::FieldCount)]
    );
}

#[test]
fn line_errors_are_capped_and_values_truncated() {
    let many = rewrite(CHF, |i, mut cells| {
        if i > 0 {
            cells[col("Date")] = "x".repeat(300);
        }
        cells
    });
    let mut text = String::from_utf8(many.clone()).unwrap();
    // Plus de 100 lignes fautives : on duplique des lignes mal datées.
    let row = text.lines().nth(1).unwrap().to_string();
    for _ in 0..150 {
        text.push_str(&row);
        text.push('\n');
    }
    match parse_wise_csv(text.as_bytes()).unwrap_err() {
        WiseCsvError::Lines(l) => {
            assert_eq!(l.len(), kesh_import::MAX_CSV_LINE_ERRORS);
            assert_eq!(
                l[0].value.as_ref().unwrap().chars().count(),
                kesh_import::MAX_CSV_LINE_ERROR_VALUE_CHARS + 1
            );
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn error_codes_are_stable() {
    assert_eq!(WiseCsvError::Empty.code(), "WISE_CSV_EMPTY");
    assert_eq!(
        WiseCsvError::MissingColumns(vec![]).code(),
        "WISE_CSV_MISSING_COLUMNS"
    );
    assert_eq!(
        WiseCsvError::MixedCurrencies(vec![]).code(),
        "WISE_CSV_MIXED_CURRENCIES"
    );
    assert_eq!(WiseLineErrorCode::SignMismatch.as_str(), "SIGN_MISMATCH");
    assert_eq!(
        serde_json::to_string(&WiseLineErrorCode::FeeSign).unwrap(),
        "\"FEE_SIGN\""
    );
}

/// Vérification locale sur de vrais relevés : `KESH_WISE_REAL_DIR` désigne un
/// dossier de CSV Wise ; chaque fichier doit se lire, et seuls les comptes de
/// lignes et la devise s'impriment (aucune donnée citée). Ignoré par défaut.
#[test]
#[ignore = "lit des relevés réels hors dépôt (KESH_WISE_REAL_DIR)"]
fn real_statements_are_read() {
    let dir = std::env::var("KESH_WISE_REAL_DIR")
        .expect("KESH_WISE_REAL_DIR doit désigner le dossier des relevés réels");
    let mut files = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "csv") {
            let bytes = std::fs::read(&path).unwrap();
            let lines = String::from_utf8_lossy(&bytes).lines().count() - 1;
            // Le code seul : le message d'une erreur peut citer une valeur.
            let s =
                parse_wise_csv(&bytes).unwrap_or_else(|e| panic!("relevé refusé : {}", e.code()));
            files += 1;
            assert_eq!(s.rows.len(), lines, "toutes les lignes sont lues");
            println!("{} : {} lignes lues", s.currency, s.rows.len());
        }
    }
    println!("{files} fichier(s) lu(s)");
    assert!(
        files > 0,
        "aucun fichier .csv dans le dossier : rien n'a été lu"
    );
}

#[test]
fn extreme_amounts_are_refused_without_panicking() {
    // Le maximum d'un `Decimal` dépasse la forme fermée (quinze chiffres
    // d'entier au plus) : refus de ligne `AMOUNT`, jamais de panique. Le
    // chemin `OVERFLOW` du chaînage n'est donc plus atteignable par un
    // fichier ; il garde sa preuve en test unitaire (`wise.rs`).
    let max = "79228162514264337593543950335";
    let f = set(&set(CHF, 0, "Amount", max), 0, "Running Balance", max);
    let codes = line_codes(parse_wise_csv(&f).unwrap_err());
    assert_eq!(codes, vec![(2, WiseLineErrorCode::Amount); 2]);
    let long = "9".repeat(5000);
    let f = set(CHF, 0, "Amount", &long);
    assert_eq!(
        line_codes(parse_wise_csv(&f).unwrap_err()),
        vec![(2, WiseLineErrorCode::Amount)]
    );
}

/// Fichier synthétique d'une devise : une ligne de données par identifiant,
/// montants et soldes à 0 (continuité et signes sans objet), sans horodatage.
fn synthetic(ids: impl IntoIterator<Item = String>) -> Vec<u8> {
    let header = std::str::from_utf8(CHF).unwrap().lines().next().unwrap();
    let mut out = String::from(header);
    for id in ids {
        out.push_str(&format!(
            "\n{id},01-01-2020,,0,CHF,,,0,,,,,,,,,,,,,,CREDIT,DEPOSIT"
        ));
    }
    out.into_bytes()
}

#[test]
fn first_duplicate_in_file_order_is_cited_with_all_its_lines() {
    // X(2) Y(3) Y(4) X(5) Z(6) Y(7) : Y est le premier identifiant revu dans
    // l'ordre du fichier, avec toutes ses lignes ; X, doublon plus tard, non.
    let ids = ["X", "Y", "Y", "X", "Z", "Y"].map(String::from);
    assert_eq!(
        parse_wise_csv(&synthetic(ids)).unwrap_err(),
        WiseCsvError::DuplicateId {
            id: "Y".into(),
            lines: vec![3, 4, 7],
        }
    );
}

#[test]
fn a_large_file_is_checked_in_linear_time() {
    // 100 000 lignes, aucun doublon : la vérification quadratique d'origine
    // (n²/2 comparaisons) dépasserait la minute ; la linéaire, la seconde.
    let ids = (0..100_000).map(|i| format!("TRANSFER-{i}"));
    let file = synthetic(ids);
    let start = std::time::Instant::now();
    let s = parse_wise_csv(&file).unwrap();
    assert_eq!(s.rows.len(), 100_000);
    assert!(
        start.elapsed() < std::time::Duration::from_secs(20),
        "lecture trop lente : {:?}",
        start.elapsed()
    );
    // Le doublon en dernière ligne est trouvé aussi vite.
    let ids = (0..100_000)
        .map(|i| format!("TRANSFER-{i}"))
        .chain(["TRANSFER-0".to_string()]);
    let start = std::time::Instant::now();
    assert!(matches!(
        parse_wise_csv(&synthetic(ids)).unwrap_err(),
        WiseCsvError::DuplicateId { .. }
    ));
    assert!(start.elapsed() < std::time::Duration::from_secs(20));
}

#[test]
fn quoted_cells_and_a_comma_in_a_description_are_read() {
    // Les relevés de Wise mettent chaque cellule entre guillemets ; une
    // description peut porter une virgule.
    let desc = "Frais, TVA incluse";
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(CHF);
    let mut writer = csv::WriterBuilder::new()
        .quote_style(csv::QuoteStyle::Always)
        .from_writer(Vec::new());
    let c = col("Description");
    for (i, r) in reader.records().enumerate() {
        let mut cells: Vec<String> = r.unwrap().iter().map(str::to_string).collect();
        if i == 1 {
            cells[c] = desc.to_string();
        }
        writer.write_record(&cells).unwrap();
    }
    let quoted = writer.into_inner().unwrap();
    assert!(quoted.starts_with(b"\"TransferWise ID\",\"Date\""));
    let s = parse_wise_csv(&quoted).unwrap();
    let mut expected = parse_wise_csv(CHF).unwrap();
    // Première ligne de données du fichier = dernière en ordre chronologique.
    let last = expected.rows.len() - 1;
    expected.rows[last].description = Some(desc.to_string());
    assert_eq!(s, expected);
}
