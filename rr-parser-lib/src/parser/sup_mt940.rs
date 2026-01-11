use chrono::{NaiveDate, NaiveDateTime};
use lazy_regex::{Lazy, lazy_regex};
use regex::Regex;
use std::collections::HashMap;

use crate::parser::{
    Wallet,
    common::{Balance, BalanceAdjustType, Transaction},
    errors::ParseError,
};

pub static RE_MT940_MSGS_ALL: Lazy<Regex> = lazy_regex!(
    r"[\[\(\{]1\:...(.{1,100})\{2\:.940(\S{1,100})[N]\}.{0,40}\{4\:([^\}]{1,750})-\}\{5.*"
);

pub static RE_MT940_MSG_LINES: Lazy<Regex> = lazy_regex!(r"(:\d{2}[A-Z]?:)");

pub static RE_MT940_TRANSACTIONS_61_86_TR: Lazy<Regex> = lazy_regex!(
    r"(?x)
\:61\:(?P<data_time>\d{6,10})
(?P<debit_credit>[cdCD])R?
        (?P<amount>\d+[\,\.]\d\d)
        (?P<transaction_type_code>\w)(?P<bank_transaction_code>.{3})
                (?P<transaction_id>[\w\/]+)
[\n\w\s]*\:86\:(?P<description_filed>[.\w\s]*)
"
);

///:61
/// Value Date: 2009-09-25
/// Entry Date: 2009-09-25
/// Debit/Credit: D → Debit
/// Amount: 583,92
/// Transaction Type: N → Normal
/// Bank Code: MSC → Debit card / POS terminal
/// Reference: 1110030403010139//1234
pub fn parse_mt940_alt(input: &str) -> Result<Vec<Wallet>, super::errors::ParseError> {
    let mut statement_data_vec: Vec<Wallet> = Vec::new();

    for cap in RE_MT940_MSGS_ALL.captures_iter(input) {
        let bank_maintainer = &cap[2];
        let body: &str = &cap[3];
        let mut fields: Vec<(&str, String)> = Vec::new();

        let mut last_tag = "";
        let mut current_value = String::new();

        for line in body.lines() {
            if let Some(m) = RE_MT940_MSG_LINES.find(line.trim()) {
                if !last_tag.is_empty() {
                    fields.push((last_tag, current_value.trim_end().to_string()));
                }
                last_tag = m.as_str();
                current_value = line[m.end()..].to_string();
            } else {
                current_value.push('\n');
                current_value.push_str(line);
            }
        }
        if !last_tag.is_empty() {
            fields.push((last_tag, current_value.trim_end().to_string()));
        }

        let field_map: HashMap<_, _> = fields.into_iter().collect();
        // Field 20: Transaction reference number
        // Field 25: Account identification
        // Field 28C: Statement number / sequence
        // Field 60F: Opening balance
        // Field 61: Statement lines (individual transactions)
        // Field 86: Optional narrative for each transaction
        // Field 62F: Closing balance
        let account_name_identification = field_map.get(":25:").cloned().unwrap_or_default();

        let parsed_opening_balance = if let Some(v) = field_map.get(":60F:") {
            Some(parse_60f(v)?)
        } else {
            None
        };

        let parsed_closing_balance = if let Some(v) = field_map.get(":62F:") {
            Some(parse_60f(v)?)
        } else {
            None
        };

        let currency = if let Some(ref bal) = parsed_opening_balance {
            bal.currency.clone()
        } else if let Some(ref bal) = parsed_closing_balance {
            bal.currency.clone()
        } else {
            return Err(ParseError::Mt940MissingCapture {
                field: ":60F: or :62F:",
            });
        };

        let transactions: Result<Vec<Transaction>, ParseError> = RE_MT940_TRANSACTIONS_61_86_TR
            .captures_iter(&body)
            .map(|caps| {
                let datetime_str = caps
                    .name("data_time")
                    .ok_or(ParseError::Mt940MissingCapture { field: "data_time" })?
                    .as_str();

                let date_time = NaiveDateTime::parse_from_str(datetime_str, "%y%m%d%H%M").map_err(
                    |source| ParseError::Mt940DateTimeParse {
                        value: datetime_str.to_string(),
                        source,
                    },
                )?;

                let debit_credit_str = caps
                    .name("debit_credit")
                    .ok_or(ParseError::Mt940MissingCapture {
                        field: "debit_credit",
                    })?
                    .as_str();

                let credit_debit = match debit_credit_str {
                    "C" | "c" => BalanceAdjustType::Credit,
                    "D" | "d" => BalanceAdjustType::Debit,
                    _ => {
                        return Err(ParseError::Mt940InvalidCreditDebitMarker {
                            marker: debit_credit_str.to_string(),
                        });
                    }
                };

                let amount_str = caps
                    .name("amount")
                    .ok_or(ParseError::Mt940MissingCapture { field: "amount" })?
                    .as_str();

                let amount = amount_str
                    .replace(',', ".")
                    .parse::<f64>()
                    .map_err(|source| ParseError::Mt940AmountParse {
                        value: amount_str.to_string(),
                        source,
                    })?;

                let description_filed = caps
                    .name("description_filed")
                    .ok_or(ParseError::Mt940MissingCapture {
                        field: "description_filed",
                    })?
                    .as_str();

                let tr_direction = description_filed
                    .split_whitespace()
                    .next()
                    .ok_or(ParseError::Mt940MissingCapture {
                        field: "transaction direction in description",
                    })?
                    .to_string();

                let transaction_id = caps
                    .name("transaction_id")
                    .ok_or(ParseError::Mt940MissingCapture {
                        field: "transaction_id",
                    })?
                    .as_str()
                    .split("//")
                    .next()
                    .ok_or(ParseError::Mt940MissingCapture {
                        field: "transaction_id (split by //)",
                    })?;

                // Note: We assume account_name_identification is non-empty; if empty, logic may break
                let (credit_account, debit_account) = match credit_debit {
                    BalanceAdjustType::Debit => {
                        (tr_direction.clone(), account_name_identification.clone())
                    }
                    BalanceAdjustType::Credit => {
                        (account_name_identification.clone(), tr_direction.clone())
                    }
                    BalanceAdjustType::WithoutInfo => {
                        // This shouldn't happen due to match above
                        return Err(ParseError::Mt940InvalidCreditDebitMarker {
                            marker: "WithoutInfo".into(),
                        });
                    }
                };

                Ok(Transaction {
                    id: transaction_id.to_owned(),
                    currency: currency.clone(),
                    date_time,
                    credit_debit,
                    amount,
                    transaction_type: caps
                        .name("bank_transaction_code")
                        .map(|m| m.as_str().to_owned()),
                    credit_account,
                    debit_account,
                    service_bank: String::new(),
                    purpose: description_filed.trim().to_owned(),
                })
            })
            .collect();

        let transactions = transactions?;

        let mut wallet = Wallet::default();
        wallet.bank_maintainer = bank_maintainer.to_string();
        wallet.account = account_name_identification;
        wallet.statement_id = field_map.get(":28C:").cloned().unwrap_or_default();
        wallet.opening_balance = parsed_opening_balance;
        wallet.closing_balance = parsed_closing_balance;
        wallet.transactions = transactions;

        statement_data_vec.push(wallet);
    }

    Ok(statement_data_vec)
}
/// Format: CYYMMDDCCYAMOUNT
/// Example: C200101EUR444,29
fn parse_60f(s: &str) -> Result<Balance, ParseError> {
    if s.is_empty() {
        return Err(ParseError::Mt940InvalidBalanceFormat {
            value: s.to_string(),
        });
    }

    let dc_mark = s.chars().next().unwrap(); // safe after empty check

    let credit_debit = match dc_mark {
        'D' => BalanceAdjustType::Debit,
        'C' => BalanceAdjustType::Credit,
        _ => {
            return Err(ParseError::Mt940InvalidCreditDebitMarker {
                marker: dc_mark.to_string(),
            });
        }
    };

    // Must have at least 6 (date) + 3 (currency) + 1 (amount) = 10 chars after D/C
    if s.len() < 10 {
        return Err(ParseError::Mt940InvalidBalanceFormat {
            value: s.to_string(),
        });
    }

    let date_str = &s[1..7]; // YYMMDD
    let date = parse_yymmdd(date_str).map_err(|source| ParseError::Mt940BalanceParse {
        field_value: s.to_string(),
        source,
    })?;

    let currency = s[7..10].to_string();
    let amount_str = &s[10..];

    // Clean up amount (remove newlines, negative signs in wrong place, etc.)
    let cleaned_amount = amount_str
        .replace("\n", "")
        .replace("\r", "")
        .replace('-', ""); // be cautious with '-'
    let amount_clean = cleaned_amount.replace(',', ".");

    let amount = amount_clean
        .parse::<f64>()
        .map_err(|source| ParseError::Mt940AmountParse {
            value: amount_str.to_string(),
            source,
        })?;

    Ok(Balance {
        amount,
        currency,
        date,
        credit_debit,
        last_ops: Vec::new(),
    })
}
fn parse_yymmdd(s: &str) -> Result<NaiveDate, chrono::ParseError> {
    NaiveDate::parse_from_str(s, "%y%m%d")
}
