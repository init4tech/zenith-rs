use alloy_consensus::{SidecarBuilder, SidecarCoder, TxEnvelope};
use alloy_primitives::{keccak256, Bytes, B256};
use std::{sync::OnceLock, time::Duration};
use tokio::{select, sync::mpsc, task::JoinHandle};
use tracing::Instrument;

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
        self.raw_encoding.get_or_init(|| alloy_rlp::encode(&self.transactions).into());
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
}

impl BlockBuilder {
    pub fn new(config: &BuilderConfig) -> Self {
        Self { incoming_transactions_buffer: config.incoming_transactions_buffer }
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
