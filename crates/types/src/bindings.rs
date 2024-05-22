#![allow(missing_docs)]
use alloy_primitives::Address;
use alloy_sol_types::sol;

use self::RollupPassage::{RollupPassageErrors, RollupPassageEvents};

sol!(
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    Zenith,
    "abi/zenith.json"
);

impl Copy for Zenith::BlockHeader {}
impl Copy for Zenith::ExitOrder {}

impl Copy for Zenith::BlockSubmitted {}
impl Copy for Zenith::Enter {}
impl Copy for Zenith::ExitFilled {}
impl Copy for Zenith::SequencerSet {}

impl Copy for Zenith::BadSequence {}
impl Copy for Zenith::BadSignature {}
impl Copy for Zenith::BlockExpired {}
impl Copy for Zenith::OneRollupBlockPerHostBlock {}
impl Copy for Zenith::OnlySequencerAdmin {}
impl Copy for Zenith::OnlyWithdrawalAdmin {}
impl Copy for Zenith::OrderExpired {}

impl Clone for Zenith::ZenithErrors {
    fn clone(&self) -> Self {
        match self {
            Self::BadSequence(inner) => Self::BadSequence(*inner),
            Self::BadSignature(inner) => Self::BadSignature(*inner),
            Self::BlockExpired(inner) => Self::BlockExpired(*inner),
            Self::OneRollupBlockPerHostBlock(inner) => Self::OneRollupBlockPerHostBlock(*inner),
            Self::OnlySequencerAdmin(inner) => Self::OnlySequencerAdmin(*inner),
            Self::OnlyWithdrawalAdmin(inner) => Self::OnlyWithdrawalAdmin(*inner),
            Self::OrderExpired(inner) => Self::OrderExpired(*inner),
        }
    }
}

impl Clone for Zenith::ZenithEvents {
    fn clone(&self) -> Self {
        match self {
            Self::BlockData(inner) => Self::BlockData(inner.clone()),
            Self::BlockSubmitted(inner) => Self::BlockSubmitted(*inner),
            Self::Enter(inner) => Self::Enter(*inner),
            Self::ExitFilled(inner) => Self::ExitFilled(*inner),
            Self::SequencerSet(inner) => Self::SequencerSet(*inner),
            Self::Withdraw(inner) => Self::Withdraw(inner.clone()),
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
        }
    }
}

impl From<&Zenith::ExitFilled> for Zenith::ExitOrder {
    fn from(event: &Zenith::ExitFilled) -> Zenith::ExitOrder {
        Zenith::ExitOrder {
            rollupChainId: event.rollupChainId,
            token: event.token,
            recipient: event.hostRecipient,
            amount: event.amount,
        }
    }
}

impl Zenith::ZenithEvents {
    /// Get the chain ID of the event (discarding high bytes), returns `None`
    /// if the event has no associated chain id.
    pub const fn chain_id(&self) -> Option<u64> {
        match self {
            Zenith::ZenithEvents::BlockSubmitted(inner) => Some(inner.rollupChainId.as_limbs()[0]),
            Zenith::ZenithEvents::Enter(inner) => Some(inner.rollupChainId.as_limbs()[0]),
            Zenith::ZenithEvents::ExitFilled(inner) => Some(inner.rollupChainId.as_limbs()[0]),
            _ => None,
        }
    }
}

impl Zenith::BlockHeader {
    /// Get the chain ID of the block (discarding high bytes).
    pub const fn chain_id(&self) -> u64 {
        self.gasLimit.as_limbs()[0]
    }

    /// Get the sequence of the block (discarding high bytes).
    pub const fn sequence(&self) -> u64 {
        self.sequence.as_limbs()[0]
    }

    /// Get the confirm by time of the block (discarding high bytes).
    pub const fn confirm_by(&self) -> u64 {
        self.gasLimit.as_limbs()[0]
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
    RollupPassage,
    "abi/passage.json"
);

impl Copy for RollupPassage::Exit {}
impl Copy for RollupPassage::Sweep {}

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
