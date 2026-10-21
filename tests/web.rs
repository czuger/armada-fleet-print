use armada_fleet_print::fleet_printer::FleetPrinter;
use armada_fleet_print::web::router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use tower::ServiceExt;

fn app() -> axum::Router {
    let cards_dir = std::env::temp_dir().join("armada-fleet-print-tests");
    router(FleetPrinter::new(cards_dir).unwrap())
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
