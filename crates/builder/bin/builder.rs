#![allow(dead_code)]

use builder::config::BuilderConfig;
use builder::service::serve_builder_with_span;
use builder::tasks::tx_poller::TxPoller;

use tokio::select;

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

    let tx_poller = TxPoller::new(&config);
    let builder = builder::tasks::block::BlockBuilder::new(&config);

    let submit = builder::tasks::submit::SubmitTask {
        provider,
        zenith,
        client: reqwest::Client::new(),
        sequencer_signer,
        config,
    };

    let (submit_channel, submit_jh) = submit.spawn();
    let (build_channel, build_jh) = builder.spawn(submit_channel);
    let tx_poller_jh = tx_poller.spawn(build_channel.clone());

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
        _ = tx_poller_jh => {
            tracing::info!("tx_poller finished");
        }
    }

    tracing::info!("shutting down");

    Ok(())
}
