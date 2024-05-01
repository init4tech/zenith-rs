#![allow(dead_code)]

mod env;
use env::LocalOrAws;

mod service;
mod tasks;

use alloy_network::EthereumSigner;
use alloy_network::TxSigner;
use alloy_primitives::{address, Address};
use alloy_provider::ProviderBuilder;
use std::borrow::Cow;
use tokio::select;
use zenith_types::Zenith;

use crate::service::serve_builder_with_span;

/// Configuration for a builder running a specific rollup on a specific host
/// chain.
pub struct ChainConfig {
    /// The chain ID of the host chain
    pub host_chain_id: u64,
    /// The chain ID of the host chain
    pub ru_chain_id: u64,
    /// Buffer time in seconds for the block to confirm
    pub confirmation_buffer: u64,
    /// address of the Zenith contract
    pub zenith: Address,
    /// URL for Quincey server to sign blocks. This prop is disregarded if a
    /// local_sequencer_signer is configured via the "SEQUENCER_KEY" env var.
    pub quincey_url: Cow<'static, str>,
    /// URL for RPC node
    pub rpc_url: Cow<'static, str>,

    /// Wallet for signing blocks locally.
    pub local_sequencer_signer: Option<LocalOrAws>,

    /// Whether to use calldata or blob for transactions
    pub use_calldata: bool,
}

const HOLESKY: ChainConfig = ChainConfig {
    host_chain_id: 17000,
    ru_chain_id: 17001,
    confirmation_buffer: 60 * 20,
    zenith: address!("97C0E40c6B5bb5d4fa3e2AA1C6b8bC7EA5ECAe31"),
    quincey_url: Cow::Borrowed("http://quincey.swanny.wtf:8080/signBlock"),
    rpc_url: Cow::Borrowed("https://ethereum-holesky-rpc.publicnode.com"),
    local_sequencer_signer: None,
    use_calldata: true,
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::try_init().unwrap();

    let span = tracing::info_span!("zenith-builder");

    // finish app config by loading key from env
    let config = ChainConfig {
        local_sequencer_signer: LocalOrAws::load("SEQUENCER_KEY", None).await.ok(),
        ..HOLESKY
    };

    // Load builder key from env
    let wallet = LocalOrAws::load("BUILDER_KEY_ID", Some(config.host_chain_id)).await?;
    let builder_rewards = wallet.address();

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .signer(EthereumSigner::from(wallet))
        .on_builtin(&config.rpc_url)
        .await?;
    let zenith = Zenith::new(config.zenith, provider.clone());

    let build = tasks::block::BlockBuilder { wait_secs: 5 };
    let submit = tasks::submit::SubmitTask {
        provider,
        zenith,
        client: reqwest::Client::new(),
        config,
        builder_rewards,
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
