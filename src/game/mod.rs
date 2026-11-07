//! Games shared by several players: a stock of miniatures pasted from a spreadsheet, fleets
//! linked from the fleet builder, and the check that the fleets fit the stock without
//! taking a unique card twice. A game is reached by its secret link only.

mod catalogue_source;
mod faction;
mod figure_kind;
mod fleet_roster;
#[allow(clippy::module_inception)]
mod game;
mod game_id;
mod game_report;
mod game_service;
mod game_store;
mod model_catalogue;
mod name_matcher;
mod stock;

pub use catalogue_source::{CARDS_JS_URL, CatalogueError, CatalogueSource};
pub use faction::Faction;
pub use figure_kind::FigureKind;
pub use fleet_roster::{FigureCount, FleetRoster};
pub use game::{GAME_SCHEMA_VERSION, Game, GameFleet, MAX_FLEETS, MAX_NAME_CHARS};
pub use game_id::{GameId, GameIdError};
pub use game_report::{
    DuplicateCard, FleetLoadError, FleetResult, GameReport, KindTotal, ModelUsage, Share,
};
pub use game_service::{GameError, GameService};
pub use game_store::{GameStore, GameStoreError};
pub use model_catalogue::{CatalogueModel, CatalogueParseError, ModelCatalogue};
pub use stock::{MAX_STOCK_LINES, Stock, StockLine, StockParseError};
