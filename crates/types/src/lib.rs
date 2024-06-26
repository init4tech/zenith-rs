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
pub use bindings::{Orders, Zenith};

mod block;
pub use block::{decode_txns, encode_txns, Alloy2718Coder, Coder, ZenithBlock, ZenithTransaction};

mod req;
pub use req::SignRequest;

mod resp;
pub use resp::SignResponse;

/// A [`RequestSigner`] signs [`SignRequest`]s by delegating to an
/// [`alloy_signer::Signer`].
pub trait RequestSigner {
    /// Signs a [`SignRequest`] and returns the [`alloy_primitives::Signature`].
    fn sign_request(
        &self,
        request: &SignRequest,
    ) -> impl std::future::Future<Output = Result<alloy_primitives::Signature, alloy_signer::Error>> + Send;
}

impl<T> RequestSigner for T
where
    T: alloy_signer::Signer + Send + Sync,
{
    async fn sign_request(
        &self,
        request: &SignRequest,
    ) -> Result<alloy_primitives::Signature, alloy_signer::Error> {
        let hash = request.signing_hash();
        self.sign_hash(&hash).await
    }
}
