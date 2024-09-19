use std::time::Duration;
use std::{collections::HashMap, time};

use alloy::consensus::TxEnvelope;
use alloy_primitives::TxHash;

use eyre::Error;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::from_slice;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub use crate::config::BuilderConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxPoolResponse {
    #[serde(rename = "key")]
    key: TxHash,
    #[serde(rename = "value")]
    value: TxEnvelope,
}

/// Implements a poller for the block builder to pull transactions from the transaction pool.
pub struct TxPoller {
    // config for the builder
    pub config: BuilderConfig,
    // Reqwest client for fetching transactions from the tx-pool
    pub client: Client,
    //  Maintain a set of transaction hashes to their expiration times
    pub seen_txns: HashMap<TxHash, time::Instant>,
}

/// TxPoller implements a poller that fetches unique transactions from the transaction pool.
impl TxPoller {
    /// returns a new TxPoller with the given config.
    pub fn new(config: &BuilderConfig) -> Self {
        Self { config: config.clone(), client: Client::new(), seen_txns: HashMap::new() }
    }

    /// polls the tx-pool for unique transactions and evicts expired transactions.
    /// unique transactions that haven't been seen before are sent into the builder pipeline.
    pub async fn check_tx_pool(&mut self) -> Result<Vec<TxEnvelope>, Error> {
        let mut unique: Vec<TxEnvelope> = Vec::new();
        let result = self.client.get(self.config.tx_pool_url.to_string() + "/get").send().await?;
        let parsed: Vec<TxPoolResponse> = from_slice(&result.bytes().await?)?;

        parsed.iter().for_each(|entry| {
            self.check_cache(entry.value.clone(), &mut unique);
        });

        Ok(unique)
    }

    /// checks if the transaction has been seen before and if not, adds it to the unique transactions list.
    fn check_cache(&mut self, tx: TxEnvelope, unique: &mut Vec<TxEnvelope>) {
        self.seen_txns.entry(*tx.tx_hash()).or_insert_with(|| {
            // add to unique transactions
            unique.push(tx.clone());
            // expiry is now + cache_duration
            time::Instant::now() + Duration::from_secs(self.config.tx_pool_cache_duration)
        });
    }

    /// removes entries from seen_txns that have lived past expiry
    fn evict(&mut self) {
        let expired_keys: Vec<TxHash> = self
            .seen_txns
            .iter()
            .filter_map(
                |(key, &expiration)| {
                    if !expiration.elapsed().is_zero() {
                        Some(*key)
                    } else {
                        None
                    }
                },
            )
            .collect();

        for key in expired_keys {
            self.seen_txns.remove(&key);
        }
    }

    /// spawns a task that polls the tx-pool for unique transactions and ingests them into the tx_channel.
    pub fn spawn(mut self, tx_channel: mpsc::UnboundedSender<TxEnvelope>) -> JoinHandle<()> {
        let handle: JoinHandle<()> = tokio::spawn(async move {
            loop {
                let channel = tx_channel.clone();
                let txns = self.check_tx_pool().await;

                // send recently discovered transactions to the builder pipeline
                match txns {
                    Ok(txns) => {
                        for txn in txns.into_iter() {
                            let result = channel.send(txn);
                            if result.is_err() {
                                tracing::debug!("tx_poller failed to send tx");
                                continue;
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
