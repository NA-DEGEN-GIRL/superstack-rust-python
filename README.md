# Superstack Rust SDK

A comprehensive Rust SDK for interacting with the Superstack API.

With Superstack API, you can retrieve account balances, manage funds, and execute trades across supported markets.

This SDK is written in Rust and provides a complete interface for trading operations, transfer operations, and account information.

## Features

- **API Information**: Retrieve user and API key information
- **Trading Operations**: Place, modify, cancel orders, leverage management, margin management
- **Asset Transfers**: USDC transfers, spot token transfers, and USD class transfers across spot and perps accounts

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
superstack-rust-sdk = { git = "https://github.com/k4-trading/superstack-rust-sdk.git" }
```

## Quick Start

### 1. Set up your API key

Create your API key on https://superstack.xyz/api with appropriate permissions.

### 2. Basic usage

```rust
use superstack_rust_sdk::SuperstackApiClient;

#[tokio::main]
async fn main() {
    let api_key = "your_api_key_here";
    
    let client = SuperstackApiClient::new(&api_key);
    
    // Get API information
    let api_info = client.get_api_info().await.unwrap();
    println!("API Info: {:?}", api_info);
}
```

## API Reference

### Client Initialization

```rust
// Basic initialization
let client = SuperstackApiClient::new(&api_key);
```

### Trading Operations

#### Place Orders

```rust
use superstack_rust_sdk::{BulkOrder, Limit, Order, OrderRequest, SuperstackApiClient};

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
```

#### Cancel Orders

```rust
use superstack_rust_sdk::{BulkCancel, CancelRequest};

// Cancel by order ID
let asset = 0; // BTC order
let oid = 12345; // your order ID
let cancel_request = BulkCancel {
    cancels: vec![CancelRequest { asset, oid }],
};
let response = client.cancel(cancel_request, None, None).await.unwrap();

// Cancel by client order ID
let asset = 0; // BTC order
let cloid = "your_cloid_here"; // your client order ID as 128 bit hex string, e.g. 0x1234567890abcdef1234567890abcdef
use superstack_rust_sdk::{BulkCancelCloid, CancelRequestCloid};
let cancel_request = BulkCancelCloid {
    cancels: vec![CancelRequestCloid { asset, cloid }],
};
let response = client.cancel_by_cloid(cancel_request, None, None).await.unwrap();
```

#### Modify Orders

```rust
use superstack_rust_sdk::{BulkModify, ModifyRequest};

let modify = BulkModify {
    modifies: vec![ModifyRequest {
        oid: 12345, // your order ID
        order: OrderRequest {
            asset: 0,
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
```

#### Schedule Cancel

Note that, only hyperliquid accounts with at least $1,000,000 volume can schedule a cancel operation

```rust
// Schedule a cancel operation 15 seconds in the future
// Use chrono to get UTC timestamp
let current_time = chrono::Utc::now().timestamp_millis() as u64;
let cancel_time = current_time + 15000; // 15 seconds from now
let response = client.schedule_cancel(Some(cancel_time), None, None).await.unwrap();
```

### Leverage Management

```rust
// Update BTC cross leverage to 5x
let response = client.update_leverage(0, true, 5, None, None).await.unwrap();

// Switch BTC to isolated margin from cross margin
let response = client.update_leverage(0, false, 5, None, None).await.unwrap();
```

### Margin Management

```rust
// Add 10.0 USDC to BTC isolated margin
let response = client.update_isolated_margin(0, 10.0, None, None).await.unwrap();
```

### Asset Transfers

#### USDC Transfers

```rust
let destination = "0x0000000000000000000000000000000000000000";
let response = client.usdc_transfer("0.01", destination).await.unwrap();
```

#### Spot Token Transfers

```rust
let destination = "0x0000000000000000000000000000000000000000";
let token = "HYPE:0x0d01dc56dcaaca66ad901c959b4011ec";
let response = client.spot_transfer("0.0001", destination, token).await.unwrap();
```

#### USD Class Transfers

```rust
// Transfer 5 USDC from perps account to spot account
let response = client.usd_class_transfer("5", false).await.unwrap();

// Transfer from spot to perps account
let response = client.usd_class_transfer("5", true).await.unwrap();
```

### API Information

```rust
let api_info = client.get_api_info().await.unwrap();

// Get EVM main wallet address
let evm_wallet = api_info.user_info.get_evm_main_wallet().unwrap().address;
println!("EVM Main Wallet: {}", evm_wallet);

// Get Solana main wallet address
let solana_wallet = api_info.user_info.get_solana_main_wallet().unwrap().address;
println!("Solana Main Wallet: {}", solana_wallet);

// API key information
println!("API Key ID: {:?}", api_info.api_info.id);
println!("Created at: {}", api_info.api_info.created_at);
println!("IP Whitelist: {:?}", api_info.api_info.ip_whitelist);
println!("Permissions: {:?}", api_info.api_info.permissions);
```

## Examples

The SDK includes comprehensive examples for all major operations:

- `api_info.rs` - Get API and user information
- `order.rs` - Place orders
- `order_and_cancel.rs` - Place and cancel orders
- `order_and_cancel_cloid.rs` - Cancel orders by client order ID
- `order_and_schedule_cancel.rs` - Schedule order cancellations
- `order_and_modify.rs` - Place and modify orders
- `leverage.rs` - Update leverage settings
- `margin.rs` - Manage isolated margin
- `usdc_transfer.rs` - USDC transfers
- `spot_transfer.rs` - Spot token transfers
- `usd_class_transfer.rs` - USD class transfers across spot and perps accounts

Run examples with:

```bash
cargo run --example api_info
cargo run --example order
cargo run --example leverage
cargo run --example margin
cargo run --example usdc_transfer
cargo run --example spot_transfer
cargo run --example usd_class_transfer
cargo run --example order_and_cancel
cargo run --example order_and_modify
cargo run --example order_and_schedule_cancel
cargo run --example order_and_cancel_cloid
```

## License

This project is licensed under the terms specified in the LICENSE file.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Support

For support and questions, please refer to the Superstack documentation or create an issue in this repository.