use alloy::primitives::Address;
use anyhow::Result;
use reqwest::Client;

use crate::actions::*;
use crate::hype_responses::{HypeExchangeResponse, HypeExchangeResponseStatus};
use crate::types::{
    ApiInfoResponse, ErrorResponse, ExchangePayload, ExchangeRequest, ExchangeResponse,
};

pub struct SuperstackApiClient {
    http_client: Client,
    base_url: String,
    api_key: String,
}

impl SuperstackApiClient {
    const BASE_URL: &str = "https://wallet-service.superstack.xyz";

    pub fn new(api_key: &str) -> Self {
        Self {
            http_client: Client::new(),
            base_url: Self::BASE_URL.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub fn with_base_url(api_key: &str, base_url: &str) -> Self {
        Self {
            http_client: Client::new(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub fn with_base_url_and_http_client(
        api_key: &str,
        base_url: &str,
        http_client: Client,
    ) -> Self {
        Self {
            http_client,
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub async fn get_api_info(&self) -> Result<ApiInfoResponse> {
        let response = self
            .http_client
            .get(&format!("{}/api/info", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            match response.text().await {
                Ok(text) => {
                    let error_response: ErrorResponse = match serde_json::from_str(&text) {
                        Ok(error_response) => error_response,
                        Err(_) => {
                            return Err(anyhow::anyhow!(
                                "Request failed with status {}, failed to parse error response: {}",
                                status,
                                text
                            ));
                        }
                    };
                    return Err(anyhow::anyhow!(
                        "Request failed with status {}, error: {}",
                        status,
                        error_response.error
                    ));
                }
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "Request failed with status {}, unknown error",
                        status
                    ));
                }
            }
        }

        let api_info: ApiInfoResponse = response.json().await?;
        Ok(api_info)
    }

    async fn post_hyperliquid_exchange(
        &self,
        exchange_payload: ExchangePayload,
    ) -> Result<HypeExchangeResponse> {
        let res = serde_json::to_string(&exchange_payload)?;

        let response = self
            .http_client
            .post("https://api.hyperliquid.xyz/exchange")
            .header("Content-Type", "application/json")
            .body(res)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            return Err(anyhow::anyhow!(
                "Request to hyperliquid exchange endpoint failed with status {}, error: {}",
                status,
                text
            ));
        }

        let response_status = response.json().await?;

        match response_status {
            HypeExchangeResponseStatus::Ok(response) => Ok(response),
            HypeExchangeResponseStatus::Err(error) => Err(anyhow::anyhow!(error)),
        }
    }

    async fn post_wallet_api_exchange(
        &self,
        action: Actions,
        vault_address: Option<Address>,
        expires_after: Option<u64>,
    ) -> Result<ExchangeResponse> {
        let full_url = format!("{}/api/exchange", self.base_url);

        let exchange_request = ExchangeRequest {
            action,
            vault_address,
            expires_after,
        };

        let response = self
            .http_client
            .post(&full_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&exchange_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            match response.text().await {
                Ok(text) => {
                    let error_response: ErrorResponse = match serde_json::from_str(&text) {
                        Ok(error_response) => error_response,
                        Err(_) => {
                            return Err(anyhow::anyhow!(
                                "Request failed with status {}, failed to parse error response: {}",
                                status,
                                text
                            ));
                        }
                    };
                    return Err(anyhow::anyhow!(
                        "Request failed with status {}, error: {}",
                        status,
                        error_response.error
                    ));
                }
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "Request failed with status {}, unknown error",
                        status
                    ));
                }
            }
        }

        let response_status = response.json().await?;
        Ok(response_status)
    }

    async fn post_exchange(
        &self,
        action: Actions,
        vault_address: Option<Address>,
        expires_after: Option<u64>,
    ) -> Result<HypeExchangeResponse> {
        let exchange_response = self
            .post_wallet_api_exchange(action, vault_address, expires_after)
            .await?;
        tracing::debug!(
            "exchange_response: {:?}",
            serde_json::to_string(&exchange_response)?
        );
        self.post_hyperliquid_exchange(exchange_response.payload)
            .await
    }

    pub async fn order(
        &self,
        mut bulk_order: BulkOrder,
        vault_address: Option<Address>,
        expires_after: Option<u64>,
    ) -> Result<HypeExchangeResponse> {
        for order in &mut bulk_order.orders {
            order.limit_px = float_to_string_for_hashing(order.limit_px.parse::<f64>()?);
            order.sz = float_to_string_for_hashing(order.sz.parse::<f64>()?);
            if let Order::Trigger(trigger) = &mut order.order_type {
                trigger.trigger_px =
                    float_to_string_for_hashing(trigger.trigger_px.parse::<f64>()?);
            }
        }

        let action = Actions::Order(bulk_order);
        self.post_exchange(action, vault_address, expires_after)
            .await
    }

    pub async fn cancel(
        &self,
        cancels: BulkCancel,
        vault_address: Option<Address>,
        expires_after: Option<u64>,
    ) -> Result<HypeExchangeResponse> {
        let action = Actions::Cancel(cancels);
        self.post_exchange(action, vault_address, expires_after)
            .await
    }

    pub async fn cancel_by_cloid(
        &self,
        cancels: BulkCancelCloid,
        vault_address: Option<Address>,
        expires_after: Option<u64>,
    ) -> Result<HypeExchangeResponse> {
        let action = Actions::CancelByCloid(cancels);
        self.post_exchange(action, vault_address, expires_after)
            .await
    }

    pub async fn schedule_cancel(
        &self,
        time: Option<u64>,
        vault_address: Option<Address>,
        expires_after: Option<u64>,
    ) -> Result<HypeExchangeResponse> {
        let action = Actions::ScheduleCancel(ScheduleCancel { time });
        self.post_exchange(action, vault_address, expires_after)
            .await
    }

    pub async fn modify(
        &self,
        mut modifies: BulkModify,
        vault_address: Option<Address>,
        expires_after: Option<u64>,
    ) -> Result<HypeExchangeResponse> {
        for modify in &mut modifies.modifies {
            modify.order.limit_px =
                float_to_string_for_hashing(modify.order.limit_px.parse::<f64>()?);
            modify.order.sz = float_to_string_for_hashing(modify.order.sz.parse::<f64>()?);
            if let Order::Trigger(trigger) = &mut modify.order.order_type {
                trigger.trigger_px =
                    float_to_string_for_hashing(trigger.trigger_px.parse::<f64>()?);
            }
        }
        let action = Actions::BatchModify(modifies);
        self.post_exchange(action, vault_address, expires_after)
            .await
    }

    pub async fn update_leverage(
        &self,
        asset: u32,
        is_cross: bool,
        leverage: u32,
        vault_address: Option<Address>,
        expires_after: Option<u64>,
    ) -> Result<HypeExchangeResponse> {
        let action = Actions::UpdateLeverage(UpdateLeverage {
            asset,
            is_cross,
            leverage,
        });

        self.post_exchange(action, vault_address, expires_after)
            .await
    }

    pub async fn update_isolated_margin(
        &self,
        asset: u32,
        amount: f64,
        vault_address: Option<Address>,
        expires_after: Option<u64>,
    ) -> Result<HypeExchangeResponse> {
        let amount = (amount * 1_000_000.0).round() as i64;
        let action = Actions::UpdateIsolatedMargin(UpdateIsolatedMargin {
            asset,
            is_buy: true,
            ntli: amount,
        });

        self.post_exchange(action, vault_address, expires_after)
            .await
    }

    pub async fn usdc_transfer(
        &self,
        amount: &str,
        destination: &str,
    ) -> Result<HypeExchangeResponse> {
        let action = Actions::UsdSend(UsdSend {
            destination: destination.to_string(),
            amount: amount.to_string(),
        });

        self.post_exchange(action, None, None).await
    }

    pub async fn spot_transfer(
        &self,
        amount: &str,
        destination: &str,
        token: &str,
    ) -> Result<HypeExchangeResponse> {
        let spot_send = SpotSend {
            destination: destination.to_string(),
            amount: amount.to_string(),
            token: token.to_string(),
        };
        let action = Actions::SpotSend(spot_send);
        self.post_exchange(action, None, None).await
    }

    pub async fn usd_class_transfer(
        &self,
        amount: &str,
        to_perp: bool,
    ) -> Result<HypeExchangeResponse> {
        let action = Actions::UsdClassTransfer(UsdClassTransfer {
            amount: amount.to_string(),
            to_perp,
        });

        self.post_exchange(action, None, None).await
    }
}

pub(crate) const WIRE_DECIMALS: u8 = 8;

pub(crate) fn float_to_string_for_hashing(x: f64) -> String {
    let mut x = format!("{:.*}", WIRE_DECIMALS.into(), x);
    while x.ends_with('0') {
        x.pop();
    }
    if x.ends_with('.') {
        x.pop();
    }
    if x == "-0" {
        "0".to_string()
    } else {
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_api_info() {
        dotenv::dotenv().ok();

        // Read base url and api key from environment variables
        let base_url =
            std::env::var("WALLET_API_BASE_URL").expect("WALLET_API_BASE_URL env var not set");
        let api_key = std::env::var("WALLET_API_KEY").expect("WALLET_API_KEY env var not set");

        let client = SuperstackApiClient::new(&base_url, &api_key);

        let result = client.get_api_info().await;

        match result {
            Ok(api_info_response) => {
                println!("api_info_response: {:?}", api_info_response);
                // Basic checks on the response
                assert!(
                    !api_info_response.user_info.wallets.is_empty()
                        || api_info_response.user_info.wallets.is_empty()
                );
                assert!(api_info_response.api_info.id != uuid::Uuid::nil());
            }
            Err(e) => {
                panic!("get_api_info failed: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_api_info_invalid_api_key() {
        dotenv::dotenv().ok();

        // Read base url from environment variable
        let base_url =
            std::env::var("WALLET_API_BASE_URL").expect("WALLET_API_BASE_URL env var not set");
        let api_key = "invalid_api_key";

        let client = SuperstackApiClient::new(&base_url, &api_key);
        let result = client.get_api_info().await;
        match result {
            Err(e) => {
                println!("Expected error: {:?}", e);
                assert!(e.to_string().contains("401 Unauthorized"));
            }
            _ => panic!("Expected error, got success"),
        }
    }
}
