use alloy_consensus::SimpleCoder;
use alloy_network::TransactionBuilder;
use alloy_primitives::{Address, FixedBytes, Signature, U256};
use alloy_provider::{Provider, WalletProvider};
use alloy_rpc_types::{BlockId, BlockNumberOrTag, TransactionRequest};
use alloy_signer::Signer;
use alloy_sol_types::SolCall;
use alloy_transport::BoxTransport;
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::instrument;
use zenith_types::SignRequest;

use crate::Zenith::{self, ZenithInstance};

use super::block::InProgressBlock;

/// Submits sidecars in ethereum txns to mainnet ethereum
pub struct SubmitTask<P> {
    /// Ethereum Provider
    pub provider: P,

    /// Zenity
    pub zenith: ZenithInstance<BoxTransport, P>,

    /// Reqwest
    pub client: reqwest::Client,

    /// Config
    pub config: crate::ChainConfig,

    /// builder address
    pub builder_rewards: Address,

    /// Gas limit for RU block
    pub gas_limit: u64,
}

impl<P> SubmitTask<P>
where
    P: Provider<BoxTransport> + WalletProvider,
{
    async fn get_confirm_by(&self) -> eyre::Result<u64> {
        self.provider
            .get_block(BlockId::Number(BlockNumberOrTag::Latest), false)
            .await
            .map_err(Into::into)
            .map(|block| {
                block.expect("latest is never none").header.timestamp
                    + self.config.confirmation_buffer
            })
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
    async fn sup_quincey(&self, sig_request: &SignRequest) -> eyre::Result<Signature> {
        tracing::info!(
            sequence = %sig_request.sequence,
            "pinging quincey for signature"
        );

        let resp: reqwest::Response = self
            .client
            .post(self.config.quincey_url.as_ref())
            .json(sig_request)
            .send()
            .await?
            .error_for_status()?;

        resp.json().await.map_err(Into::into)
    }

    #[instrument(skip_all, err)]
    async fn construct_sig_request(&self, contents: &InProgressBlock) -> eyre::Result<SignRequest> {
        let sequence = self.get_next_sequence().await?;
        let confirm_by = self.get_confirm_by().await?;

        Ok(SignRequest {
            host_chain_id: U256::from(self.config.host_chain_id),
            ru_chain_id: U256::from(self.config.ru_chain_id),
            sequence: U256::from(sequence),
            confirm_by: U256::from(confirm_by),
            gas_limit: U256::from(self.gas_limit),
            ru_reward_address: self.builder_rewards,
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
    ) -> TransactionRequest {
        let data = Zenith::submitBlockCall {
            header,
            blockDataHash: in_progress.contents_hash(),
            v,
            r,
            s,
            blockData: Default::default(),
        }
        .abi_encode();
        TransactionRequest::default()
            .with_blob_sidecar(in_progress.encode_blob::<SimpleCoder>().build().unwrap())
            .with_input(data)
    }

    fn build_calldata_tx(
        &self,
        header: Zenith::BlockHeader,
        v: u8,
        r: FixedBytes<32>,
        s: FixedBytes<32>,
        in_progress: &InProgressBlock,
    ) -> TransactionRequest {
        let data = Zenith::submitBlockCall {
            header,
            blockDataHash: in_progress.contents_hash(),
            v,
            r,
            s,
            blockData: in_progress.encode_calldata().clone(),
        }
        .abi_encode();
        TransactionRequest::default().with_input(data)
    }

    async fn submit_transaction(
        &self,
        sig_request: SignRequest,
        signature: &Signature,
        in_progress: &InProgressBlock,
    ) -> eyre::Result<()> {
        let v: u8 = signature.v().y_parity_byte() + 27;
        let r: FixedBytes<32> = signature.r().into();
        let s: FixedBytes<32> = signature.s().into();

        let header = Zenith::BlockHeader {
            rollupChainId: U256::from(self.config.ru_chain_id),
            sequence: sig_request.sequence,
            gasLimit: sig_request.gas_limit,
            confirmBy: sig_request.confirm_by,
            rewardAddress: sig_request.ru_reward_address,
        };

        let tx = if self.config.use_calldata {
            self.build_calldata_tx(header, v, r, s, in_progress)
        } else {
            self.build_blob_tx(header, v, r, s, in_progress)
        }
        .with_from(self.provider.default_signer_address())
        .with_to(self.config.zenith);

        tracing::debug!(
            sequence = %sig_request.sequence,
            gas_limit = %sig_request.gas_limit,
            "sending transaction to network"
        );

        let result = self.provider.send_transaction(tx).await?;

        let tx_hash = result.tx_hash();

        tracing::info!(
            %tx_hash,
            sequence = %sig_request.sequence,
            gas_limit = %sig_request.gas_limit,
            "dispatched to network"
        );

        Ok(())
    }

    #[instrument(skip_all, err)]
    async fn handle_inbound(&self, in_progress: &InProgressBlock) -> eyre::Result<()> {
        tracing::info!(txns = in_progress.len(), "handling inbound block");
        let sig_request = self.construct_sig_request(in_progress).await?;

        tracing::debug!(
            sequence = %sig_request.sequence,
            confirm_by = %sig_request.confirm_by,
            "constructed signature request"
        );

        // If configured with a local signer, we use it. Otherwise, we ask
        // quincey (politely)
        let signature = if let Some(signer) = &self.config.local_sequencer_signer {
            let sig = signer.sign_hash(&sig_request.signing_hash()).await?;
            tracing::debug!(
                sig = hex::encode(sig.as_bytes()),
                "acquied signature from local signer"
            );
            sig
        } else {
            let sig = self.sup_quincey(&sig_request).await?;
            tracing::debug!(
                sig = hex::encode(sig.as_bytes()),
                "acquied signature from quincey"
            );
            sig
        };

        self.submit_transaction(sig_request, &signature, in_progress)
            .await
    }
}

impl<P> SubmitTask<P>
where
    P: Provider<BoxTransport> + WalletProvider + 'static,
{
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
