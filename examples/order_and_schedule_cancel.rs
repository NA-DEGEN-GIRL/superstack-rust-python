use superstack_rust_sdk::{BulkOrder, Limit, Order, OrderRequest, SuperstackApiClient};

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Read configuration from environment variables
    let api_key = std::env::var("API_KEY").unwrap();

    // Create a new superstack API client
    let client = SuperstackApiClient::new(&api_key);

    // Create a new order request and place it
    let order = BulkOrder {
        orders: vec![OrderRequest {
            asset: 0, // BTC
            is_buy: true,
            reduce_only: false,
            limit_px: "100000".to_string(),
            sz: "0.0001".to_string(),
            cloid: None,
            order_type: Order::Limit(Limit {
                tif: "Gtc".to_string(),
            }),
        }],
        grouping: "na".to_string(),
    };
    let response = client.order(order, None, None).await.unwrap();
    println!("Order response: {:?}", response);

    // So you can see the order before it's cancelled
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // Schedule a cancel operation 15 seconds in the future
    // Use chrono to for UTC timestamp
    let current_time = chrono::Utc::now().timestamp_millis() as u64;
    let cancel_time = current_time + 15000; // 15 seconds from now

    // Note that, only hyperliquid accounts with at least $1,000,000 volume can schedule a cancel operation
    let response = client
        .schedule_cancel(Some(cancel_time), None, None)
        .await
        .unwrap();
    println!("schedule_cancel response: {:?}", response);
    tokio::time::sleep(std::time::Duration::from_secs(20)).await;
}
