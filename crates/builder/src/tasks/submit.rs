use alloy_consensus::SimpleCoder;
use alloy_network::TransactionBuilder;
use alloy_primitives::{FixedBytes, U256};
use alloy_provider::{Provider as _, WalletProvider};
use alloy_rpc_types::{BlockId, BlockNumberOrTag, TransactionRequest};
use alloy_signer::Signer;
use alloy_sol_types::SolCall;
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::{debug, instrument, trace};
use zenith_types::{SignRequest, SignResponse, Zenith};
use Zenith::BlockHeader as ZenithHeader;

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
    async fn get_confirm_by(&self) -> eyre::Result<u64> {
        self.provider
            .get_block(BlockId::Number(BlockNumberOrTag::Latest), false)
            .await
            .map_err(Into::<eyre::Report>::into)?
            .ok_or_else(|| eyre::eyre!("latest block is none"))
            .map(|block| block.header.timestamp + self.config.block_confirmation_buffer)
    }

    /// Get the next sequence number from the chain
    ///
    /// # Note
    ///
    /// Produces bad output if the rollup has more than 18446744073709551615
    /// blocks. Seems fine lol.
    async fn get_next_sequence(&self) -> eyre::Result<u64> {
        self.zenith
            .nextSequence(U256::from(self.config.ru_chain_id))
            .call()
            .await
            .map(|resp| resp._0.as_limbs()[0])
            .map_err(Into::into)
    }

    /// Get the signature from our main man quincey.
    async fn sup_quincey(&self, sig_request: &SignRequest) -> eyre::Result<SignResponse> {
        tracing::info!(
            sequence = %sig_request.header.sequence(),
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

    #[instrument(skip_all, err)]
    async fn construct_sig_request(&self, block: &InProgressBlock) -> eyre::Result<SignRequest> {
        let sequence = self.get_next_sequence().await?;
        let confirm_by = self.get_confirm_by().await?;

        Ok(SignRequest {
            host_chain_id: U256::from(self.config.host_chain_id),
            header: ZenithHeader {
                rollupChainId: U256::from(self.config.ru_chain_id),
                sequence: U256::from(sequence),
                confirmBy: U256::from(confirm_by),
                gasLimit: U256::from(self.config.rollup_block_gas_limit),
                rewardAddress: self.config.builder_rewards_address,
                blockDataHash: block.block_data_hash(),
            },
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

        let tx = self
            .build_blob_tx(resp.req.header, v, r, s, in_progress)?
            .with_from(self.provider.default_signer_address())
            .with_to(self.config.zenith_address);

        tracing::debug!(
            sequence = %resp.req.header.sequence(),
            gas_limit = %resp.req.header.gas_limit(),
            "sending transaction to network"
        );

        let result = self.provider.send_transaction(tx).await?;

        let tx_hash = result.tx_hash();

        tracing::info!(
            %tx_hash,
            sequence = %resp.req.header.sequence(),
            gas_limit = %resp.req.header.gas_limit(),
            "dispatched to network"
        );

        Ok(())
    }

    #[instrument(skip_all, err)]
    async fn handle_inbound(&self, in_progress: &InProgressBlock) -> eyre::Result<()> {
        tracing::info!(txns = in_progress.len(), "handling inbound block");
        let sig_request = self.construct_sig_request(in_progress).await?;

        tracing::debug!(
            sequence = %sig_request.header.sequence(),
            confirm_by = %sig_request.header.confirm_by(),
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
                        tracing::error!(%e, "error in block submission. Dropping block.");
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
