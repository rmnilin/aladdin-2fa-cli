use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub accepted: bool,
    pub device_time: String,
    pub hmac: String,
    pub jassessionid: String,
    pub language: String,
    pub mobile_local_time: String,
    pub mobile_os: String,
    pub mobile_version: String,
    #[serde(rename = "sessionID")]
    pub session_id: String,
    #[serde(rename = "tokenUID")]
    pub token_uid: String,
    pub valueotp: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {}
