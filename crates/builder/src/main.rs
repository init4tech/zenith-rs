#![allow(dead_code)]

mod config;
use config::load_aws_signer;

mod bindings;
use bindings::ZenithContract;

mod service;
mod tasks;

use std::borrow::Cow;
use tokio::select;

use alloy_network::EthereumSigner;
use alloy_network::TxSigner;
use alloy_primitives::{address, Address};
use alloy_provider::ProviderBuilder;

use crate::service::serve_builder_with_span;

pub struct ChainConfig {
    /// The chain ID of the host chain
    pub host_chain_id: u64,
    /// The chain ID of the host chain
    pub ru_chain_id: u64,
    /// Buffer time in seconds for the block to confirm
    pub confirmation_buffer: u64,
    /// address of the Zenith contract
    pub zenith: Address,
    /// URL for Quincey server to sign blocks
    pub quincey_url: Cow<'static, str>,
    /// URL for RPC node
    pub rpc_url: Cow<'static, str>,

    /// Wallet for signing blocks locally.
    pub local_sequencer_signer: Option<alloy_signer_wallet::LocalWallet>,

    /// Whether to use calldata or blob for transactions
    pub use_calldata: bool,
}

const HOLESKY: ChainConfig = ChainConfig {
    host_chain_id: 17000,
    ru_chain_id: 17001,
    confirmation_buffer: 60 * 20,
    zenith: address!("74ae65DF20cB0e3BF8c022051d0Cdd79cc60890C"),
    quincey_url: Cow::Borrowed("http://quincey.swanny.wtf:8080/signBlock"),
    rpc_url: Cow::Borrowed("https://ethereum-holesky-rpc.publicnode.com"),
    local_sequencer_signer: None,
    use_calldata: true,
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::try_init().unwrap();

    let span = tracing::span!(tracing::Level::INFO, "zenith-builder");

    let wallet = load_aws_signer("BUILDER_KEY_ID", Some(HOLESKY.host_chain_id)).await?;
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .signer(EthereumSigner::from(wallet.clone()))
        .on_builtin(&HOLESKY.rpc_url)
        .await?;
    let zenith = ZenithContract::new(HOLESKY.zenith, provider.clone());

    let build = tasks::block::BlockBuilder { wait_secs: 5 };
    let submit = tasks::submit::SubmitTask {
        provider,
        zenith,
        client: reqwest::Client::new(),
        config: HOLESKY,
        builder_rewards: wallet.address(),
        gas_limit: 30_000_000,
    };

    let (submit_channel, submit_jh) = submit.spawn();
    let (build_channel, build_jh) = build.spawn(submit_channel);

    let server = serve_builder_with_span(build_channel, ([0, 0, 0, 0], 6969), span);

    select! {
        _ = submit_jh => {
            tracing::info!("submit finished");
        },
        _ = build_jh => {
            tracing::info!("build finished");
        }
        _ = server => {
            tracing::info!("server finished");
        }
    }

    tracing::info!("shutting down");

    Ok(())
}
