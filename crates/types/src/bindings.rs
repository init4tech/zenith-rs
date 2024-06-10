#![allow(clippy::too_many_arguments)]
#![allow(missing_docs)]
use alloy_primitives::{Address, B256, U256};
use alloy_sol_types::sol;

use self::RollupPassage::{RollupPassageErrors, RollupPassageEvents};

sol!(
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    Zenith,
    "abi/Zenith.json"
);

impl Copy for Zenith::BlockHeader {}

impl Copy for Zenith::BlockSubmitted {}
impl Copy for Zenith::Enter {}
impl Copy for Zenith::SequencerSet {}
impl Copy for Zenith::Withdrawal {}
impl Copy for Zenith::SwapFulfilled {}

impl Copy for Zenith::BadSequence {}
impl Copy for Zenith::BadSignature {}
impl Copy for Zenith::BlockExpired {}
impl Copy for Zenith::OneRollupBlockPerHostBlock {}
impl Copy for Zenith::OnlySequencerAdmin {}
impl Copy for Zenith::OnlyWithdrawalAdmin {}
impl Copy for Zenith::ZenithErrors {}

impl Clone for Zenith::ZenithErrors {
    fn clone(&self) -> Self {
        *self
    }
}

impl Clone for Zenith::ZenithEvents {
    fn clone(&self) -> Self {
        match self {
            Self::BlockSubmitted(inner) => Self::BlockSubmitted(*inner),
            Self::Enter(inner) => Self::Enter(*inner),
            Self::SwapFulfilled(inner) => Self::SwapFulfilled(*inner),
            Self::SequencerSet(inner) => Self::SequencerSet(*inner),
            Self::Withdrawal(inner) => Self::Withdrawal(*inner),
        }
    }
}

impl From<&Zenith::BlockSubmitted> for Zenith::BlockHeader {
    fn from(event: &Zenith::BlockSubmitted) -> Zenith::BlockHeader {
        Zenith::BlockHeader {
            rollupChainId: event.rollupChainId,
            sequence: event.sequence,
            confirmBy: event.confirmBy,
            gasLimit: event.gasLimit,
            rewardAddress: event.rewardAddress,
            blockDataHash: event.blockDataHash,
        }
    }
}

impl Zenith::SwapFulfilled {
    /// Get the target chain ID of the swap (discarding high bytes).
    pub const fn origin_chain_id(&self) -> u64 {
        self.originChainId.as_limbs()[0]
    }
}

impl Zenith::ZenithEvents {
    /// Get the chain ID of the event (discarding high bytes), returns `None`
    /// if the event has no associated chain id.
    pub const fn rollup_chain_id(&self) -> Option<u64> {
        match self {
            Zenith::ZenithEvents::BlockSubmitted(inner) => Some(inner.rollupChainId.as_limbs()[0]),
            Zenith::ZenithEvents::Enter(inner) => Some(inner.rollupChainId.as_limbs()[0]),
            Zenith::ZenithEvents::SwapFulfilled(inner) => Some(inner.originChainId.as_limbs()[0]),
            _ => None,
        }
    }
}

impl Zenith::BlockHeader {
    /// Get the chain ID of the block (discarding high bytes).
    pub const fn chain_id(&self) -> U256 {
        self.rollupChainId
    }

    /// Get the sequence of the block (discarding high bytes).
    pub const fn sequence(&self) -> U256 {
        self.sequence
    }

    /// Get the confirm by time of the block (discarding high bytes).
    pub const fn confirm_by(&self) -> U256 {
        self.confirmBy
    }

    /// Get the gas limit of the block (discarding high bytes).
    pub const fn gas_limit(&self) -> U256 {
        self.gasLimit
    }

    /// Get the reward address of the block.
    pub const fn reward_address(&self) -> Address {
        self.rewardAddress
    }

    /// Get the data hash of the block.
    pub const fn block_data_hash(&self) -> B256 {
        self.blockDataHash
    }
}

sol!(
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    RollupPassage,
    "abi/RollupPassage.json"
);

impl Copy for RollupPassage::Swap {}
impl Copy for RollupPassage::Sweep {}
impl Copy for RollupPassage::SwapFulfilled {}

impl Copy for RollupPassage::OrderExpired {}
impl Copy for RollupPassageEvents {}
impl Copy for RollupPassageErrors {}

impl Clone for RollupPassage::RollupPassageEvents {
    fn clone(&self) -> Self {
        *self
    }
}

impl Clone for RollupPassage::RollupPassageErrors {
    fn clone(&self) -> Self {
        *self
    }
}

impl RollupPassage::SwapFulfilled {
    /// Get the target chain ID of the swap (discarding high bytes).
    pub const fn origin_chain_id(&self) -> u64 {
        self.originChainId.as_limbs()[0]
    }
}

impl RollupPassage::Swap {
    /// Get the target chain ID of the swap (discarding high bytes).
    pub const fn target_chain_id(&self) -> u64 {
        self.targetChainId.as_limbs()[0]
    }
}
