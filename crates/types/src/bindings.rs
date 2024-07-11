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
    RollupOrders,
    "abi/RollupOrders.json"
);

// Zenith types
impl Copy for Zenith::BlockHeader {}
impl Copy for Zenith::BlockSubmitted {}
impl Copy for Zenith::SequencerSet {}
impl Copy for Zenith::BadSignature {}
impl Copy for Zenith::OneRollupBlockPerHostBlock {}
impl Copy for Zenith::OnlySequencerAdmin {}
impl Copy for Zenith::IncorrectHostBlock {}

impl Zenith::BlockSubmitted {
    pub const fn sequencer(&self) -> Address {
        self.sequencer
    }

    pub const fn rollup_chain_id(&self) -> u64 {
        self.rollupChainId.as_limbs()[0]
    }

    pub const fn gas_limit(&self) -> u64 {
        self.gasLimit.as_limbs()[0]
    }

    pub const fn reward_address(&self) -> Address {
        self.rewardAddress
    }

    pub const fn block_data_hash(&self) -> FixedBytes<32> {
        self.blockDataHash
    }
}

impl Zenith::BlockHeader {
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
}

// returns a BlockHeader from a BlockSubmitted event with the given host block number
#[allow(dead_code)]
pub(crate) const fn header_from_block_submitted(
    event: &Zenith::BlockSubmitted,
    host_block_number: U256,
) -> Zenith::BlockHeader {
    Zenith::BlockHeader {
        rollupChainId: event.rollupChainId,
        hostBlockNumber: host_block_number,
        gasLimit: event.gasLimit,
        rewardAddress: event.rewardAddress,
        blockDataHash: event.blockDataHash,
    }
}

// Passage types
impl Copy for Passage::EnterConfigured {}
impl Copy for Passage::Withdrawal {}
impl Copy for Passage::OnlyTokenAdmin {}
impl Copy for Passage::Enter {}
impl Copy for Passage::EnterToken {}

impl Clone for Passage::PassageEvents {
    fn clone(&self) -> Self {
        match self {
            Self::Enter(inner) => Self::Enter(*inner),
            Self::Withdrawal(inner) => Self::Withdrawal(*inner),
            Self::EnterConfigured(inner) => Self::EnterConfigured(*inner),
            Self::EnterToken(inner) => Self::EnterToken(*inner),
            Self::Transact(inner) => Self::Transact(inner.clone()),
        }
    }
}

impl Passage::Enter {
    /// Get the chain ID of the event (discarding high bytes), returns `None`
    /// if the event has no associated chain id.
    pub const fn rollup_chain_id(&self) -> u64 {
        self.rollupChainId.as_limbs()[0]
    }
}

impl Passage::PassageErrors {}
impl Passage::OnlyTokenAdmin {}
impl Passage::EnterToken {}
impl Passage::EnterConfigured {}
impl Passage::Enter {}
impl Passage::DisallowedEnter {}

impl Passage::Transact {
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

impl Passage::Withdrawal {
    pub const fn token(&self) -> Address {
        self.token
    }
    pub const fn recipient(&self) -> Address {
        self.recipient
    }
    pub const fn amount(&self) -> u64 {
        self.amount.as_limbs()[0]
    }
}

impl Zenith::BlockHeader {
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

// RollupOrders types
impl RollupOrders::OrderExpired {}
impl RollupOrders::OnlyBuilder {}

impl Copy for RollupOrders::Input {}
impl Copy for RollupOrders::Output {}

impl RollupOrders::Input {
    pub const fn token(&self) -> Address {
        self.token
    }

    pub const fn amount(&self) -> u64 {
        self.amount.as_limbs()[0]
    }
}

impl RollupOrders::Output {
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

impl RollupOrders::Order {
    pub fn inputs(&self) -> &[RollupOrders::Input] {
        &self.inputs
    }

    pub fn outputs(&self) -> &[RollupOrders::Output] {
        &self.outputs
    }

    pub const fn deadline(&self) -> u64 {
        self.deadline.as_limbs()[0]
    }
}

impl RollupOrders::Sweep {
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

impl RollupOrders::Filled {
    pub fn outputs(&self) -> &[RollupOrders::Output] {
        self.outputs.as_slice()
    }
}
