//! # Streaming Quotes Project
//!
//! A real-time stock quote streaming system with TCP subscription and UDP data delivery.
//!
//! ## Architecture
//!
//! - **Server**: Listens for TCP subscriptions and streams quotes via UDP
//! - **Client**: Subscribes via TCP and receives quotes via UDP
//! - **Protocol**: `STREAM udp://<addr> <ticker1>,<ticker2>,...`
//!
//! ## Supported Tickers
//!
//! AAPL, MSFT, GOOGL, AMZN, NVDA, META, TSLA, JPM, JNJ, V, PG, UNH, HD, DIS, PYPL, NFLX, ADBE, CRM
//!
//! ## Examples
//! ### help
//! ```sh
//! ./quote_client --help
//! ./quote_server --help
//! ```
//! ### Start server
//! ```sh
//! ./quote_server 8000 8001
//! ``` 
//! ### Start client
//! With qoutes filter as string list: 
//! ```sh
//!  ./quote_client --target-quote-server 127.0.0.1:8001 --filer-lint AAPL,TSLA
//! ```
//! Import qoutes filter from file:
//! ```sh
//! RUST_LOG=info ./target/debug/quote_client --target-quote-server 127.0.0.1:8001 --tickers-file streaming_quotes_project/tests/test_quotes.lst
//! ```
//!

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

use std::time::{SystemTime, UNIX_EPOCH};
pub mod logging;

pub use logging::*;
pub mod errors;
pub use errors::*;

#[derive(Debug, Clone)]
pub struct StockQuote {
    pub ticker: String,
    pub price: f64,
    pub volume: u32,
    pub timestamp: u64,
}

impl StockQuote {
    pub fn to_string(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.ticker, self.price, self.volume, self.timestamp
        )
    }

    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() == 4 {
            Some(StockQuote {
                ticker: parts[0].to_string(),
                price: parts[1].parse().ok()?,
                volume: parts[2].parse().ok()?,
                timestamp: parts[3].parse().ok()?,
            })
        } else {
            None
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.ticker.as_bytes());
        bytes.push(b'|');
        bytes.extend_from_slice(self.price.to_string().as_bytes());
        bytes.push(b'|');
        bytes.extend_from_slice(self.volume.to_string().as_bytes());
        bytes.push(b'|');
        bytes.extend_from_slice(self.timestamp.to_string().as_bytes());
        bytes
    }
}

// Список поддерживаемых тикеров
pub const SUPPORTED_TICKERS: &[&str] = &[
    "AAPL", "MSFT", "GOOGL", "AMZN", "NVDA", "META", "TSLA", "JPM", "JNJ", "V", "PG", "UNH", "HD",
    "DIS", "PYPL", "NFLX", "ADBE", "CRM",
];

pub fn is_supported_ticker(ticker: &str) -> bool {
    SUPPORTED_TICKERS.contains(&ticker)
}


fn get_cur_timestamp () -> u64 {
    SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
}

/// Batch of quotes generated at the same time for all tickers.
#[derive(Debug, Clone)]
pub struct QuoteBatch {
    /// Quotes for all supported tickers.
    pub quotes: Vec<StockQuote>,
    /// Generation timestamp.
    pub timestamp: u64,
}

impl QuoteBatch {
    /// Creates a new batch from individual quotes.
    pub fn new(quotes: Vec<StockQuote>) -> Self {
        let timestamp = get_cur_timestamp();

        QuoteBatch { quotes, timestamp }
    }

    /// Filters the batch to include only specified tickers.
    pub fn filter_tickers(&self, tickers: &[String]) -> Vec<StockQuote> {
        self.quotes
            .iter()
            .filter(|q| tickers.contains(&q.ticker))
            .cloned()
            .collect()
    }
}

// Генератор котировок с "рыночной" логикой
pub struct QuoteGenerator {
    base_prices: std::collections::HashMap<String, f64>,
}

/// Categories of tickers by typical trading volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeCategory {
    /// Very high volume: mega-cap tech stocks
    High,
    /// Medium-high volume: large-cap stocks
    MediumHigh,
    /// Medium volume: established large-cap non-tech
    Medium,
    /// Lower volume: mid-cap or more specialized stocks
    Low,
}

/// Разные категории для большей наглядности
impl VolumeCategory {
    /// Returns the volume category for a given ticker.
    pub fn for_ticker(ticker: &str) -> Self {
        match ticker {
            // Mega-cap tech: очень высокий объём
            "AAPL" | "MSFT" | "NVDA" | "TSLA" | "META" | "AMZN" | "GOOGL" => VolumeCategory::High,

            // Large-cap financials/healthcare: высокий объём
            "JPM" | "V" | "UNH" | "JNJ" => VolumeCategory::MediumHigh,

            // Large-cap consumer/industrial: средний-высокий объём
            "PG" | "HD" | "DIS" => VolumeCategory::Medium,

            // Mid-cap / growth / specialized: средний-низкий объём
            "PYPL" | "NFLX" | "ADBE" | "CRM" => VolumeCategory::Low,

            // Fallback to medium
            _ => VolumeCategory::Medium,
        }
    }

    /// Generates a realistic volume for this category.
    pub fn generate_volume(self) -> u32 {
        match self {
            VolumeCategory::High => 2000 + (rand::random::<f64>() * 8000.0) as u32,
            VolumeCategory::MediumHigh => 1000 + (rand::random::<f64>() * 4000.0) as u32,
            VolumeCategory::Medium => 500 + (rand::random::<f64>() * 2000.0) as u32,
            VolumeCategory::Low => 100 + (rand::random::<f64>() * 900.0) as u32,
        }
    }
}

impl QuoteGenerator {
    pub fn new() -> Self {
        // Базовые цены для тикеров (условные)
        let mut base_prices = std::collections::HashMap::new();
        base_prices.insert("AAPL".to_string(), 175.0);
        base_prices.insert("MSFT".to_string(), 380.0);
        base_prices.insert("GOOGL".to_string(), 140.0);
        base_prices.insert("AMZN".to_string(), 178.0);
        base_prices.insert("NVDA".to_string(), 875.0);
        base_prices.insert("META".to_string(), 490.0);
        base_prices.insert("TSLA".to_string(), 175.0);
        base_prices.insert("JPM".to_string(), 195.0);
        base_prices.insert("JNJ".to_string(), 155.0);
        base_prices.insert("V".to_string(), 275.0);
        base_prices.insert("PG".to_string(), 160.0);
        base_prices.insert("UNH".to_string(), 520.0);
        base_prices.insert("HD".to_string(), 350.0);
        base_prices.insert("DIS".to_string(), 110.0);
        base_prices.insert("PYPL".to_string(), 62.0);
        base_prices.insert("NFLX".to_string(), 600.0);
        base_prices.insert("ADBE".to_string(), 560.0);
        base_prices.insert("CRM".to_string(), 260.0);

        QuoteGenerator { base_prices }
    }

    pub fn generate_quote(&mut self, ticker: &str) -> Option<StockQuote> {
        if !is_supported_ticker(ticker) {
            return None;
        }

        // Получаем или инициализируем последнюю цену
        let last_price = self
            .base_prices
            .entry(ticker.to_string())
            .or_insert_with(|| 100.0 + rand::random::<f64>() * 400.0);

        // Случайное изменение цены: ±2%
        let change_percent = (rand::random::<f64>() - 0.5) * 0.04;
        *last_price *= 1.0 + change_percent;
        *last_price = (*last_price * 100.0).round() / 100.0; // Округляем до центов

        // Генерация volume  с помощью VolumeCategory
        let volume_category = VolumeCategory::for_ticker(ticker);
        let volume = volume_category.generate_volume();

        let timestamp = get_cur_timestamp();

        Some(StockQuote {
            ticker: ticker.to_string(),
            price: *last_price,
            volume,
            timestamp,
        })
    }

    /// Generates quotes for ALL supported tickers.
    ///
    /// # Returns
    ///
    /// A `QuoteBatch` containing quotes for every supported ticker.
    pub fn generate_all_quotes(&mut self) -> QuoteBatch {
        let quotes = SUPPORTED_TICKERS
            .iter()
            .filter_map(|&ticker| self.generate_quote(ticker))
            .collect();

        QuoteBatch::new(quotes)
    }
}
