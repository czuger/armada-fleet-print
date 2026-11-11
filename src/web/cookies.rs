//! Reading request cookies.

use axum::http::HeaderMap;
use axum::http::header::COOKIE;

/// Value of the cookie `name` in the request headers, if present and readable.
pub(crate) fn cookie_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(found, _)| *found == name)
        .map(|(_, value)| value)
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

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
        let headers = headers(&["a=1; lang=en; b=2", "c=3"]);
        assert_eq!(cookie_value(&headers, "lang"), Some("en"));
        assert_eq!(cookie_value(&headers, "c"), Some("3"));
    }

    #[test]
    fn needs_the_exact_name() {
        let headers = headers(&["xlang=en"]);
        assert_eq!(cookie_value(&headers, "lang"), None);
        assert_eq!(cookie_value(&HeaderMap::new(), "lang"), None);
    }
}
