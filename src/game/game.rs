//! A game shared by several players: their stock and the fleets drawing on it.

use serde::{Deserialize, Serialize};

use crate::game::stock::Stock;

/// Version of the saved game format.
pub const GAME_SCHEMA_VERSION: u32 = 1;
/// Most fleets in one game.
pub const MAX_FLEETS: usize = 30;
/// Longest game or player name, in characters.
pub const MAX_NAME_CHARS: usize = 100;

/// A game, saved as one JSON file named after its [`GameId`](crate::game::GameId).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    /// Version of the saved format.
    pub schema_version: u32,
    /// Name chosen at creation.
    pub name: String,
    /// Creation time, RFC 3339 in UTC.
    pub created_at: String,
    /// Miniatures available to the fleets.
    #[serde(default)]
    pub stock: Stock,
    /// Fleets of the game, in the order they were added.
    #[serde(default)]
    pub fleets: Vec<GameFleet>,
}

/// A fleet of the game, read from its fleet builder link whenever results are shown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameFleet {
    /// Identifier within the game.
    pub id: u32,
    /// Who plays the fleet.
    pub player: String,
    /// Link to the saved fleet, validated when added.
    pub url: String,
}

impl Game {
    /// Creates an empty game named `name`, created now.
    pub fn new(name: String) -> Self {
        Self {
            schema_version: GAME_SCHEMA_VERSION,
            name,
            created_at: jiff::Timestamp::now()
                .strftime("%Y-%m-%dT%H:%M:%SZ")
                .to_string(),
            stock: Stock::default(),
            fleets: Vec::new(),
        }
    }

    /// Adds a fleet and returns its identifier.
    pub fn add_fleet(&mut self, player: String, url: String) -> u32 {
        let id = self
            .fleets
            .iter()
            .map(|fleet| fleet.id)
            .max()
            .map_or(1, |id| id + 1);
        self.fleets.push(GameFleet { id, player, url });
        id
    }

    /// Gives the fleet `id` a new player and link, keeping its place in the list.
    ///
    /// # Returns
    /// Whether the fleet existed.
    pub fn replace_fleet(&mut self, id: u32, player: String, url: String) -> bool {
        let Some(fleet) = self.fleets.iter_mut().find(|fleet| fleet.id == id) else {
            return false;
        };
        fleet.player = player;
        fleet.url = url;
        true
    }

    /// Removes the fleet `id`.
    ///
    /// # Returns
    /// Whether the fleet existed.
    pub fn remove_fleet(&mut self, id: u32) -> bool {
        let before = self.fleets.len();
        self.fleets.retain(|fleet| fleet.id != id);
        self.fleets.len() != before
    }
}
