//! Armada Fleet Print: turns a fleet saved on <https://armada.ryankingston.com/> into a
//! printable PDF of its cards.
#![warn(missing_docs)]

pub mod card_image;
pub mod card_name;
pub mod card_store;
pub mod cli;
pub mod error_chain;
pub mod fleet;
pub mod fleet_printer;
pub mod fleet_url;
pub mod layout;
pub mod pdf;
pub mod web;

mod fleet_json;
mod fleet_script;
