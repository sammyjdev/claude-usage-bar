//! Token from the macOS Keychain item `Claude Code-credentials`.

use std::process::Command;

use crate::error::WidgetError;

pub fn fetch_token() -> Result<String, WidgetError> {
    let output = Command::new("/usr/bin/security")
        .args([
            "find-generic-password",
            "-s",
            "Claude Code-credentials",
            "-w",
        ])
        .output()
        .map_err(|_| WidgetError::TokenNotFound)?;

    if !output.status.success() {
        return Err(WidgetError::TokenNotFound);
    }
    let body = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value =
        serde_json::from_str(body.trim()).map_err(|_| WidgetError::TokenMalformed)?;
    value
        .get("claudeAiOauth")
        .and_then(|o| o.get("accessToken"))
        .and_then(|t| t.as_str())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .ok_or(WidgetError::TokenMalformed)
}
