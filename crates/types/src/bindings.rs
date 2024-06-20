#![allow(clippy::too_many_arguments)]
#![allow(missing_docs)]
use alloy_primitives::Address;
use alloy_sol_types::sol;

use self::Orders::{OrdersErrors, OrdersEvents};

sol!(
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    Zenith,
    "abi/Zenith.json"
);

// Zenith types
impl Copy for Zenith::BlockHeader {}
impl Copy for Zenith::BlockSubmitted {}
impl Copy for Zenith::SequencerSet {}
impl Copy for Zenith::BadSequence {}
impl Copy for Zenith::BadSignature {}
impl Copy for Zenith::BlockExpired {}
impl Copy for Zenith::OneRollupBlockPerHostBlock {}
impl Copy for Zenith::OnlySequencerAdmin {}

// Passage types
impl Copy for Zenith::Enter {}
impl Copy for Zenith::Withdrawal {}
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
            Self::SequencerSet(inner) => Self::SequencerSet(*inner),
            Self::Transact(inner) => Self::Transact(inner.clone()),
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

impl Zenith::ZenithEvents {
    /// Get the chain ID of the event (discarding high bytes), returns `None`
    /// if the event has no associated chain id.
    pub const fn rollup_chain_id(&self) -> Option<u64> {
        match self {
            Zenith::ZenithEvents::BlockSubmitted(inner) => Some(inner.rollupChainId.as_limbs()[0]),
            Zenith::ZenithEvents::Enter(inner) => Some(inner.rollupChainId.as_limbs()[0]),
            _ => None,
        }
    }
}

impl Zenith::BlockHeader {
    /// Get the chain ID of the block (discarding high bytes).
    pub const fn chain_id(&self) -> u64 {
        self.rollupChainId.as_limbs()[0]
    }

    /// Get the sequence of the block (discarding high bytes).
    pub const fn sequence(&self) -> u64 {
        self.sequence.as_limbs()[0]
    }

    /// Get the confirm by time of the block (discarding high bytes).
    pub const fn confirm_by(&self) -> u64 {
        self.confirmBy.as_limbs()[0]
    }

    /// Get the gas limit of the block (discarding high bytes).
    pub const fn gas_limit(&self) -> u64 {
        self.gasLimit.as_limbs()[0]
    }

    /// Get the reward address of the block.
    pub const fn reward_address(&self) -> Address {
        self.rewardAddress
    }
}

sol!(
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    Orders,
    "abi/Orders.json"
);

impl Copy for Orders::Swap {}
impl Copy for Orders::Sweep {}
impl Copy for Orders::SwapFulfilled {}
impl Copy for Orders::OrderExpired {}
impl Copy for OrdersEvents {}
impl Copy for OrdersErrors {}

impl Clone for Orders::OrdersEvents {
    fn clone(&self) -> Self {
        *self
    }
}

impl Clone for Orders::OrdersErrors {
    fn clone(&self) -> Self {
        *self
    }
}

impl Orders::SwapFulfilled {
    /// Get the target chain ID of the swap (discarding high bytes).
    pub const fn origin_chain_id(&self) -> u64 {
        self.originChainId.as_limbs()[0]
    }
}

impl Orders::Swap {
    /// Get the target chain ID of the swap (discarding high bytes).
    pub const fn target_chain_id(&self) -> u64 {
        self.targetChainId.as_limbs()[0]
    }
}
