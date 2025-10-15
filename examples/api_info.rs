use superstack_rust_sdk::SuperstackApiClient;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Read configuration from environment variables
    let api_key = std::env::var("API_KEY").unwrap();

    // Create a new superstack API client
    let client = SuperstackApiClient::new(&api_key);

    // Get API information
    let api_info = client.get_api_info().await.unwrap();

    println!(
        "API information:\n{}",
        serde_json::to_string_pretty(&api_info).unwrap()
    );

    let evm_main_wallet = &api_info.user_info.get_evm_main_wallet().unwrap().address;
    println!("EVM Main Wallet: {}", evm_main_wallet);

    println!("API Key ID: {:?}", api_info.api_info.id);
    println!(
        "API Key created at: {}",
        chrono::DateTime::from_timestamp(api_info.api_info.created_at, 0)
            .unwrap()
            .format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("API Key IP whitelist: {:?}", api_info.api_info.ip_whitelist);
    println!("API Key permissions: {:?}", api_info.api_info.permissions);
}
