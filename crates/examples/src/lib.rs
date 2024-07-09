use eyre;
use reqwest::Client;
use reth_primitives::Transaction;

const ROOT_TEST_URL: &str = "http://localhost:9000";

pub async fn run() -> eyre::Result<()> {
    let client = Client::new();

    // CREATE A TRANSACTION
    //
    // Make a transaction to add to the cache
    let tx = Transaction::default();

    // Build a post request with the transaction
    let req = client.post(ROOT_TEST_URL.to_owned() + "/add").json(&tx);
    let result = req.send().await.unwrap();
    debug_assert!(result.status().is_success());

    // GET ALL TRANSACTIONS
    //
    // Make a request to the cache
    // NB: This is intentionally a post request to the `/get` endpoint
    let req = client.post(ROOT_TEST_URL.to_owned() + "/get");
    let result = req.send().await.unwrap();
    debug_assert!(result.status().is_success());

    let raw = result.text().await.unwrap();
    assert!(raw.len() > 0);

    // TODO parse raw into a list of Items
    // This is blocked by the other PR

    Ok(())
}
