use crate::signer::{LocalOrAws, SignerError};
use alloy::network::{Ethereum, EthereumWallet};
use alloy::providers::fillers::BlobGasFiller;
use alloy::providers::{
    fillers::{ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller},
    Identity, ProviderBuilder, RootProvider,
};
use alloy::transports::BoxTransport;
use alloy_primitives::Address;
use std::{borrow::Cow, env, num, str::FromStr};
use zenith_types::Zenith;

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
const BUILDER_REWARDS_ADDRESS: &str = "BUILDER_REWARDS_ADDRESS";
const ROLLUP_BLOCK_GAS_LIMIT: &str = "ROLLUP_BLOCK_GAS_LIMIT";
const TX_POOL_URL: &str = "TX_POOL_URL";
const TX_POOL_POLL_INTERVAL: &str = "TX_POOL_POLL_INTERVAL";
const TX_POOL_CACHE_DURATION: &str = "TX_POOL_CACHE_DURATION";
const OAUTH_CLIENT_ID: &str = "OAUTH_CLIENT_ID";
const OAUTH_CLIENT_SECRET: &str = "OAUTH_CLIENT_SECRET";
const OAUTH_AUTHENTICATE_URL: &str = "OAUTH_AUTHENTICATE_URL";
const OAUTH_TOKEN_URL: &str = "OAUTH_TOKEN_URL";
const OAUTH_AUDIENCE: &str = "OAUTH_AUDIENCE";

/// Configuration for a builder running a specific rollup on a specific host
/// chain.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
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
    /// Key to access Sequencer Wallet - AWS Key ID _OR_ local private key.
    /// Set IFF using local Sequencer signing instead of remote Quincey signing.
    pub sequencer_key: Option<String>,
    /// Key to access Builder transaction submission wallet - AWS Key ID _OR_ local private key.
    pub builder_key: String,
    /// Buffer in seconds that Builder will wait & accept incoming transactions before bundling them and submitting as a block.
    pub incoming_transactions_buffer: u64,
    /// Buffer in seconds in which the `submitBlock` transaction must confirm on the Host chain.
    pub block_confirmation_buffer: u64,
    /// Address on Rollup to which Builder will receive user transaction fees.
    pub builder_rewards_address: Address,
    /// Gas limit for RU block.
    /// NOTE: a "smart" builder would determine this programmatically by simulating the block.
    pub rollup_block_gas_limit: u64,
    /// URL of the tx pool to poll for incoming transactions.
    pub tx_pool_url: Cow<'static, str>,
    //// Interval in seconds to poll the tx-pool for new transactions.
    pub tx_pool_poll_interval: u64,
    /// Duration in seconds transactions can live in the tx-pool cache.
    pub tx_pool_cache_duration: u64,
    /// OAuth client ID for the builder.
    pub oauth_client_id: String,
    /// OAuth client secret for the builder.
    pub oauth_client_secret: String,
    /// OAuth authenticate URL for the builder for performing OAuth logins.
    pub oauth_authenticate_url: String,
    /// OAuth token URL for the builder to get an OAuth2 access token
    pub oauth_token_url: String,
    /// OAuth audience for the builder.
    pub oauth_audience: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Error loading from environment variable
    #[error("missing or non-unicode environment variable: {0}")]
    Var(String),
    /// Error parsing environment variable
    #[error("failed to parse environment variable: {0}")]
    Parse(#[from] num::ParseIntError),
    /// Error parsing boolean environment variable
    #[error("failed to parse boolean environment variable")]
    ParseBool,
    /// Error parsing hex from environment variable
    #[error("failed to parse hex: {0}")]
    Hex(#[from] hex::FromHexError),
    /// Error connecting to the provider
    #[error("failed to connect to provider: {0}")]
    Provider(#[from] alloy::transports::TransportError),
    /// Error connecting to the signer
    #[error("failed to connect to signer: {0}")]
    Signer(#[from] SignerError),
}

impl ConfigError {
    /// Missing or non-unicode env var.
    pub fn missing(s: &str) -> Self {
        ConfigError::Var(s.to_string())
    }
}

/// Provider type used by this transaction
pub type Provider = FillProvider<
    JoinFill<
        JoinFill<
            Identity,
            JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
        >,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider<BoxTransport>,
    BoxTransport,
    Ethereum,
>;

pub type ZenithInstance = Zenith::ZenithInstance<BoxTransport, Provider>;

impl BuilderConfig {
    /// Load the builder configuration from environment variables.
    pub fn load_from_env() -> Result<BuilderConfig, ConfigError> {
        Ok(BuilderConfig {
            host_chain_id: load_u64(HOST_CHAIN_ID)?,
            ru_chain_id: load_u64(RU_CHAIN_ID)?,
            host_rpc_url: load_url(HOST_RPC_URL)?,
            zenith_address: load_address(ZENITH_ADDRESS)?,
            quincey_url: load_url(QUINCEY_URL)?,
            builder_port: load_u16(BUILDER_PORT)?,
            sequencer_key: load_string_option(SEQUENCER_KEY),
            builder_key: load_string(BUILDER_KEY)?,
            incoming_transactions_buffer: load_u64(INCOMING_TRANSACTIONS_BUFFER)?,
            block_confirmation_buffer: load_u64(BLOCK_CONFIRMATION_BUFFER)?,
            builder_rewards_address: load_address(BUILDER_REWARDS_ADDRESS)?,
            rollup_block_gas_limit: load_u64(ROLLUP_BLOCK_GAS_LIMIT)?,
            tx_pool_url: load_url(TX_POOL_URL)?,
            tx_pool_poll_interval: load_u64(TX_POOL_POLL_INTERVAL)?,
            tx_pool_cache_duration: load_u64(TX_POOL_CACHE_DURATION)?,
            oauth_client_id: load_string(OAUTH_CLIENT_ID)?,
            oauth_client_secret: load_string(OAUTH_CLIENT_SECRET)?,
            oauth_authenticate_url: load_string(OAUTH_AUTHENTICATE_URL)?,
            oauth_token_url: load_string(OAUTH_TOKEN_URL)?,
            oauth_audience: load_string(OAUTH_AUDIENCE)?,
        })
    }

    pub async fn connect_builder_signer(&self) -> Result<LocalOrAws, ConfigError> {
        LocalOrAws::load(&self.builder_key, Some(self.host_chain_id)).await.map_err(Into::into)
    }

    pub async fn connect_sequencer_signer(&self) -> Result<Option<LocalOrAws>, ConfigError> {
        match &self.sequencer_key {
            Some(sequencer_key) => LocalOrAws::load(sequencer_key, Some(self.host_chain_id))
                .await
                .map_err(Into::into)
                .map(Some),
            None => Ok(None),
        }
    }

    /// Connect to the provider using the configuration.
    pub async fn connect_provider(&self) -> Result<Provider, ConfigError> {
        let builder_signer = self.connect_builder_signer().await?;
        ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(builder_signer))
            .on_builtin(&self.host_rpc_url)
            .await
            .map_err(Into::into)
    }

    pub fn connect_zenith(&self, provider: Provider) -> ZenithInstance {
        Zenith::new(self.zenith_address, provider)
    }
}

fn load_string(key: &str) -> Result<String, ConfigError> {
    env::var(key).map_err(|_| ConfigError::missing(key))
}

fn load_string_option(key: &str) -> Option<String> {
    load_string(key).ok()
}

fn load_u64(key: &str) -> Result<u64, ConfigError> {
    let val = load_string(key)?;
    val.parse::<u64>().map_err(Into::into)
}

fn load_u16(key: &str) -> Result<u16, ConfigError> {
    let val = load_string(key)?;
    val.parse::<u16>().map_err(Into::into)
}

fn load_url(key: &str) -> Result<Cow<'static, str>, ConfigError> {
    load_string(key).map_err(Into::into).map(Into::into)
}

fn load_address(key: &str) -> Result<Address, ConfigError> {
    let address = load_string(key)?;
    Address::from_str(&address).map_err(Into::into)
}
