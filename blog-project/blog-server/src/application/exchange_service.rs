use std::collections::HashMap;
use std::sync::Arc;

use reqwest::Client;
use serde::Deserialize;
use tracing::{error, info};

#[derive(Debug, Deserialize)]
struct ExchangeResponse {
    rates: HashMap<String, f64>,
}

#[derive(Clone)]
pub struct ExchangeService {
    client: Arc<Client>,
    base_url: String,
}

impl ExchangeService {
    pub fn new(client: Arc<Client>, base_url: String) -> Self {
        Self { client, base_url }
    }

    pub async fn get_rate(&self, from: &str, to: &str) -> anyhow::Result<f64> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), from.to_uppercase());
        let resp = self.client.get(url).send().await?;

        if !resp.status().is_success() {
            error!("failed to fetch exchange rate: {}", resp.status());
            anyhow::bail!("exchange api returned {}", resp.status());
        }

        let body: ExchangeResponse = resp.json().await?;
        match body.rates.get(&to.to_uppercase()) {
            Some(rate) => {
                info!(from = %from, to = %to, rate, "exchange rate fetched");
                Ok(*rate)
            }
            None => anyhow::bail!("currency {} not found", to),
        }
    }
}

