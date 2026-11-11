//! Errors returned by request handlers, answered with an error page in the request language.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::error_chain::error_chain;
use crate::fleet::FleetParseError;
use crate::fleet_printer::PrintError;
use crate::fleet_url::FleetUrlError;
use crate::game::{GameError, GameIdError, GameStoreError};
use crate::web::text::ErrorMessage;

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

    fn message(&self) -> ErrorMessage {
        match self {
            Self::InvalidUrl(_) => ErrorMessage::InvalidFleetLink,
            Self::Print(PrintError::FetchPage(err))
                if err.status() == Some(StatusCode::NOT_FOUND) =>
            {
                ErrorMessage::FleetNotFound
            }
            Self::Print(PrintError::FetchPage(_)) => ErrorMessage::FleetUnreachable,
            Self::Print(PrintError::Parse(FleetParseError::NoFleetData)) => {
                ErrorMessage::NoFleetData
            }
            Self::Print(PrintError::Parse(_)) => ErrorMessage::UnreadableFleet,
            Self::Print(PrintError::Cards(_)) => ErrorMessage::CardsUnavailable,
            Self::Print(_) => ErrorMessage::PdfFailed,
            Self::InvalidGameId(_) | Self::Game(GameError::Store(GameStoreError::NotFound)) => {
                ErrorMessage::GameNotFound
            }
            Self::Game(GameError::InvalidName) => ErrorMessage::GameName,
            Self::Game(_) => ErrorMessage::GameFailed,
            Self::Template(_) => ErrorMessage::Internal,
        }
    }
}

impl IntoResponse for WebError {
    /// Logs the error and answers its status, leaving the page to the `localize` middleware,
    /// which knows the language: the response carries the [`ErrorMessage`] to show.
    fn into_response(self) -> Response {
        let status = self.status();
        if status.is_server_error() {
            tracing::error!("{}", error_chain(&self));
        } else {
            tracing::warn!("{}", error_chain(&self));
        }
        let mut response = status.into_response();
        response.extensions_mut().insert(self.message());
        response
    }
}
