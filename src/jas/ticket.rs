use std::{fs, io::Write};

use color_eyre::Result;
use serde::{Deserialize, Serialize};

/// JAS ticket data.
#[derive(Debug, Deserialize, Serialize)]
pub struct Ticket {
    pub request_id: String,
    pub server_root_uri: String,
    pub pin_length: u8,
    pub auth_key: String,
    pub totp_uri: String,
    pub token_id: String,
}

impl Ticket {
    /// Save the ticket data to a file.
    pub fn save(&self, path: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = fs::File::create(path)?;
        writeln!(file, "{}", json)?;
        Ok(())
    }

    /// Load the ticket data from a file.
    pub fn load(path: &str) -> Result<Self> {
        let json = fs::read_to_string(path)?;
        let ticket = serde_json::from_str::<Self>(&json)?;
        Ok(ticket)
    }
}
