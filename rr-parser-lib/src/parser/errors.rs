// use std::fmt;

// /// // Wrapping another error
// /// let dt = NaiveDateTime::parse_from_str(date_str, "%d.%m.%Y")
// ///     .map_err(|e| ParseError::from(Box::new(e)))?;

// /// // Or just a custom message
// /// return Err(ParseError::from("Failed to match header regex"));

use chrono::{NaiveDate, NaiveDateTime};
use thiserror::Error;
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Failed to match header regex")]
    ExtraFinHeaderNotMatched,

    #[error("Invalid creation time '{date_str}': {source}")]
    ExtraFinInvalidCreationTime {
        date_str: String,
        #[source]
        source: chrono::ParseError,
    },

    #[error("Invalid parse russian data: {source}")]
    ExtraFinInvalidParseRussianDate {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Missing or invalid client ID: {source}")]
    ExtraFinInvalidClientId {
        #[source]
        source: std::num::ParseIntError,
    },
    //  XML/camt053  section
    #[error("Failed to parse XML document: {source}")]
    Camt053XmlParse {
        #[source]
        source: roxmltree::Error,
    },

    #[error("Missing required XML element: {element}")]
    Camt053MissingElement { element: &'static str },

    #[error("Failed to parse datetime '{value}' with format '{format}': {source}")]
    Camt053DateTimeParse {
        value: String,
        format: &'static str,
        #[source]
        source: chrono::ParseError,
    },

    #[error("Failed to parse date '{value}' with format '{format}': {source}")]
    Camt053DateParse {
        value: String,
        format: &'static str,
        #[source]
        source: chrono::ParseError,
    },

    #[error("Failed to parse number '{value}': {source}")]
    Camt053NumberParse {
        value: String,
        #[source]
        source: std::num::ParseFloatError,
    },

    #[error("Unexpected or missing transaction code")]
    Camt053InvalidTransactionCode,

    #[error("Missing required text content in XML node")]
    Camt053MissingTextContent,

    // MT940 sectoin
    #[error("Failed to parse mt940: '{value}': {source}")]
    Mt940DateTimeParse {
        value: String,
        #[source]
        source: chrono::ParseError,
    },

    #[error("Missing required capture group in MT940 regex: {field}")]
    Mt940MissingCapture { field: &'static str },

    #[error("Failed to parse balance in MT940 field '{field_value}': {source}")]
    Mt940BalanceParse {
        field_value: String,
        #[source]
        source: chrono::ParseError,
    },

    #[error("Failed to parse amount in MT940: '{value}': {source}")]
    Mt940AmountParse {
        value: String,
        #[source]
        source: std::num::ParseFloatError,
    },

    #[error("Unexpected credit/debit marker in MT940: '{marker}'")]
    Mt940InvalidCreditDebitMarker { marker: String },

    #[error("Invalid MT940 balance format: expected format like CYYMMDDCCYAMOUNT, got '{value}'")]
    Mt940InvalidBalanceFormat { value: String },
}
