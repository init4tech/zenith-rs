use alloy::primitives::{Address, Keccak256, B256, U256};
use serde::{Deserialize, Serialize};

/// The domain binding for the signing service.
const DOMAIN_BINDING: &str = "init4.sequencer.v0";

/// A request to sign a rollup block.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SignRequest {
    /// The block number of the host.
    pub host_block_number: U256,
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
        hasher.update(self.host_chain_id.to_be_bytes::<32>());
        hasher.update(self.ru_chain_id.to_be_bytes::<32>());
        hasher.update(self.host_block_number.to_be_bytes::<32>());
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
            "SignRequest {{ host_chain_id: {}, host_block_number: {}, ru_chain_id: {}, gas_limit: {}, ru_reward_address: {}, contents: {} }}",
            self.host_chain_id,
            self.host_block_number,
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
    use alloy::primitives::b256;

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
            b256!("74388c53a86cf15b3e8b11fa5f499dac87819fd00c20cfec4557b7d551b2c445")
        );

        assert_eq!(de.signing_hash(), req.signing_hash());
    }
}
