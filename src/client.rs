//! Fetches the usage payload from the undocumented `/api/oauth/usage` endpoint.

use std::io::Read;
use std::time::Duration;

use crate::error::WidgetError;
use crate::token;
use crate::usage::{decode_usage, Usage};

const ENDPOINT: &str = "https://api.anthropic.com/api/oauth/usage";

pub fn fetch_usage() -> Result<Usage, WidgetError> {
    let token = token::fetch_token()?;

    let response = ureq::get(ENDPOINT)
        .set("Authorization", &format!("Bearer {token}"))
        .set("anthropic-beta", "oauth-2025-04-20")
        .timeout(Duration::from_secs(15))
        .call();

    match response {
        Ok(resp) => {
            let mut bytes = Vec::new();
            resp.into_reader()
                .read_to_end(&mut bytes)
                .map_err(|e| WidgetError::Network(e.to_string()))?;
            decode_usage(&bytes)
        }
        Err(ureq::Error::Status(401, _)) => Err(WidgetError::Auth),
        Err(ureq::Error::Status(code, _)) => {
            Err(WidgetError::Network(format!("HTTP {code}")))
        }
        Err(ureq::Error::Transport(t)) => Err(WidgetError::Network(t.to_string())),
    }
}
