use std::sync::atomic::{AtomicU32, Ordering};

use armada_fleet_print::fleet_printer::FleetPrinter;
use armada_fleet_print::game::{CatalogueSource, GameService, ModelCatalogue};
use armada_fleet_print::http_client::build_http_client;
use armada_fleet_print::web::{AppState, router};
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use tower::ServiceExt;

const CARDS_JS: &str = include_str!("fixtures/builder_cards.js");

/// An app saving games in a directory of its own, with the card list fixture.
fn app() -> axum::Router {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let cards_dir = std::env::temp_dir().join("armada-fleet-print-tests");
    let games_dir = std::env::temp_dir().join(format!(
        "armada-fleet-print-web-games-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let catalogue = CatalogueSource::fixed(ModelCatalogue::parse_cards_js(CARDS_JS).unwrap());
    router(AppState {
        printer: FleetPrinter::new(cards_dir).unwrap(),
        games: GameService::with_catalogue(games_dir, build_http_client().unwrap(), catalogue),
    })
}

fn form(path: &str, body: &str) -> Request<Body> {
    Request::post(path)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body.to_owned()))
        .unwrap()
}

fn location(response: &axum::response::Response) -> &str {
    response.headers()[header::LOCATION].to_str().unwrap()
}

/// Creates a game and returns its page path, `/games/<id>/`.
async fn create_game(app: &axum::Router) -> String {
    let response = app
        .clone()
        .oneshot(form("/games", "name=Campagne"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let target = location(&response);
    assert!(
        target.starts_with("games/") && target.ends_with('/'),
        "{target}"
    );
    format!("/{target}")
}

async fn body_text(response: axum::response::Response) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test]
async fn home_page_shows_the_form() {
    let request = Request::get("/").body(Body::empty()).unwrap();
    let response = app().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains("Impression de flotte Armada"));
    assert!(body.contains(r#"<form action="print" method="post">"#));
}

#[tokio::test]
async fn print_rejects_links_to_other_sites() {
    let request = Request::post("/print")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from("url=https%3A%2F%2Fexample.com%2Ffleet%2F1%2F"))
        .unwrap();
    let response = app().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(
        body_text(response)
            .await
            .contains("Ce lien n'est pas une flotte")
    );
}

#[tokio::test]
async fn home_page_offers_to_create_a_game() {
    let request = Request::get("/").body(Body::empty()).unwrap();
    let body = body_text(app().oneshot(request).await.unwrap()).await;
    assert!(body.contains(r#"<form action="games" method="post">"#));
}

#[tokio::test]
async fn new_game_page_is_empty() {
    let app = app();
    let page = create_game(&app).await;
    let request = Request::get(&page).body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains("<h1>Campagne</h1>"));
    assert!(body.contains("Aucune flotte pour l'instant."));
}

#[tokio::test]
async fn game_link_without_slash_redirects() {
    let app = app();
    let page = create_game(&app).await;
    let bare = page.trim_end_matches('/');
    let request = Request::get(bare).body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
    let id = bare.rsplit('/').next().unwrap();
    assert_eq!(location(&response), format!("{id}/"));
}

#[tokio::test]
async fn unknown_or_malformed_games_are_not_found() {
    for path in [
        "/games/0123456789abcdef0123456789abcdef/",
        "/games/..%2F..%2Fetc/",
    ] {
        let request = Request::get(path).body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
        assert!(
            body_text(response)
                .await
                .contains("Cette partie n'existe pas")
        );
    }
}

#[tokio::test]
async fn game_without_name_is_rejected() {
    let response = app().oneshot(form("/games", "name=++")).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn stock_is_saved_and_shown() {
    let app = app();
    let page = create_game(&app).await;
    let stock = "Owner%09Faction%09Figurine+Name%09Size%09Type%09Qty%0A\
                 Manu%09Empire%09Tie+Fighter%09-%09Squadron%0916%0A\
                 Manu%09Empire%09Millennium+Pigeon%09-%09Squadron%091%0A";
    let response = app
        .clone()
        .oneshot(form(&format!("{page}stock"), &format!("text={stock}")))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(location(&response), "./");

    let request = Request::get(&page).body(Body::empty()).unwrap();
    let body = body_text(app.oneshot(request).await.unwrap()).await;
    assert!(body.contains("Tie Fighter"));
    assert!(body.contains("Stock : Manu ×16"));
    assert!(body.contains("Millennium Pigeon (Manu, Empire, Squadron)"));
}

#[tokio::test]
async fn invalid_stock_is_explained_and_kept() {
    let app = app();
    let page = create_game(&app).await;
    let stock = "Manu%09Empire%09Tie+%3CFighter%3E%09-%09Squadron%09seize%0A";
    let response = app
        .oneshot(form(&format!("{page}stock"), &format!("text={stock}")))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = body_text(response).await;
    assert!(body.contains("Ligne 1 : quantité « seize » invalide."));
    // The paste is shown again, escaped.
    assert!(body.contains("Tie &#60;Fighter&#62;") || body.contains("Tie &lt;Fighter&gt;"));
}

#[tokio::test]
async fn fleet_from_another_site_is_rejected() {
    let app = app();
    let page = create_game(&app).await;
    let body = "player=Alice&url=https%3A%2F%2Fexample.com%2Ffleet%2F1%2F";
    let response = app
        .oneshot(form(&format!("{page}fleets"), body))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = body_text(response).await;
    assert!(body.contains("Ce lien n&#39;est pas une flotte"));
    assert!(body.contains(r#"value="Alice""#));
}
