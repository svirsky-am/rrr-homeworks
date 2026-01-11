use chrono::{Datelike, NaiveDate};

use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;

use crate::parser::{Wallet, common::BalanceAdjustType};

pub fn render_content_as_yaml(
    input_vec: Vec<Wallet>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let iner_result_content = serde_yaml::to_string(&input_vec).expect("Can't convert to YAML");
    Ok(iner_result_content.as_bytes().to_vec())
}

type SharedDepth = Rc<RefCell<usize>>;

pub struct RrXmlTag {
    node_name: String,
    depth_at_open: usize,
    writer: *mut Writer<Cursor<Vec<u8>>>,
    depth_ref: SharedDepth,
}

impl RrXmlTag {
    pub fn open(
        node_name: String,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        depth_ref: SharedDepth,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let current_depth = *depth_ref.borrow();
        write_indent(writer, current_depth)?;
        writer.write_event(Event::Start(BytesStart::new(&node_name)))?;
        writer.write_event(Event::Text(BytesText::from_escaped("\n")))?;
        *depth_ref.borrow_mut() += 1;

        Ok(RrXmlTag {
            node_name,
            depth_at_open: current_depth,
            writer: writer as *mut _,
            depth_ref,
        })
    }

    pub fn close(self) -> Result<(), Box<dyn std::error::Error>> {
        let writer = unsafe { &mut *self.writer };
        let depth_ref = self.depth_ref;

        *depth_ref.borrow_mut() = self.depth_at_open;

        write_indent(writer, self.depth_at_open)?;
        writer.write_event(Event::End(BytesEnd::new(&self.node_name)))?;
        writer.write_event(Event::Text(BytesText::from_escaped("\n")))?;
        // std::mem::forget(self);
        Ok(())
    }
}

fn write_indent(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    level: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let indent = "  ".repeat(level);
    writer.write_event(Event::Text(BytesText::from_escaped(&indent)))?;
    Ok(())
}

fn add_child_event_with_text(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    depth: usize,
    node_name: &str,
    node_text: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    write_indent(writer, depth)?;
    writer.write_event(Event::Start(BytesStart::new(node_name)))?;
    writer.write_event(Event::Text(BytesText::from_escaped(node_text)))?;
    writer.write_event(Event::End(BytesEnd::new(node_name)))?;
    writer.write_event(Event::Text(BytesText::from_escaped("\n")))?;
    Ok(())
}

fn add_child_event_with_attrs_and_text(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    depth: usize,
    node_name: &str,
    node_text: &str,
    inner_attr: (&str, &str),
) -> Result<(), Box<dyn std::error::Error>> {
    write_indent(writer, depth)?;

    let mut start_tag = BytesStart::new(node_name);
    start_tag.push_attribute((inner_attr.0, inner_attr.1));
    writer.write_event(Event::Start(start_tag))?;
    writer.write_event(Event::Text(BytesText::from_escaped(node_text)))?;
    writer.write_event(Event::End(BytesEnd::new(node_name)))?;
    writer.write_event(Event::Text(BytesText::from_escaped("\n")))?;
    Ok(())
}

pub fn render_content_as_camt053(
    input_vec: Vec<Wallet>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let depth_ref = Rc::new(RefCell::new(0));

    // XML declaration
    let decl = BytesDecl::new("1.0", Some("UTF-8"), None);
    writer.write_event(Event::Decl(decl))?;
    writer.write_event(Event::Text(BytesText::from_escaped("\n")))?;

    // <Document>
    write_indent(&mut writer, *depth_ref.borrow())?;
    *depth_ref.borrow_mut() += 1;

    let mut document = BytesStart::new("Document");
    document.push_attribute(("xmlns", "urn:iso:std:iso:20022:tech:xsd:camt.053.001.02"));
    document.push_attribute(("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"));
    document.push_attribute((
        "xsi:schemaLocation",
        "urn:iso:std:iso:20022:tech:xsd:camt.053.001.02 camt.053.001.02.xsd",
    ));
    writer.write_event(Event::Start(document))?;
    writer.write_event(Event::Text(BytesText::from_escaped("\n")))?;

    for cash_statement_data in &input_vec {
        let statement_id = &cash_statement_data.statement_id;
        let creation_time = cash_statement_data
            .creation_time
            .expect("Can't get creation time.")
            .format("%Y-%m-%dT%H:%M:%S")
            .to_string();

        let bk_to_cstmr_stmt_tag =
            RrXmlTag::open("BkToCstmrStmt".to_string(), &mut writer, depth_ref.clone())?;

        {
            let grp_hdr_tag = RrXmlTag::open("GrpHdr".to_string(), &mut writer, depth_ref.clone())?;
            add_child_event_with_text(&mut writer, *depth_ref.borrow(), "MsgId", statement_id)?;
            add_child_event_with_text(&mut writer, *depth_ref.borrow(), "CreDtTm", &creation_time)?;
            grp_hdr_tag.close()?;
        }

        {
            let stmt_tag = RrXmlTag::open("Stmt".to_string(), &mut writer, depth_ref.clone())?;
            add_child_event_with_text(&mut writer, *depth_ref.borrow(), "Id", statement_id)?;
            add_child_event_with_text(&mut writer, *depth_ref.borrow(), "CreDtTm", &creation_time)?;

            let fr_to_dt_tag =
                RrXmlTag::open("FrToDt".to_string(), &mut writer, depth_ref.clone())?;
            add_child_event_with_text(
                &mut writer,
                *depth_ref.borrow(),
                "FrDtTm",
                &cash_statement_data
                    .statement_period_start
                    .format("%Y-%m-%dT%H:%M:%S")
                    .to_string(),
            )?;
            add_child_event_with_text(
                &mut writer,
                *depth_ref.borrow(),
                "ToDtTm",
                &cash_statement_data
                    .statement_period_end
                    .format("%Y-%m-%dT%H:%M:%S")
                    .to_string(),
            )?;
            fr_to_dt_tag.close()?;

            let acct_tag = RrXmlTag::open("Acct".to_string(), &mut writer, depth_ref.clone())?;
            let id_tag = RrXmlTag::open("Id".to_string(), &mut writer, depth_ref.clone())?;
            add_child_event_with_text(
                &mut writer,
                *depth_ref.borrow(),
                "IBAN",
                &cash_statement_data.account,
            )?;
            add_child_event_with_text(
                &mut writer,
                *depth_ref.borrow(),
                "Ccy",
                &cash_statement_data.currency,
            )?;

            id_tag.close()?;
            acct_tag.close()?;
            {
                let open_bal_tag =
                    RrXmlTag::open("Bal".to_string(), &mut writer, depth_ref.clone())?;
                add_child_event_with_attrs_and_text(
                    &mut writer,
                    *depth_ref.borrow(),
                    "Amt",
                    &cash_statement_data
                        .opening_balance
                        .clone()
                        .expect("Can't get balance.")
                        .amount
                        .to_string(),
                    ("Ccy", &cash_statement_data.currency),
                )?;
                let open_dt = match &cash_statement_data
                    .opening_balance
                    .clone()
                    .expect("Can't get balance.")
                    .credit_debit
                {
                    BalanceAdjustType::Debit => "DBIT",
                    BalanceAdjustType::Credit => "CRDT",
                    BalanceAdjustType::WithoutInfo => "DBIT",
                };
                add_child_event_with_text(&mut writer, *depth_ref.borrow(), "CdtDbtInd", open_dt)?;
                let open_dt_1_tag =
                    RrXmlTag::open("Dt".to_string(), &mut writer, depth_ref.clone())?;
                add_child_event_with_text(
                    &mut writer,
                    *depth_ref.borrow(),
                    "Dt",
                    &cash_statement_data
                        .opening_balance
                        .clone()
                        .expect("Can't get balance.")
                        .date
                        .format("%Y-%m-%d")
                        .to_string(),
                )?;
                open_dt_1_tag.close()?;
                open_bal_tag.close()?;

                let close_bal_tag =
                    RrXmlTag::open("Bal".to_string(), &mut writer, depth_ref.clone())?;
                add_child_event_with_text(
                    &mut writer,
                    *depth_ref.borrow(),
                    "Amt",
                    &cash_statement_data
                        .closing_balance
                        .clone()
                        .expect("Can't get balance.")
                        .amount
                        .to_string(),
                )?;
                add_child_event_with_attrs_and_text(
                    &mut writer,
                    *depth_ref.borrow(),
                    "Amt",
                    &cash_statement_data
                        .closing_balance
                        .clone()
                        .expect("Can't get balance.")
                        .amount
                        .to_string(),
                    ("Ccy", &cash_statement_data.currency),
                )?;
                let close_dt = match &cash_statement_data
                    .closing_balance
                    .clone()
                    .expect("Can't get debet_credit info.")
                    .credit_debit
                {
                    BalanceAdjustType::Debit => "DBIT",
                    BalanceAdjustType::Credit => "CRDT",
                    BalanceAdjustType::WithoutInfo => "DBIT",
                };
                add_child_event_with_text(&mut writer, *depth_ref.borrow(), "CdtDbtInd", close_dt)?;
                let close_dt_1_tag =
                    RrXmlTag::open("Dt".to_string(), &mut writer, depth_ref.clone())?;
                add_child_event_with_text(
                    &mut writer,
                    *depth_ref.borrow(),
                    "Dt",
                    &cash_statement_data
                        .closing_balance
                        .clone()
                        .expect("Can't get datetime of Balance.")
                        .date
                        .format("%Y-%m-%d")
                        .to_string(),
                )?;
                close_dt_1_tag.close()?;
                close_bal_tag.close()?;
            }
            {
                let mut transaction_count = 1;
                for tr in &cash_statement_data.transactions {
                    let ntry_tag =
                        RrXmlTag::open("Ntry".to_string(), &mut writer, depth_ref.clone())?;
                    add_child_event_with_text(
                        &mut writer,
                        *depth_ref.borrow(),
                        "NtryRef",
                        &transaction_count.to_string(),
                    )?;
                    let d_c = match &tr.credit_debit {
                        BalanceAdjustType::Debit => "DBIT",
                        BalanceAdjustType::Credit => "CRDT",
                        BalanceAdjustType::WithoutInfo => "DBIT",
                    };
                    add_child_event_with_attrs_and_text(
                        &mut writer,
                        *depth_ref.borrow(),
                        "Amt",
                        &tr.amount.to_string(),
                        ("Ccy", &tr.currency),
                    )?;
                    add_child_event_with_text(&mut writer, *depth_ref.borrow(), "CdtDbtInd", d_c)?;
                    add_child_event_with_text(
                        &mut writer,
                        *depth_ref.borrow(),
                        "AcctSvcrRef",
                        &tr.service_bank,
                    )?;
                    let _bk_tx_cd_tag =
                        RrXmlTag::open("BkTxCd".to_string(), &mut writer, depth_ref.clone())?;
                    let domn_tag =
                        RrXmlTag::open("Domn".to_string(), &mut writer, depth_ref.clone())?;
                    let domn_cd_tag =
                        RrXmlTag::open("Cd".to_string(), &mut writer, depth_ref.clone())?;
                    let transaction_type = &tr
                        .transaction_type
                        .clone()
                        .unwrap_or("None".to_string())
                        .to_string();
                    add_child_event_with_text(
                        &mut writer,
                        *depth_ref.borrow(),
                        "SubFmlyCd",
                        transaction_type,
                    )?;
                    let _ = domn_cd_tag.close();
                    let _ = domn_tag.close();

                    let _prtry_tag =
                        RrXmlTag::open("Prtry".to_string(), &mut writer, depth_ref.clone())?;
                    let tr_direction = match &tr.credit_debit {
                        BalanceAdjustType::Debit => &tr.debit_account,
                        BalanceAdjustType::Credit => &tr.credit_account,
                        BalanceAdjustType::WithoutInfo => &tr.credit_account,
                    };
                    add_child_event_with_text(
                        &mut writer,
                        *depth_ref.borrow(),
                        "Cd",
                        tr_direction,
                    )?;
                    let _ = _prtry_tag.close()?;

                    transaction_count += 1;
                    ntry_tag.close()?;
                }
            }

            stmt_tag.close()?;
        }

        bk_to_cstmr_stmt_tag.close()?;
    }
    *depth_ref.borrow_mut() = 0;
    write_indent(&mut writer, 0)?;
    writer.write_event(Event::End(BytesEnd::new("Document")))?;
    writer.write_event(Event::Text(BytesText::from_escaped("\n")))?;

    let xml_bytes = writer.into_inner().into_inner();
    Ok(xml_bytes)
}

pub fn render_content_as_mt940(
    input_vec: Vec<Wallet>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut iner_result_content = String::new();
    for cash_statement_data in &input_vec {
        let _date_of_statemant = cash_statement_data
            .creation_time
            .clone()
            .expect("Can't get datetime of Balance.");
        let account_id = &cash_statement_data.id;
        let bank_maintainer = &cash_statement_data.bank_maintainer;

        let account_name = &cash_statement_data.account;
        let _currency: &String = &cash_statement_data.currency;
        let statement_id = &cash_statement_data.statement_id;
        let _statement_start_format = cash_statement_data
            .statement_period_start
            .date()
            .format("%y%m%d")
            .to_string();

        let opening_balance = &cash_statement_data
            .opening_balance
            .clone()
            .expect("Can't get datetime of Balance.");
        let open_balance_amount = opening_balance.amount.to_string().replace(".", ",");
        let open_balance_data = opening_balance.date.format("%y%m%d").to_string();
        let open_balance_currency = &opening_balance.currency;

        iner_result_content.push_str(&format!(
            "{{1:F01{bank_maintainer}0000000000}}{{2:O940{bank_maintainer}N}}{{3:}}{{4:
:20:{account_id}
:25:{account_name}
:28C:{statement_id}
:60F:C{open_balance_data}{open_balance_currency}{open_balance_amount}
"
        ));
        for tr in &cash_statement_data.transactions {
            let date_time = tr.date_time.format("%y%m%d%m%d").to_string();
            let amount = tr.amount.to_string().replace(".", ",");

            let transaction_type = tr.transaction_type.clone().unwrap_or("non".to_owned());
            let _transaction_id = &tr.id;
            let (debit_credit, tr_direction) = match tr.credit_debit {
                BalanceAdjustType::Debit => ("D".to_owned(), tr.credit_account.clone()),

                BalanceAdjustType::Credit => ("C".to_owned(), tr.debit_account.clone()),
                BalanceAdjustType::WithoutInfo => ("C".to_owned(), tr.credit_account.clone()),
            };
            let description = &tr.purpose;
            iner_result_content.push_str(&format!(
                ":61:{date_time}R{debit_credit}{amount}N{transaction_type}{tr_direction}
:86:{description}\n"
            ));
        }

        let closing_balance = &cash_statement_data
            .closing_balance
            .clone()
            .expect("Can't get clising Balance.");
        let closing_balance_amount = closing_balance.amount.to_string().replace(".", ",");
        let closing_balance_data = closing_balance.date.format("%y%m%d").to_string();
        let closing_balance_currency = &closing_balance.currency;

        let _statement_end_format = cash_statement_data
            .statement_period_end
            .date()
            .format("%y%m%d")
            .to_string();

        iner_result_content.push_str(&format!(
            ":62F:C{closing_balance_data}{closing_balance_currency}{closing_balance_amount}
-}}{{5:}}\n"
        ));
        iner_result_content.push_str(&format!(
            ":62F:C{closing_balance_data}{closing_balance_currency}{closing_balance_amount}
-}}{{5:}}\n"
        ));
    }

    Ok(iner_result_content.as_bytes().to_vec())
}

fn format_russian_naive_date(input_date: NaiveDate) -> String {
    static MONTHS: [&str; 12] = [
        "января",
        "февраля",
        "марта",
        "апреля",
        "мая",
        "июня",
        "июля",
        "августа",
        "сентября",
        "октября",
        "ноября",
        "декабря",
    ];
    let m_index = input_date
        .month()
        .to_string()
        .parse::<usize>()
        .expect("Can't parse date as russian fromat.")
        - 1;
    let month_name = MONTHS[m_index];
    format!(
        "{:02} {} {}",
        input_date.day(),
        month_name,
        input_date.year()
    )
}

pub fn render_content_as_csv_extra_fin(
    input_vec: Vec<Wallet>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut iner_result_content = String::new();
    for cash_statement_data in &input_vec {
        let datetime_of_statemant = cash_statement_data
            .creation_time
            .clone()
            .expect("Can't get datetime of Balance.");
        let creation_date = &datetime_of_statemant.format("%d.%m.%Y").to_string();
        let creation_datetime = &datetime_of_statemant
            .format("%d.%m.%Y в %H:%M:%S")
            .to_string();
        let account_id = &cash_statement_data.id;
        let account_name = &cash_statement_data.account;
        let bank_maintainer = &cash_statement_data.bank_maintainer;
        let currency = &cash_statement_data.currency;
        let statement_id = &cash_statement_data.statement_id;
        let statement_period_start =
            format_russian_naive_date(cash_statement_data.statement_period_start.date());
        let statement_period_end =
            format_russian_naive_date(cash_statement_data.statement_period_end.date());
        iner_result_content.push_str(&format!(",,,,,,,,,,,,,,,,,,,,,,
,{creation_date},,,,СберБизнес. {statement_id},,,,,,,,,,,,,,,,,
,\"{bank_maintainer}\",,,,,,,,,,,,,,,,,,,,,
,Дата формирования выписки {creation_datetime},,,,,,,,,,,,,,,,,,,,,
,ВЫПИСКА ОПЕРАЦИЙ ПО ЛИЦЕВОМУ СЧЕТУ,,,,,,,,,,,{account_id},,,,,,,,,,
,,,,,,,,,,,,{account_name},,,,,,,,,,
,,за период с {statement_period_start} г.,,,,,,,,,,,, по ,{statement_period_end} г.,,,,,,,
,,{currency},,,,,,,,,,Дата предыдущей операции по счету TODo г. ,,,,,,,,,,
,,,,,,,,,,,,,,,,,,,,,,
,Дата проводки,,,Счет,,,,,Сумма по дебету,,,,Сумма по кредиту,№ документа,,ВО,Банк (БИК и наименование),,,Назначение платежа,,
,,,,Дебет,,,,Кредит,,,,,,,,,,,,,,\n"
));

        let mut wtr = csv::Writer::from_writer(Vec::new());
        for statement in &input_vec {
            for tr in &statement.transactions {
                let mut record = vec![String::new(); 23];
                record[1] = tr.date_time.format("%d.%m.%Y").to_string();
                record[4] = tr.debit_account.to_string();
                record[8] = tr.credit_account.to_owned();
                record[14] = tr.id.to_string();
                record[16] = "01".to_string(); // ВО 1/17 ?
                record[17] = tr.service_bank.to_owned();
                record[20] = tr.purpose.replace(['\n', '\r'], " "); // sanitize
                match tr.credit_debit {
                    BalanceAdjustType::Debit => {
                        record[9] = format!("{:.2}", tr.amount);
                    }
                    BalanceAdjustType::Credit => {
                        record[13] = format!("{:.2}", tr.amount);
                    }
                    BalanceAdjustType::WithoutInfo => {
                        continue;
                    }
                }
                let _ = wtr.write_record(&record);
            }
        }
        wtr.flush().expect("Can't flush CSV data to output buff.");
        let csv_bytes = wtr
            .into_inner()
            .expect("Can't convert buff with CSV content  to raw bytes.");
        let csv_string = String::from_utf8(csv_bytes).expect("Can't convert raw bytes to String.");
        iner_result_content.push_str(&csv_string);
        let opening_balance = &cash_statement_data
            .opening_balance
            .clone()
            .expect("Can't get info of Balance.");
        let open_balance_amount = opening_balance.amount.to_string().replace(".", ",");
        let closing_balance = &cash_statement_data
            .closing_balance
            .clone()
            .expect("Can't get info of Balance.");
        let closing_balance_amount = closing_balance.amount.to_string().replace(".", ",");

        let open_balance_data_format = format_russian_naive_date(opening_balance.date);
        let closing_balance_format = format_russian_naive_date(closing_balance.date);

        iner_result_content.push_str(&format!(",,,,,,,,,,,,,,,,,,,,,,,
,б/с,,40702,,,,Дебет,,,,Кредит,,,,,,,,Всего,,,
,,,,,,,,,,,,,,,,,,,,,,
,Количество операций,,,,,,26,,,,6,,,,,,,,32,,,
,Входящий остаток,,,,,,\"0,00\",,,,{open_balance_amount},,,,,,(П),,{open_balance_data_format} г.,,,
,Итого оборотов,,,,,,TODO_DIFF,,,,TODO_DIFF,,,,,,,,,,,
,Исходящий остаток,,,,,,\"0,00\",,,,{closing_balance_amount},,,,,,(П),,{closing_balance_format} г.,,,
,,,,,,,,,,,,,,,,,,,,,,
,,,,,,,,,,,,,,,,,,,,,,\n"
        ));
    }
    Ok(iner_result_content.as_bytes().to_vec())
}
