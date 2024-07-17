use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{collections::HashMap, time};

use alloy_consensus::TxEnvelope;
use eyre::Error;
use reqwest::Client;
use serde_json::from_str;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub use crate::config::BuilderConfig;

/// Implements a poller for the block builder to pull transactions from the transaction pool.
pub struct TxPoller {
    // config for the builder
    pub config: BuilderConfig,
    // Reqwest client for fetching from the tx-pool
    pub client: Client,
    //  Maintain a set of txn hash to expiration time
    pub seen_txns: Arc<Mutex<HashMap<String, time::Instant>>>,
}

/// TxPoller implements a poller that fetches unique transactions from the transaction pool.
impl TxPoller {
    /// returns a new TxPoller with the given config.
    pub fn new(config: &BuilderConfig) -> Self {
        Self {
            config: config.clone(),
            client: Client::new(),
            seen_txns: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// polls the tx-pool for unique transactions and evicts expired transactions.
    /// unique transactions that haven't been seen before are sent into the builder pipeline.
    pub async fn check_tx_pool(&self) -> Result<Vec<TxEnvelope>, Error> {
        let mut unique: Vec<TxEnvelope> = Vec::new();
        let result = self.client.get(self.config.tx_pool_url.to_string() + "/get").send().await?;

        // parse the response as a JSON array of key/value pairs of transaction hashes to transaction envelopes
        let parsed: Value = from_str(&result.text().await?)?;
        if let Value::Array(items) = parsed.clone() {
            for item in items {
                if let Value::Object(map) = item {
                    if let Some(Value::String(value)) = map.get("value") {
                        if let Ok(parsed) = from_str::<TxEnvelope>(value) {
                            self.check_cache(parsed, &mut unique);
                        }
                    }
                }
            }
        }

        Ok(unique)
    }

    /// checks if the transaction has been seen before and if not, adds it to the unique transactions list.
    fn check_cache(&self, tx: TxEnvelope, unique: &mut Vec<TxEnvelope>) {
        self.seen_txns.lock().unwrap().entry(tx.tx_hash().to_string()).or_insert_with(|| {
            // add to unique transactions
            unique.push(tx.clone());
            // expiry is now + cache_duration
            time::Instant::now() + Duration::from_secs(self.config.tx_pool_cache_duration)
        });
    }

    /// removes entries from seen_txns that have lived past expiry
    fn evict(&self) {
        for txn in self.seen_txns.lock().unwrap().iter() {
            if txn.1.elapsed() > Duration::from_secs(0) {
                self.seen_txns.lock().unwrap().remove(txn.0);
            }
        }
    }

    /// spawns a task that polls the tx-pool for unique transactions and ingests them into the tx_channel.
    pub fn spawn(self, tx_channel: mpsc::UnboundedSender<TxEnvelope>) -> JoinHandle<()> {
        let handle: JoinHandle<()> = tokio::spawn(async move {
            loop {
                let channel = tx_channel.clone();
                let txns = self.check_tx_pool().await;

                // send recently discovered transactions to the builder pipeline
                match txns {
                    Ok(txns) => {
                        for txn in txns.iter() {
                            let result = channel.send(txn.clone());
                            if result.is_err() {
                                tracing::debug!("tx_poller failed to send tx");
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error polling transactions: {}", e);
                    }
                }

                // evict expired txns once every loop
                self.evict();

                tokio::time::sleep(Duration::from_secs(self.config.tx_pool_poll_interval)).await;
            }
        });

        handle
    }
}
