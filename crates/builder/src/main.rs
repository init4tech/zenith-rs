#![allow(dead_code)]

mod config;
mod service;
mod signer;
mod tasks;

use alloy_network::EthereumSigner;
use alloy_provider::ProviderBuilder;
use tokio::select;
use zenith_types::Zenith;

use crate::config::load_builder_config;
use crate::service::serve_builder_with_span;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::try_init().unwrap();
    let span = tracing::info_span!("zenith-builder");

    // load config from env
    let config = load_builder_config().await?;

    // build provider from config
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .signer(EthereumSigner::from(config.builder_signer.clone()))
        .on_builtin(&config.host_rpc_url.clone())
        .await?;

    tracing::debug!(
        rpc_url = config.host_rpc_url.as_ref(),
        "instantiated provider"
    );

    // build zenith from config
    let zenith = Zenith::new(config.zenith_address, provider.clone());

    let port = config.builder_port;

    let build = tasks::block::BlockBuilder::new(&config);

    let submit = tasks::submit::SubmitTask {
        provider,
        zenith,
        client: reqwest::Client::new(),
        config,
    };

    let (submit_channel, submit_jh) = submit.spawn();

    let (build_channel, build_jh) = build.spawn(submit_channel);

    // server
    let server = serve_builder_with_span(build_channel, ([0, 0, 0, 0], port), span);

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
