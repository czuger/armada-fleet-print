use std::sync::atomic::{AtomicU32, Ordering};

use armada_fleet_print::fleet_printer::FleetPrinter;
use armada_fleet_print::game::{CatalogueSource, GameId, GameService, GameStore, ModelCatalogue};
use armada_fleet_print::http_client::build_http_client;
use armada_fleet_print::web::{AppState, router};
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use tower::ServiceExt;

const CARDS_JS: &str = include_str!("fixtures/builder_cards.js");

/// An app saving games in a directory of its own, with the card list fixture.
fn app() -> axum::Router {
    app_with_games().0
}

/// [`app`], with the directory its games are saved in.
fn app_with_games() -> (axum::Router, std::path::PathBuf) {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let cards_dir = std::env::temp_dir().join("armada-fleet-print-tests");
    let games_dir = std::env::temp_dir().join(format!(
        "armada-fleet-print-web-games-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let catalogue = CatalogueSource::fixed(ModelCatalogue::parse_cards_js(CARDS_JS).unwrap());
    let app = router(AppState {
        printer: FleetPrinter::new(cards_dir).unwrap(),
        games: GameService::with_catalogue(
            games_dir.clone(),
            build_http_client().unwrap(),
            catalogue,
        ),
    });
    (app, games_dir)
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
            .contains("Ce lien n&#39;est pas une flotte")
    );
}

#[tokio::test]
async fn home_page_checks_portrait_by_default() {
    let request = Request::get("/").body(Body::empty()).unwrap();
    let body = body_text(app().oneshot(request).await.unwrap()).await;
    assert!(body.contains(r#"value="portrait" checked>"#));
    assert!(body.contains(r#"value="landscape">"#));
}

#[tokio::test]
async fn home_page_checks_the_saved_orientation() {
    let request = Request::get("/")
        .header(header::COOKIE, "orientation=landscape")
        .body(Body::empty())
        .unwrap();
    let body = body_text(app().oneshot(request).await.unwrap()).await;
    assert!(body.contains(r#"value="portrait">"#));
    assert!(body.contains(r#"value="landscape" checked>"#));
}

#[tokio::test]
async fn print_saves_the_chosen_orientation() {
    let response = app()
        .oneshot(form(
            "/print",
            "url=https%3A%2F%2Fexample.com%2Ffleet%2F1%2F&orientation=landscape",
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let cookie = response.headers()[header::SET_COOKIE].to_str().unwrap();
    assert!(cookie.starts_with("orientation=landscape;"), "{cookie}");
    let content_type = response.headers()[header::CONTENT_TYPE].to_str().unwrap();
    assert!(content_type.starts_with("text/html"), "{content_type}");
}

#[tokio::test]
async fn print_without_orientation_keeps_the_cookie() {
    let response = app()
        .oneshot(form(
            "/print",
            "url=https%3A%2F%2Fexample.com%2Ffleet%2F1%2F",
        ))
        .await
        .unwrap();
    assert!(!response.headers().contains_key(header::SET_COOKIE));
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
    assert!(body.contains("Aucune flotte pour l&#39;instant."));
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
                .contains("Cette partie n&#39;existe pas")
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

fn get(path: &str, headers: &[(header::HeaderName, &str)]) -> Request<Body> {
    let mut request = Request::get(path);
    for (name, value) in headers {
        request = request.header(name, *value);
    }
    request.body(Body::empty()).unwrap()
}

#[tokio::test]
async fn home_page_follows_the_browser_language() {
    let request = get("/", &[(header::ACCEPT_LANGUAGE, "en-US,en;q=0.9,fr;q=0.8")]);
    let body = body_text(app().oneshot(request).await.unwrap()).await;
    assert!(body.contains(r#"<html lang="en">"#));
    assert!(body.contains("Armada fleet printing"));
    assert!(body.contains(r#"<a href="lang?to=fr""#));

    let body = body_text(app().oneshot(get("/", &[])).await.unwrap()).await;
    assert!(body.contains(r#"<html lang="fr">"#));
    assert!(body.contains(r#"<a href="lang?to=en""#));
}

#[tokio::test]
async fn saved_language_beats_the_browser() {
    let request = get(
        "/",
        &[(header::ACCEPT_LANGUAGE, "en"), (header::COOKIE, "lang=fr")],
    );
    let body = body_text(app().oneshot(request).await.unwrap()).await;
    assert!(body.contains("Impression de flotte Armada"));
}

#[tokio::test]
async fn language_switch_saves_the_choice_and_goes_back() {
    let app = app();
    let page = create_game(&app).await;
    let id = page.trim_start_matches("/games/").trim_end_matches('/');

    let response = app
        .clone()
        .oneshot(get(&format!("/lang?to=en&game={id}"), &[]))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(location(&response), format!("games/{id}/"));
    let cookie = response.headers()[header::SET_COOKIE].to_str().unwrap();
    assert!(cookie.starts_with("lang=en;"), "{cookie}");

    let response = app
        .oneshot(get("/lang?to=fr&game=https://evil.example/", &[]))
        .await
        .unwrap();
    assert_eq!(location(&response), "./");
}

#[tokio::test]
async fn unknown_language_changes_nothing() {
    let response = app().oneshot(get("/lang?to=de", &[])).await.unwrap();
    assert_eq!(location(&response), "./");
    assert!(!response.headers().contains_key(header::SET_COOKIE));
}

#[tokio::test]
async fn error_page_is_in_the_request_language() {
    let mut request = form("/print", "url=https%3A%2F%2Fexample.com%2Ffleet%2F1%2F");
    request
        .headers_mut()
        .insert(header::COOKIE, "lang=en".parse().unwrap());
    let response = app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_text(response).await;
    assert!(body.contains("This link is not a fleet"));
    assert!(body.contains(">Back</a>"));
}

#[tokio::test]
async fn game_page_in_english() {
    let app = app();
    let page = create_game(&app).await;
    let id = page.trim_start_matches("/games/").trim_end_matches('/');
    let response = app
        .clone()
        .oneshot(get(&page, &[(header::COOKIE, "lang=en")]))
        .await
        .unwrap();
    let body = body_text(response).await;
    assert!(body.contains("No fleet yet."));
    assert!(body.contains(&format!(r#"href="../../lang?to=fr&#38;game={id}""#)));

    let mut request = form(
        &format!("{page}stock"),
        "text=Manu%09Empire%09Tie%09Small%09Squadron%09seize",
    );
    request
        .headers_mut()
        .insert(header::COOKIE, "lang=en".parse().unwrap());
    let body = body_text(app.oneshot(request).await.unwrap()).await;
    assert!(body.contains("Line 1: invalid quantity “seize”."), "{body}");
}

/// Creates a game holding one fleet of Alice, saved without downloading it, and returns
/// the game page path and the fleet id.
async fn game_with_fleet() -> (axum::Router, String, u32) {
    let (app, games_dir) = app_with_games();
    let page = create_game(&app).await;
    let id: GameId = page
        .trim_start_matches("/games/")
        .trim_end_matches('/')
        .parse()
        .unwrap();
    let fleet = GameStore::new(games_dir)
        .update(&id, |game| {
            game.add_fleet(
                "Alice".to_owned(),
                "https://armada.ryankingston.com/fleet/1/".to_owned(),
            )
        })
        .await
        .unwrap();
    (app, page, fleet)
}

#[tokio::test]
async fn replace_button_opens_the_filled_form() {
    let (app, page, fleet) = game_with_fleet().await;
    let body = body_text(app.clone().oneshot(get(&page, &[])).await.unwrap()).await;
    assert!(body.contains(&format!(
        r#"href="./?replace={fleet}#fleet-form">Remplacer</a>"#
    )));
    assert!(body.contains(r#"<form id="fleet-form" action="fleets""#));

    let replace = format!("{page}?replace={fleet}");
    let body = body_text(app.oneshot(get(&replace, &[])).await.unwrap()).await;
    assert!(body.contains(r#"<form id="fleet-form" action="replace""#));
    assert!(body.contains(&format!(r#"name="fleet" value="{fleet}""#)));
    assert!(body.contains(r#"name="player" value="Alice""#));
    assert!(body.contains(r#"value="https://armada.ryankingston.com/fleet/1/""#));
    assert!(body.contains(r#"class="table-active""#));
}

#[tokio::test]
async fn unknown_fleet_to_replace_shows_the_add_form() {
    let (app, page, _) = game_with_fleet().await;
    let body = body_text(
        app.oneshot(get(&format!("{page}?replace=99"), &[]))
            .await
            .unwrap(),
    )
    .await;
    assert!(body.contains(r#"<form id="fleet-form" action="fleets""#));
}

#[tokio::test]
async fn rejected_replacement_keeps_the_form_and_typed_text() {
    let (app, page, fleet) = game_with_fleet().await;
    let body = format!("fleet={fleet}&player=Bob&url=https%3A%2F%2Fexample.com%2Ffleet%2F2%2F");
    let response = app
        .oneshot(form(&format!("{page}replace"), &body))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = body_text(response).await;
    assert!(body.contains("Ce lien n&#39;est pas une flotte"));
    assert!(body.contains(r#"<form id="fleet-form" action="replace""#));
    assert!(body.contains(r#"name="player" value="Bob""#));
    assert!(body.contains(r#"value="https://example.com/fleet/2/""#));
}

#[tokio::test]
async fn replacing_a_removed_fleet_is_explained() {
    let (app, page, _) = game_with_fleet().await;
    let body = "fleet=99&player=Bob&url=https%3A%2F%2Farmada.ryankingston.com%2Ffleet%2F2%2F";
    let response = app
        .oneshot(form(&format!("{page}replace"), body))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = body_text(response).await;
    assert!(body.contains("retirée de la partie entre-temps"));
    assert!(body.contains(r#"<form id="fleet-form" action="fleets""#));
    assert!(body.contains(r#"name="player" value="Bob""#));
}

#[tokio::test]
async fn health_answers_ok_without_cookie() {
    let request = Request::get("/health").body(Body::empty()).unwrap();
    let response = app().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_text(response).await, "OK");
}
