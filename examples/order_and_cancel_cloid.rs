use superstack_rust_sdk::{
    BulkCancelCloid, BulkOrder, CancelRequestCloid, Limit, Order, OrderRequest, SuperstackApiClient,
};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Read configuration from environment variables
    let api_key = std::env::var("API_KEY").unwrap();

    // Create a new superstack API client
    let client = SuperstackApiClient::new(&api_key);

    // Create a new order request and place it
    // cloid should be a 128 bit hex string like 0x1234567890abcdef1234567890abcdef
    let cloid = uuid_to_hex_string(Uuid::new_v4());
    let order = BulkOrder {
        orders: vec![OrderRequest {
            asset: 0, // BTC
            is_buy: true,
            reduce_only: false,
            limit_px: "100000".to_string(),
            sz: "0.0001".to_string(),
            cloid: Some(cloid.clone()),
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

    // Create a new cancel request and cancel the order
    let cancel = BulkCancelCloid {
        cancels: vec![CancelRequestCloid { asset: 0, cloid }],
    };
    // This response will return an error if the order was filled (since you can't cancel a filled order), otherwise it will cancel the order
    let response = client.cancel_by_cloid(cancel, None, None).await.unwrap();
    println!("Order cancelled: {:?}", response);
}

fn uuid_to_hex_string(uuid: Uuid) -> String {
    let hex_string = uuid
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<String>>()
        .join("");
    format!("0x{hex_string}")
}
