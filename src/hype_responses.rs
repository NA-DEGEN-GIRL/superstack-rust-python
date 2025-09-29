use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct HypeRestingOrder {
    pub oid: u64,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HypeFilledOrder {
    pub total_sz: String,
    pub avg_px: String,
    pub oid: u64,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum HypeExchangeDataStatus {
    Success,
    WaitingForFill,
    WaitingForTrigger,
    Error(String),
    Resting(HypeRestingOrder),
    Filled(HypeFilledOrder),
}

#[derive(Deserialize, Debug, Clone)]
pub struct HypeExchangeDataStatuses {
    pub statuses: Vec<HypeExchangeDataStatus>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct HypeExchangeResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub data: Option<HypeExchangeDataStatuses>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "status", content = "response")]
pub enum HypeExchangeResponseStatus {
    Ok(HypeExchangeResponse),
    Err(String),
}
