use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Request {
    #[serde(rename = "reqID")]
    pub req_id: String,
    pub language: String,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub mobileauth: String,
}

#[derive(Debug, Deserialize)]
pub struct ResponseDecrypted {
    pub authkey: String,
    pub otp: String,
    #[serde(rename = "tokenUID")]
    pub token_uid: String,
}
