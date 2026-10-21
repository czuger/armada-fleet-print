//! Routes and request handlers.

use axum::extract::State;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::Deserialize;

use crate::fleet_printer::FleetPrinter;
use crate::fleet_url::FleetUrl;
use crate::web::pages::INDEX;
use crate::web::web_error::WebError;

/// Builds the application: `GET /` shows the form, `POST /print` returns the fleet PDF.
///
/// # Arguments
/// * `printer` - Printer shared by all requests.
pub fn router(printer: FleetPrinter) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/print", post(print))
        .with_state(printer)
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
