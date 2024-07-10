use alloy_primitives::{Address, Keccak256, B256, U256};
use serde::{Deserialize, Serialize};

/// The domain binding for the signing service.
const DOMAIN_BINDING: &str = "init4.sequencer.v0";

/// A request to sign a rollup block.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SignRequest {
    /// The block number of the host.
    pub host_block_number: U256, // TODO assign this correctly?
    /// The chain ID of the host.
    pub host_chain_id: U256,
    /// The chain ID of the rollup.
    pub ru_chain_id: U256,
    /// The gas limit of the rollup block.
    pub gas_limit: U256,
    /// The reward address for the builder.
    pub ru_reward_address: Address,
    /// Encoded transactions to be signed
    pub contents: B256,
}

impl SignRequest {
    /// Compute the signing hash for this sig request
    pub fn signing_hash(&self) -> B256 {
        let mut hasher = Keccak256::new();
        hasher.update(DOMAIN_BINDING);
        hasher.update(self.host_block_number.to_be_bytes::<32>());
        hasher.update(self.host_chain_id.to_be_bytes::<32>());
        hasher.update(self.ru_chain_id.to_be_bytes::<32>());
        hasher.update(self.gas_limit.to_be_bytes::<32>());
        hasher.update(self.ru_reward_address);
        hasher.update(self.contents);
        hasher.finalize()
    }
}

impl core::fmt::Display for SignRequest {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "SignRequest {{ host_block_number: {}, host_chain_id: {}, ru_chain_id: {}, gas_limit: {}, ru_reward_address: {}, contents: {} }}",
            self.host_block_number,
            self.host_chain_id,
            self.ru_chain_id,
            self.gas_limit,
            self.ru_reward_address,
            self.contents
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloy_primitives::b256;

    #[test]
    fn roundtrip() {
        let req = SignRequest {
            host_block_number: U256::from(0),            
            host_chain_id: U256::from(1),
            ru_chain_id: U256::from(2),
            gas_limit: U256::from(5),
            ru_reward_address: Address::repeat_byte(6),
            contents: B256::repeat_byte(7),
        };

        let ser = serde_json::to_string(&req).unwrap();
        let de: SignRequest = serde_json::from_str(&ser).unwrap();
        assert_eq!(req, de);

        assert_eq!(
            req.signing_hash(),
            b256!("8c89d2c9e8d725ee335a4f35869a001db64d2f6ce2effe7f09d3ef92f6d251ec")
        );

        assert_eq!(de.signing_hash(), req.signing_hash());
    }
}
