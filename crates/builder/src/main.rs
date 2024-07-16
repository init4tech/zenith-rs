#![allow(dead_code)]

mod config;
mod service;
mod signer;
mod tasks;

use tokio::select;

use crate::config::BuilderConfig;
use crate::service::serve_builder_with_span;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::try_init().unwrap();
    let span = tracing::info_span!("zenith-builder");

    let config = BuilderConfig::load_from_env()?;
    let provider = config.connect_provider().await?;

    tracing::debug!(rpc_url = config.host_rpc_url.as_ref(), "instantiated provider");

    let sequencer_signer = config.connect_sequencer_signer().await?;
    let zenith = config.connect_zenith(provider.clone());

    let port = config.builder_port;

    let tx_poller = tasks::block::TxPoller::new(&config);
    let build = tasks::block::BlockBuilder::new(&config, tx_poller);

    let submit = tasks::submit::SubmitTask {
        provider,
        zenith,
        client: reqwest::Client::new(),
        sequencer_signer,
        config,
    };

    let (submit_channel, submit_jh) = submit.spawn();
    let (build_channel, build_jh) = build.spawn(submit_channel);

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
