use chrono::NaiveDate;
use csv::ReaderBuilder;

use crate::parser::common::{BalanceAdjustType, Transaction};

/// Дебет – ушло, Кредит – пришло (если смотреть на счет клиента в его банке, то наоборот, Дт - пришло, Кт - ушло).

/// Если рассматривать счет клиента в банке (зеркально):
/// Дебет (ДТ): Операции, которые увеличивают ваш баланс (поступление зарплаты, перевод от друга, зачисление средств).
/// Кредит (КТ): Операции, которые уменьшают ваш баланс (оплата покупок, снятие наличных, переводы другому человеку).
/// Для счета компании (в бухгалтерии, где банк — кредитор):
/// Дебет (ДТ): Поступления на баланс компании (например, от покупателей).
/// Кредит (КТ): Расходы компании (зарплата, аренда, закупка).
/// Пример из выписки (для личного счета):
/// ДТ 1000 руб. (Покупка в магазине) – деньги ушли с вашего счета.
/// КТ 5000 руб. (Зарплата) – деньги пришли на ваш счет.
/// Для удобства, многие банки заменяют «Дебет/Кредит» на колонки «Приход» (поступления) и «Расход» (списания), чтобы избежать путаницы.

pub fn normalyze_csv_str(input_data: String) -> String {
    let mut work_data = input_data.replace(",\n,", ",__,");
    work_data = work_data.replace("\n", " ");
    work_data = work_data.replace(",__,", ",\n,");
    work_data = work_data.replace(",,,,Дебет,,,,Кредит,,,,,,,,,,,,,,\n", "");
    work_data
}

pub fn parsr_csv_str(
    input_data: String,
) -> Result<Vec<Transaction>, Box<dyn std::error::Error + Send + Sync>> {
    let mut transactions = Vec::new();

    let mut rdr = ReaderBuilder::new()
        .delimiter(b',')
        .has_headers(false) // no proper header row
        .flexible(true) // allow varying number of columns per row
        .from_reader(input_data.as_bytes());

    // Skip first two rows (they are headers/metadata)
    let mut records = rdr.records();
    let _ = records.next(); // first row: empty headers
    let _ = records.next(); // second row: sub-headers

    for result in records {
        let record = result?;
        let date_str = record.get(1).unwrap_or("").trim().to_string();
        if date_str.is_empty() {
            continue;
        } // skip empty lines
        let parsed_date = NaiveDate::parse_from_str(&date_str, "%d.%m.%Y")?;
        let date_time = parsed_date
            .and_hms_opt(0, 0, 0)
            .ok_or("Invalid time (should never happen)")?;

        let debit_account = record
            .get(4)
            .unwrap_or("")
            .replace("\n", " ")
            .trim()
            .to_string();
        let credit_account = record
            .get(8)
            .unwrap_or("")
            .replace("\n", " ")
            .trim()
            .to_string();

        let debit_str = record.get(9).unwrap_or("").trim();
        let credit_str = record.get(13).unwrap_or("").trim();

        let currency = "Rub".to_string();

        let (amount, credit_debit) = match (!credit_str.is_empty(), !debit_str.is_empty()) {
            (true, true) => (0.0, BalanceAdjustType::WithoutInfo),
            (true, false) => (credit_str.parse::<f64>()?, BalanceAdjustType::Credit),
            (false, true) => (debit_str.parse::<f64>()?, BalanceAdjustType::Debit),
            (false, false) => (0.0, BalanceAdjustType::WithoutInfo),
        };

        let doc_number = record.get(14).unwrap_or("").trim().to_string();
        let id_transaction: String = doc_number;
        let service_bank = record
            .get(17)
            .unwrap_or("")
            .replace("\n", " ")
            .trim()
            .to_string();
        let purpose = record
            .get(20)
            .unwrap_or("")
            .replace("\n", " ")
            .trim()
            .to_string();

        let tx = Transaction {
            id: id_transaction,
            credit_account,
            debit_account,
            date_time,
            amount,
            currency,
            credit_debit,
            service_bank,
            purpose,
            transaction_type: None,
        };

        transactions.push(tx);
    }
    Ok(transactions)
}

pub fn write_debug_file(dir: &std::path::Path, name: &str, data: &[u8]) -> std::io::Result<()> {
    let path = dir.join(name);
    let mut file = std::fs::File::create(path)?;
    std::io::Write::write_all(&mut file, data)
}
