//! Routes and request handlers.

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE, SET_COOKIE};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Extension, Form, Router, middleware};
use serde::Deserialize;

use crate::fleet_printer::FleetPrinter;
use crate::fleet_url::FleetUrl;
use crate::game::GameId;
use crate::lang::Lang;
use crate::layout::Orientation;
use crate::web::app_state::AppState;
use crate::web::game_routes::{
    add_fleet, add_trailing_slash, create_game, remove_fleet, replace_fleet, replace_stock,
    show_game,
};
use crate::web::lang_cookie;
use crate::web::localize::localize;
use crate::web::orientation_cookie::{self, saved_orientation};
use crate::web::pages::index_page;
use crate::web::web_error::WebError;

/// Builds the application: `GET /` shows the form, `POST /print` returns the fleet PDF,
/// `GET /lang` switches the language, and `/games` holds the multi-player games. Every page
/// is in the language of the request (see `localize`). `GET /health` answers `OK` for the
/// deployment health check; it is added after the `localize` layer, so it skips it.
///
/// # Arguments
/// * `state` - Services shared by all requests.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/print", post(print))
        .route("/lang", get(switch_lang))
        .route("/games", post(create_game))
        .route("/games/{id}", get(add_trailing_slash))
        .route("/games/{id}/", get(show_game))
        .route("/games/{id}/stock", post(replace_stock))
        .route("/games/{id}/fleets", post(add_fleet))
        .route("/games/{id}/replace", post(replace_fleet))
        .route("/games/{id}/fleets/{fleet}/delete", post(remove_fleet))
        .layer(middleware::from_fn(localize))
        .route("/health", get(health))
        .with_state(state)
}

/// Reads nothing, so it answers as soon as the server accepts requests.
async fn health() -> &'static str {
    "OK"
}

#[derive(Debug, Deserialize)]
struct PrintForm {
    #[serde(default)]
    url: String,
    /// Chosen on the home page; the game pages leave it out and use the saved choice.
    orientation: Option<Orientation>,
}

async fn index(
    Extension(lang): Extension<Lang>,
    headers: HeaderMap,
) -> Result<Html<String>, WebError> {
    let orientation = saved_orientation(&headers).unwrap_or_default();
    Ok(Html(index_page(lang, orientation)?))
}

#[derive(Debug, Deserialize)]
struct SwitchQuery {
    /// Language to switch to.
    #[serde(default)]
    to: String,
    /// Game whose page the switch was clicked on.
    #[serde(default)]
    game: String,
}

/// Saves the chosen language and goes back to the page the switch was on: the game named by
/// `game`, else the home page. Only a valid game id is followed, so the redirect stays on
/// this site.
async fn switch_lang(Query(query): Query<SwitchQuery>) -> Response {
    let back = match query.game.parse::<GameId>() {
        Ok(id) => Redirect::to(&format!("games/{id}/")),
        Err(_) => Redirect::to("./"),
    };
    match query.to.parse::<Lang>() {
        Ok(lang) => ([(SET_COOKIE, lang_cookie::set_cookie(lang))], back).into_response(),
        Err(err) => {
            tracing::warn!("{err}");
            back.into_response()
        }
    }
}

/// Prints the fleet, and saves the orientation chosen in the form even when printing fails.
async fn print(
    State(printer): State<FleetPrinter>,
    Extension(lang): Extension<Lang>,
    headers: HeaderMap,
    Form(form): Form<PrintForm>,
) -> Response {
    let orientation = form
        .orientation
        .or_else(|| saved_orientation(&headers))
        .unwrap_or_default();
    let response = print_fleet(&printer, &form.url, orientation, lang).await;
    match form.orientation {
        Some(chosen) => {
            let cookie = orientation_cookie::set_cookie(chosen);
            ([(SET_COOKIE, cookie)], response).into_response()
        }
        None => response.into_response(),
    }
}

async fn print_fleet(
    printer: &FleetPrinter,
    url: &str,
    orientation: Orientation,
    lang: Lang,
) -> Result<Response, WebError> {
    let url = FleetUrl::parse(url)?;
    tracing::info!(%url, %orientation, %lang, "printing fleet");
    let pdf = printer.print(&url, orientation, lang).await?;
    let disposition = match lang {
        Lang::Fr => "inline; filename=\"flotte.pdf\"",
        Lang::En => "inline; filename=\"fleet.pdf\"",
    };
    let headers = [
        (CONTENT_TYPE, "application/pdf"),
        (CONTENT_DISPOSITION, disposition),
    ];
    Ok((headers, pdf).into_response())
}
