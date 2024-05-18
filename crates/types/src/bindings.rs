#![allow(missing_docs)]
use alloy_primitives::Address;
use alloy_sol_types::sol;

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
