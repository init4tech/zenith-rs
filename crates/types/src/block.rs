use std::{marker::PhantomData, sync::OnceLock};

use crate::Zenith::BlockHeader as ZenithHeader;
use alloy_consensus::TxEnvelope;
use alloy_eips::eip2718::{Decodable2718, Eip2718Error, Encodable2718};
use alloy_primitives::{keccak256, Address, B256};

/// Zenith processes normal Ethereum txns.
pub type ZenithTransaction = TxEnvelope;

/// Encode/Decode trait for inner tx type
pub trait Coder {
    /// The inner tx type.
    type Tx: std::fmt::Debug + Clone + PartialEq + Eq;

    /// Encode the tx.
    fn encode(t: &Self::Tx) -> Vec<u8>;

    /// Decode the tx.
    fn decode(buf: &mut &[u8]) -> Option<Self::Tx>
    where
        Self: Sized;
}

#[derive(Copy, Clone, Debug)]
pub struct Alloy2718Coder;

impl Coder for Alloy2718Coder {
    type Tx = ZenithTransaction;

    fn encode(t: &ZenithTransaction) -> Vec<u8> {
        t.encoded_2718()
    }

    fn decode(buf: &mut &[u8]) -> Option<ZenithTransaction>
    where
        Self: Sized,
    {
        ZenithTransaction::decode_2718(buf).ok()
    }
}

/// A Zenith block is just a list of transactions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZenithBlock<C: Coder = Alloy2718Coder> {
    /// The zenith block header, which may be extracted from a
    /// [`crate::Zenith::BlockSubmitted`] event.
    header: ZenithHeader,
    /// The transactions in the block, which are extracted from the calldata or
    /// blob data.
    transactions: Vec<<C as Coder>::Tx>,

    // memoization fields
    encoded: OnceLock<Vec<u8>>,
    block_data_hash: OnceLock<B256>,

    /// The coder
    _pd: std::marker::PhantomData<C>,
}

impl<C> ZenithBlock<C>
where
    C: Coder,
{
    /// Create a new zenith block.
    pub fn new(header: ZenithHeader, transactions: Vec<<C as Coder>::Tx>) -> Self {
        ZenithBlock {
            header,
            transactions,
            encoded: OnceLock::new(),
            block_data_hash: OnceLock::new(),
            _pd: PhantomData,
        }
    }

    /// Decode tx data in the block.
    pub fn from_header_and_data(
        header: ZenithHeader,
        buf: impl AsRef<[u8]>,
    ) -> Result<Self, Eip2718Error> {
        let b = buf.as_ref();
        let transactions = decode_txns::<C>(b)?;
        let h = keccak256(b);
        Ok(ZenithBlock {
            header,
            transactions,
            encoded: b.to_owned().into(),
            block_data_hash: h.into(),
            _pd: PhantomData,
        })
    }

    /// Break the block into its parts.
    pub fn into_parts(self) -> (ZenithHeader, Vec<C::Tx>) {
        (self.header, self.transactions)
    }

    /// Encode the transactions in the block.
    pub fn encoded_txns(&self) -> &[u8] {
        self.seal();
        self.encoded.get().unwrap().as_slice()
    }

    /// The hash of the encoded transactions.
    pub fn block_data_hash(&self) -> B256 {
        self.seal();
        *self.block_data_hash.get().unwrap()
    }

    /// Push a transaction into the block.
    pub fn push_transaction(&mut self, tx: C::Tx) {
        self.unseal();
        self.transactions.push(tx);
    }

    /// Access to the transactions.
    pub fn transactions(&self) -> &[C::Tx] {
        &self.transactions
    }

    /// Mutable access to the transactions.
    pub fn transactions_mut(&mut self) -> &mut Vec<C::Tx> {
        self.unseal();
        &mut self.transactions
    }

    /// Iterate over the transactions.
    pub fn transactions_iter(&self) -> std::slice::Iter<'_, C::Tx> {
        self.transactions.iter()
    }

    /// Iterate over mut transactions.
    pub fn transactions_iter_mut(&mut self) -> std::slice::IterMut<'_, C::Tx> {
        self.unseal();
        self.transactions.iter_mut()
    }

    /// Access to the header.
    pub const fn header(&self) -> &ZenithHeader {
        &self.header
    }

    /// Mutable access to the header.
    pub fn header_mut(&mut self) -> &mut ZenithHeader {
        &mut self.header
    }

    fn seal(&self) {
        let encoded = self.encoded.get_or_init(|| encode_transactions::<C>(&self.transactions));
        self.block_data_hash.get_or_init(|| keccak256(encoded));
    }

    fn unseal(&mut self) {
        self.encoded.take();
        self.block_data_hash.take();
    }

    /// Get the chain ID of the block (discarding high bytes).
    pub const fn chain_id(&self) -> u64 {
        self.header.chain_id()
    }

    /// Get the sequence of the block (discarding high bytes).
    pub const fn sequence(&self) -> u64 {
        self.header.sequence()
    }

    /// Get the confirm by time of the block (discarding high bytes).
    pub const fn confirm_by(&self) -> u64 {
        self.header.confirm_by()
    }

    /// Get the gas limit of the block (discarding high bytes).
    pub const fn gas_limit(&self) -> u64 {
        self.header.gas_limit()
    }

    /// Get the reward address of the block.
    pub const fn reward_address(&self) -> Address {
        self.header.reward_address()
    }
}

/// Decode transactions.
///
/// A transaction is an RLP-encoded list of EIP-2718-encoded transaction
/// envelopes.
///
/// A [`encode_txns`] has been provided for completeness.
pub(crate) fn decode_txns<C>(block_data: impl AsRef<[u8]>) -> Result<Vec<C::Tx>, Eip2718Error>
where
    C: Coder,
{
    let mut bd = block_data.as_ref();

    let rlp: Vec<Vec<u8>> = alloy_rlp::Decodable::decode(&mut bd)?;

    Ok(rlp.into_iter().flat_map(|buf| C::decode(&mut buf.as_slice())).collect())
}

/// Encode a set of transactions into a single RLP-encoded buffer.
pub(crate) fn encode_transactions<'a, C>(
    transactions: impl IntoIterator<Item = &'a C::Tx>,
) -> Vec<u8>
where
    C: Coder,
    C::Tx: 'a,
{
    let encoded_txns = transactions.into_iter().map(|tx| C::encode(tx)).collect::<Vec<Vec<u8>>>();

    let mut buf = Vec::new();
    alloy_rlp::Encodable::encode(&encoded_txns, &mut buf);
    buf
}

#[cfg(test)]
mod test {
    use alloy_consensus::{Signed, TxEip1559};
    use alloy_primitives::{b256, bytes, Address, U256};
    use alloy_signer::Signature;

    use super::*;

    #[test]
    fn encode_decode() {
        let sig = Signature::from_scalars_and_parity(
            b256!("840cfc572845f5786e702984c2a582528cad4b49b2a10b9db1be7fca90058565"),
            b256!("25e7109ceb98168d95b09b18bbf6b685130e0562f233877d492b94eee0c5b6d1"),
            false,
        )
        .unwrap();

        let tx = ZenithTransaction::Eip1559(Signed::new_unchecked(
            TxEip1559 {
                chain_id: 1,
                nonce: 2,
                gas_limit: 3,
                max_fee_per_gas: 4,
                max_priority_fee_per_gas: 5,
                to: Address::repeat_byte(6).into(),
                value: U256::from(7),
                access_list: Default::default(),
                input: bytes!("08090a0b0c0d0e0f"),
            },
            sig,
            b256!("87fdda4563f2f98ac9c3f076bca48a59309df94f13fb8abf8471b3b8b51a2816"),
        ));

        let mut txs = vec![tx.clone()];
        let encoded = encode_transactions::<Alloy2718Coder>(&txs);
        let decoded = decode_txns::<Alloy2718Coder>(encoded).unwrap();

        assert_eq!(txs, decoded);

        txs.push(tx.clone());
        let encoded = encode_transactions::<Alloy2718Coder>(&txs);
        let decoded = decode_txns::<Alloy2718Coder>(encoded).unwrap();

        assert_eq!(txs, decoded);
    }
}
