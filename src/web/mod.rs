//! Web front end, in French or English: a form taking a fleet link and returning its PDF,
//! and the pages of multi-player games.

mod app_state;
mod cookies;
mod game_form_error;
mod game_page;
mod game_routes;
mod lang_cookie;
mod localize;
mod orientation_cookie;
mod pages;
mod router;
mod server;
mod text;
mod web_error;

pub use app_state::AppState;
pub use router::router;
pub use server::serve;
pub use web_error::WebError;
