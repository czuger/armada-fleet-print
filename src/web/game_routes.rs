//! Handlers of the game pages.
//!
//! The game page lives at `games/<id>/`, with a trailing slash, so its forms and redirects
//! can use paths relative to it and keep working under any URL prefix.

use axum::Form;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::Deserialize;

use crate::error_chain::error_chain;
use crate::game::{GameError, GameId, GameService};
use crate::web::game_page::{Drafts, form_error, render_game_page};
use crate::web::web_error::WebError;

#[derive(Debug, Deserialize)]
pub(crate) struct CreateForm {
    #[serde(default)]
    name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct StockForm {
    #[serde(default)]
    text: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FleetForm {
    #[serde(default)]
    player: String,
    #[serde(default)]
    url: String,
}

/// `POST games`: creates a game and opens its page.
pub(crate) async fn create_game(
    State(games): State<GameService>,
    Form(form): Form<CreateForm>,
) -> Result<Redirect, WebError> {
    let id = games.create(&form.name).await?;
    tracing::info!(game = %id, "game created");
    Ok(Redirect::to(&format!("games/{id}/")))
}

/// `GET games/<id>`: adds the trailing slash the page needs.
pub(crate) async fn add_trailing_slash(Path(id): Path<String>) -> Result<Redirect, WebError> {
    let id: GameId = id.parse()?;
    Ok(Redirect::permanent(&format!("{id}/")))
}

/// `GET games/<id>/`: shows the game with results computed from the current fleets.
pub(crate) async fn show_game(
    State(games): State<GameService>,
    Path(id): Path<String>,
) -> Result<Html<String>, WebError> {
    let id: GameId = id.parse()?;
    game_page(&games, &id, &Drafts::default(), None).await
}

/// `POST games/<id>/stock`: replaces the stock.
pub(crate) async fn replace_stock(
    State(games): State<GameService>,
    Path(id): Path<String>,
    Form(form): Form<StockForm>,
) -> Result<Response, WebError> {
    let id: GameId = id.parse()?;
    match games.replace_stock(&id, &form.text).await {
        Ok(()) => Ok(Redirect::to("./").into_response()),
        Err(err) => {
            let drafts = Drafts {
                stock: form.text,
                ..Drafts::default()
            };
            rejected(&games, &id, err, &drafts).await
        }
    }
}

/// `POST games/<id>/fleets`: adds a fleet.
pub(crate) async fn add_fleet(
    State(games): State<GameService>,
    Path(id): Path<String>,
    Form(form): Form<FleetForm>,
) -> Result<Response, WebError> {
    let id: GameId = id.parse()?;
    match games.add_fleet(&id, &form.player, &form.url).await {
        Ok(()) => Ok(Redirect::to("./").into_response()),
        Err(err) => {
            let drafts = Drafts {
                player: form.player,
                url: form.url,
                ..Drafts::default()
            };
            rejected(&games, &id, err, &drafts).await
        }
    }
}

/// `POST games/<id>/fleets/<fleet>/delete`: removes a fleet. Removing it twice, as a
/// double click does, is not an error.
pub(crate) async fn remove_fleet(
    State(games): State<GameService>,
    Path((id, fleet_id)): Path<(String, u32)>,
) -> Result<Redirect, WebError> {
    let id: GameId = id.parse()?;
    match games.remove_fleet(&id, fleet_id).await {
        Ok(()) | Err(GameError::FleetNotFound) => Ok(Redirect::to("../../")),
        Err(err) => Err(err.into()),
    }
}

/// Shows the game page again with the rejected input and why it was rejected, or fails
/// with `err` when it is not about the input.
async fn rejected(
    games: &GameService,
    id: &GameId,
    err: GameError,
    drafts: &Drafts,
) -> Result<Response, WebError> {
    let Some((status, message)) = form_error(&err) else {
        return Err(err.into());
    };
    log_rejection(status, &err);
    let page = game_page(games, id, drafts, Some(&message)).await?;
    Ok((status, page).into_response())
}

fn log_rejection(status: StatusCode, err: &GameError) {
    if status.is_server_error() {
        tracing::error!("{}", error_chain(err));
    } else {
        tracing::warn!("{}", error_chain(err));
    }
}

async fn game_page(
    games: &GameService,
    id: &GameId,
    drafts: &Drafts,
    error: Option<&str>,
) -> Result<Html<String>, WebError> {
    let game = games.load(id).await?;
    let report = games.report(&game).await;
    Ok(Html(render_game_page(&game, &report, drafts, error)?))
}
