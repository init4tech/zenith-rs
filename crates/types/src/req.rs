use alloy_primitives::{Address, Keccak256, B256, U256};
use serde::{Deserialize, Serialize};

/// The domain binding for the signing service.
const DOMAIN_BINDING: &str = "init4.sequencer.v0";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignRequest {
    /// The chain ID of the host.
    pub host_chain_id: U256,
    /// The chain ID of the rollup.
    pub ru_chain_id: U256,
    /// The sequence number of the rollup block.
    pub sequence: U256,
    /// The rollup block must be confirmed by this time.
    pub confirm_by: U256,
    /// The gas limit of the rollup block.
    pub gas_limit: U256,
    /// The reward address for the builder.
    pub ru_reward_address: Address,
    #[serde(flatten)]
    /// Encoded transactions to be signed
    pub contents: B256,
}

impl SignRequest {
    /// Compute the signing hash for this sig request
    pub fn signing_hash(&self) -> B256 {
        let mut hasher = Keccak256::new();
        hasher.update(DOMAIN_BINDING);
        hasher.update(self.host_chain_id.to_be_bytes::<32>());
        hasher.update(self.ru_chain_id.to_be_bytes::<32>());
        hasher.update(self.sequence.to_be_bytes::<32>());
        hasher.update(self.gas_limit.to_be_bytes::<32>());
        hasher.update(self.confirm_by.to_be_bytes::<32>());
        hasher.update(self.ru_reward_address);
        hasher.update(self.contents);
        hasher.finalize()
    }
}
