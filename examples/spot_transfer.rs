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
    let token = "HYPE:0x0d01dc56dcaaca66ad901c959b4011ec";
    // Send 0.0001 HYPE to the destination address
    let response = client
        .spot_transfer("0.0001", destination, token)
        .await
        .unwrap();
    println!("Spot transfer response: {:?}", response);
}
