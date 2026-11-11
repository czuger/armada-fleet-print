//! Cookie remembering the page orientation last chosen on the home page.
//!
//! The cookie has no `Path`, so the browser scopes it to the directory of `print`: the app
//! root, whatever URL prefix it is served under. The game pages, below that root, get it too.

use axum::http::HeaderMap;

use crate::layout::Orientation;
use crate::web::cookies::cookie_value;

const NAME: &str = "orientation";
/// One year, in seconds.
const MAX_AGE: u32 = 365 * 24 * 60 * 60;

/// Orientation saved in the request cookies, if any is valid.
///
/// An unreadable or unknown value counts as no choice, so the page falls back to portrait.
pub(crate) fn saved_orientation(headers: &HeaderMap) -> Option<Orientation> {
    cookie_value(headers, NAME)?.parse().ok()
}

/// `Set-Cookie` value saving `orientation`.
pub(crate) fn set_cookie(orientation: Orientation) -> String {
    format!("{NAME}={orientation}; Max-Age={MAX_AGE}; SameSite=Lax; HttpOnly")
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;
    use axum::http::header::COOKIE;

    use super::*;

    fn headers(cookies: &[&str]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for cookie in cookies {
            headers.append(COOKIE, HeaderValue::from_str(cookie).unwrap());
        }
        headers
    }

    #[test]
    fn finds_the_cookie_among_others() {
        let found = saved_orientation(&headers(&["a=1; orientation=landscape; b=2"]));
        assert_eq!(found, Some(Orientation::Landscape));
        let found = saved_orientation(&headers(&["a=1", "orientation=portrait"]));
        assert_eq!(found, Some(Orientation::Portrait));
    }

    #[test]
    fn ignores_missing_or_unknown_values() {
        assert_eq!(saved_orientation(&headers(&[])), None);
        assert_eq!(saved_orientation(&headers(&["orientation=sideways"])), None);
        assert_eq!(
            saved_orientation(&headers(&["xorientation=landscape"])),
            None
        );
    }

    #[test]
    fn saved_value_reads_back() {
        let cookie = set_cookie(Orientation::Landscape);
        let pair = cookie.split(';').next().unwrap();
        assert_eq!(
            saved_orientation(&headers(&[pair])),
            Some(Orientation::Landscape)
        );
    }
}
