use crate::bindings::HostOrders::{Output, Permit2Batch};
use serde::{Deserialize, Serialize};

/// A signed order.
/// TODO: Link to docs.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SignedOrder {
    /// The permit batch.
    #[serde(flatten)]
    pub permit: Permit2Batch,
    /// The desired outputs.
    pub outputs: Vec<Output>,
}

impl SignedOrder {
    /// Creates a new signed order.
    pub fn new(permit: Permit2Batch, outputs: Vec<Output>) -> Self {
        Self { permit, outputs }
    }
}
