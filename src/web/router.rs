//! Routes and request handlers.

use axum::extract::State;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::Deserialize;

use crate::fleet_printer::FleetPrinter;
use crate::fleet_url::FleetUrl;
use crate::web::app_state::AppState;
use crate::web::game_routes::{
    add_fleet, add_trailing_slash, create_game, remove_fleet, replace_stock, show_game,
};
use crate::web::pages::INDEX;
use crate::web::web_error::WebError;

/// Builds the application: `GET /` shows the form, `POST /print` returns the fleet PDF,
/// and `/games` holds the multi-player games.
///
/// # Arguments
/// * `state` - Services shared by all requests.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/print", post(print))
        .route("/games", post(create_game))
        .route("/games/{id}", get(add_trailing_slash))
        .route("/games/{id}/", get(show_game))
        .route("/games/{id}/stock", post(replace_stock))
        .route("/games/{id}/fleets", post(add_fleet))
        .route("/games/{id}/fleets/{fleet}/delete", post(remove_fleet))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct PrintForm {
    #[serde(default)]
    url: String,
}

async fn index() -> Html<&'static str> {
    Html(INDEX)
}

async fn print(
    State(printer): State<FleetPrinter>,
    Form(form): Form<PrintForm>,
) -> Result<Response, WebError> {
    let url = FleetUrl::parse(&form.url)?;
    tracing::info!(%url, "printing fleet");
    let pdf = printer.print(&url).await?;
    let headers = [
        (CONTENT_TYPE, "application/pdf"),
        (CONTENT_DISPOSITION, "inline; filename=\"flotte.pdf\""),
    ];
    Ok((headers, pdf).into_response())
}
