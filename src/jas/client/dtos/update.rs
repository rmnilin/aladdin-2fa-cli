use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub available_tokens: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResponseAvailableTokens {
    #[serde(rename = "tokenUID")]
    pub token_uid: String,
    #[serde(rename = "sessionID")]
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub available_tokens: Vec<ResponseAvailableTokens>,
}
