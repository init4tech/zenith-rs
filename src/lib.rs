#![doc = include_str!("../README.md")]
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    clippy::missing_const_for_fn,
    rustdoc::all
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

mod bindings;
pub use bindings::{
    mintCall, BundleHelper, HostOrders, Passage, RollupOrders, RollupPassage, Transactor, Zenith,
};

mod block;
pub use block::{decode_txns, encode_txns, Alloy2718Coder, Coder, ZenithBlock, ZenithTransaction};

mod orders;
pub use orders::{AggregateOrders, SignedOrder};

mod bundle;
pub use bundle::{
    ZenithCallBundle, ZenithCallBundleResponse, ZenithEthBundle, ZenithEthBundleResponse,
};

mod req;
pub use req::SignRequest;

mod resp;
pub use resp::SignResponse;

use alloy::primitives::{address, Address};

/// System address with permission to mint tokens on pre-deploys.
pub const MINTER_ADDRESS: Address = address!("00000000000000000000746f6b656e61646d696e");

/// A [`RequestSigner`] signs [`SignRequest`]s by delegating to an
/// [`alloy::signers::Signer`].
pub trait RequestSigner {
    /// Signs a [`SignRequest`] and returns the [`alloy::primitives::Signature`].
    fn sign_request(
        &self,
        request: &SignRequest,
    ) -> impl std::future::Future<
        Output = Result<alloy::primitives::PrimitiveSignature, alloy::signers::Error>,
    > + Send;
}

impl<T> RequestSigner for T
where
    T: alloy::signers::Signer + Send + Sync,
{
    async fn sign_request(
        &self,
        request: &SignRequest,
    ) -> Result<alloy::primitives::PrimitiveSignature, alloy::signers::Error> {
        let hash = request.signing_hash();
        self.sign_hash(&hash).await
    }
}
