use crate::Zenith::BlockHeader as ZenithHeader;
use alloy_primitives::{Keccak256, B256, U256};
use serde::{Deserialize, Serialize};

/// The domain binding for the signing service.
const DOMAIN_BINDING: &str = "init4.sequencer.v0";

/// A request to sign a rollup block.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignRequest {
    /// The chain ID of the host.
    pub host_chain_id: U256,
    /// The Zenith BlockHeader for the rollup block.
    pub header: ZenithHeader,
}

impl SignRequest {
    /// Compute the signing hash for this sig request
    pub fn signing_hash(&self) -> B256 {
        let mut hasher = Keccak256::new();
        hasher.update(DOMAIN_BINDING);
        hasher.update(self.host_chain_id.to_be_bytes::<32>());
        hasher.update(self.header.chain_id().to_be_bytes::<32>());
        hasher.update(self.header.sequence().to_be_bytes::<32>());
        hasher.update(self.header.gas_limit().to_be_bytes::<32>());
        hasher.update(self.header.confirm_by().to_be_bytes::<32>());
        hasher.update(self.header.reward_address());
        hasher.update(self.header.block_data_hash());
        hasher.finalize()
    }
}

impl core::fmt::Display for SignRequest {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "SignRequest {{ host_chain_id: {}, ru_chain_id: {}, sequence: {}, confirm_by: {}, gas_limit: {}, ru_reward_address: {}, contents: {} }}",
            self.host_chain_id,
            self.header.chain_id(),
            self.header.sequence(),
            self.header.confirm_by(),
            self.header.gas_limit(),
            self.header.reward_address(),
            self.header.block_data_hash()
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloy_primitives::{Address, b256};

    #[test]
    fn roundtrip() {
        let req = SignRequest {
            host_chain_id: U256::from(1),
            header: ZenithHeader {
                rollupChainId: U256::from(2),
                sequence: U256::from(3),
                confirmBy: U256::from(4),
                gasLimit: U256::from(5),
                rewardAddress: Address::repeat_byte(6),
                blockDataHash: B256::repeat_byte(7),
            },
        };

        let ser = serde_json::to_string(&req).unwrap();
        let de: SignRequest = serde_json::from_str(&ser).unwrap();
        assert_eq!(req, de);
        assert_eq!(
            req.signing_hash(),
            b256!("eabffc9ed79f68618d3628a804d40199c7888cb5274407ee0ad9ef95c7144d0f")
        );
        assert_eq!(de.signing_hash(), req.signing_hash());
    }
}
