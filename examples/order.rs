use superstack_rust_sdk::{BulkOrder, Limit, Order, OrderRequest, SuperstackApiClient};

#[tokio::main]
async fn main() {
    let api_key = std::env::var("API_KEY").unwrap();
    let client = SuperstackApiClient::new(&api_key);

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
    let response = client.order(order, None, None).await.unwrap();
    println!("Order response: {:?}", response);
}
