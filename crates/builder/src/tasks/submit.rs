use alloy_consensus::SimpleCoder;
use alloy_network::TransactionBuilder;
use alloy_primitives::{FixedBytes, U256};
use alloy_provider::{Provider as _, WalletProvider};
use alloy_rpc_types_eth::{BlockId, BlockNumberOrTag, TransactionRequest};
use alloy_signer::Signer;
use alloy_sol_types::SolCall;
use alloy_transport::TransportError;
use eyre::bail;
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::{debug, error, instrument, trace};
use zenith_types::{SignRequest, SignResponse, Zenith};

use crate::{
    config::{Provider, ZenithInstance},
    signer::LocalOrAws,
    tasks::block::InProgressBlock,
};

/// Submits sidecars in ethereum txns to mainnet ethereum
pub struct SubmitTask {
    /// Ethereum Provider
    pub provider: Provider,

    /// Zenity
    pub zenith: ZenithInstance,

    /// Reqwest
    pub client: reqwest::Client,

    /// Sequencer Signer
    pub sequencer_signer: Option<LocalOrAws>,

    /// Config
    pub config: crate::config::BuilderConfig,
}

impl SubmitTask {
    async fn sup_quincey(&self, sig_request: &SignRequest) -> eyre::Result<SignResponse> {
        tracing::info!(
            // sequence = %sig_request.TODO(), // TODO log correct sequence number
            "pinging quincey for signature"
        );

        let resp: reqwest::Response = self
            .client
            .post(self.config.quincey_url.as_ref())
            .json(sig_request)
            .send()
            .await?
            .error_for_status()?;

        let body = resp.bytes().await?;

        debug!(bytes = body.len(), "retrieved response body");
        trace!(body = %String::from_utf8_lossy(&body), "response body");

        serde_json::from_slice(&body).map_err(Into::into)
    }

    #[instrument(skip_all)]
    async fn construct_sig_request(&self, contents: &InProgressBlock) -> eyre::Result<SignRequest> {
        Ok(SignRequest {
            host_block_number: U256::from(0), // TODO get and set proper sequence number
            host_chain_id: U256::from(self.config.host_chain_id),
            ru_chain_id: U256::from(self.config.ru_chain_id),
            gas_limit: U256::from(self.config.rollup_block_gas_limit),
            ru_reward_address: self.config.builder_rewards_address,
            contents: contents.contents_hash(),
        })
    }

    fn build_blob_tx(
        &self,
        header: Zenith::BlockHeader,
        v: u8,
        r: FixedBytes<32>,
        s: FixedBytes<32>,
        in_progress: &InProgressBlock,
    ) -> eyre::Result<TransactionRequest> {
        let data = Zenith::submitBlockCall { header, v, r, s, _4: Default::default() }.abi_encode();
        let sidecar = in_progress.encode_blob::<SimpleCoder>().build()?;
        Ok(TransactionRequest::default().with_blob_sidecar(sidecar).with_input(data))
    }

    async fn submit_transaction(
        &self,
        resp: &SignResponse,
        in_progress: &InProgressBlock,
    ) -> eyre::Result<()> {
        let v: u8 = resp.sig.v().y_parity_byte() + 27;
        let r: FixedBytes<32> = resp.sig.r().into();
        let s: FixedBytes<32> = resp.sig.s().into();

        let header = Zenith::BlockHeader {
            hostBlockNumber: todo!(),
            gasLimit: todo!(),
            rollupChainId: todo!(),
            blockDataHash: todo!(),
            rewardAddress: todo!(),
            // rollupChainId: U256::from(self.config.ru_chain_id),
            // sequence: resp.req.sequence,
            // gasLimit: resp.req.gas_limit,
            // confirmBy: resp.req.confirm_by,
            // rewardAddress: resp.req.ru_reward_address,
            // blockDataHash: in_progress.contents_hash(),
        };

        let tx = self
            .build_blob_tx(header, v, r, s, in_progress)?
            .with_from(self.provider.default_signer_address())
            .with_to(self.config.zenith_address);

        if let Err(TransportError::ErrorResp(e)) = self.provider.call(&tx).await {
            error!(
                code = e.code,
                message = %e.message,
                data = ?e.data,
                "error in transaction submission"
            );

            bail!("bailing transaction submission")
        }

        tracing::debug!(
            // sequence = %resp.req.sequence, // TODO log correct host block number
            gas_limit = %resp.req.gas_limit,
            "sending transaction to network"
        );

        let result = self.provider.send_transaction(tx).await?;

        let tx_hash = result.tx_hash();

        tracing::info!(
            %tx_hash,
            // sequence = %resp.req.sequence, // TODO log correct host block number
            gas_limit = %resp.req.gas_limit,
            "dispatched to network"
        );

        Ok(())
    }

    #[instrument(skip_all, err)]
    async fn handle_inbound(&self, in_progress: &InProgressBlock) -> eyre::Result<()> {
        tracing::info!(txns = in_progress.len(), "handling inbound block");
        let sig_request = self.construct_sig_request(in_progress).await?;

        tracing::debug!(
            // sequence = %sig_request.sequence,// TODO log correct host block number and rollup chain id
            "constructed signature request"
        );

        // If configured with a local signer, we use it. Otherwise, we ask
        // quincey (politely)
        let signed = if let Some(signer) = &self.sequencer_signer {
            let sig = signer.sign_hash(&sig_request.signing_hash()).await?;
            tracing::debug!(
                sig = hex::encode(sig.as_bytes()),
                "acquied signature from local signer"
            );
            SignResponse { req: sig_request, sig }
        } else {
            let resp: SignResponse = self.sup_quincey(&sig_request).await?;
            tracing::debug!(
                sig = hex::encode(resp.sig.as_bytes()),
                "acquied signature from quincey"
            );
            resp
        };

        self.submit_transaction(&signed, in_progress).await
    }
}

impl SubmitTask {
    /// Spawn the task.
    pub fn spawn(self) -> (mpsc::UnboundedSender<InProgressBlock>, JoinHandle<()>) {
        let (sender, mut inbound) = mpsc::unbounded_channel();
        let handle = tokio::spawn(async move {
            loop {
                if let Some(in_progress) = inbound.recv().await {
                    if let Err(e) = self.handle_inbound(&in_progress).await {
                        error!(%e, "error in block submission. Dropping block.");
                    }
                } else {
                    tracing::debug!("upstream task gone");
                    break;
                }
            }
        });

        (sender, handle)
    }
}
