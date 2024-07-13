use alloy_consensus::{SidecarBuilder, SidecarCoder, Transaction, TxEip1559, TxEnvelope};
use alloy_primitives::{keccak256, Bytes, B256};
use serde_json::{from_str, Value};
use std::{sync::OnceLock, time::Duration};
use tokio::{select, sync::mpsc, task::JoinHandle};
use reqwest::Client;
use tracing::Instrument;
use zenith_types::{encode_txns, Alloy2718Coder};
use eyre::Error;

use crate::config::BuilderConfig;

#[derive(Debug, Default, Clone)]
/// A block in progress.
pub struct InProgressBlock {
    transactions: Vec<TxEnvelope>,

    raw_encoding: OnceLock<Bytes>,
    hash: OnceLock<B256>,
}

impl InProgressBlock {
    /// Create a new `InProgressBlock`
    pub fn new() -> Self {
        Self { transactions: Vec::new(), raw_encoding: OnceLock::new(), hash: OnceLock::new() }
    }

    /// Get the number of transactions in the block.
    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    /// Check if the block is empty.
    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }

    /// Unseal the block
    fn unseal(&mut self) {
        self.raw_encoding.take();
        self.hash.take();
    }

    /// Seal the block by encoding the transactions and calculating the contentshash.
    fn seal(&self) {
        self.raw_encoding.get_or_init(|| encode_txns::<Alloy2718Coder>(&self.transactions).into());
        self.hash.get_or_init(|| keccak256(self.raw_encoding.get().unwrap().as_ref()));
    }

    /// Ingest a transaction into the in-progress block. Fails
    pub fn ingest_tx(&mut self, tx: &TxEnvelope) {
        tracing::info!(hash = %tx.tx_hash(), "ingesting tx");
        self.unseal();
        self.transactions.push(tx.clone());
    }

    /// Encode the in-progress block
    fn encode_raw(&self) -> &Bytes {
        self.seal();
        self.raw_encoding.get().unwrap()
    }

    /// Calculate the hash of the in-progress block, finishing the block.
    pub fn contents_hash(&self) -> alloy_primitives::B256 {
        self.seal();
        *self.hash.get().unwrap()
    }

    /// Convert the in-progress block to sign request contents.
    pub fn encode_calldata(&self) -> &Bytes {
        self.encode_raw()
    }

    /// Convert the in-progress block to a blob transaction sidecar.
    pub fn encode_blob<T: SidecarCoder + Default>(&self) -> SidecarBuilder<T> {
        let mut coder = SidecarBuilder::<T>::default();
        coder.ingest(self.encode_raw());
        coder
    }
}

pub struct BlockBuilder {
    pub incoming_transactions_buffer: u64,
    pub config: BuilderConfig,
}

impl BlockBuilder {
    // create a new block builder with the given config.
    pub fn new(config: &BuilderConfig) -> Self {
        Self { 
            config: config.clone(),
            incoming_transactions_buffer: config.incoming_transactions_buffer, 
        }
    }

    // polls the transaction pool for transactions.
    pub async fn poll_transactions(&self) ->  Result<Vec<TxEnvelope>, Error> {
        let client = Client::new();
        let result = client.get(self.config.tx_pool_url.to_string() + "/get").send().await?;
        let raw = result.text().await?;

        let parsed: Value = from_str(&raw)?;
        // dbg!(parsed.clone());

        let mut _transactions: Vec<TxEnvelope> = Vec::new();

        if let Value::Array(items) = parsed.clone() {   
            // dbg!(items.clone());

            for item in items { 
                if let Value::Object(map) = item { 
                    if let Some(Value::String(value)) = map.get("key") {
                        if let Some(Value::String(value)) = map.get("value") {
                            dbg!(value);
                        }
                        // TODO if value is parseable as a transaction envelope, add it to the list of transactions
                    }
                }
            }
        
            todo!()
        }

        todo!();
    }

    /// Spawn the block builder task, returning the inbound channel to it, and
    /// a handle to the running task.
    pub fn spawn(
        self,
        outbound: mpsc::UnboundedSender<InProgressBlock>,
    ) -> (mpsc::UnboundedSender<TxEnvelope>, JoinHandle<()>) {
        let mut in_progress = InProgressBlock::default();

        let (sender, mut inbound) = mpsc::unbounded_channel();

        let handle = tokio::spawn(
            async move {
                loop {
                    let sleep = tokio::time::sleep(Duration::from_secs(self.incoming_transactions_buffer));
                    tokio::pin!(sleep);

                    select! {
                        biased;
                        _ = &mut sleep => {
                            if !in_progress.is_empty() {
                                tracing::debug!(txns = in_progress.len(), "sending block to submit task");
                                let in_progress_block = std::mem::take(&mut in_progress);
                                if outbound.send(in_progress_block).is_err() {
                                    tracing::debug!("downstream task gone");
                                    break
                                }
                            }
                        }
                        // TODO - Option 1 - pump transactions from the tx pool into the inbound channel
                        item_res = inbound.recv() => {
                            match item_res {
                                Some(item) => in_progress.ingest_tx(&item),
                                None => {
                                    tracing::debug!("upstream task gone");
                                    break
                                }
                            }
                        }
                    }
                }
            }
            .in_current_span(),
        );

        (sender, handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BuilderConfig;
    use std::{str::FromStr, time::Duration};

    use alloy_consensus::TxEip1559;
    use alloy_primitives::{bytes, Address, U256};

    #[tokio::test]
    async fn test_tx_roundtrip() {
        let config = BuilderConfig {
            host_chain_id: 17001,
            ru_chain_id: 17002,
            host_rpc_url: "http://rpc.api.signet.sh".into(),
            zenith_address: Address::from_str("0x0000000000000000000000000000000000000000").unwrap(),
            quincey_url: "http://localhost:8080".into(),
            builder_port: 8080,
            sequencer_key: None,
            builder_key: "0000000000000000000000000000000000000000000000000000000000000000".into(),
            incoming_transactions_buffer: 1,
            block_confirmation_buffer: 1,
            builder_rewards_address: Address::from_str("0x0000000000000000000000000000000000000000").unwrap(),
            rollup_block_gas_limit: 100_000,
            tx_pool_url: "https://transactions.api.signet.sh".into(),
        };
        let builder = BlockBuilder::new(&config);

        let client = reqwest::Client::new();

        let tx = TxEip1559 {
            chain_id: 17001,
            nonce: 1,
            gas_limit: 50000,
            value: U256::from(1_f64),
            input: bytes!(""),
            ..Default::default()
        };

        client.post(config.tx_pool_url.to_string() + "/put").json(&tx).send().await.unwrap();

        println!("sending transaction {:?} to tx pool", tx);

        // poll the transaction pool and check that we got that same transaction
        let result = builder.poll_transactions().await.unwrap();
        dbg!(result);
        // TODO parse the result and compare it to the original transaction

        let _interval = tokio::time::interval(Duration::from_millis(100));
    }
}