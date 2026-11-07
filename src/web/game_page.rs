//! Page of a game: results, fleets and stock.

use askama::Template;
use axum::http::StatusCode;

use crate::fleet::FleetParseError;
use crate::game::{
    FigureKind, FleetLoadError, Game, GameError, GameReport, MAX_FLEETS, MAX_NAME_CHARS,
    MAX_STOCK_LINES, Share, StockLine, StockParseError,
};

/// Text typed in the page forms, shown again when a submission is rejected.
#[derive(Debug, Default)]
pub(crate) struct Drafts {
    /// Pasted stock; when empty, the saved stock is shown instead.
    pub stock: String,
    /// Player of the fleet to add.
    pub player: String,
    /// Link of the fleet to add.
    pub url: String,
}

#[derive(Template)]
#[template(path = "game.html")]
struct GamePage<'a> {
    game: &'a Game,
    report: &'a GameReport,
    unmatched: Vec<&'a StockLine>,
    error: Option<&'a str>,
    stock_draft: String,
    player_draft: &'a str,
    url_draft: &'a str,
}

/// Renders the page of `game`.
///
/// # Arguments
/// * `game` - The game.
/// * `report` - Its results.
/// * `drafts` - Text to put back in the forms.
/// * `error` - Message explaining why the last submission was rejected.
pub(crate) fn render_game_page(
    game: &Game,
    report: &GameReport,
    drafts: &Drafts,
    error: Option<&str>,
) -> Result<String, askama::Error> {
    let stock_draft = if drafts.stock.is_empty() && !game.stock.lines.is_empty() {
        game.stock.to_sheet()
    } else {
        drafts.stock.clone()
    };
    GamePage {
        game,
        report,
        unmatched: game.stock.unmatched().collect(),
        error,
        stock_draft,
        player_draft: &drafts.player,
        url_draft: &drafts.url,
    }
    .render()
}

/// Helpers called by the template, which resolves bare calls as methods.
impl GamePage<'_> {
    /// "Cyrille ×4, Manu ×1".
    fn shares(&self, shares: &[Share]) -> String {
        shares
            .iter()
            .map(|share| format!("{} ×{}", share.who, share.count))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn fleet_error(&self, err: &FleetLoadError) -> &'static str {
        fleet_error(err)
    }
}

/// French explanation of why a fleet of the game cannot be read.
pub(crate) fn fleet_error(err: &FleetLoadError) -> &'static str {
    match err {
        FleetLoadError::Url(_) => {
            "Ce lien n'est pas une flotte de https://armada.ryankingston.com/. Copiez le lien \
             complet de la flotte depuis la barre d'adresse."
        }
        FleetLoadError::Fetch(err) if err.status() == Some(reqwest::StatusCode::NOT_FOUND) => {
            "Cette flotte n'existe pas. Vérifiez le lien."
        }
        FleetLoadError::Fetch(_) => {
            "Impossible de récupérer la flotte sur https://armada.ryankingston.com/. Réessayez \
             dans quelques instants."
        }
        FleetLoadError::Parse(FleetParseError::NoFleetData) => {
            "Aucune flotte trouvée à cette adresse. Vérifiez que la flotte a bien été sauvegardée."
        }
        FleetLoadError::Parse(_) => "Les données de cette flotte sont illisibles.",
    }
}

/// Status and French message for an error caused by what was typed in a game form, `None`
/// for any other error.
pub(crate) fn form_error(err: &GameError) -> Option<(StatusCode, String)> {
    let invalid = |message: String| Some((StatusCode::UNPROCESSABLE_ENTITY, message));
    match err {
        GameError::Stock(err) => invalid(stock_error(err)),
        GameError::InvalidName => invalid(format!(
            "Indiquez un nom de joueur ({MAX_NAME_CHARS} caractères au plus)."
        )),
        GameError::TooManyFleets => invalid(format!(
            "Une partie ne peut pas avoir plus de {MAX_FLEETS} flottes."
        )),
        GameError::Catalogue(_) => Some((
            StatusCode::BAD_GATEWAY,
            "Impossible de récupérer la liste des cartes sur https://armada.ryankingston.com/. \
             Réessayez dans quelques instants."
                .to_owned(),
        )),
        GameError::Fleet(err) => {
            let status = match err {
                FleetLoadError::Fetch(fetch) if fetch.status() != Some(StatusCode::NOT_FOUND) => {
                    StatusCode::BAD_GATEWAY
                }
                FleetLoadError::Parse(FleetParseError::Json(_) | FleetParseError::Quantity(..)) => {
                    StatusCode::BAD_GATEWAY
                }
                _ => StatusCode::UNPROCESSABLE_ENTITY,
            };
            Some((status, fleet_error(err).to_owned()))
        }
        GameError::Store(_) | GameError::FleetNotFound => None,
    }
}

fn stock_error(err: &StockParseError) -> String {
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
