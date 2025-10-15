use superstack_rust_sdk::SuperstackApiClient;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Read configuration from environment variables
    let api_key = std::env::var("API_KEY").unwrap();

    // Create a new superstack API client
    let client = SuperstackApiClient::new(&api_key);

    // Update the BTC cross leverage to 5x
    let response = client
        .update_leverage(0, true, 5, None, None)
        .await
        .unwrap();
    println!("Update leverage response: {:?}", response);

    // Update the ETH cross leverage to 20x
    let response = client
        .update_leverage(1, true, 20, None, None)
        .await
        .unwrap();
    println!("Update leverage response: {:?}", response);

    // Switch the ETH cross margin to isolated margin
    let response = client
        .update_leverage(1, false, 20, None, None)
        .await
        .unwrap();
    println!("Update leverage response: {:?}", response);
}
