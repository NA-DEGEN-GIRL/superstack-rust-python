use superstack_rust_sdk::SuperstackApiClient;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Read configuration from environment variables
    let api_key = std::env::var("API_KEY").unwrap();

    // Create a new superstack API client
    let client = SuperstackApiClient::new(&api_key);

    // If you have a isolated position on ETH, you can add or remove margin to it

    // Add 10.0 USDC to the ETH isolated margin
    // It will fail if you don't have a isolated position on ETH
    let response = client
        .update_isolated_margin(1, 10.0, None, None)
        .await
        .unwrap();
    println!("Update isolated margin response: {:?}", response);
}
