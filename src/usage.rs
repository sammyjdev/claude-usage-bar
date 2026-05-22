use serde::Deserialize;

use crate::error::WidgetError;

#[derive(Debug, Clone, Deserialize)]
pub struct Window {
    pub utilization: u32,
    pub resets_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExtraUsage {
    pub is_enabled: bool,
    pub used_credits: Option<f64>,
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub five_hour: Window,
    pub seven_day: Window,
    pub seven_day_opus: Option<Window>,
    pub seven_day_sonnet: Option<Window>,
    pub extra_usage: Option<ExtraUsage>,
}

/// Decode the `/api/oauth/usage` JSON body.
pub fn decode_usage(body: &[u8]) -> Result<Usage, WidgetError> {
    serde_json::from_slice(body).map_err(|_| WidgetError::Format)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "five_hour":{"utilization":17,"resets_at":"2026-05-21T20:50:00.508668+00:00"},
        "seven_day":{"utilization":18,"resets_at":"2026-05-24T07:00:00.508691+00:00"},
        "seven_day_opus":null,
        "seven_day_sonnet":{"utilization":5,"resets_at":"2026-05-24T07:00:00.508699+00:00"},
        "extra_usage":{"is_enabled":true,"monthly_limit":null,"used_credits":82,
                       "utilization":null,"currency":"BRL","disabled_reason":null}
    }"#;

    #[test]
    fn decodes_sample_payload() {
        let u = decode_usage(SAMPLE.as_bytes()).expect("should decode");
        assert_eq!(u.five_hour.utilization, 17);
        assert_eq!(u.seven_day.utilization, 18);
        assert_eq!(u.seven_day_sonnet.unwrap().utilization, 5);
        assert!(u.seven_day_opus.is_none());
        assert_eq!(u.extra_usage.unwrap().used_credits, Some(82.0));
        assert!(u.five_hour.resets_at.is_some());
    }

    #[test]
    fn rejects_garbage() {
        assert!(matches!(decode_usage(b"not json"), Err(WidgetError::Format)));
    }
}
