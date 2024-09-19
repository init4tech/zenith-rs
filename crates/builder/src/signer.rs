use alloy::consensus::SignableTransaction;
use alloy::signers::aws::{AwsSigner, AwsSignerError};
use alloy::signers::local::{LocalSignerError, PrivateKeySigner};
use alloy::signers::Signature;
use alloy_primitives::{Address, ChainId, B256};
use aws_config::BehaviorVersion;

/// Abstraction over local signer or
#[derive(Debug, Clone)]
pub enum LocalOrAws {
    Local(PrivateKeySigner),
    Aws(AwsSigner),
}

#[derive(Debug, thiserror::Error)]
pub enum SignerError {
    /// Error during [`AwsSigner`] instantiation
    #[error("failed to connect AWS signer: {0}")]
    AwsSigner(#[from] AwsSignerError),
    /// Error loading the private key
    #[error("failed to load private key: {0}")]
    Wallet(#[from] LocalSignerError),
    /// Error parsing hex
    #[error("failed to parse hex: {0}")]
    Hex(#[from] hex::FromHexError),
}

impl LocalOrAws {
    /// Load a privkey or AWS signer from environment variables.
    pub async fn load(key: &str, chain_id: Option<u64>) -> Result<Self, SignerError> {
        if let Ok(wallet) = LocalOrAws::wallet(key) {
            Ok(LocalOrAws::Local(wallet))
        } else {
            let signer = LocalOrAws::aws_signer(key, chain_id).await?;
            Ok(LocalOrAws::Aws(signer))
        }
    }

    /// Load the wallet from environment variables.
    ///
    /// # Panics
    ///
    /// Panics if the env var contents is not a valid secp256k1 private key.
    fn wallet(private_key: &str) -> Result<PrivateKeySigner, SignerError> {
        let bytes = hex::decode(private_key.strip_prefix("0x").unwrap_or(private_key))?;
        Ok(PrivateKeySigner::from_slice(&bytes).unwrap())
    }

    /// Load the AWS signer from environment variables./s
    async fn aws_signer(key_id: &str, chain_id: Option<u64>) -> Result<AwsSigner, SignerError> {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let client = aws_sdk_kms::Client::new(&config);
        AwsSigner::new(client, key_id.to_string(), chain_id).await.map_err(Into::into)
    }
}

#[async_trait::async_trait]
impl alloy::network::TxSigner<Signature> for LocalOrAws {
    fn address(&self) -> Address {
        match self {
            LocalOrAws::Local(signer) => signer.address(),
            LocalOrAws::Aws(signer) => signer.address(),
        }
    }

    async fn sign_transaction(
        &self,
        tx: &mut dyn SignableTransaction<Signature>,
    ) -> alloy::signers::Result<Signature> {
        match self {
            LocalOrAws::Local(signer) => signer.sign_transaction(tx).await,
            LocalOrAws::Aws(signer) => signer.sign_transaction(tx).await,
        }
    }
}

#[async_trait::async_trait]
impl alloy::signers::Signer<Signature> for LocalOrAws {
    /// Signs the given hash.
    async fn sign_hash(&self, hash: &B256) -> alloy::signers::Result<Signature> {
        match self {
            LocalOrAws::Local(signer) => signer.sign_hash(hash).await,
            LocalOrAws::Aws(signer) => signer.sign_hash(hash).await,
        }
    }

    /// Returns the signer's Ethereum Address.
    fn address(&self) -> Address {
        match self {
            LocalOrAws::Local(signer) => signer.address(),
            LocalOrAws::Aws(signer) => signer.address(),
        }
    }

    /// Returns the signer's chain ID.
    fn chain_id(&self) -> Option<ChainId> {
        match self {
            LocalOrAws::Local(signer) => signer.chain_id(),
            LocalOrAws::Aws(signer) => signer.chain_id(),
        }
    }

    /// Sets the signer's chain ID.
    fn set_chain_id(&mut self, chain_id: Option<ChainId>) {
        match self {
            LocalOrAws::Local(signer) => signer.set_chain_id(chain_id),
            LocalOrAws::Aws(signer) => signer.set_chain_id(chain_id),
        }
    }
}
