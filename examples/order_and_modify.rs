use superstack_rust_sdk::{
    hype_responses::HypeExchangeDataStatus, BulkModify, BulkOrder, Limit, ModifyRequest, Order,
    OrderRequest, SuperstackApiClient,
};

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Read configuration from environment variables
    let api_key = std::env::var("API_KEY").unwrap();

    // Create a new superstack API client
    let client = SuperstackApiClient::new(&api_key);

    // Create a new order request
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

    // Place the order
    let response = client.order(order, None, None).await.unwrap();
    println!("Order response: {:?}", response);

    let status = response.data.unwrap().statuses[0].clone();
    let oid = match status {
        HypeExchangeDataStatus::Resting(order) => order.oid,
        _ => panic!("Error: {status:?}"),
    };

    // So you can see the order before it's modified
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // Modify the order
    let modify = BulkModify {
        modifies: vec![ModifyRequest {
            oid,
            order: OrderRequest {
                asset: 0, // BTC
                is_buy: true,
                reduce_only: false,
                limit_px: "90000".to_string(),
                sz: "0.0001".to_string(),
                cloid: None,
                order_type: Order::Limit(Limit {
                    tif: "Gtc".to_string(),
                }),
            },
        }],
    };
    let response = client.modify(modify, None, None).await.unwrap();
    println!("Modify response: {:?}", response);
}
