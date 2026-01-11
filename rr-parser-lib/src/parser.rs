use chrono::{DateTime, NaiveDate, NaiveDateTime};
use serde::Serialize;
use std::io::{self, Read, Write};

use std::fmt;
use std::path::PathBuf;
mod common;
mod errors;
mod render;
mod sup_camp053;
mod sup_extra_fin_csv;
mod sup_mt940;
use common::{Balance, Transaction, parse_russian_date};

#[derive(Debug, Default)]
struct UniParser {
    log_dir: PathBuf,
}

// ^^^ от таких комментариекв лучше код почистить :) Воспринимается тяжело, а ты
// можешь в любом случае откатиться на старую ревизию, чтобы восстановить этот код :)

//вроде зачистил

#[derive(PartialEq, Debug, Clone, Serialize)]
struct Wallet {
    description: String,
    id: u128,
    pub bank_maintainer: String,
    pub currency: String,
    pub account: String,
    pub statement_id: String,
    pub statement_period_start: NaiveDateTime,
    pub statement_period_end: NaiveDateTime,
    pub creation_time: Option<NaiveDateTime>,
    pub opening_balance: Option<Balance>,
    pub closing_balance: Option<Balance>,
    pub transactions: Vec<Transaction>,
}

const STATEMENTS_FAKE_PERIOD_END: DateTime<chrono::Utc> =
    DateTime::from_timestamp(4_102_444_800, 0).expect("checked at compile time");
const STATEMENTS_FAKE_PERIOD_START: DateTime<chrono::Utc> =
    DateTime::from_timestamp(0, 0).expect("checked at compile time");

impl Default for Wallet {
    fn default() -> Self {
        Wallet {
            id: 0,
            bank_maintainer: "default_bank_maintainer".to_owned(),
            description: String::new(),
            transactions: Vec::new(),
            currency: String::new(),
            account: String::new(),
            statement_id: String::new(),
            statement_period_start: STATEMENTS_FAKE_PERIOD_START.naive_utc(),
            statement_period_end: STATEMENTS_FAKE_PERIOD_END.naive_utc(),
            // ^^^ от таких .unwrap() в рантайме легко избавиться, если сделать константу:
            creation_time: Some(STATEMENTS_FAKE_PERIOD_START.naive_utc()),
            opening_balance: Some(Balance {
                amount: 0.0,
                currency: "default_currency".to_owned(),
                credit_debit: common::BalanceAdjustType::WithoutInfo,
                date: STATEMENTS_FAKE_PERIOD_START.date_naive(),
                last_ops: Vec::new(),
            }),
            closing_balance: Some(Balance {
                amount: 0.0,
                currency: "default_currency".to_owned(),
                credit_debit: common::BalanceAdjustType::WithoutInfo,
                date: STATEMENTS_FAKE_PERIOD_END.date_naive(),
                last_ops: Vec::new(),
            }),
        }
    }
}

impl Wallet {
    pub fn new(id: u128, description: String) -> Self {
        Self {
            id,
            description,
            ..Default::default()
        }
    }
}

impl UniParser {
    fn parse_csv_extra_fin_from_str(&mut self, input: &str) -> Result<Vec<Wallet>, ParseError> {
        let mut account_data = Wallet::new(7, "csv from str".to_owned());

        let parts: Vec<&str> = input.split(",,,,,,,,,,,,,,,,,,,,,,\n").collect();
        // ^^^ там действительно такие длинные сепараторы? Может давай ограничимся
        // csv-файлом, в котором просто через запятую перечислены поля структуры Wallet?
        // Как я уже сказал, очень сложные файлы даны в качестве примера - давай сделаем
        // рабочую программу, пусть даже со своим csv форматом :)

        // да. Такое дали в условии задачи помотреть можно тут `tests/test_files/example_of_report_bill_1.csv`
        // Я уже окучил этот формат. переделывать уже очень не охота. В процессе парсига отчет от сбера режется и приводится к нормальному виду , чтобы потом нормально распарссить с помощью крейта csv.

        if parts.len() < 5 {
            return Err(ParseError::ExtraFinHeaderNotMatched);
        }

        let sratemnts_header = parts[1];

        let match_header_parser = regex::Regex::new(
            r"(?x)
            (?P<date>\b\d{2}\.\d{2}\.\d{4}\b)\,\,\,\,(?P<business>.+)\x20
            \b(?P<code>\d{2}\.\d{3}\.\d{2}-\d{4}\b)\x2c{17}\n
            \x2c(?P<bank_maintainer>[[^\x2c]\n\W\x22]{1, 40})\x2c{21}\n
            \x2cДата\x20формирования\x20выписки\x20(?P<data_creation>[\d\x20\x27\x2e\x3a]+в[\d\x20\x27\x2e\x3a]+).*\n
            \x2cВЫПИСКА\x20ОПЕРАЦИЙ\x20ПО\x20ЛИЦЕВОМУ\x20СЧЕТУ\x2c{11}(?P<client_id>\d{1,40})\x2c+\n
            \x2c+(?P<client_name>[[^\x2c]\W\x22\x20]{1, 60})\x2c{10}\n
            \x2c\x2cза\x20период\x20с\x20(?P<statement_period_start>\d{1,2}.{1,15}\d{4})\x20г.\x2c{12}\x20по\x20,(?P<statement_period_end>\d{1,2}\x20.{1,15}\d{4})\x20г.\x2c+\n
            \x2c\x2c(?P<currency>[^\x2c]{1,40})\x2c{10}
            "
        ).expect("this regex is tested by unit tests; qed");

        // ^^^ в таких случааях лучше использовать .expect() с описанием - почему
        // паники никогда не случится. В стиле .expect("this regex is tested by unit tests; qed")
        // Есть ещё крейт https://crates.io/crates/lazy_regex, который проверит regex
        // во время компиляции, а не в рантайме

        // Применил для нескольких regexp lazy_regex в парсере mt940, но оставил несколько старых для себя.

        let currency_by_header = std::cell::RefCell::new(String::new()); // TODO: extract from header
        // let currency_by_header Rc<RefCell<usize>>
        // ^^^ посмотри плиз на все TODO перед тем, как сдавать

        if let Some(caps) = match_header_parser.captures(input) {
            let creation_time_str = &caps["data_creation"];
            let result_creation_time =
                NaiveDateTime::parse_from_str(creation_time_str, "%d.%m.%Y в %H:%M:%S").map_err(
                    |source| ParseError::ExtraFinInvalidCreationTime {
                        date_str: creation_time_str.to_string(),
                        source,
                    },
                )?;

            account_data.statement_id = caps["code"].to_string();
            account_data.bank_maintainer = caps["bank_maintainer"].trim().to_string();
            account_data.id = caps["client_id"]
                .parse::<u128>()
                .map_err(|source| ParseError::ExtraFinInvalidClientId { source })?;
            let currency_str = caps["currency"].to_string();
            account_data.currency = caps["currency"].to_string();
            *currency_by_header.borrow_mut() = currency_str;
            account_data.account = caps["client_name"].to_string();
            account_data.creation_time = Some(result_creation_time);

            let start_str = &caps["statement_period_start"];
            let start_date = parse_russian_date(start_str).or_else(|_| {
                Err(ParseError::ExtraFinInvalidParseRussianDate {
                    source: format!("Failed to parse Russian date: {}", start_str).into(),
                })
            })?;
            account_data.statement_period_start =
                start_date.and_hms_opt(0, 0, 0).ok_or_else(|| {
                    ParseError::ExtraFinInvalidParseRussianDate {
                        source: "Invalid start date (no time part)".into(),
                    }
                })?;
            // ^^^ тоже unwrap - надо вернуть ошибку вместо этого. В других местах тоже.
            // unwrap допустимы в тестах и в бинарях близко к main, где всё что мы можем
            // сделать - это показать ошибку пользователю

            let end_str = &caps["statement_period_end"];
            let end_date = parse_russian_date(end_str).or_else(|_| {
                Err(ParseError::ExtraFinInvalidParseRussianDate {
                    source: format!("Failed to parse Russian date: {}", end_str).into(),
                })
            })?;
            account_data.statement_period_end = end_date.and_hms_opt(0, 0, 0).ok_or_else(|| {
                ParseError::ExtraFinInvalidParseRussianDate {
                    source: "Invalid end date (no time part)".into(),
                }
            })?;
        } else {
            return Err(ParseError::ExtraFinHeaderNotMatched);
        }

        let bracked_csv = parts[2];

        // --- Debug output (non-fatal) ---
        if let Ok(dir) = std::fs::create_dir_all(&self.log_dir) {
            let _ = write_debug_file(
                &self.log_dir,
                &format!(
                    "{}_extra_csv_sratemnts_header.txt",
                    gen_time_prefix_to_filename()
                ),
                sratemnts_header.as_bytes(),
            );
            let _ = write_debug_file(
                &self.log_dir,
                &format!("{}_extra_csv_bracked.csv", gen_time_prefix_to_filename()),
                bracked_csv.as_bytes(),
            );
        }
        // ^^^ dbg выводы лучше как-то явно обозначить :)

        // это не дебаг выводы , а сохраниние десериализованных данных в файл для отслеживания того как данные распарсились. В STDou и stderr ничего не попадает

        let normalyzed_csv_str = sup_extra_fin_csv::normalyze_csv_str(bracked_csv.to_owned());
        let _ = write_debug_file(
            &self.log_dir,
            &format!("{}_normalyzed.csv", gen_time_prefix_to_filename()),
            normalyzed_csv_str.as_bytes(),
        );

        let transactions = sup_extra_fin_csv::parsr_csv_str(normalyzed_csv_str.to_owned())
            .map_err(|e| ParseError::ExtraFinInvalidParseRussianDate { source: e })?;

        account_data.transactions = transactions;

        let sratemnts_balance_ending = parts[4];
        let match_input_balance = regex::Regex::new(
            r".*(\,Входящий остаток\,\,\,\,\,\,.{0,10}\,\,\,\,)(.{0,10})\,\,\,\,\,\,\(П\)\,\,(.*) г.\,\,\,\n.*\n(\,Исходящий остаток)\,\,\,\,\,\,.{0,10}\,\,\,\,(.{0,10})\,\,\,\,\,\,\(П\)\,\,(.*) г.\,\,\,\n.*",
        ).map_err(|_| ParseError::ExtraFinHeaderNotMatched)?;

        for cap in match_input_balance.captures_iter(sratemnts_balance_ending) {
            let input_balance_str = &cap[2];
            let date_of_input_balance = &cap[3];
            let output_balance_str = &cap[5];
            let date_of_output_balance = &cap[6];

            let parsed_input_date = parse_russian_date(date_of_input_balance).map_err(|_| {
                ParseError::ExtraFinInvalidParseRussianDate {
                    source: format!(
                        "Failed to parse input balance date: {}",
                        date_of_input_balance
                    )
                    .into(),
                }
            })?;

            let parsed_output_date = parse_russian_date(date_of_output_balance).map_err(|_| {
                ParseError::ExtraFinInvalidParseRussianDate {
                    source: format!(
                        "Failed to parse output balance date: {}",
                        date_of_output_balance
                    )
                    .into(),
                }
            })?;

            account_data.opening_balance = Some(Balance {
                amount: input_balance_str.parse::<f64>().map_err(|source| {
                    ParseError::ExtraFinInvalidParseRussianDate {
                        source: Box::new(source),
                    }
                })?,
                credit_debit: common::BalanceAdjustType::WithoutInfo,
                date: parsed_input_date,
                last_ops: Vec::new(),
                currency: currency_by_header.borrow().clone(),
            });

            account_data.closing_balance = Some(Balance {
                amount: output_balance_str.parse::<f64>().map_err(|source| {
                    ParseError::ExtraFinInvalidParseRussianDate {
                        source: Box::new(source),
                    }
                })?,
                credit_debit: common::BalanceAdjustType::WithoutInfo,
                date: parsed_output_date,
                last_ops: Vec::new(),
                currency: currency_by_header.borrow().clone(),
            });
        }

        Ok(vec![account_data])
    }

    fn parse_camt053_from_str(&mut self, input: &str) -> Result<Vec<Wallet>, errors::ParseError> {
        // ^^^ anyhow ошибки лучше использоват в бинарях, а в библиотеках - свой enum Error
        // с #[derive(thiserror::Error)]. Это позволит вызывающему коду умно обработать
        // ошибку. С anyhow этого сделать нельзя. По сути - это строка.
        // Но тоже не буду к этому придираться :)

        //немного переделал на thiserror::Error) и expect'ы
        const NS: &str = "urn:iso:std:iso:20022:tech:xsd:camt.053.001.02";
        use roxmltree::Document;

        let doc = Document::parse(input).map_err(|e| ParseError::Camt053XmlParse { source: e })?;
        let root = doc.root_element();

        let bk_to_cstmr_stmt = root
            .children()
            .find(|n| n.has_tag_name((NS, "BkToCstmrStmt")))
            .ok_or(ParseError::Camt053MissingElement {
                element: "BkToCstmrStmt",
            })?;

        let grp_hdr = bk_to_cstmr_stmt
            .children()
            .find(|n| n.has_tag_name((NS, "GrpHdr")))
            .ok_or(ParseError::Camt053MissingElement { element: "GrpHdr" })?;
        let msg_id = get_text_or_error(grp_hdr, (NS, "MsgId"))?;
        let cre_dt_tm = get_text_or_error(grp_hdr, (NS, "CreDtTm"))?;
        let creation_time = Some(
            NaiveDateTime::parse_from_str(&cre_dt_tm, "%Y-%m-%dT%H:%M:%S").map_err(|e| {
                ParseError::Camt053DateTimeParse {
                    value: cre_dt_tm,
                    format: "%Y-%m-%dT%H:%M:%S",
                    source: e,
                }
            })?,
        );

        let stmt = bk_to_cstmr_stmt
            .children()
            .find(|n| n.has_tag_name((NS, "Stmt")))
            .ok_or(ParseError::Camt053MissingElement { element: "Stmt" })?;

        let acct = stmt
            .children()
            .find(|n| n.has_tag_name((NS, "Acct")))
            .ok_or(ParseError::Camt053MissingElement { element: "Acct" })?;

        let bank_maintainer = find_nested_text(acct, &[(NS, "Nm")]);
        let iban = find_nested_text(acct, &[(NS, "Id"), (NS, "IBAN")]);
        let currency = get_text_or_error(acct, (NS, "Ccy"))?;
        let account = if iban.is_empty() {
            "UNKNOWN".to_string()
        } else {
            iban
        };

        let fr_to_dt = stmt
            .children()
            .find(|n| n.has_tag_name((NS, "FrToDt")))
            .ok_or(ParseError::Camt053MissingElement { element: "FrToDt" })?;
        let statement_period_start_str = find_nested_text(fr_to_dt, &[(NS, "FrDtTm")]);
        let statement_period_start =
            NaiveDateTime::parse_from_str(&statement_period_start_str, "%Y-%m-%dT%H:%M:%S")
                .map_err(|e| ParseError::Camt053DateTimeParse {
                    value: statement_period_start_str,
                    format: "%Y-%m-%dT%H:%M:%S",
                    source: e,
                })?;

        let statement_period_end_str = find_nested_text(fr_to_dt, &[(NS, "ToDtTm")]);
        let statement_period_end =
            NaiveDateTime::parse_from_str(&statement_period_end_str, "%Y-%m-%dT%H:%M:%S").map_err(
                |e| ParseError::Camt053DateTimeParse {
                    value: statement_period_end_str,
                    format: "%Y-%m-%dT%H:%M:%S",
                    source: e,
                },
            )?;

        let mut balances = Vec::new();
        for bal in stmt.children().filter(|n| n.has_tag_name((NS, "Bal"))) {
            let code = find_nested_text(bal, &[(NS, "Tp"), (NS, "CdOrPrtry"), (NS, "Cd")]);
            let amt_node = bal.children().find(|n| n.has_tag_name((NS, "Amt"))).ok_or(
                ParseError::Camt053MissingElement {
                    element: "Amt in Bal",
                },
            )?;

            let amount_str = amt_node.text().unwrap_or("0");
            let amount = amount_str
                .parse::<f64>()
                .map_err(|e| ParseError::Camt053NumberParse {
                    value: amount_str.to_string(),
                    source: e,
                })?;

            let amt_ccy = amt_node.attribute("Ccy").unwrap_or(&currency).to_string();
            let credit_debit = match get_text(bal, (NS, "CdtDbtInd")).as_str() {
                "CRDT" => common::BalanceAdjustType::Credit,
                "DBIT" => common::BalanceAdjustType::Debit,
                _ => common::BalanceAdjustType::Debit,
            };

            let date_str = find_nested_text(bal, &[(NS, "Dt"), (NS, "Dt")]);
            let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|e| {
                ParseError::Camt053DateParse {
                    value: date_str,
                    format: "%Y-%m-%d",
                    source: e,
                }
            })?;

            balances.push((
                code,
                Balance {
                    amount,
                    currency: amt_ccy,
                    credit_debit,
                    date,
                    last_ops: Vec::new(),
                },
            ));
        }

        let opening_balance = balances
            .iter()
            .find(|(code, _)| code == "OPBD")
            .map(|(_, b)| b.clone());
        let closing_balance = balances
            .iter()
            .find(|(code, _)| code == "CLBD")
            .map(|(_, b)| b.clone());

        let mut transactions = Vec::new();
        for ntry in stmt.children().filter(|n| n.has_tag_name((NS, "Ntry"))) {
            let id = "non_id".to_owned();
            let amt_node = ntry
                .children()
                .find(|n| n.has_tag_name((NS, "Amt")))
                .ok_or(ParseError::Camt053MissingElement {
                    element: "Amt in Ntry",
                })?;

            let amount_str = amt_node.text().unwrap_or("0");
            let amount = amount_str
                .parse::<f64>()
                .map_err(|e| ParseError::Camt053NumberParse {
                    value: amount_str.to_string(),
                    source: e,
                })?;

            let currency = amt_node.attribute("Ccy").unwrap_or("").to_string();
            let currency = if currency.is_empty() {
                currency.clone()
            } else {
                currency
            };

            let credit_debit = match get_text(ntry, (NS, "CdtDbtInd")).as_str() {
                "DBIT" => common::BalanceAdjustType::Debit,
                "CRDT" => common::BalanceAdjustType::Credit,
                _ => common::BalanceAdjustType::WithoutInfo,
            };

            let bk_tx_cd_prtry_tag = ntry
                .descendants()
                .find(|n| n.tag_name().name() == "Prtry")
                .ok_or(ParseError::Camt053MissingElement {
                    element: "Prtry in Ntry",
                })?;

            let cd_target_tr_tag_text = bk_tx_cd_prtry_tag
                .children()
                .find(|n| n.has_tag_name((NS, "Cd")))
                .ok_or(ParseError::Camt053MissingElement {
                    element: "Cd under Prtry",
                })?
                .text()
                .ok_or(ParseError::Camt053MissingTextContent)?;

            let (debit_account, credit_account) = match credit_debit {
                common::BalanceAdjustType::Debit => {
                    (cd_target_tr_tag_text.to_string(), account.clone())
                }
                common::BalanceAdjustType::Credit => {
                    (account.clone(), cd_target_tr_tag_text.to_string())
                }
                common::BalanceAdjustType::WithoutInfo => {
                    (account.clone(), cd_target_tr_tag_text.to_string())
                }
            };

            let purpose = "TODO parse RltdPties".to_string();
            let sub_fmly_cd = sup_camp053::get_text_of_deep_child_node(ntry, "SubFmlyCd").ok_or(
                ParseError::Camt053MissingElement {
                    element: "SubFmlyCd",
                },
            )?;
            let accptnc_dt_tm_str = sup_camp053::get_text_of_deep_child_node(ntry, "AccptncDtTm")
                .ok_or(ParseError::Camt053MissingElement {
                element: "AccptncDtTm",
            })?;
            let service_bank = sup_camp053::get_text_of_deep_child_node(ntry, "AcctSvcrRef")
                .unwrap_or_default()
                .to_string();

            let date_time = NaiveDateTime::parse_from_str(accptnc_dt_tm_str, "%Y-%m-%dT%H:%M:%S")
                .map_err(|e| ParseError::Camt053DateTimeParse {
                value: accptnc_dt_tm_str.to_string(),
                format: "%Y-%m-%dT%H:%M:%S",
                source: e,
            })?;

            transactions.push(Transaction {
                id,
                credit_account,
                debit_account,
                date_time,
                amount,
                currency,
                credit_debit,
                service_bank,
                purpose,
                transaction_type: Some(sub_fmly_cd.to_owned()),
            });
        }

        let output = vec![Wallet {
            id: 0, // not found in test sample
            bank_maintainer,
            description: "0".to_owned(),
            account,
            currency,
            statement_id: msg_id,
            statement_period_start,
            statement_period_end,
            creation_time,
            opening_balance,
            closing_balance,
            transactions,
        }];

        Ok(output)
    }
    fn parse_mt940_from_str(&mut self, input: &str) -> Result<Vec<Wallet>, errors::ParseError> {
        sup_mt940::parse_mt940_alt(input)
    }
}

/// Supported input formats for financial data parsing.
#[derive(Debug, Clone)]
pub enum InputParserFormat {
    /// Extended CSV format by Sberbank with additional financial fields.
    CsvExtraFin,
    /// ISO 20022 camt.053 XML bank statement format.
    Camt053,
    /// SWIFT MT940 customer statement message format.
    Mt940,
}

impl fmt::Display for InputParserFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InputParserFormat::CsvExtraFin => write!(f, "csv_extra_fin"),
            InputParserFormat::Mt940 => write!(f, "mt_940"),
            InputParserFormat::Camt053 => write!(f, "camt_053"),
        }
    }
}

impl std::str::FromStr for InputParserFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "csv_extra_fin" => Ok(InputParserFormat::CsvExtraFin),
            "camt_053" => Ok(InputParserFormat::Camt053),
            "mt_940" => Ok(InputParserFormat::Mt940),
            _ => Err(format!(
                "Unsupported format: {}. Supported: csv_extra_fin, camt_053, mt_940",
                s
            )),
        }
    }
}

impl InputParserFormat {
    /// Returns a slice oёf all supported input parser formats.
    ///
    /// This list includes only the formats currently enabled for parsing input data.
    pub fn all_variants() -> &'static [InputParserFormat] {
        &[
            InputParserFormat::CsvExtraFin,
            InputParserFormat::Mt940,
            InputParserFormat::Camt053,
        ]
    }
}

/// Specifies the output format for financial data conversion.
///
/// This enum defines the supported serialization formats that the converter can
/// produce. Each variant corresponds to a specific structured representation
/// commonly used in financial data interchange.
///
/// The enum implements [`strum_macros::EnumString`], allowing parsing from
/// string representations (case-sensitive) such as:
/// - `"csv_extra_fin"` or `"CsvExtraFin"` → [`OutputParserFormat::CsvExtraFin`]
/// - `"yaml"` → [`OutputParserFormat::Yaml`]
/// - `"camt_053"` → [`OutputParserFormat::Camt053`]
/// - `"mt_940"` → [`OutputParserFormat::Mt940`]
///
/// # Variants
/// - **`CsvExtraFin`**: Extended CSV format tailored for financial records by Sberbank,
///   typically including additional metadata or normalized fields beyond basic CSV.
/// - **`Yaml`**: Human-readable YAML serialization of the financial data structure.
/// - **`Camt053`**: ISO 20022 `camt.053` XML message format, used for bank statement reporting.
/// - **`Mt940`**: SWIFT MT940 structured narrative format, commonly used in bank-to-customer stat
#[derive(Debug, Clone, strum_macros::EnumString)]
pub enum OutputParserFormat {
    /// Extended CSV format by sberbank with additional financial fields.
    #[strum(serialize = "csv_extra_fin", serialize = "CsvExtraFin")]
    CsvExtraFin,
    /// YAML serialization of parsed financial data.
    #[strum(serialize = "yaml")]
    Yaml,
    /// ISO 20022 camt.053 XML bank statement format.
    #[strum(serialize = "camt_053")]
    Camt053,
    /// SWIFT MT940 customer statement message format.
    #[strum(serialize = "mt_940")]
    Mt940,
}
impl fmt::Display for OutputParserFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OutputParserFormat::CsvExtraFin => write!(f, "csv_extra_fin"),
            OutputParserFormat::Yaml => write!(f, "yaml"),
            OutputParserFormat::Mt940 => write!(f, "mt_940"),
            OutputParserFormat::Camt053 => write!(f, "camt_053"),
        }
    }
}

impl OutputParserFormat {
    /// Returns a slice of all currently supported output formats.
    pub fn all_variants() -> &'static [OutputParserFormat] {
        &[
            OutputParserFormat::CsvExtraFin,
            OutputParserFormat::Yaml,
            OutputParserFormat::Mt940,
            OutputParserFormat::Camt053,
        ]
    }
}

/// A bidirectional I/O adapter that converts financial data from one format to another.
///
/// Implements [`std::io::Write`] to accept input data (e.g., MT940, CAMT.053, EXTRAFINCSV),
/// buffers and decodes it (with UTF-8 and encoding fallback support), then parses
/// and converts it to the target output format. Also implements [`std::io::Read`]
/// to emit the resulting serialized data.
///
/// The conversion is triggered on [`flush()`], after which the output can be read.
/// Intermediate results are also written to YAML files in the `log_dir` for debugging.
pub struct FinConverter {
    // Input state (for Write)
    process_input_type: InputParserFormat,
    process_output_type: OutputParserFormat,
    input_byte_buffer: Vec<u8>, // накапливаем сырые байты для поддержки кириллицы
    flushed: bool,
    input_buffer: String,
    output_bytes: Vec<u8>,
    read_pos: usize,
    log_dir: std::path::PathBuf,
}

impl FinConverter {
    /// Creates a new `FinConverter` for converting between the specified input and output formats.
    pub fn new(
        process_input_type: InputParserFormat,
        process_output_type: OutputParserFormat,
    ) -> Self {
        Self {
            process_input_type,
            process_output_type,
            input_byte_buffer: Vec::new(),
            input_buffer: String::new(),
            flushed: false,
            output_bytes: Vec::new(),
            read_pos: 0,
            log_dir: std::path::PathBuf::from("output").join("debug_yamls"),
        }
    }

    fn process_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.flushed {
            return Ok(());
        }

        let mut parser: UniParser = UniParser::default();
        parser.log_dir = self.log_dir.clone();
        let result_statement_data = match self.process_input_type {
            InputParserFormat::CsvExtraFin => {
                parser.parse_csv_extra_fin_from_str(&self.input_buffer)
            }
            InputParserFormat::Camt053 => parser.parse_camt053_from_str(&self.input_buffer),
            InputParserFormat::Mt940 => parser.parse_mt940_from_str(&self.input_buffer),
        };

        let parsed_account_data = result_statement_data?;
        let report_string: String = serde_yaml::to_string(&parsed_account_data)?;

        let gen_output_name = format!(
            "from_{}_to_{}_{}.yaml",
            self.process_input_type,
            self.process_output_type,
            gen_time_prefix_to_filename()
        );
        let output_path = self.log_dir.join(gen_output_name);

        std::fs::create_dir_all(&self.log_dir)?;
        let mut file = std::fs::File::create(output_path)?;
        let _ = file.write_all(report_string.as_bytes());

        // std::fs::write(&output_path, yaml_string)
        //     .with_context(|| format!("Failed to write YAML: {}", output_path))?;

        // self.output_bytes = parser.account_to_yaml_bytes(account_data);
        let rendered_result = match self.process_output_type {
            // OutputParserFormat::Csv => render::render_content_as_csv(parsed_account_data),
            OutputParserFormat::CsvExtraFin => {
                render::render_content_as_csv_extra_fin(parsed_account_data)
            }
            OutputParserFormat::Yaml => render::render_content_as_yaml(parsed_account_data),
            OutputParserFormat::Camt053 => render::render_content_as_camt053(parsed_account_data),
            OutputParserFormat::Mt940 => render::render_content_as_mt940(parsed_account_data),
        };

        // ^^^ вот тут - клёво. Я бы, если честно, вот это и оставил в parse_input_and_serialize_via_trait,
        // убрав всю flush-магию :) Но ты - автор, волен делать как хочешь :)

        self.output_bytes = rendered_result?;
        let mut output_format_str = format!("output_format: {}\n", self.process_output_type)
            .as_bytes()
            .to_vec();
        let mut input_format_str = format!("input_format: {}\n", self.process_input_type)
            .as_bytes()
            .to_vec();
        // self.output_bytes.append(&mut input_format_str);
        // self.output_bytes.append(&mut output_format_str);

        self.flushed = true;
        Ok(())
    }
}

use chardetng::EncodingDetector;

use crate::parser::common::gen_time_prefix_to_filename;
use crate::parser::errors::ParseError;
use crate::parser::sup_camp053::{find_nested_text, get_text, get_text_or_error};
use crate::parser::sup_extra_fin_csv::write_debug_file;
// use quick_xml::events::Event;

fn detect_and_decode(buf: &[u8]) -> String {
    let mut detector = EncodingDetector::new();
    detector.feed(buf, true); // true = last buffer
    let encoding = detector.guess(None, true);
    let (cow, ..) = encoding.decode(buf);
    cow.into_owned()
}
// detector.

impl Write for FinConverter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Просто добавляем байты — не пытаемся сразу декодировать
        self.input_byte_buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // Теперь, на flush, можно безопасно попытаться декодировать
        // Всё накопленное как UTF-8 (или через detect_and_decode)
        // Потому что flush означает: "вход завершён"
        let input_str = match std::str::from_utf8(&self.input_byte_buffer) {
            Ok(s) => s.to_string(),
            Err(_) => detect_and_decode(&self.input_byte_buffer),
        };
        self.input_buffer.push_str(&input_str);
        let _precess_result = self.process_data();
        // self.process_string_input();
        // self.flushed = true;

        self.flushed = true;
        Ok(())
    }
}

// Read apply to buffer of converter
impl Read for FinConverter {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.read_pos >= self.output_bytes.len() {
            return Ok(0); // EOF
        }

        let remaining = self.output_bytes.len() - self.read_pos;
        let to_copy = std::cmp::min(buf.len(), remaining);
        buf[..to_copy].copy_from_slice(&self.output_bytes[self.read_pos..self.read_pos + to_copy]);
        self.read_pos += to_copy;
        Ok(to_copy)
    }
}

/// ===== Example usage with stdio and BufReader/BufWriter =====
/// Parses financial input data and serializes it to the specified output format.
///
/// Reads from `input_buff_reader`, converts the data from `process_input_type`
/// to `process_output_type` using an internal `FinConverter`, and writes the
/// result to `output_buff_writer`.
///
/// # Errors
/// Returns an `io::Error` if reading, writing, parsing, or conversion fails.
pub fn parse_input_and_serialize_via_trait<TypeOfBuffInput: Read, TypeOfBuffOutput: Write>(
    mut input_buff_reader: TypeOfBuffInput,
    mut output_buff_writer: TypeOfBuffOutput,
    process_input_type: InputParserFormat,
    process_output_type: OutputParserFormat,
) -> io::Result<()> {
    // Create our transformer
    let mut converter = FinConverter::new(process_input_type, process_output_type);

    std::io::copy(&mut input_buff_reader, &mut converter)?;

    converter.flush()?;

    std::io::copy(&mut converter, &mut output_buff_writer)?;

    Ok(())
}

mod tests;
