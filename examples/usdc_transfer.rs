use superstack_rust_sdk::SuperstackApiClient;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Read configuration from environment variables
    let api_key = std::env::var("API_KEY").unwrap();

    // Create a new superstack API client
    let client = SuperstackApiClient::new(&api_key);

    let destination = "0x0000000000000000000000000000000000000000";
    // Send 0.01 USDC to the destination address
    let response = client.usdc_transfer("0.01", destination).await.unwrap();
    println!("Usdc transfer response: {:?}", response);
}
