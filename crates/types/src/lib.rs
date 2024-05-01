mod req;
pub use req::SignRequest;

pub trait RequestSigner {
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
