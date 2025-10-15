use superstack_rust_sdk::SuperstackApiClient;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Read configuration from environment variables
    let api_key = std::env::var("API_KEY").unwrap();

    // Create a new superstack API client
    let client = SuperstackApiClient::new(&api_key);

    // Transfer 5 USDC from perps account to the spot account
    let to_perp = false;
    let response = client.usd_class_transfer("5", to_perp).await.unwrap();
    println!("Usd class transfer response: {:?}", response);
}
