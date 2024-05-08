use alloy_sol_types::sol;

sol!(
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq)]
    Zenith,
    "abi/zenith.json"
);

impl From<&Zenith::BlockSubmitted> for Zenith::BlockHeader {
    fn from(event: &Zenith::BlockSubmitted) -> Zenith::BlockHeader {
        Zenith::BlockHeader {
            rollupChainId: event.rollupChainId,
            sequence: event.sequence,
            confirmBy: event.confirmBy,
            gasLimit: event.gasLimit,
            rewardAddress: event.rewardAddress,
        }
    }
}
