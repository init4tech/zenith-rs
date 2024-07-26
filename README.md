# zenith-rs

Rust utilites for working with [Zenith](https://github.com/init4tech/zenith).

![rust](https://github.com/init4tech/zenith-rs/actions/workflows/rust-ci.yml/badge.svg) ![ecr](https://github.com/init4tech/zenith-rs/actions/workflows/ecr-cd.yml/badge.svg)

## Crates

- `zenith-types`: Common types used by other crates.
- `zenith-builder-example`: a block building example that uses the `builder` library.

## Development

This crate contains an exmaple block builder in the Signet ecosystem.

### Requirements

- Rust 1.79.0
- Cargo [Lambda](https://www.cargo-lambda.info/)
- AWS CLI and credentials

### Environment

The following environment variables are exposed to configure the Builder:

```bash
# Builder Configs
HOST_CHAIN_ID="17000" # Holesky Testnet
RU_CHAIN_ID="17001"
HOST_RPC_URL="http://host.url.here"
ZENITH_ADDRESS="ZENITH_ADDRESS_HERE"
QUINCEY_URL="http://signer.url.here"
BUILDER_PORT="8080"
BUILDER_KEY="YOUR_BUILDER_KEY_HERE"
INCOMING_TRANSACTIONS_BUFFER="10"
BLOCK_CONFIRMATION_BUFFER="10"
BUILDER_REWARDS_ADDRESS="BUILDER_REWARDS_ADDRESS_HERE"
ROLLUP_BLOCK_GAS_LIMIT="30000000"
# Transaction Pool Configs
TX_POOL_URL="http://pool.url.here/" # trailing slash is required
TX_POOL_POLL_INTERVAL="5" # seconds
TX_POOL_CACHE_DURATION="600" # seconds
```
