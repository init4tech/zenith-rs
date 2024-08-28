#![allow(clippy::too_many_arguments)]
#![allow(missing_docs)]
use alloy_primitives::{Address, Bytes, FixedBytes, U256};
use alloy_sol_types::sol;

sol!(
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    Zenith,
    "abi/Zenith.json"
);

sol!(
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    Passage,
    "abi/Passage.json"
);

sol!(
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    Orders,
    "abi/RollupOrders.json"
);

sol!(
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    Transactor,
    "abi/Transactor.json"
);

sol!(
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    RollupPassage,
    "abi/RollupPassage.json"
);

/// Module alias for [`Orders`].
#[allow(non_snake_case)]
pub mod RollupOrders {
    pub use super::Orders::OrdersCalls as RollupOrdersCalls;
    pub use super::Orders::OrdersErrors as RollupOrdersErrors;
    pub use super::Orders::OrdersEvents as RollupOrdersEvents;
    pub use super::Orders::OrdersInstance as RollupOrdersInstance;
    pub use super::Orders::*;
}

/// Module alias for [`Orders`].
#[allow(non_snake_case)]
pub mod HostOrders {
    pub use super::Orders::OrdersCalls as HostOrdersCalls;
    pub use super::Orders::OrdersErrors as HostOrdersErrors;
    pub use super::Orders::OrdersEvents as HostOrdersEvents;
    pub use super::Orders::OrdersInstance as HostOrdersInstance;
    pub use super::Orders::*;
}

// Zenith types
impl Copy for Zenith::BlockHeader {}
impl Copy for Zenith::BlockSubmitted {}
impl Copy for Zenith::SequencerSet {}
impl Copy for Zenith::BadSignature {}
impl Copy for Zenith::OneRollupBlockPerHostBlock {}
impl Copy for Zenith::OnlySequencerAdmin {}
impl Copy for Zenith::IncorrectHostBlock {}

impl Zenith::BlockSubmitted {
    /// Get the sequencer address that signed the block.
    pub const fn sequencer(&self) -> Address {
        self.sequencer
    }

    /// Get the chain id of the rollup.
    pub const fn rollup_chain_id(&self) -> u64 {
        self.rollupChainId.as_limbs()[0]
    }

    /// Get the gas limit of the block
    pub const fn gas_limit(&self) -> u64 {
        self.gasLimit.as_limbs()[0]
    }

    /// Get the reward address of the block.
    pub const fn reward_address(&self) -> Address {
        self.rewardAddress
    }

    /// Get the block data hash, i.e. the committment to the data of the block.
    pub const fn block_data_hash(&self) -> FixedBytes<32> {
        self.blockDataHash
    }

    /// Convert the BlockSubmitted event to a BlockHeader with the given host
    /// block number.
    pub const fn to_header(self, host_block_number: U256) -> Zenith::BlockHeader {
        Zenith::BlockHeader::from_block_submitted(self, host_block_number)
    }
}

impl Zenith::BlockHeader {
    /// Create a BlockHeader from a BlockSubmitted event with the given host
    /// block number
    pub const fn from_block_submitted(
        host_block_submitted: Zenith::BlockSubmitted,
        host_block_number: U256,
    ) -> Zenith::BlockHeader {
        Zenith::BlockHeader {
            rollupChainId: host_block_submitted.rollupChainId,
            hostBlockNumber: host_block_number,
            gasLimit: host_block_submitted.gasLimit,
            rewardAddress: host_block_submitted.rewardAddress,
            blockDataHash: host_block_submitted.blockDataHash,
        }
    }

    /// Get the host block number of the block
    pub const fn host_block_number(&self) -> u64 {
        self.hostBlockNumber.as_limbs()[0]
    }

    /// Get the chain ID of the block (discarding high bytes).
    pub const fn chain_id(&self) -> u64 {
        self.rollupChainId.as_limbs()[0]
    }

    /// Get the gas limit of the block (discarding high bytes).
    pub const fn gas_limit(&self) -> u64 {
        self.gasLimit.as_limbs()[0]
    }

    /// Get the reward address of the block.
    pub const fn reward_address(&self) -> Address {
        self.rewardAddress
    }

    /// Get the block data hash, i.e. the committment to the data of the block.
    pub const fn block_data_hash(&self) -> FixedBytes<32> {
        self.blockDataHash
    }
}

// Passage types
impl Copy for Passage::EnterConfigured {}
impl Copy for Passage::Withdrawal {}
impl Copy for Passage::OnlyTokenAdmin {}
impl Copy for Passage::Enter {}
impl Copy for Passage::EnterToken {}
impl Copy for Passage::DisallowedEnter {}
impl Copy for Passage::FailedCall {}
impl Copy for Passage::InsufficientBalance {}
impl Copy for Passage::SafeERC20FailedOperation {}
impl Copy for Passage::AddressEmptyCode {}

impl Copy for Passage::PassageEvents {}

impl Clone for Passage::PassageEvents {
    fn clone(&self) -> Self {
        *self
    }
}

impl Passage::EnterToken {
    /// Get the chain ID of the event (discarding high bytes), returns `None`
    /// if the event has no associated chain id.
    pub const fn rollup_chain_id(&self) -> u64 {
        self.rollupChainId.as_limbs()[0]
    }

    /// Get the token address of the event.
    pub const fn token(&self) -> Address {
        self.token
    }

    /// Get the recipient of the event.
    pub const fn recipient(&self) -> Address {
        self.rollupRecipient
    }

    /// Get the amount of the event.
    pub const fn amount(&self) -> U256 {
        self.amount
    }
}

impl Passage::Enter {
    /// Get the chain ID of the event (discarding high bytes), returns `None`
    /// if the event has no associated chain id.
    pub const fn rollup_chain_id(&self) -> u64 {
        self.rollupChainId.as_limbs()[0]
    }

    /// Get the recipient of the event.
    pub const fn recipient(&self) -> Address {
        self.rollupRecipient
    }

    /// Get the amount of the event.
    pub const fn amount(&self) -> U256 {
        self.amount
    }
}

impl Passage::Withdrawal {
    /// Get the token address of the request.
    pub const fn token(&self) -> Address {
        self.token
    }

    /// Get the recipient of the request.
    pub const fn recipient(&self) -> Address {
        self.recipient
    }

    /// Get the amount of the request.
    pub const fn amount(&self) -> U256 {
        self.amount
    }
}

impl Passage::EnterConfigured {
    /// Get the token address of the event.
    pub const fn token(&self) -> Address {
        self.token
    }

    /// Get if the token has been configured to allow or disallow enters.
    pub const fn can_enter(&self) -> bool {
        self.canEnter
    }
}

// RollupOrders types

impl Copy for Orders::Input {}
impl Copy for Orders::Output {}
impl Copy for Orders::Sweep {}
impl Copy for Orders::InsufficientBalance {}
impl Copy for Orders::AddressEmptyCode {}
impl Copy for Orders::LengthMismatch {}
impl Copy for Orders::OrderExpired {}
impl Copy for Orders::OutputMismatch {}
impl Copy for Orders::SafeERC20FailedOperation {}

impl Clone for Orders::OrdersEvents {
    fn clone(&self) -> Self {
        match self {
            Self::Order(event) => Self::Order(event.clone()),
            Self::Sweep(event) => Self::Sweep(*event),
            Self::Filled(event) => Self::Filled(event.clone()),
        }
    }
}

impl Orders::Input {
    pub const fn token(&self) -> Address {
        self.token
    }

    pub const fn amount(&self) -> u64 {
        self.amount.as_limbs()[0]
    }
}

impl Orders::Output {
    pub const fn token(&self) -> Address {
        self.token
    }

    pub const fn amount(&self) -> u64 {
        self.amount.as_limbs()[0]
    }

    pub const fn recipient(&self) -> Address {
        self.recipient
    }

    pub const fn chain_id(&self) -> u32 {
        self.chainId
    }
}

impl Orders::Order {
    /// Get the inputs of the order.
    pub fn inputs(&self) -> &[Orders::Input] {
        &self.inputs
    }

    /// Get the outputs of the order.
    pub fn outputs(&self) -> &[Orders::Output] {
        &self.outputs
    }

    /// Get the deadline of the order.
    pub const fn deadline(&self) -> u64 {
        self.deadline.as_limbs()[0]
    }
}

impl Orders::Sweep {
    pub const fn recipient(&self) -> Address {
        self.recipient
    }

    pub const fn token(&self) -> Address {
        self.token
    }

    pub const fn amount(&self) -> u64 {
        self.amount.as_limbs()[0]
    }
}

impl Orders::Filled {
    pub fn outputs(&self) -> &[Orders::Output] {
        self.outputs.as_slice()
    }
}

// Transactor
impl Copy for Transactor::GasConfigured {}

impl Clone for Transactor::TransactorEvents {
    fn clone(&self) -> Self {
        match self {
            Transactor::TransactorEvents::Transact(event) => {
                Transactor::TransactorEvents::Transact(event.clone())
            }
            Transactor::TransactorEvents::GasConfigured(event) => {
                Transactor::TransactorEvents::GasConfigured(*event)
            }
        }
    }
}

impl Transactor::Transact {
    pub const fn rollup_chain_id(&self) -> u64 {
        self.rollupChainId.as_limbs()[0]
    }

    pub const fn sender(&self) -> Address {
        self.sender
    }

    pub const fn to(&self) -> Address {
        self.to
    }

    pub const fn data(&self) -> &Bytes {
        &self.data
    }

    pub const fn value(&self) -> U256 {
        self.value
    }

    pub fn max_fee_per_gas(&self) -> u128 {
        self.maxFeePerGas.to::<u128>()
    }

    pub fn gas(&self) -> u128 {
        self.gas.to::<u128>()
    }
}

// RollupPassage
impl Copy for RollupPassage::Exit {}
impl Copy for RollupPassage::ExitToken {}
impl Copy for RollupPassage::AddressEmptyCode {}
impl Copy for RollupPassage::InsufficientBalance {}
impl Copy for RollupPassage::SafeERC20FailedOperation {}

impl Copy for RollupPassage::RollupPassageEvents {}

impl Clone for RollupPassage::RollupPassageEvents {
    fn clone(&self) -> Self {
        *self
    }
}
