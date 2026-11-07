//! Operations on games: creation, stock, fleets and results.

use std::path::PathBuf;

use futures_util::{StreamExt, stream};
use reqwest::Client;

use crate::error_chain::error_chain;
use crate::fleet_page::fetch_fleet_page;
use crate::fleet_url::FleetUrl;
use crate::game::catalogue_source::{CatalogueError, CatalogueSource};
use crate::game::fleet_roster::FleetRoster;
use crate::game::game::{Game, GameFleet, MAX_FLEETS, MAX_NAME_CHARS};
use crate::game::game_id::GameId;
use crate::game::game_report::{FleetLoadError, FleetResult, GameReport};
use crate::game::game_store::{GameStore, GameStoreError};
use crate::game::stock::{Stock, StockParseError};
use crate::http_client::build_http_client;

/// Fleet pages downloaded at the same time when computing results.
const CONCURRENT_FLEETS: usize = 4;

/// Manages the games saved in a directory.
#[derive(Debug, Clone)]
pub struct GameService {
    client: Client,
    store: GameStore,
    catalogue: CatalogueSource,
}

/// Error returned when a game operation fails.
#[derive(Debug, thiserror::Error)]
pub enum GameError {
    /// The game could not be read or saved, or does not exist.
    #[error("game storage failed")]
    Store(#[from] GameStoreError),
    /// A game or player name is empty or longer than [`MAX_NAME_CHARS`].
    #[error("invalid name")]
    InvalidName,
    /// The game already has [`MAX_FLEETS`] fleets.
    #[error("too many fleets")]
    TooManyFleets,
    /// The game has no fleet with this identifier.
    #[error("fleet not found")]
    FleetNotFound,
    /// The pasted stock is unreadable.
    #[error("invalid stock")]
    Stock(#[from] StockParseError),
    /// The builder card list, needed to read the stock, is unavailable.
    #[error("card list unavailable")]
    Catalogue(#[from] CatalogueError),
    /// The fleet to add cannot be read.
    #[error("invalid fleet")]
    Fleet(#[from] FleetLoadError),
}

impl GameService {
    /// Creates a service saving games in `dir` and reading the card list from the fleet builder.
    ///
    /// # Errors
    /// Returns [`reqwest::Error`] if the HTTP client cannot be initialized.
    pub fn new(dir: PathBuf) -> Result<Self, reqwest::Error> {
        let client = build_http_client()?;
        let catalogue = CatalogueSource::new(client.clone());
        Ok(Self::with_catalogue(dir, client, catalogue))
    }

    /// Creates a service with its own card list source, for instance a fixed one in tests.
    pub fn with_catalogue(dir: PathBuf, client: Client, catalogue: CatalogueSource) -> Self {
        Self {
            client,
            store: GameStore::new(dir),
            catalogue,
        }
    }

    /// Creates an empty game.
    ///
    /// # Errors
    /// Returns [`GameError::InvalidName`] for an empty or too long name, or a storage error.
    pub async fn create(&self, name: &str) -> Result<GameId, GameError> {
        let game = Game::new(clean_name(name)?);
        Ok(self.store.create(&game).await?)
    }

    /// Reads the game `id`.
    ///
    /// # Errors
    /// Returns [`GameError::Store`] if the game does not exist or cannot be read.
    pub async fn load(&self, id: &GameId) -> Result<Game, GameError> {
        Ok(self.store.load(id).await?)
    }

    /// Replaces the stock of the game `id` with the one pasted in `text`.
    ///
    /// # Errors
    /// Returns [`GameError`] if the stock is unreadable, the card list unavailable, or the
    /// game cannot be updated.
    pub async fn replace_stock(&self, id: &GameId, text: &str) -> Result<(), GameError> {
        let catalogue = self.catalogue.get().await?;
        let stock = Stock::parse(text, &catalogue)?;
        self.store.update(id, |game| game.stock = stock).await?;
        Ok(())
    }

    /// Adds the fleet at `url`, played by `player`, after checking it can be read.
    ///
    /// # Errors
    /// Returns [`GameError`] if the name or fleet is invalid, the game is full, or the game
    /// cannot be updated.
    pub async fn add_fleet(&self, id: &GameId, player: &str, url: &str) -> Result<(), GameError> {
        let player = clean_name(player)?;
        let url = FleetUrl::parse(url).map_err(FleetLoadError::from)?;
        if self.store.load(id).await?.fleets.len() >= MAX_FLEETS {
            return Err(GameError::TooManyFleets);
        }
        self.read_roster(&url).await?;
        let url = url.to_string();
        let added = self
            .store
            .update(id, |game| {
                // Checked again: another fleet may have been added during the download.
                (game.fleets.len() < MAX_FLEETS).then(|| game.add_fleet(player, url))
            })
            .await?;
        added.map(|_| ()).ok_or(GameError::TooManyFleets)
    }

    /// Removes the fleet `fleet_id` from the game `id`.
    ///
    /// # Errors
    /// Returns [`GameError::FleetNotFound`] for an unknown fleet, or a storage error.
    pub async fn remove_fleet(&self, id: &GameId, fleet_id: u32) -> Result<(), GameError> {
        let removed = self
            .store
            .update(id, |game| game.remove_fleet(fleet_id))
            .await?;
        if removed {
            Ok(())
        } else {
            Err(GameError::FleetNotFound)
        }
    }

    /// Downloads every fleet of `game` and compares them with its stock. A fleet that cannot
    /// be read is reported as such and left out of the counts.
    pub async fn report(&self, game: &Game) -> GameReport {
        let fleets: Vec<FleetResult> = stream::iter(game.fleets.iter().cloned())
            .map(|fleet| self.fleet_result(fleet))
            .buffered(CONCURRENT_FLEETS)
            .collect()
            .await;
        let catalogue = match self.catalogue.get().await {
            Ok(catalogue) => Some(catalogue),
            Err(err) => {
                tracing::warn!(
                    "card list unavailable for model names: {}",
                    error_chain(&err)
                );
                None
            }
        };
        GameReport::build(&game.stock, fleets, catalogue.as_deref())
    }

    async fn fleet_result(&self, fleet: GameFleet) -> FleetResult {
        let roster = match FleetUrl::parse(&fleet.url) {
            Ok(url) => self.read_roster(&url).await,
            Err(err) => Err(err.into()),
        };
        if let Err(err) = &roster {
            tracing::warn!(url = %fleet.url, "reading game fleet failed: {}", error_chain(err));
        }
        FleetResult { fleet, roster }
    }

    async fn read_roster(&self, url: &FleetUrl) -> Result<FleetRoster, FleetLoadError> {
        let html = fetch_fleet_page(&self.client, url).await?;
        Ok(FleetRoster::from_page_html(&html)?)
    }
}

/// Trims `name` and checks it is neither empty nor longer than [`MAX_NAME_CHARS`].
fn clean_name(name: &str) -> Result<String, GameError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME_CHARS {
        return Err(GameError::InvalidName);
    }
    Ok(name.to_owned())
}
