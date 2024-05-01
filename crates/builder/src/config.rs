use alloy_signer_aws::{AwsSigner, AwsSignerError};
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
