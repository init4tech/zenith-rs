mod tests {
    use std::str::FromStr;

    use alloy::consensus::{SignableTransaction, TxEip1559, TxEnvelope};
    use alloy::signers::{local::PrivateKeySigner, SignerSync};
    use alloy_primitives::{bytes, Address, TxKind, U256};
    use builder::config::BuilderConfig;
    use builder::tasks::{block::BlockBuilder, tx_poller};
    use eyre::Result;

    #[ignore = "integration test"]
    #[tokio::test]
    async fn test_tx_roundtrip() -> Result<()> {
        // Create a new test environment
        let (_, config) = setup_test_builder().await?;

        // Post a transaction to the cache
        post_tx(&config).await?;

        // Create a new poller
        let mut poller = tx_poller::TxPoller::new(&config);

        // Fetch transactions the pool
        let transactions = poller.check_tx_pool().await?;

        // Ensure at least one transaction exists
        assert!(!transactions.is_empty());

        Ok(())
    }

    async fn post_tx(config: &BuilderConfig) -> Result<()> {
        let client = reqwest::Client::new();
        let wallet = PrivateKeySigner::random();
        let tx_envelope = new_test_tx(&wallet)?;

        let url = format!("{}/transactions", config.tx_pool_url);
        let response = client.post(&url).json(&tx_envelope).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            eyre::bail!("Failed to post transaction: {}", error_text);
        }

        Ok(())
    }

    // Returns a new signed test transaction with default values
    fn new_test_tx(wallet: &PrivateKeySigner) -> Result<TxEnvelope> {
        let tx = TxEip1559 {
            chain_id: 17001,
            nonce: 1,
            gas_limit: 50000,
            to: TxKind::Call(
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap(),
            ),
            value: U256::from(1_f64),
            input: bytes!(""),
            ..Default::default()
        };
        let signature = wallet.sign_hash_sync(&tx.signature_hash())?;
        Ok(TxEnvelope::Eip1559(tx.into_signed(signature)))
    }

    // Sets up a block builder with test values
    async fn setup_test_builder() -> Result<(BlockBuilder, BuilderConfig)> {
        let config = BuilderConfig {
            host_chain_id: 17000,
            ru_chain_id: 17001,
            host_rpc_url: "http://rpc.holesky.signet.sh".into(),
            zenith_address: Address::default(),
            quincey_url: "http://localhost:8080".into(),
            builder_port: 8080,
            sequencer_key: None,
            builder_key: "0000000000000000000000000000000000000000000000000000000000000000".into(),
            incoming_transactions_buffer: 1,
            block_confirmation_buffer: 1,
            builder_rewards_address: Address::default(),
            rollup_block_gas_limit: 100_000,
            tx_pool_url: "http://localhost:9000/".into(),
            tx_pool_cache_duration: 5,
            tx_pool_poll_interval: 5,
            oauth_client_id: "some_client_id".into(),
            oauth_client_secret: "some_client_secret".into(),
            oauth_authenticate_url: "http://localhost:8080".into(),
            oauth_token_url: "http://localhost:8080".into(),
            oauth_audience: "https://transactions.holesky.signet.sh".into(),
        };
        Ok((BlockBuilder::new(&config), config))
    }
}
