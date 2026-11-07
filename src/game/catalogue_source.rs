//! The builder card list, downloaded on first use and refreshed daily.

use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::Client;
use tokio::sync::Mutex;

use crate::error_chain::error_chain;
use crate::game::model_catalogue::{CatalogueParseError, ModelCatalogue};

/// Script of the fleet builder holding every card.
pub const CARDS_JS_URL: &str = "https://armada.ryankingston.com/site/templates/js/cards.js";
/// Age after which the card list is downloaded again.
const MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

/// Provides the [`ModelCatalogue`], caching it in memory.
#[derive(Debug, Clone)]
pub struct CatalogueSource(Origin);

#[derive(Debug, Clone)]
enum Origin {
    Fixed(Arc<ModelCatalogue>),
    Builder {
        client: Client,
        cached: Arc<Mutex<Option<CachedCatalogue>>>,
    },
}

#[derive(Debug)]
struct CachedCatalogue {
    fetched_at: Instant,
    catalogue: Arc<ModelCatalogue>,
}

/// Error returned when the card list cannot be obtained.
#[derive(Debug, thiserror::Error)]
pub enum CatalogueError {
    /// The card list could not be downloaded.
    #[error("downloading card list failed")]
    Fetch(#[from] reqwest::Error),
    /// The card list could not be read.
    #[error("reading card list failed")]
    Parse(#[from] CatalogueParseError),
}

impl CatalogueSource {
    /// Creates a source downloading the card list from the fleet builder with `client`.
    pub fn new(client: Client) -> Self {
        Self(Origin::Builder {
            client,
            cached: Arc::new(Mutex::new(None)),
        })
    }

    /// Creates a source always returning `catalogue`, without network access.
    pub fn fixed(catalogue: ModelCatalogue) -> Self {
        Self(Origin::Fixed(Arc::new(catalogue)))
    }

    /// Returns the catalogue, downloading it if it is missing or older than a day. When a
    /// refresh fails, the previous catalogue is returned and the failure logged.
    ///
    /// # Errors
    /// Returns [`CatalogueError`] if there is no previous catalogue and the download fails.
    pub async fn get(&self) -> Result<Arc<ModelCatalogue>, CatalogueError> {
        let (client, cached) = match &self.0 {
            Origin::Fixed(catalogue) => return Ok(Arc::clone(catalogue)),
            Origin::Builder { client, cached } => (client, cached),
        };
        // Held during the download, so concurrent requests wait for it instead of repeating it.
        let mut cached = cached.lock().await;
        if let Some(fresh) = cached.as_ref().filter(|c| c.fetched_at.elapsed() < MAX_AGE) {
            return Ok(Arc::clone(&fresh.catalogue));
        }
        match download(client).await {
            Ok(catalogue) => {
                let catalogue = Arc::new(catalogue);
                *cached = Some(CachedCatalogue {
                    fetched_at: Instant::now(),
                    catalogue: Arc::clone(&catalogue),
                });
                Ok(catalogue)
            }
            Err(err) => match cached.as_ref() {
                Some(stale) => {
                    tracing::warn!(
                        "refreshing card list failed, keeping the previous one: {}",
                        error_chain(&err)
                    );
                    Ok(Arc::clone(&stale.catalogue))
                }
                None => Err(err),
            },
        }
    }
}

async fn download(client: &Client) -> Result<ModelCatalogue, CatalogueError> {
    let script = client
        .get(CARDS_JS_URL)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    Ok(ModelCatalogue::parse_cards_js(&script)?)
}
