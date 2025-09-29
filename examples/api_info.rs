use superstack_rust_sdk::SuperstackApiClient;

#[tokio::main]
async fn main() {
    let api_key = std::env::var("API_KEY").unwrap();
    let client = SuperstackApiClient::new(&api_key);

    let api_info = client.get_api_info().await.unwrap();
    println!("API information: {:?}", api_info);
}
