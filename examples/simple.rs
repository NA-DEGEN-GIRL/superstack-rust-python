use anyhow::Result;

use superstack_rust_sdk::hype_responses::HypeExchangeDataStatus;
use superstack_rust_sdk::{
    BulkCancel, BulkOrder, CancelRequest, Limit, Order, OrderRequest, SuperstackApiClient,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    tracing_subscriber::fmt::init();

    // Read configuration from environment variables
    let base_url = std::env::var("API_BASE_URL").unwrap();
    let api_key = std::env::var("API_KEY").unwrap();
    let client = reqwest::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    // Create a new superstack API client
    let client = SuperstackApiClient::with_base_url_and_http_client(&api_key, &base_url, client);

    // Get API information
    println!("Fetching API information...");
    let api_info = client.get_api_info().await?;
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

    let order = BulkOrder {
        orders: vec![OrderRequest {
            asset: 0, // BTC
            is_buy: true,
            reduce_only: false,
            limit_px: "110000.0".to_string(),
            sz: "0.0001".to_string(),
            cloid: None,
            order_type: Order::Limit(Limit {
                tif: "Gtc".to_string(),
            }),
        }],
        grouping: "na".to_string(),
    };
    let response = client.order(order, None, None).await?;
    println!("Order response: {:?}", response);

    let status = response.data.unwrap().statuses[0].clone();
    let oid = match status {
        HypeExchangeDataStatus::Filled(order) => order.oid,
        HypeExchangeDataStatus::Resting(order) => order.oid,
        _ => panic!("Error: {status:?}"),
    };

    // So you can see the order before it's cancelled
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    let cancel = BulkCancel {
        cancels: vec![CancelRequest { asset: 0, oid }],
    };

    // This response will return an error if order was filled (since you can't cancel a filled order), otherwise it will cancel the order
    let response = client.cancel(cancel, None, None).await.unwrap();
    println!("Order potentially cancelled: {response:?}");

    let destination = "0x0000000000000000000000000000000000000000".to_string();
    let amount = "0.01".to_string();
    let response = client.usdc_transfer(&amount, &destination).await?;
    println!("USD send response: {:?}", response);

    Ok(())
}
