//! Errors returned by request handlers, answered with a French error page.

use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

use crate::error_chain::error_chain;
use crate::fleet::FleetParseError;
use crate::fleet_printer::PrintError;
use crate::fleet_url::FleetUrlError;
use crate::game::{GameError, GameIdError, GameStoreError};
use crate::web::pages::error_page;

/// Asked for when a game is created without a usable name; states `MAX_NAME_CHARS`.
const GAME_NAME_MESSAGE: &str = "Donnez un nom à la partie (100 caractères au plus).";

/// Error answered to a web request.
#[derive(Debug, thiserror::Error)]
pub enum WebError {
    /// The submitted link is not a fleet builder link.
    #[error("invalid fleet link")]
    InvalidUrl(#[from] FleetUrlError),
    /// The fleet could not be printed.
    #[error("printing fleet failed")]
    Print(#[from] PrintError),
    /// The link names no game.
    #[error("invalid game link")]
    InvalidGameId(#[from] GameIdError),
    /// A game operation failed.
    #[error("game operation failed")]
    Game(#[from] GameError),
    /// A page could not be rendered.
    #[error("rendering page failed")]
    Template(#[from] askama::Error),
}

impl WebError {
    fn status(&self) -> StatusCode {
        match self {
            Self::InvalidUrl(_) => StatusCode::BAD_REQUEST,
            Self::Print(PrintError::FetchPage(err))
                if err.status() == Some(StatusCode::NOT_FOUND) =>
            {
                StatusCode::NOT_FOUND
            }
            Self::Print(PrintError::Parse(FleetParseError::NoFleetData)) => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            Self::Print(PrintError::FetchPage(_) | PrintError::Parse(_) | PrintError::Cards(_)) => {
                StatusCode::BAD_GATEWAY
            }
            Self::Print(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidGameId(_) | Self::Game(GameError::Store(GameStoreError::NotFound)) => {
                StatusCode::NOT_FOUND
            }
            Self::Game(GameError::InvalidName) => StatusCode::BAD_REQUEST,
            Self::Game(_) | Self::Template(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn user_message(&self) -> &'static str {
        match self {
            Self::InvalidUrl(_) => {
                "Ce lien n'est pas une flotte de https://armada.ryankingston.com/. Copiez le lien \
                 complet de la flotte depuis la barre d'adresse."
            }
            Self::Print(PrintError::FetchPage(err))
                if err.status() == Some(StatusCode::NOT_FOUND) =>
            {
                "Cette flotte n'existe pas. Vérifiez le lien."
            }
            Self::Print(PrintError::FetchPage(_)) => {
                "Impossible de récupérer la flotte sur https://armada.ryankingston.com/. Réessayez \
                 dans quelques instants."
            }
            Self::Print(PrintError::Parse(FleetParseError::NoFleetData)) => {
                "Aucune flotte trouvée à cette adresse. Vérifiez que la flotte a bien été sauvegardée."
            }
            Self::Print(PrintError::Parse(_)) => "Les données de cette flotte sont illisibles.",
            Self::Print(PrintError::Cards(_)) => {
                "Impossible de télécharger les images des cartes. Réessayez dans quelques instants."
            }
            Self::Print(_) => "Erreur interne lors de la création du PDF.",
            Self::InvalidGameId(_) | Self::Game(GameError::Store(GameStoreError::NotFound)) => {
                "Cette partie n'existe pas. Vérifiez le lien."
            }
            Self::Game(GameError::InvalidName) => GAME_NAME_MESSAGE,
            Self::Game(_) | Self::Template(_) => "Erreur interne lors du traitement de la partie.",
        }
    }
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let status = self.status();
        if status.is_server_error() {
            tracing::error!("{}", error_chain(&self));
        } else {
            tracing::warn!("{}", error_chain(&self));
        }
        (status, Html(error_page(self.user_message()))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::MAX_NAME_CHARS;

    #[test]
    fn game_name_message_states_the_limit() {
        assert!(GAME_NAME_MESSAGE.contains(&MAX_NAME_CHARS.to_string()));
    }
}
