use alloy_signer_aws::{AwsSigner, AwsSignerError};
use alloy_signer_wallet::LocalWallet;
use aws_config::BehaviorVersion;
use std::{env, num};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Error loading from environment variable
    #[error("missing environment variable: {0}")]
    Var(#[from] env::VarError),
    /// Error parsing environment variable
    #[error("failed to parse environment variable: {0}")]
    Parse(#[from] num::ParseIntError),
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

/// Load the private key from environment variables.
pub fn load_privkey(key: &str) -> Result<String, ConfigError> {
    env::var(key).map_err(Into::into)
}

/// Load the wallet from environment variables.
///
/// # Panics
///
/// Panics if the env var contents is not a valid secp256k1 private key.
pub fn load_wallet(key: &str) -> Result<LocalWallet, ConfigError> {
    let key = load_privkey(key)?;
    let bytes = hex::decode(key.strip_prefix("0x").unwrap_or(&key))?;
    Ok(LocalWallet::from_slice(&bytes).unwrap())
}

/// Load the AWS signer from environment variables./s
pub async fn load_aws_signer(key: &str, chain_id: Option<u64>) -> Result<AwsSigner, ConfigError> {
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = aws_sdk_kms::Client::new(&config);

    let key_id = load_key_id(key)?;

    AwsSigner::new(client, key_id, chain_id)
        .await
        .map_err(Into::into)
}

pub fn load_key_id(key: &str) -> Result<String, ConfigError> {
    env::var(key).map_err(Into::into)
}

/// Load the server port from environment variables.
pub fn load_port() -> Result<u16, ConfigError> {
    let port = env::var("ZENITH_PORT")?;
    let port = port.parse::<u16>()?;
    Ok(port)
}
