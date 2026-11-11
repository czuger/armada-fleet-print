//! Middleware giving every request its language and writing error pages in it.

use axum::body::Body;
use axum::extract::Request;
use axum::http::HeaderValue;
use axum::http::header::{CONTENT_LENGTH, CONTENT_TYPE};
use axum::middleware::Next;
use axum::response::Response;

use crate::web::lang_cookie::request_lang;
use crate::web::pages::error_page;
use crate::web::text::ErrorMessage;

/// Adds the request [`Lang`](crate::lang::Lang) to the request extensions, for handlers to
/// take with `Extension<Lang>`, and turns a response carrying an [`ErrorMessage`], as
/// [`WebError`](crate::web::WebError) answers, into the error page in that language.
pub(crate) async fn localize(mut request: Request, next: Next) -> Response {
    let lang = request_lang(request.headers());
    request.extensions_mut().insert(lang);
    let mut response = next.run(request).await;
    let Some(message) = response.extensions_mut().remove::<ErrorMessage>() else {
        return response;
    };
    let (body, content_type) = match error_page(message, lang) {
        Ok(page) => (page, "text/html; charset=utf-8"),
        Err(err) => {
            tracing::error!("rendering error page failed: {err}");
            (message.text(lang).to_owned(), "text/plain; charset=utf-8")
        }
    };
    // Keep the status and headers, such as a cookie set by the handler; replace the body.
    let (mut parts, _) = response.into_parts();
    parts.headers.remove(CONTENT_LENGTH);
    parts
        .headers
        .insert(CONTENT_TYPE, HeaderValue::from_static(content_type));
    Response::from_parts(parts, Body::from(body))
}
