# zenith-rs

Rust utilites for working with [Zenith](https://github.com/init4tech/zenith).

![rust](https://github.com/init4tech/zenith-rs/actions/workflows/rust-ci.yml/badge.svg) ![ecr](https://github.com/init4tech/zenith-rs/actions/workflows/ecr-cd.yml/badge.svg)

## Crates

- `zenith-types`: Common types used by other crates.
- `zenith-builder-example`: Example Zenith block builder.

## Development

This crate contains an exmaple block builder in the Signet ecosystem.

### Requirements

- Rust 1.79.0
- Cargo [Lambda](https://www.cargo-lambda.info/)
- AWS CLI and credentials

### Environment

The following environment variables are required to run the Builder locally for development:

```bash
HOST_CHAIN_ID="17000" # Holesky Testnet
RU_CHAIN_ID="17001"
HOST_RPC_URL="http://your.url.here"
ZENITH_ADDRESS="ZENITH_ADDRESS_HERE"
QUINCEY_URL="http://your.url.here"
BUILDER_PORT="8080"
BUILDER_KEY="YOUR_BUILDER_KEY_HERE"
INCOMING_TRANSACTIONS_BUFFER="10"
BLOCK_CONFIRMATION_BUFFER="10"
BUILDER_REWARDS_ADDRESS="BUILDER_REWARDS_ADDRESS_HERE"
ROLLUP_BLOCK_GAS_LIMIT="30000000"
TX_POOL_URL="http://your.url.here"
```
