//! OAuth token retrieval, dispatched by platform. Read fresh on every poll —
//! Claude Code rotates the token, so a cached value would go stale.

#[cfg(target_os = "macos")]
mod macos;

// Always compiled so its unit tests run on every platform; only used at
// runtime off macOS.
#[cfg_attr(target_os = "macos", allow(dead_code))]
mod file;

use crate::error::WidgetError;

pub fn fetch_token() -> Result<String, WidgetError> {
    #[cfg(target_os = "macos")]
    {
        macos::fetch_token()
    }
    #[cfg(not(target_os = "macos"))]
    {
        file::fetch_token()
    }
}
