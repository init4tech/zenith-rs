use alloy::{
    eips::{eip2718::Encodable2718, BlockNumberOrTag},
    rpc::types::mev::{EthCallBundle, EthCallBundleResponse, EthSendBundle},
};
use alloy_primitives::{keccak256, Address, Bytes, B256, U256};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::SignedOrder;

/// Wraps a flashbots style EthSendBundle with host fills to make a Zenith compatible bundle
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZenithEthBundle {
    /// The bundle of transactions to simulate. Same structure as a Flashbots [EthSendBundle] bundle.
    /// see <https://github.com/alloy-rs/alloy/blob/main/crates/rpc-types-mev/src/eth_calls.rs#L121-L139>
    #[serde(flatten)]
    pub bundle: EthSendBundle,
    /// Host fills to be applied with the bundle, represented as a signed permit2 order.
    pub host_fills: Option<SignedOrder>,
}

impl ZenithEthBundle {
    /// Returns the transactions in this bundle.
    pub fn txs(&self) -> &[Bytes] {
        &self.bundle.txs
    }

    /// Returns the block number for this bundle.
    pub const fn block_number(&self) -> u64 {
        self.bundle.block_number
    }

    /// Returns the minimum timestamp for this bundle.
    pub const fn min_timestamp(&self) -> Option<u64> {
        self.bundle.min_timestamp
    }

    /// Returns the maximum timestamp for this bundle.
    pub const fn max_timestamp(&self) -> Option<u64> {
        self.bundle.max_timestamp
    }

    /// Returns the reverting tx hashes for this bundle.
    pub fn reverting_tx_hashes(&self) -> &[B256] {
        self.bundle.reverting_tx_hashes.as_slice()
    }

    /// Returns the replacement uuid for this bundle.
    pub fn replacement_uuid(&self) -> Option<&str> {
        self.bundle.replacement_uuid.as_deref()
    }
}

/// Response for `zenith_sendBundle`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZenithEthBundleResponse {
    /// The bundle hash of the sent bundle.
    ///
    /// This is calculated as keccak256(tx_hashes) where tx_hashes are the concatenated transaction hashes.
    pub bundle_hash: B256,
}

/// Bundle of transactions for `zenith_callBundle`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZenithCallBundle {
    /// The bundle of transactions to simulate. Same structure as a Flashbots [EthCallBundle] bundle.
    /// see <https://github.com/alloy-rs/alloy/blob/main/crates/rpc-types-mev/src/eth_calls.rs#L13-L33>
    #[serde(flatten)]
    pub bundle: EthCallBundle,
    /// Host fills to be applied to the bundle for simulation. The mapping corresponds
    /// to asset => user => amount.
    pub host_fills: BTreeMap<Address, BTreeMap<Address, U256>>,
}

impl ZenithCallBundle {
    /// Returns the host fills for this bundle.
    pub const fn host_fills(&self) -> &BTreeMap<Address, BTreeMap<Address, U256>> {
        &self.host_fills
    }

    /// Returns the transactions in this bundle.
    pub fn txs(&self) -> &[Bytes] {
        &self.bundle.txs
    }

    /// Returns the block number for this bundle.
    pub const fn block_number(&self) -> u64 {
        self.bundle.block_number
    }

    /// Returns the state block number for this bundle.
    pub const fn state_block_number(&self) -> BlockNumberOrTag {
        self.bundle.state_block_number
    }

    /// Returns the timestamp for this bundle.
    pub const fn timestamp(&self) -> Option<u64> {
        self.bundle.timestamp
    }

    /// Returns the gas limit for this bundle.
    pub const fn gas_limit(&self) -> Option<u64> {
        self.bundle.gas_limit
    }

    /// Returns the difficulty for this bundle.
    pub const fn difficulty(&self) -> Option<U256> {
        self.bundle.difficulty
    }

    /// Returns the base fee for this bundle.
    pub const fn base_fee(&self) -> Option<u128> {
        self.bundle.base_fee
    }

    /// Creates a new bundle from the given [`Encodable2718`] transactions.
    pub fn from_2718_and_host_fills<I, T>(
        txs: I,
        host_fills: BTreeMap<Address, BTreeMap<Address, U256>>,
    ) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Encodable2718,
    {
        Self::from_raw_txs_and_host_fills(txs.into_iter().map(|tx| tx.encoded_2718()), host_fills)
    }

    /// Creates a new bundle with the given transactions and host fills.
    pub fn from_raw_txs_and_host_fills<I, T>(
        txs: I,
        host_fills: BTreeMap<Address, BTreeMap<Address, U256>>,
    ) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Bytes>,
    {
        Self {
            bundle: EthCallBundle {
                txs: txs.into_iter().map(Into::into).collect(),
                ..Default::default()
            },
            host_fills,
        }
    }

    /// Adds an [`Encodable2718`] transaction to the bundle.
    pub fn append_2718_tx(self, tx: impl Encodable2718) -> Self {
        self.append_raw_tx(tx.encoded_2718())
    }

    /// Adds an EIP-2718 envelope to the bundle.
    pub fn append_raw_tx(mut self, tx: impl Into<Bytes>) -> Self {
        self.bundle.txs.push(tx.into());
        self
    }

    /// Adds multiple [`Encodable2718`] transactions to the bundle.
    pub fn extend_2718_txs<I, T>(self, tx: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Encodable2718,
    {
        self.extend_raw_txs(tx.into_iter().map(|tx| tx.encoded_2718()))
    }

    /// Adds multiple calls to the block.
    pub fn extend_raw_txs<I, T>(mut self, txs: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Bytes>,
    {
        self.bundle.txs.extend(txs.into_iter().map(Into::into));
        self
    }

    /// Sets the block number for the bundle.
    pub const fn with_block_number(mut self, block_number: u64) -> Self {
        self.bundle.block_number = block_number;
        self
    }

    /// Sets the state block number for the bundle.
    pub fn with_state_block_number(
        mut self,
        state_block_number: impl Into<BlockNumberOrTag>,
    ) -> Self {
        self.bundle.state_block_number = state_block_number.into();
        self
    }

    /// Sets the timestamp for the bundle.
    pub const fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.bundle.timestamp = Some(timestamp);
        self
    }

    /// Sets the gas limit for the bundle.
    pub const fn with_gas_limit(mut self, gas_limit: u64) -> Self {
        self.bundle.gas_limit = Some(gas_limit);
        self
    }

    /// Sets the difficulty for the bundle.
    pub const fn with_difficulty(mut self, difficulty: U256) -> Self {
        self.bundle.difficulty = Some(difficulty);
        self
    }

    /// Sets the base fee for the bundle.
    pub const fn with_base_fee(mut self, base_fee: u128) -> Self {
        self.bundle.base_fee = Some(base_fee);
        self
    }

    /// Make a bundle hash from the given deserialized transaction array and host fills from this bundle.
    /// The hash is calculated as keccak256(tx_preimage + host_preimage).
    /// The tx_preimage is calculated as `keccak(tx_hash1 + tx_hash2 + ... + tx_hashn)`.
    /// The host_preimage is calculated as
    /// `keccak(NUM_OF_ASSETS_LE + asset1 + NUM_OF_FILLS_LE + asset1_user1 + user1_amount2 + ... + asset1_usern + asset1_amountn + ...)`.
    /// For the number of users/fills and amounts in the host_preimage, the amounts are serialized as little-endian U256 slice.
    pub fn bundle_hash(&self) -> B256 {
        let mut hasher = alloy_primitives::Keccak256::new();

        // Concatenate the transaction hashes, to then hash them. This is the tx_preimage.
        for tx in self.bundle.txs.iter() {
            // Calculate the tx hash (keccak256(encoded_signed_tx)) and append it to the tx_bytes.
            hasher.update(keccak256(tx).as_slice());
        }
        let tx_preimage = hasher.finalize();

        // Now, let's build the host_preimage. We do it in steps:
        // 1. Prefix the number of assets, encoded as a little-endian U256 slice.
        // 2. For each asset:
        // 3. Concatenate the asset address.
        // 4. Prefix the number of fills.
        // 5. For each fill, concatenate the user and amount, the latter encoded as a little-endian U256 slice.
        let mut hasher = alloy_primitives::Keccak256::new();

        // Prefix the list of users with the number of assets.
        hasher.update(U256::from(self.host_fills.len()).as_le_slice());

        for (asset, fills) in self.host_fills.iter() {
            // Concatenate the asset address.
            hasher.update(asset.as_slice());

            // Prefix the list of fills with the number of fills
            hasher.update(U256::from(fills.len()).as_le_slice());

            for (user, amount) in fills.iter() {
                // Concatenate the user address and amount for each fill.
                hasher.update(user.as_slice());
                hasher.update(amount.as_le_slice());
            }
        }

        // Hash the host pre-image.
        let host_preimage = hasher.finalize();

        let mut pre_image = alloy_primitives::Keccak256::new();
        pre_image.update(tx_preimage.as_slice());
        pre_image.update(host_preimage.as_slice());

        // Hash both tx and host hashes to get the final bundle hash.
        pre_image.finalize()
    }
}

/// Response for `zenith_callBundle`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ZenithCallBundleResponse {
    /// The flattened "vanilla" response which comes from `eth_callBundle`
    #[serde(flatten)]
    pub response: EthCallBundleResponse,
}
