/// Every failure mode the widget can present to the user.
#[derive(Debug, Clone)]
pub enum WidgetError {
    /// Token store (Keychain / credentials file) absent or unreadable.
    TokenNotFound,
    /// Token store present but not in the expected shape.
    TokenMalformed,
    /// HTTP 401 — token rejected by the endpoint.
    Auth,
    /// Transport failure or non-401 non-200 HTTP status.
    Network(String),
    /// HTTP 200 but the JSON body did not match the expected schema.
    Format,
}
