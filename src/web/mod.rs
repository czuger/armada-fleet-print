//! Web front end: a form taking a fleet link and returning its PDF.

mod pages;
mod router;
mod server;
mod web_error;

pub use router::router;
pub use server::serve;
pub use web_error::WebError;
