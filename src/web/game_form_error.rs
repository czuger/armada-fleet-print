//! Messages explaining why a game form was rejected or a fleet of the game cannot be read.

use axum::http::StatusCode;

use crate::fleet::FleetParseError;
use crate::game::{
    FleetLoadError, GameError, MAX_FLEETS, MAX_NAME_CHARS, MAX_STOCK_LINES, StockParseError,
};
use crate::lang::Lang;
use crate::web::text::ErrorMessage;

/// Why a fleet of the game cannot be read.
pub(crate) fn fleet_error(err: &FleetLoadError) -> ErrorMessage {
    match err {
        FleetLoadError::Url(_) => ErrorMessage::InvalidFleetLink,
        FleetLoadError::Fetch(err) if err.status() == Some(reqwest::StatusCode::NOT_FOUND) => {
            ErrorMessage::FleetNotFound
        }
        FleetLoadError::Fetch(_) => ErrorMessage::FleetUnreachable,
        FleetLoadError::Parse(FleetParseError::NoFleetData) => ErrorMessage::NoFleetData,
        FleetLoadError::Parse(_) => ErrorMessage::UnreadableFleet,
    }
}

/// Status and message in `lang` for an error caused by what was typed in a game form,
/// `None` for any other error.
pub(crate) fn form_error(err: &GameError, lang: Lang) -> Option<(StatusCode, String)> {
    let invalid = |message: String| Some((StatusCode::UNPROCESSABLE_ENTITY, message));
    match err {
        GameError::Stock(err) => invalid(stock_error(err, lang)),
        GameError::InvalidName => invalid(match lang {
            Lang::Fr => format!("Indiquez un nom de joueur ({MAX_NAME_CHARS} caractères au plus)."),
            Lang::En => format!("Give a player name ({MAX_NAME_CHARS} characters at most)."),
        }),
        GameError::TooManyFleets => invalid(match lang {
            Lang::Fr => format!("Une partie ne peut pas avoir plus de {MAX_FLEETS} flottes."),
            Lang::En => format!("A game cannot have more than {MAX_FLEETS} fleets."),
        }),
        GameError::Catalogue(_) => Some((
            StatusCode::BAD_GATEWAY,
            ErrorMessage::CatalogueUnreachable.text(lang).to_owned(),
        )),
        GameError::Fleet(err) => Some((fleet_status(err), fleet_error(err).text(lang).to_owned())),
        GameError::FleetNotFound => Some((
            StatusCode::NOT_FOUND,
            ErrorMessage::FleetRemoved.text(lang).to_owned(),
        )),
        GameError::Store(_) => None,
    }
}

/// Bad gateway when the builder failed us, unprocessable when the link was wrong.
fn fleet_status(err: &FleetLoadError) -> StatusCode {
    match err {
        FleetLoadError::Fetch(fetch) if fetch.status() != Some(StatusCode::NOT_FOUND) => {
            StatusCode::BAD_GATEWAY
        }
        FleetLoadError::Parse(FleetParseError::Json(_) | FleetParseError::Quantity(..)) => {
            StatusCode::BAD_GATEWAY
        }
        _ => StatusCode::UNPROCESSABLE_ENTITY,
    }
}

fn stock_error(err: &StockParseError, lang: Lang) -> String {
    match lang {
        Lang::Fr => french_stock_error(err),
        Lang::En => english_stock_error(err),
    }
}

fn french_stock_error(err: &StockParseError) -> String {
    match err {
        StockParseError::Empty => "Le stock collé ne contient aucune figurine.".to_owned(),
        StockParseError::TooManyLines => {
            format!("Le stock ne peut pas dépasser {MAX_STOCK_LINES} lignes.")
        }
        StockParseError::Columns { line, found } => format!(
            "Ligne {line} : 6 colonnes attendues (Owner, Faction, Figurine Name, Size, Type, Qty), \
             {found} trouvée(s). Copiez les cellules depuis le tableur pour garder les tabulations."
        ),
        StockParseError::Kind { line, value } => {
            format!("Ligne {line} : type « {value} » inconnu, Ship ou Squadron attendu.")
        }
        StockParseError::Quantity { line, value } => {
            format!("Ligne {line} : quantité « {value} » invalide.")
        }
    }
}

fn english_stock_error(err: &StockParseError) -> String {
    match err {
        StockParseError::Empty => "The pasted stock has no miniature.".to_owned(),
        StockParseError::TooManyLines => {
            format!("The stock cannot have more than {MAX_STOCK_LINES} lines.")
        }
        StockParseError::Columns { line, found } => format!(
            "Line {line}: 6 columns expected (Owner, Faction, Figurine Name, Size, Type, Qty), \
             {found} found. Copy the cells from the spreadsheet to keep the tabs."
        ),
        StockParseError::Kind { line, value } => {
            format!("Line {line}: unknown type “{value}”, Ship or Squadron expected.")
        }
        StockParseError::Quantity { line, value } => {
            format!("Line {line}: invalid quantity “{value}”.")
        }
    }
}
