//! On-disk cache of card images downloaded from the fleet builder.

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use futures_util::{StreamExt, TryStreamExt, stream};
use reqwest::Client;

use crate::atomic_file::write_atomic;
use crate::card_name::CardName;

/// Where the fleet builder serves card images from.
const CARD_IMAGES_URL: &str = "https://armada.ryankingston.com/img/cards/";

/// How many card images are downloaded at the same time.
const MAX_CONCURRENT_DOWNLOADS: usize = 4;

/// Card image cache backed by a local directory. Images are downloaded once, then read from disk.
#[derive(Debug, Clone)]
pub struct CardStore {
    dir: PathBuf,
    client: Client,
}

/// Error returned when a card image can be neither read from the cache nor downloaded.
#[derive(Debug, thiserror::Error)]
pub enum CardStoreError {
    /// Reading or writing the cache directory failed.
    #[error("card cache I/O failed on {path}")]
    Io {
        /// File or directory involved.
        path: PathBuf,
        /// Underlying error.
        #[source]
        source: io::Error,
    },
    /// Downloading the image from the fleet builder failed.
    #[error("downloading card {card} failed")]
    Download {
        /// Card being downloaded.
        card: CardName,
        /// Underlying error.
        #[source]
        source: reqwest::Error,
    },
}

impl CardStore {
    /// Creates a store caching images in `dir`, which is created on first download.
    ///
    /// # Arguments
    /// * `dir` - Cache directory.
    /// * `client` - HTTP client used for downloads.
    pub fn new(dir: PathBuf, client: Client) -> Self {
        Self { dir, client }
    }

    /// Returns the encoded bytes of every card in `cards`, downloading missing ones.
    ///
    /// # Errors
    /// Returns the first [`CardStoreError`] met.
    pub async fn load_all(
        &self,
        cards: &[CardName],
    ) -> Result<HashMap<CardName, Vec<u8>>, CardStoreError> {
        // Owned names: borrowed ones trip the higher-ranked `Send` check of axum handlers.
        stream::iter(cards.iter().cloned())
            .map(|card| async move {
                let bytes = self.load(&card).await?;
                Ok((card, bytes))
            })
            .buffer_unordered(MAX_CONCURRENT_DOWNLOADS)
            .try_collect()
            .await
    }

    /// Returns the encoded bytes of `card`, downloading it into the cache if needed.
    ///
    /// # Errors
    /// Returns [`CardStoreError`] if the cache cannot be read or written, or the download fails.
    pub async fn load(&self, card: &CardName) -> Result<Vec<u8>, CardStoreError> {
        let path = self.dir.join(card.as_str());
        match tokio::fs::read(&path).await {
            Ok(bytes) => return Ok(bytes),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(source) => return Err(CardStoreError::Io { path, source }),
        }
        let bytes = self.download(card).await?;
        self.save(&path, &bytes).await?;
        Ok(bytes)
    }

    /// Fetches `card` from the fleet builder.
    async fn download(&self, card: &CardName) -> Result<Vec<u8>, CardStoreError> {
        let to_error = |source| CardStoreError::Download {
            card: card.clone(),
            source,
        };
        let url = format!("{CARD_IMAGES_URL}{card}");
        let response = self.client.get(url).send().await.map_err(to_error)?;
        let bytes = response.error_for_status().map_err(to_error)?.bytes().await;
        Ok(bytes.map_err(to_error)?.to_vec())
    }

    /// Writes `bytes` to `path` atomically so readers never see partial images.
    async fn save(&self, path: &Path, bytes: &[u8]) -> Result<(), CardStoreError> {
        write_atomic(path, bytes)
            .await
            .map_err(|err| CardStoreError::Io {
                path: err.path,
                source: err.source,
            })
    }
}
