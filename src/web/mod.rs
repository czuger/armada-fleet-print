//! Web front end: a form taking a fleet link and returning its PDF, and the pages of
//! multi-player games.

mod app_state;
mod game_page;
mod game_routes;
mod pages;
mod router;
mod server;
mod web_error;

pub use app_state::AppState;
pub use router::router;
pub use server::serve;
pub use web_error::WebError;
