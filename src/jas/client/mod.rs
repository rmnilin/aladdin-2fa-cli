use aes_gcm::aead::Aead;
use base64::{Engine as _, engine::general_purpose as base64_engines};
use chrono::{SecondsFormat, Utc};
use color_eyre::{Result, eyre::eyre};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;

use crate::jas::Ticket;

mod dtos;

pub struct Client {
    http: reqwest::Client,
    retries: u8,
}

impl Client {
    pub fn new(ca_cert: Option<&[u8]>, retries: Option<u8>) -> Result<Self> {
        let http = reqwest::Client::builder().pool_idle_timeout(None);

        let http = if let Some(ca_cert) = ca_cert {
            let ca_cert = reqwest::Certificate::from_pem(ca_cert)?;
            http.add_root_certificate(ca_cert)
        } else {
            http
        };

        let http = http.build()?;
        Ok(Client {
            http,
            retries: retries.unwrap_or(3),
        })
    }

    async fn execute_request(&self, request: reqwest::Request) -> Result<reqwest::Response> {
        let mut error = Option::<reqwest::Error>::None;

        for _ in 0..self.retries {
            let response = self
                .http
                .execute(
                    request
                        .try_clone()
                        .ok_or(eyre!("Failed to clone request for retry"))?,
                )
                .await;

            match response {
                Ok(response) => return Ok(response),
                Err(e) => error = Some(e),
            }
        }

        Err(error
            .expect("Should have an error if all retries failed")
            .into())
    }

    pub async fn register(&self, uri: &str) -> Result<Ticket> {
        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct TokenUriQuery {
            address: String,
            key: String,
            pin_len: u8,
        }

        let uri = url::Url::parse(uri)?;

        if uri.scheme() != "jasticket" {
            return Err(eyre!("Invalid token URI scheme"));
        }

        let request_id = uri
            .host_str()
            .ok_or(eyre!("URI token host is required"))?
            .to_string();
        let query = serde_qs::from_str::<TokenUriQuery>(
            uri.query().ok_or(eyre!("URI token query is required"))?,
        )?;

        let server_root_uri = query.address;
        let pin_length = query.pin_len;

        let request = self
            .http
            .post(format!("{}/register", server_root_uri))
            .json(&dtos::register::Request {
                req_id: request_id.clone(),
                language: "en".to_string(),
            })
            .build()?;

        let response = self.execute_request(request).await?;
        if !response.status().is_success() {
            return Err(Self::get_error(response).await);
        }
        let response = response.json::<dtos::register::Response>().await?;

        let (auth_key, totp_uri, token_id) =
            Self::decrypt_register_response(&response, &query.key)?;

        Ok(Ticket {
            request_id,
            server_root_uri,
            pin_length,
            auth_key,
            totp_uri,
            token_id,
        })
    }

    async fn get_error(response: reqwest::Response) -> color_eyre::Report {
        let url = response.url().to_owned();
        let path = url.path();
        let status = response.status();

        if let Ok(text) = response.text().await {
            #[derive(Debug, Deserialize)]
            struct R {
                message: String,
            }
            if let Ok(r) = serde_json::from_str::<R>(&text) {
                return eyre!(
                    "Request failed\nPath: {}\nHTTP status code: {}\nMessage: {}",
                    path,
                    status,
                    r.message
                );
            }

            return eyre!(
                "Request failed\nPath: {}\nHTTP status code: {}\nBody: {}",
                path,
                status,
                text
            );
        }

        eyre!(
            "Request failed\nPath: {}\nHTTP status code: {}",
            path,
            status
        )
    }

    fn decrypt_register_response(
        dtos::register::Response { mobileauth: data }: &dtos::register::Response,
        key: &str,
    ) -> Result<(String, String, String)> {
        use aes_gcm::KeyInit;

        let key: &aes_gcm::Key<aes_gcm::Aes256Gcm> = key.as_bytes().into();

        let parts = data.split('.').collect::<Vec<_>>();
        let [nonce, cipher, tag] = *parts else {
            return Err(eyre!("Invalid encrypted auth data format:\n{}", data));
        };

        let cipher = format!("{}{}", cipher, tag);

        let data = aes_gcm::Aes256Gcm::new(key)
            .decrypt(nonce.as_bytes().into(), cipher.as_bytes())
            .map_err(|err| eyre!("Failed to decrypt auth data:\n{}", err))?;

        let data = String::from_utf8(data)?;

        let dtos::register::ResponseDecrypted {
            authkey: auth_key,
            otp: totp_uri,
            token_uid: token_id,
        } = serde_json::from_str(&data)?;

        Ok((auth_key, totp_uri, token_id))
    }

    pub async fn update(&self, ticket: &Ticket) -> Result<Vec<String>> {
        let request = self
            .http
            .post(format!("{}/updateTokens", ticket.server_root_uri))
            .json(&dtos::update::Request {
                available_tokens: vec![ticket.token_id.clone()],
            })
            .build()?;

        let response = self.execute_request(request).await?;
        if !response.status().is_success() {
            return Err(Self::get_error(response).await);
        }
        let response = response.json::<dtos::update::Response>().await?;

        let result = response
            .available_tokens
            .into_iter()
            .filter(|t| t.token_uid == ticket.token_id)
            .map(|t| t.session_id)
            .collect::<Vec<_>>();

        Ok(result)
    }

    pub async fn accept(&self, ticket: &Ticket, session_id: &str) -> Result<()> {
        let accepted = true;

        let totp_generator = totp_rs::TOTP::from_url(&ticket.totp_uri)?;
        let valueotp = totp_generator.generate_current()?;

        let hmac = Self::get_hmac(
            &ticket.auth_key,
            &[
                &ticket.token_id,
                session_id,
                &accepted.to_string(),
                &valueotp,
            ],
        );

        let device_time = Utc::now().to_rfc3339_opts(SecondsFormat::AutoSi, true); // TODO: yyyy-MM-dd HH:mm:ss,SSS+HH:mm,
        let mobile_local_time = device_time.clone(); // TODO: keep ISO format

        let request = dtos::accept::Request {
            accepted,
            device_time,
            hmac,
            jassessionid: String::new(), // TODO: is it okay to be empty?
            language: "en".to_string(),
            mobile_local_time,
            mobile_os: "ios".to_string(),
            mobile_version: "1.3.1.124".to_string(),
            session_id: session_id.to_string(),
            token_uid: ticket.token_id.clone(),
            valueotp,
        };
        let request = self
            .http
            .post(format!("{}/acceptToken", ticket.server_root_uri))
            .json(&request)
            .build()?;

        let response = self.execute_request(request).await?;
        if !response.status().is_success() {
            return Err(Self::get_error(response).await);
        }

        Ok(())
    }

    /// Get base64 encoded HMAC SHA256 hash
    pub fn get_hmac(key: &str, values: &[&str]) -> String {
        let message = values.join(":");

        let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes())
            .expect("HMAC should take key of any size");
        mac.update(message.as_bytes());
        let hash = mac.finalize();
        let hash = hash.into_bytes();

        base64_engines::STANDARD.encode(hash)
    }
}
