//! Errors returned by request handlers, answered with a French error page.

use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

use crate::error_chain::error_chain;
use crate::fleet::FleetParseError;
use crate::fleet_printer::PrintError;
use crate::fleet_url::FleetUrlError;
use crate::web::pages::error_page;

/// Error answered to a web request.
#[derive(Debug, thiserror::Error)]
pub enum WebError {
    /// The submitted link is not a fleet builder link.
    #[error("invalid fleet link")]
    InvalidUrl(#[from] FleetUrlError),
    /// The fleet could not be printed.
    #[error("printing fleet failed")]
    Print(#[from] PrintError),
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
