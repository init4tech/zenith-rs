use crate::{
    config::{Provider, ZenithInstance},
    signer::LocalOrAws,
    tasks::block::InProgressBlock,
};
use alloy::consensus::{constants::GWEI_TO_WEI, SimpleCoder};
use alloy::eips::BlockNumberOrTag;
use alloy::network::{TransactionBuilder, TransactionBuilder4844};
use alloy::providers::{Provider as _, WalletProvider};
use alloy::rpc::types::eth::TransactionRequest;
use alloy::signers::Signer;
use alloy::sol_types::SolCall;
use alloy::transports::TransportError;
use alloy_primitives::{FixedBytes, U256};
use eyre::{bail, eyre};
use oauth2::{
    basic::BasicClient, basic::BasicTokenType, reqwest::http_client, AuthUrl, ClientId,
    ClientSecret, EmptyExtraTokenFields, StandardTokenResponse, TokenResponse, TokenUrl,
};
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::{debug, error, instrument, trace};
use zenith_types::{SignRequest, SignResponse, Zenith};

/// OAuth Audience Claim Name, required param by IdP for client credential grant
const OAUTH_AUDIENCE_CLAIM: &str = "audience";

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
            host_block_number = %sig_request.host_block_number,
            ru_chain_id = %sig_request.ru_chain_id,
            "pinging quincey for signature"
        );

        let token = self.fetch_oauth_token().await?;

        let resp: reqwest::Response = self
            .client
            .post(self.config.quincey_url.as_ref())
            .json(sig_request)
            .bearer_auth(token.access_token().secret())
            .send()
            .await?
            .error_for_status()?;

        let body = resp.bytes().await?;

        debug!(bytes = body.len(), "retrieved response body");
        trace!(body = %String::from_utf8_lossy(&body), "response body");

        serde_json::from_slice(&body).map_err(Into::into)
    }

    async fn fetch_oauth_token(
        &self,
    ) -> eyre::Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
        let client = BasicClient::new(
            ClientId::new(self.config.oauth_client_id.clone()),
            Some(ClientSecret::new(self.config.oauth_client_secret.clone())),
            AuthUrl::new(self.config.oauth_authenticate_url.clone())?,
            Some(TokenUrl::new(self.config.oauth_token_url.clone())?),
        );

        let token_result = client
            .exchange_client_credentials()
            .add_extra_param(OAUTH_AUDIENCE_CLAIM, self.config.oauth_audience.clone())
            .request(http_client)?;

        Ok(token_result)
    }

    #[instrument(skip_all)]
    async fn construct_sig_request(&self, contents: &InProgressBlock) -> eyre::Result<SignRequest> {
        let ru_chain_id = U256::from(self.config.ru_chain_id);
        let next_block_height = self.next_host_block_height().await?;

        Ok(SignRequest {
            host_block_number: U256::from(next_block_height),
            host_chain_id: U256::from(self.config.host_chain_id),
            ru_chain_id,
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
        Ok(TransactionRequest::default()
            .with_blob_sidecar(sidecar)
            .with_input(data)
            .with_max_priority_fee_per_gas((GWEI_TO_WEI * 16) as u128))
    }

    async fn next_host_block_height(&self) -> eyre::Result<u64> {
        let result = self.provider.get_block_number().await?;
        let next = result.checked_add(1).ok_or_else(|| eyre!("next host block height overflow"))?;
        Ok(next)
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
            hostBlockNumber: resp.req.host_block_number,
            rollupChainId: U256::from(self.config.ru_chain_id),
            gasLimit: resp.req.gas_limit,
            rewardAddress: resp.req.ru_reward_address,
            blockDataHash: in_progress.contents_hash(),
        };

        let tx = self
            .build_blob_tx(header, v, r, s, in_progress)?
            .with_from(self.provider.default_signer_address())
            .with_to(self.config.zenith_address)
            .with_gas_limit(1_000_000);

        if let Err(TransportError::ErrorResp(e)) =
            self.provider.call(&tx).block(BlockNumberOrTag::Pending.into()).await
        {
            error!(
                code = e.code,
                message = %e.message,
                data = ?e.data,
                "error in transaction submission"
            );

            bail!("simulation failed, bailing transaction submission")
        }

        tracing::debug!(
            host_block_number = %resp.req.host_block_number,
            gas_limit = %resp.req.gas_limit,
            "sending transaction to network"
        );

        let result = self.provider.send_transaction(tx).await?;

        let tx_hash = result.tx_hash();

        tracing::info!(
            %tx_hash,
            ru_chain_id = %resp.req.ru_chain_id,
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
            host_block_number = %sig_request.host_block_number,
            ru_chain_id = %sig_request.ru_chain_id,
            "constructed signature request for host block"
        );

        // If configured with a local signer, we use it. Otherwise, we ask
        // quincey (politely)
        let signed = if let Some(signer) = &self.sequencer_signer {
            let sig = signer.sign_hash(&sig_request.signing_hash()).await?;
            tracing::debug!(
                sig = hex::encode(sig.as_bytes()),
                "acquired signature from local signer"
            );
            SignResponse { req: sig_request, sig }
        } else {
            let resp: SignResponse = self.sup_quincey(&sig_request).await?;
            tracing::debug!(
                sig = hex::encode(resp.sig.as_bytes()),
                "acquired signature from quincey"
            );
            resp
        };

        self.submit_transaction(&signed, in_progress).await
    }

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
