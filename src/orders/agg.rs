use crate::RollupOrders;
use alloy_primitives::{Address, U256};
use std::collections::HashMap;

/// Aggregated orders for a transaction or set of transactions.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct AggregateOrders {
    /// Outputs to be transferred to the user. These may be on the rollup or
    /// the host or potentially elsewhere in the future.
    pub outputs: HashMap<(u64, Address), HashMap<Address, U256>>,
    /// Inputs to be transferred to the filler. These are always on the
    /// rollup.
    pub inputs: HashMap<Address, U256>,
}

impl AggregateOrders {
    /// Instantiate a new [`AggregateOrders`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Instantiate a new [`AggregateOrders`] with a custom capacity. The
    /// capcity is for the number of assets in inputs or outputs.
    pub fn with_capacity(capacity: usize) -> Self {
        Self { outputs: HashMap::with_capacity(capacity), inputs: HashMap::with_capacity(capacity) }
    }

    /// Ingest an output into the aggregate orders.
    fn ingest_output(&mut self, output: &RollupOrders::Output) {
        let entry = self
            .outputs
            .entry((output.chain_id() as u64, output.token))
            .or_default()
            .entry(output.recipient)
            .or_default();
        *entry = entry.saturating_add(output.amount);
    }

    /// Ingest an input into the aggregate orders.
    fn ingest_input(&mut self, input: &RollupOrders::Input) {
        let entry = self.inputs.entry(input.token).or_default();
        *entry = entry.saturating_add(input.amount);
    }

    /// Ingest a new order into the aggregate orders.
    pub fn ingest(&mut self, order: &RollupOrders::Order) {
        order.outputs.iter().for_each(|o| self.ingest_output(o));
        order.inputs.iter().for_each(|i| self.ingest_input(i));
    }

    /// Extend the orders with a new set of orders.
    pub fn extend<'a>(&mut self, orders: impl IntoIterator<Item = &'a RollupOrders::Order>) {
        for order in orders {
            self.ingest(order);
        }
    }
}

impl<'a> FromIterator<&'a RollupOrders::Order> for AggregateOrders {
    fn from_iter<T: IntoIterator<Item = &'a RollupOrders::Order>>(iter: T) -> Self {
        let mut orders = AggregateOrders::new();
        orders.extend(iter);
        orders
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloy_primitives::{Address, U256};

    const ASSET_A: Address = Address::repeat_byte(1);
    const ASSET_B: Address = Address::repeat_byte(2);
    const ASSET_C: Address = Address::repeat_byte(3);

    const USER_A: Address = Address::repeat_byte(4);
    const USER_B: Address = Address::repeat_byte(5);
    const USER_C: Address = Address::repeat_byte(6);

    fn input(asset: Address, amount: u64) -> RollupOrders::Input {
        RollupOrders::Input { token: asset, amount: U256::from(amount) }
    }

    fn output(asset: Address, recipient: Address, amount: u64) -> RollupOrders::Output {
        RollupOrders::Output { chainId: 1, token: asset, recipient, amount: U256::from(amount) }
    }

    #[test]
    fn test_single_order() {
        let order = RollupOrders::Order {
            inputs: vec![input(ASSET_A, 100), input(ASSET_B, 200)],
            outputs: vec![
                output(ASSET_A, USER_A, 50),
                output(ASSET_A, USER_B, 50),
                output(ASSET_B, USER_B, 100),
                output(ASSET_C, USER_C, 200),
                output(ASSET_C, USER_C, 200),
            ],
            deadline: U256::ZERO,
        };

        let agg: AggregateOrders = [&order].into_iter().collect();
        assert_eq!(agg.inputs.get(&ASSET_A), Some(&U256::from(100)), "ASSET_A input");
        assert_eq!(agg.inputs.get(&ASSET_B), Some(&U256::from(200)), "ASSET_B input");

        assert_eq!(
            agg.outputs.get(&(1, ASSET_A)).map(|m| m.get(&USER_A)),
            Some(Some(&U256::from(50))),
            "ASSET_A USER_A output"
        );
        assert_eq!(
            agg.outputs.get(&(1, ASSET_A)).map(|m| m.get(&USER_B)),
            Some(Some(&U256::from(50))),
            "ASSET_A USER_B output"
        );
        assert_eq!(
            agg.outputs.get(&(1, ASSET_B)).map(|m| m.get(&USER_B)),
            Some(Some(&U256::from(100))),
            "ASSET_B USER_B output"
        );
        assert_eq!(
            agg.outputs.get(&(1, ASSET_C)).map(|m| m.get(&USER_C)),
            Some(Some(&U256::from(400))),
            "ASSET_C USER_C output"
        );
    }

    #[test]
    fn test_two_orders() {
        let order_1 = RollupOrders::Order {
            inputs: vec![input(ASSET_A, 100), input(ASSET_B, 200)],
            outputs: vec![
                output(ASSET_A, USER_A, 50),
                output(ASSET_A, USER_B, 50),
                output(ASSET_B, USER_B, 100),
                output(ASSET_C, USER_C, 200),
                output(ASSET_C, USER_C, 200),
            ],
            deadline: U256::ZERO,
        };
        let order_2 = RollupOrders::Order {
            inputs: vec![input(ASSET_A, 50), input(ASSET_C, 100)],
            outputs: vec![
                output(ASSET_A, USER_A, 50),
                output(ASSET_B, USER_B, 100),
                output(ASSET_C, USER_C, 100),
            ],
            deadline: U256::ZERO,
        };

        let agg: AggregateOrders = [&order_1, &order_2].into_iter().collect();

        assert_eq!(agg.inputs.get(&ASSET_A), Some(&U256::from(150)), "ASSET_A input");
        assert_eq!(agg.inputs.get(&ASSET_B), Some(&U256::from(200)), "ASSET_B input");
        assert_eq!(agg.inputs.get(&ASSET_C), Some(&U256::from(100)), "ASSET_C input");

        assert_eq!(
            agg.outputs.get(&(1, ASSET_A)).map(|m| m.get(&USER_A)),
            Some(Some(&U256::from(100))),
            "ASSET_A USER_A output"
        );
        assert_eq!(
            agg.outputs.get(&(1, ASSET_A)).map(|m| m.get(&USER_B)),
            Some(Some(&U256::from(50))),
            "ASSET_A USER_B output"
        );
        assert_eq!(
            agg.outputs.get(&(1, ASSET_B)).map(|m| m.get(&USER_B)),
            Some(Some(&U256::from(200))),
            "ASSET_B USER_B output"
        );
        assert_eq!(
            agg.outputs.get(&(1, ASSET_C)).map(|m| m.get(&USER_C)),
            Some(Some(&U256::from(500))),
            "ASSET_C USER_C output"
        );
    }
}
