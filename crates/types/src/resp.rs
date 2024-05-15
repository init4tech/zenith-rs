use std::sync::OnceLock;

use crate::SignRequest;
use alloy_primitives::{Address, Signature};

/// A signature response from a [`RequestSigner`].
pub struct SignResponse {
    /// The request that was signed.
    pub req: SignRequest,
    /// The signature over that request.
    pub sig: Signature,

    /// Memoized signer of the request.
    signer: OnceLock<Address>,
}

impl SignResponse {
    /// Get the signer of the request.
    ///
    /// # Panics - if recovery fails
    pub fn signer(&self) -> Address {
        *self.signer.get_or_init(|| {
            self.sig
                .recover_address_from_prehash(&self.req.signing_hash())
                .unwrap()
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{RequestSigner, SignRequest};
    use alloy_primitives::U256;

    #[tokio::test]
    async fn test_sign_response() {
        let req = SignRequest {
            host_chain_id: U256::from(1u64),
            ru_chain_id: U256::from(2u64),
            sequence: U256::from(3u64),
            confirm_by: U256::from(4u64),
            gas_limit: U256::from(5u64),
            ru_reward_address: Address::repeat_byte(6),
            contents: [7u8; 32].into(),
        };
        let signer = alloy_signer_wallet::LocalWallet::from_slice(&[8u8; 32]).unwrap();

        let sig = signer.sign_request(&req).await.unwrap();

        let resp = SignResponse {
            req,
            sig,
            signer: OnceLock::new(),
        };
        let addr = resp.signer();

        assert_eq!(addr, signer.address());
    }
}
