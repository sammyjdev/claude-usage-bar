/// Every failure mode the widget can present to the user.
#[derive(Debug, Clone)]
pub enum WidgetError {
    /// Claude Code logs directory could not be located, or no usage events
    /// were found inside it. Surfaces in the UI as `⚠ logs`.
    LogsNotFound,
}
