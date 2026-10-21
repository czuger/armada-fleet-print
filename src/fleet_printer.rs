//! End-to-end printing: fleet page → card images → layout → PDF.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use reqwest::Client;

use crate::card_image::CardImage;
use crate::card_name::CardName;
use crate::card_store::{CardStore, CardStoreError};
use crate::fleet::{Fleet, FleetParseError};
use crate::fleet_url::FleetUrl;
use crate::layout::{LayoutError, lay_out};
use crate::pdf::{RenderError, render};

/// Timeout of each request to the fleet builder.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Produces the printable PDF of saved fleets.
#[derive(Debug, Clone)]
pub struct FleetPrinter {
    client: Client,
    cards: CardStore,
}

/// Error returned when a fleet cannot be printed.
#[derive(Debug, thiserror::Error)]
pub enum PrintError {
    /// The fleet page could not be downloaded.
    #[error("downloading fleet page failed")]
    FetchPage(#[source] reqwest::Error),
    /// The fleet page has no readable fleet.
    #[error("reading fleet failed")]
    Parse(#[from] FleetParseError),
    /// A card image could not be obtained.
    #[error("getting card images failed")]
    Cards(#[from] CardStoreError),
    /// A card image could not be decoded.
    #[error("decoding card {card} failed")]
    Decode {
        /// Card whose image is corrupt.
        card: CardName,
        /// Underlying error.
        #[source]
        source: image::ImageError,
    },
    /// The layout step failed.
    #[error("laying out fleet failed")]
    Layout(#[from] LayoutError),
    /// The PDF step failed.
    #[error("rendering PDF failed")]
    Render(#[from] RenderError),
    /// The background rendering task panicked or was cancelled.
    #[error("rendering task failed")]
    Task(#[from] tokio::task::JoinError),
}

impl FleetPrinter {
    /// Creates a printer caching card images in `cards_dir`.
    ///
    /// # Errors
    /// Returns [`reqwest::Error`] if the HTTP client cannot be initialized.
    pub fn new(cards_dir: PathBuf) -> Result<Self, reqwest::Error> {
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent(concat!("armada-fleet-print/", env!("CARGO_PKG_VERSION")))
            .build()?;
        let cards = CardStore::new(cards_dir, client.clone());
        Ok(Self { client, cards })
    }

    /// Downloads the fleet at `url` and returns its PDF.
    ///
    /// # Errors
    /// Returns [`PrintError`] describing the failed step.
    pub async fn print(&self, url: &FleetUrl) -> Result<Vec<u8>, PrintError> {
        let html = self.fetch_page(url).await.map_err(PrintError::FetchPage)?;
        let fleet = Fleet::from_page_html(&html)?;
        let encoded = self.cards.load_all(&fleet.cards()).await?;
        // Decoding and compressing images is CPU-bound, so keep it off the async workers.
        tokio::task::spawn_blocking(move || render_fleet(&fleet, &encoded)).await?
    }

    async fn fetch_page(&self, url: &FleetUrl) -> Result<String, reqwest::Error> {
        let response = self.client.get(url.as_url().clone()).send().await?;
        response.error_for_status()?.text().await
    }
}

/// Decodes the card images, lays out `fleet` and renders the PDF.
fn render_fleet(
    fleet: &Fleet,
    encoded: &HashMap<CardName, Vec<u8>>,
) -> Result<Vec<u8>, PrintError> {
    let images = encoded
        .iter()
        .map(|(card, bytes)| {
            let image = CardImage::decode(bytes).map_err(|source| PrintError::Decode {
                card: card.clone(),
                source,
            })?;
            Ok((card.clone(), image))
        })
        .collect::<Result<HashMap<_, _>, PrintError>>()?;
    let aspect_ratios = images
        .iter()
        .map(|(card, image)| (card.clone(), image.aspect_ratio()))
        .collect();
    let plan = lay_out(fleet, &aspect_ratios)?;
    Ok(render(&plan, &images, &fleet.name)?)
}
