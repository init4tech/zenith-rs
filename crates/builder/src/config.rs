use crate::signer::LocalOrAws;
use alloy_primitives::Address;
use alloy_signer_aws::AwsSignerError;
use std::{borrow::Cow, env, num, str::FromStr};

// Keys for .env variables that need to be set to configure the builder.
const HOST_CHAIN_ID: &str = "HOST_CHAIN_ID";
const RU_CHAIN_ID: &str = "RU_CHAIN_ID";
const HOST_RPC_URL: &str = "HOST_RPC_URL";
const ZENITH_ADDRESS: &str = "ZENITH_ADDRESS";
const QUINCEY_URL: &str = "QUINCEY_URL";
const BUILDER_PORT: &str = "BUILDER_PORT";
const SEQUENCER_KEY: &str = "SEQUENCER_KEY"; // empty (to use Quincey) OR AWS key ID (to use AWS signer) OR raw private key (to use local signer)
const BUILDER_KEY: &str = "BUILDER_KEY"; // AWS key ID (to use AWS signer) OR raw private key (to use local signer)
const INCOMING_TRANSACTIONS_BUFFER: &str = "INCOMING_TRANSACTIONS_BUFFER";
const BLOCK_CONFIRMATION_BUFFER: &str = "BLOCK_CONFIRMATION_BUFFER";
const SUBMIT_VIA_CALLDATA: &str = "SUBMIT_VIA_CALLDATA";
const BUILDER_REWARDS_ADDRESS: &str = "BUILDER_REWARDS_ADDRESS";
const ROLLUP_BLOCK_GAS_LIMIT: &str = "ROLLUP_BLOCK_GAS_LIMIT";

/// Configuration for a builder running a specific rollup on a specific host
/// chain.
pub struct BuilderConfig {
    /// The chain ID of the host chain
    pub host_chain_id: u64,
    /// The chain ID of the host chain
    pub ru_chain_id: u64,
    /// URL for Host RPC node.
    pub host_rpc_url: Cow<'static, str>,
    /// address of the Zenith contract on Host.
    pub zenith_address: Address,
    /// URL for remote Quincey Sequencer server to sign blocks.
    /// Disregarded if a sequencer_signer is configured.
    pub quincey_url: Cow<'static, str>,
    /// Port for the Builder server.
    pub builder_port: u16,
    /// Wallet for local Sequencer to sign blocks.
    /// Set IFF using local Sequencer signing instead of remote Quincey signing.
    pub sequencer_signer: Option<LocalOrAws>,
    /// Wallet for submitting blocks to Host chain.
    pub builder_signer: LocalOrAws,
    /// Buffer in seconds that Builder will wait & accept incoming transactions before bundling them and submitting as a block.
    pub incoming_transactions_buffer: u64,
    /// Buffer in seconds in which the `submitBlock` transaction must confirm on the Host chain.
    pub block_confirmation_buffer: u64,
    /// TRUE to use calldata as DA for submitting blocks; FALSE to use blobs.
    /// NOTE: a "smart" builder would determine this programmatically based on current lowest fees.
    pub submit_via_calldata: bool,
    /// Address on Rollup to which Builder will receive user transaction fees.
    pub builder_rewards_address: Address,
    /// Gas limit for RU block.
    /// NOTE: a "smart" builder would determine this programmatically by simulating the block.
    pub rollup_block_gas_limit: u64,
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

pub async fn load_builder_config() -> Result<BuilderConfig, ConfigError> {
    let host_chain = load_u64(HOST_CHAIN_ID)?;
    Ok(BuilderConfig {
        host_chain_id: host_chain,
        ru_chain_id: load_u64(RU_CHAIN_ID)?,
        host_rpc_url: load_url(HOST_RPC_URL)?,
        zenith_address: load_address(ZENITH_ADDRESS)?,
        quincey_url: load_url(QUINCEY_URL)?,
        builder_port: load_u16(BUILDER_PORT)?,
        sequencer_signer: LocalOrAws::load_option(SEQUENCER_KEY, Some(host_chain)).await?,
        builder_signer: LocalOrAws::load(BUILDER_KEY, Some(host_chain)).await?,
        incoming_transactions_buffer: load_u64(INCOMING_TRANSACTIONS_BUFFER)?,
        block_confirmation_buffer: load_u64(BLOCK_CONFIRMATION_BUFFER)?,
        submit_via_calldata: load_bool(SUBMIT_VIA_CALLDATA)?,
        builder_rewards_address: load_address(BUILDER_REWARDS_ADDRESS)?,
        rollup_block_gas_limit: load_u64(ROLLUP_BLOCK_GAS_LIMIT)?,
    })
}

pub fn load_string(key: &str) -> Result<String, ConfigError> {
    env::var(key).map_err(Into::into)
}

pub fn load_string_option(key: &str) -> Result<Option<String>, ConfigError> {
    let val = env::var(key);
    match val {
        Ok(val) => Ok(Some(val)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn load_u64(key: &str) -> Result<u64, ConfigError> {
    let val = env::var(key)?;
    val.parse::<u64>().map_err(Into::into)
}

pub fn load_u16(key: &str) -> Result<u16, ConfigError> {
    let val = env::var(key)?;
    val.parse::<u16>().map_err(Into::into)
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
