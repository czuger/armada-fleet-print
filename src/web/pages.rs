//! HTML pages served to users.

/// Home page with the fleet link form.
pub(crate) const INDEX: &str = include_str!("index.html");

const ERROR_TEMPLATE: &str = include_str!("error.html");

/// Error page showing `message`, which must be trusted HTML-safe text.
pub(crate) fn error_page(message: &'static str) -> String {
    ERROR_TEMPLATE.replace("{message}", message)
}
