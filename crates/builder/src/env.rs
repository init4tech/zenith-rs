use alloy_consensus::SignableTransaction;
use alloy_primitives::{Address, ChainId, B256};
use alloy_signer::Signature;
use alloy_signer_aws::{AwsSigner, AwsSignerError};
use alloy_signer_wallet::LocalWallet;
use aws_config::BehaviorVersion;
use std::{borrow::Cow, env, num, str::FromStr};

/// Configuration for a builder running a specific rollup on a specific host
/// chain.
pub struct BuilderConfig {
    /// The chain ID of the host chain
    pub host_chain_id: u64,
    /// The chain ID of the host chain
    pub ru_chain_id: u64,
    /// Buffer time in seconds for the block to confirm
    pub confirmation_buffer: u64,
    /// Buffer time in seconds that Builder will wait before submitting a block
    /// containing whatever transactions it has received in the meantime.
    pub wait_before_submitting: u64,
    /// address of the Zenith contract
    pub zenith: Address,
    /// URL for RPC node
    pub rpc_url: Cow<'static, str>,
    /// URL for Quincey server to sign blocks. This prop is disregarded if a
    /// local_sequencer_signer is configured via the "SEQUENCER_KEY" env var.
    pub quincey_url: Cow<'static, str>,
    /// Wallet for signing blocks locally.
    pub local_sequencer_signer: Option<LocalOrAws>,
    /// Wallet for signing blocks locally.
    pub builder_wallet: LocalOrAws,
    /// address to which Builder will receive transaction fees on Rollup
    pub builder_rewards_address: Address,
    /// Whether to use calldata or blob for transactions
    pub use_calldata: bool,
    /// Gas limit for RU block
    pub gas_limit: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Error loading from environment variable
    #[error("missing environment variable: {0}")]
    Var(#[from] env::VarError),
    /// Error parsing environment variable
    #[error("failed to parse environment variable: {0}")]
    Parse(#[from] num::ParseIntError),
    /// Error parsing boolean environment variable
    #[error("failed to parse boolean environment variable")]
    ParseBool(),
    /// Error during [`AwsSigner`] instantiation
    #[error("failed to connect AWS signer: {0}")]
    AwsSigner(#[from] AwsSignerError),
    /// Error parsing hex
    #[error("failed to parse hex: {0}")]
    Hex(#[from] hex::FromHexError),
    /// Error loading the private key
    #[error("failed to load private key: {0}")]
    Wallet(#[from] alloy_signer_wallet::WalletError),
}

/// Abstraction over local signer or
#[derive(Debug, Clone)]
pub enum LocalOrAws {
    Local(LocalWallet),
    Aws(AwsSigner),
}

impl LocalOrAws {
    /// Load a privkey or AWS signer from environment variables.
    pub async fn load(key: &str, chain_id: Option<u64>) -> Result<Self, ConfigError> {
        if let Ok(wallet) = load_wallet(key) {
            Ok(LocalOrAws::Local(wallet))
        } else {
            let signer = load_aws_signer(key, chain_id).await?;
            Ok(LocalOrAws::Aws(signer))
        }
    }
}

#[async_trait::async_trait]
impl alloy_network::TxSigner<Signature> for LocalOrAws {
    fn address(&self) -> Address {
        match self {
            LocalOrAws::Local(signer) => signer.address(),
            LocalOrAws::Aws(signer) => signer.address(),
        }
    }

    async fn sign_transaction(
        &self,
        tx: &mut dyn SignableTransaction<Signature>,
    ) -> alloy_signer::Result<Signature> {
        match self {
            LocalOrAws::Local(signer) => signer.sign_transaction(tx).await,
            LocalOrAws::Aws(signer) => signer.sign_transaction(tx).await,
        }
    }
}

#[async_trait::async_trait]
impl alloy_signer::Signer<Signature> for LocalOrAws {
    /// Signs the given hash.
    async fn sign_hash(&self, hash: &B256) -> alloy_signer::Result<Signature> {
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

pub fn load_string(key: &str) -> Result<String, ConfigError> {
    env::var(key).map_err(Into::into)
}

pub fn load_u64(key: &str) -> Result<u64, ConfigError> {
    let val = env::var(key)?;
    val.parse::<u64>().map_err(Into::into)
}

pub fn load_url(key: &str) -> Result<Cow<'static, str>, ConfigError> {
    env::var(key).map_err(Into::into).map(Into::into)
}

pub fn load_address(key: &str) -> Result<Address, ConfigError> {
    let address = env::var(key)?;
    Address::from_str(&address).map_err(Into::into)
}

pub fn load_bool(key: &str) -> Result<bool, ConfigError> {
    let val = env::var(key)?;
    match val.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(ConfigError::ParseBool()),
    }
}

pub async fn load_builder_config() -> Result<BuilderConfig, ConfigError> {
    let host_chain = load_u64("HOST_CHAIN_ID")?;
    Ok(BuilderConfig {
        host_chain_id: host_chain,
        ru_chain_id: load_u64("RU_CHAIN_ID")?,
        confirmation_buffer: load_u64("CONFIRMATION_BUFFER")?,
        zenith: load_address("ZENITH_ADDRESS")?,
        quincey_url: load_url("QUINCEY_URL")?,
        rpc_url: load_url("RPC_URL")?,
        use_calldata: load_bool("USE_CALLDATA")?,
        wait_before_submitting: load_u64("WAIT_BEFORE_SUBMITTING")?,
        local_sequencer_signer: None, // TODO: i think we don't want this to be configurable in our builder?
        builder_wallet: LocalOrAws::load("BUILDER_KEY_ID", Some(host_chain)).await?,
        builder_rewards_address: load_address("BUILDER_REWARDS_ADDRESS")?,
        gas_limit: load_u64("GAS_LIMIT")?,
    })
}

/// Load the wallet from environment variables.
///
/// # Panics
///
/// Panics if the env var contents is not a valid secp256k1 private key.
pub fn load_wallet(key: &str) -> Result<LocalWallet, ConfigError> {
    let key = load_string(key)?;
    let bytes = hex::decode(key.strip_prefix("0x").unwrap_or(&key))?;
    Ok(LocalWallet::from_slice(&bytes).unwrap())
}

/// Load the AWS signer from environment variables./s
pub async fn load_aws_signer(key: &str, chain_id: Option<u64>) -> Result<AwsSigner, ConfigError> {
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = aws_sdk_kms::Client::new(&config);

    let key_id = load_string(key)?;

    AwsSigner::new(client, key_id, chain_id)
        .await
        .map_err(Into::into)
}
