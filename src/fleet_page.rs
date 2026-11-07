//! Download of fleet builder pages.

use reqwest::Client;

use crate::fleet_url::FleetUrl;

/// Downloads the HTML page of the fleet at `url`.
///
/// # Errors
/// Returns [`reqwest::Error`] if the request fails or the server answers with an error status.
pub async fn fetch_fleet_page(client: &Client, url: &FleetUrl) -> Result<String, reqwest::Error> {
    let response = client.get(url.as_url().clone()).send().await?;
    response.error_for_status()?.text().await
}
