//! Token from `~/.claude/.credentials.json` (Linux and Windows).

use std::path::PathBuf;

use crate::error::WidgetError;

fn credentials_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join(".credentials.json"))
}

/// Extract `claudeAiOauth.accessToken` from a credentials-file body.
fn token_from_json(body: &str) -> Result<String, WidgetError> {
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|_| WidgetError::TokenMalformed)?;
    value
        .get("claudeAiOauth")
        .and_then(|o| o.get("accessToken"))
        .and_then(|t| t.as_str())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .ok_or(WidgetError::TokenMalformed)
}

pub fn fetch_token() -> Result<String, WidgetError> {
    let path = credentials_path().ok_or(WidgetError::TokenNotFound)?;
    let body = std::fs::read_to_string(&path).map_err(|_| WidgetError::TokenNotFound)?;
    token_from_json(&body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_credentials() {
        let body = r#"{"claudeAiOauth":{"accessToken":"abc123"}}"#;
        assert_eq!(token_from_json(body).unwrap(), "abc123");
    }

    #[test]
    fn rejects_missing_token() {
        assert!(matches!(
            token_from_json(r#"{"claudeAiOauth":{}}"#),
            Err(WidgetError::TokenMalformed)
        ));
        assert!(matches!(
            token_from_json("not json"),
            Err(WidgetError::TokenMalformed)
        ));
    }
}
