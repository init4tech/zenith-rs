mod tests {
    use std::str::FromStr;

    use alloy::consensus::{SignableTransaction, TxEip1559, TxEnvelope};
    use alloy::signers::{local::PrivateKeySigner, SignerSync};
    use alloy_primitives::{bytes, Address, TxKind, U256};

    use builder::config::BuilderConfig;
    use builder::tasks::{block::BlockBuilder, tx_poller};

    #[ignore = "integration test"]
    #[tokio::test]
    async fn test_tx_roundtrip() {
        // create a new test environment
        let client = reqwest::Client::new();
        let (_, config) = setup_test_builder().await;

        // create a new signed test transaction
        let wallet = PrivateKeySigner::random();
        let tx_envelope = new_test_tx(&wallet);

        // send that transaction to ensure there is at least one tx in pool to parse
        let _ = client
            .post(config.tx_pool_url.to_string() + "/add")
            .json(&tx_envelope)
            .send()
            .await
            .unwrap();

        // assert that we parsed at least one transaction
        let got = tx_poller::TxPoller::new(&config).check_tx_pool().await.unwrap();
        assert!(!got.is_empty());
    }

    // returns a new signed test transaction with blank values
    fn new_test_tx(wallet: &PrivateKeySigner) -> TxEnvelope {
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
        let tx_hash = wallet.sign_hash_sync(&tx.signature_hash()).unwrap();
        TxEnvelope::Eip1559(tx.into_signed(tx_hash))
    }

    // sets up a block builder with test values
    async fn setup_test_builder() -> (BlockBuilder, BuilderConfig) {
        let config = BuilderConfig {
            host_chain_id: 17000,
            ru_chain_id: 17001,
            host_rpc_url: "http://rpc.api.signet.sh".into(),
            zenith_address: Address::from_str("0x0000000000000000000000000000000000000000")
                .unwrap(),
            quincey_url: "http://localhost:8080".into(),
            builder_port: 8080,
            sequencer_key: None,
            builder_key: "0000000000000000000000000000000000000000000000000000000000000000".into(),
            incoming_transactions_buffer: 1,
            block_confirmation_buffer: 1,
            builder_rewards_address: Address::from_str(
                "0x0000000000000000000000000000000000000000",
            )
            .unwrap(),
            rollup_block_gas_limit: 100_000,
            tx_pool_url: "http://localhost:9000".into(),
            tx_pool_cache_duration: 5,
            tx_pool_poll_interval: 5,
        };
        (BlockBuilder::new(&config), config)
    }
}
