//! Local database of every Star Wars: Armada card, built offline from the community
//! BattleScribe data of <https://github.com/BSData/star-wars-armada>. The web server does not
//! use it.

mod armada_database;
mod bs_catalogue;
mod card_kind;
mod characteristics;
mod data_sync;
mod dice;
mod github_source;
mod objective;
mod raw_entry;
mod ship;
mod speed_chart;
mod squadron;
mod upgrade;

pub use armada_database::{ArmadaDatabase, DataSource, SCHEMA_VERSION};
pub use bs_catalogue::{BsCatalogue, BsCatalogueError};
pub use card_kind::CardKind;
pub use data_sync::{DataSync, DataSyncError};
pub use dice::Dice;
pub use github_source::{BSDATA_REPO, GithubError, GithubSource, ResolvedRef, SOURCE_FILES};
pub use objective::Objective;
pub use raw_entry::{CardTitle, RawEntry, RawProfile};
pub use ship::Ship;
pub use speed_chart::SpeedChart;
pub use squadron::Squadron;
pub use upgrade::Upgrade;
