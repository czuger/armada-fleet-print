//! HTTP client shared by every command.

use std::time::Duration;

use reqwest::Client;

/// Timeout of each outgoing request.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Builds the HTTP client used for all downloads, with a timeout and a generic User-Agent.
///
/// # Errors
/// Returns [`reqwest::Error`] if the client cannot be initialized.
pub fn build_http_client() -> Result<Client, reqwest::Error> {
    Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .user_agent(concat!("armada-fleet-print/", env!("CARGO_PKG_VERSION")))
        .build()
}
