use alloy::primitives::Address;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::actions::Actions;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Network {
    Ethereum,
    Solana,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub enum WalletSet {
    Main,
    Secondary(u32),
    Forward(u32),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletInfo {
    pub address: String,
    pub network: Network,
    pub wallet_set: WalletSet,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub wallets: Vec<WalletInfo>,
}

impl UserInfo {
    pub fn get_evm_main_wallet(&self) -> Option<&WalletInfo> {
        self.wallets
            .iter()
            .find(|w| w.network == Network::Ethereum && w.wallet_set == WalletSet::Main)
    }

    pub fn get_solana_main_wallet(&self) -> Option<&WalletInfo> {
        self.wallets
            .iter()
            .find(|w| w.network == Network::Solana && w.wallet_set == WalletSet::Main)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiInfo {
    pub id: Uuid,
    pub created_at: i64, // Unix timstamp
    pub ip_whitelist: Option<Vec<IpNetwork>>,
    pub permissions: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiInfoResponse {
    pub user_info: UserInfo,
    pub api_info: ApiInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeRequest {
    pub action: Actions,
    pub vault_address: Option<Address>,
    pub expires_after: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExchangePayload {
    pub action: serde_json::Value,
    pub nonce: u64,
    pub signature: serde_json::Value,
    pub vault_address: Option<Address>,
    pub expires_after: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeResponse {
    pub payload: ExchangePayload,
}
