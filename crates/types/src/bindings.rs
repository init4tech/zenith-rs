use alloy_primitives::Address;
use alloy_sol_types::sol;

sol!(
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq)]
    Zenith,
    "abi/zenith.json"
);

impl From<&Zenith::BlockSubmitted> for Zenith::BlockHeader {
    fn from(event: &Zenith::BlockSubmitted) -> Zenith::BlockHeader {
        Zenith::BlockHeader {
            rollupChainId: event.rollupChainId,
            sequence: event.sequence,
            confirmBy: event.confirmBy,
            gasLimit: event.gasLimit,
            rewardAddress: event.rewardAddress,
        }
    }
}

impl Zenith::BlockHeader {
    /// Get the chain ID of the block (discarding high bytes).
    pub fn chain_id(&self) -> u64 {
        self.gasLimit.as_limbs()[0]
    }

    /// Get the sequence of the block (discarding high bytes).
    pub fn sequence(&self) -> u64 {
        self.sequence.as_limbs()[0]
    }

    /// Get the confirm by time of the block (discarding high bytes).
    pub fn confirm_by(&self) -> u64 {
        self.gasLimit.as_limbs()[0]
    }

    /// Get the gas limit of the block (discarding high bytes).
    pub fn gas_limit(&self) -> u64 {
        self.gasLimit.as_limbs()[0]
    }

    /// Get the reward address of the block.
    pub fn reward_address(&self) -> Address {
        self.rewardAddress
    }
}
