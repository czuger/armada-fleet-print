//! Games saved as JSON files, one per game, in a directory.

use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::atomic_file::{AtomicWriteError, write_atomic};
use crate::game::game::Game;
use crate::game::game_id::GameId;

/// Directory of saved games.
#[derive(Debug, Clone)]
pub struct GameStore {
    dir: PathBuf,
    /// Serializes read-modify-write cycles, so two players editing at once lose nothing.
    write_lock: Arc<Mutex<()>>,
}

/// Error returned when a game cannot be read or saved.
#[derive(Debug, thiserror::Error)]
pub enum GameStoreError {
    /// No game has this identifier.
    #[error("game not found")]
    NotFound,
    /// No random identifier could be drawn.
    #[error("generating game identifier failed")]
    Random(#[source] getrandom::Error),
    /// The game file could not be read.
    #[error("reading {path} failed")]
    Read {
        /// Game file.
        path: PathBuf,
        /// Underlying error.
        #[source]
        source: io::Error,
    },
    /// The game file is not a valid game.
    #[error("game file {path} is corrupt")]
    Corrupt {
        /// Game file.
        path: PathBuf,
        /// Underlying error.
        #[source]
        source: serde_json::Error,
    },
    /// The game could not be serialized.
    #[error("serializing game failed")]
    Serialize(#[source] serde_json::Error),
    /// The game file could not be written.
    #[error("saving game failed")]
    Write(#[from] AtomicWriteError),
}

impl GameStore {
    /// Creates a store saving games in `dir`, which is created on the first save.
    pub fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            write_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Saves `game` under a new identifier.
    ///
    /// # Errors
    /// Returns [`GameStoreError`] if no identifier can be drawn or the file cannot be written.
    pub async fn create(&self, game: &Game) -> Result<GameId, GameStoreError> {
        let id = GameId::generate().map_err(GameStoreError::Random)?;
        let _guard = self.write_lock.lock().await;
        self.save(&id, game).await?;
        Ok(id)
    }

    /// Reads the game `id`.
    ///
    /// # Errors
    /// Returns [`GameStoreError::NotFound`] for an unknown game, another variant if the
    /// file cannot be read.
    pub async fn load(&self, id: &GameId) -> Result<Game, GameStoreError> {
        let path = self.path(id);
        let bytes = match tokio::fs::read(&path).await {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                return Err(GameStoreError::NotFound);
            }
            Err(source) => return Err(GameStoreError::Read { path, source }),
        };
        serde_json::from_slice(&bytes).map_err(|source| GameStoreError::Corrupt { path, source })
    }

    /// Applies `change` to the game `id` and saves it, with no other update in between.
    ///
    /// # Returns
    /// What `change` returned.
    ///
    /// # Errors
    /// Returns [`GameStoreError`] if the game cannot be read or saved.
    pub async fn update<T>(
        &self,
        id: &GameId,
        change: impl FnOnce(&mut Game) -> T,
    ) -> Result<T, GameStoreError> {
        let _guard = self.write_lock.lock().await;
        let mut game = self.load(id).await?;
        let result = change(&mut game);
        self.save(id, &game).await?;
        Ok(result)
    }

    async fn save(&self, id: &GameId, game: &Game) -> Result<(), GameStoreError> {
        let mut json = serde_json::to_vec_pretty(game).map_err(GameStoreError::Serialize)?;
        json.push(b'\n');
        write_atomic(&self.path(id), &json).await?;
        Ok(())
    }

    fn path(&self, id: &GameId) -> PathBuf {
        self.dir.join(format!("{id}.json"))
    }
}
