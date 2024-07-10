use crate::SignRequest;
use alloy_primitives::{Address, Signature, SignatureError};
use serde::{Deserialize, Serialize};

/// A signature response from a [`RequestSigner`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SignResponse {
    /// The request that was signed.
    pub req: SignRequest,
    /// The signature over that request.
    pub sig: Signature,
}

impl SignResponse {
    /// Get the signer of the request.
    ///
    /// # Panics
    ///
    /// - If recovery fails due to a k256 error.
    pub fn signer(&self) -> Result<Address, SignatureError> {
        self.sig.recover_address_from_prehash(&self.req.signing_hash())
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
            host_block_number: U256::from(0), // TODO assign correct host block number
            host_chain_id: U256::from(1u64),
            ru_chain_id: U256::from(2u64),
            gas_limit: U256::from(5u64),
            ru_reward_address: Address::repeat_byte(6),
            contents: [7u8; 32].into(),
        };
        let signer = alloy_signer_local::PrivateKeySigner::from_slice(&[8u8; 32]).unwrap();
        let sig = signer.sign_request(&req).await.unwrap();
        let resp = SignResponse { req, sig };
        let addr = resp.signer().unwrap();

        assert_eq!(addr, signer.address());
    }

    #[tokio::test]
    async fn deser_roundtrip() {
        let req = SignRequest {
            host_block_number: U256::from(0),
            host_chain_id: U256::from(1u64),
            ru_chain_id: U256::from(2u64),
            gas_limit: U256::from(5u64),
            ru_reward_address: Address::repeat_byte(6),
            contents: [7u8; 32].into(),
        };
        let signer = alloy_signer_local::PrivateKeySigner::from_slice(&[8u8; 32]).unwrap();

        let sig = signer.sign_request(&req).await.unwrap();

        let resp = SignResponse { req, sig };

        let json = serde_json::to_string(&resp).unwrap();
        let resp2: SignResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(resp, resp2);
    }
}
