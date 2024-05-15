#![allow(dead_code)]

mod env;

mod service;
mod tasks;

use alloy_network::EthereumSigner;
use alloy_provider::ProviderBuilder;
use tokio::select;
use zenith_types::Zenith;

use crate::env::load_builder_config;
use crate::service::serve_builder_with_span;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::try_init().unwrap();

    let span = tracing::info_span!("zenith-builder");

    // finish app config by loading key from env
    let config = load_builder_config().await?;

    // provider is built from config
    let provider: alloy_provider::fillers::FillProvider<
        alloy_provider::fillers::JoinFill<
            alloy_provider::fillers::JoinFill<
                alloy_provider::fillers::JoinFill<
                    alloy_provider::fillers::JoinFill<
                        alloy_provider::Identity,
                        alloy_provider::fillers::GasFiller,
                    >,
                    alloy_provider::fillers::NonceFiller,
                >,
                alloy_provider::fillers::ChainIdFiller,
            >,
            alloy_provider::fillers::SignerFiller<EthereumSigner>,
        >,
        alloy_provider::RootProvider<alloy_transport::BoxTransport>,
        alloy_transport::BoxTransport,
        alloy_network::Ethereum,
    > = ProviderBuilder::new()
        .with_recommended_fillers()
        .signer(EthereumSigner::from(config.builder_wallet.clone()))
        .on_builtin(&config.rpc_url.clone())
        .await?;
    tracing::debug!(rpc_url = config.rpc_url.as_ref(), "instantiated provider");

    // zenith is built from config
    let zenith = Zenith::new(config.zenith, provider.clone());

    let build = tasks::block::BlockBuilder {
        wait_secs: config.wait_before_submitting,
    };
    let submit = tasks::submit::SubmitTask {
        provider,
        zenith,
        client: reqwest::Client::new(),
        config,
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
