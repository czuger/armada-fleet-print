//! Language of a request: the `lang` cookie set by the language switch, else the browser's
//! preference.
//!
//! The cookie is set by `GET lang` at the app root and has no `Path`, so the browser scopes it
//! to the app root, whatever prefix nginx serves it under, game pages included.

use axum::http::HeaderMap;
use axum::http::header::ACCEPT_LANGUAGE;

use crate::lang::Lang;
use crate::web::cookies::cookie_value;

const NAME: &str = "lang";
/// One year, in seconds.
const MAX_AGE: u32 = 365 * 24 * 60 * 60;

/// Language to answer the request in: the saved choice, else the first supported language
/// of `Accept-Language`, else French.
pub(crate) fn request_lang(headers: &HeaderMap) -> Lang {
    saved_lang(headers)
        .or_else(|| {
            let accepted = headers.get(ACCEPT_LANGUAGE)?.to_str().ok()?;
            preferred_lang(accepted)
        })
        .unwrap_or_default()
}

/// Language saved in the request cookies. An unknown value counts as no choice.
fn saved_lang(headers: &HeaderMap) -> Option<Lang> {
    cookie_value(headers, NAME)?.parse().ok()
}

/// `Set-Cookie` value saving `lang`.
pub(crate) fn set_cookie(lang: Lang) -> String {
    format!("{NAME}={lang}; Max-Age={MAX_AGE}; SameSite=Lax; HttpOnly")
}

/// Supported language with the highest weight in an `Accept-Language` value such as
/// `de-CH, en;q=0.8, fr;q=0.5`. Ties keep the header order; `q=0` means "not this one".
fn preferred_lang(accepted: &str) -> Option<Lang> {
    let mut candidates: Vec<(f32, Lang)> = accepted
        .split(',')
        .filter_map(|entry| {
            let mut parts = entry.split(';');
            let tag = parts.next()?.trim();
            let primary = tag.split('-').next()?.to_ascii_lowercase();
            let lang = primary.parse().ok()?;
            Some((weight(parts), lang))
        })
        .filter(|(weight, _)| *weight > 0.0)
        .collect();
    // Stable sort, so equal weights keep the header order.
    candidates.sort_by(|a, b| b.0.total_cmp(&a.0));
    candidates.first().map(|(_, lang)| *lang)
}

/// `q` parameter among the parameters of an `Accept-Language` entry, 1 when absent or
/// unreadable.
fn weight<'a>(params: impl Iterator<Item = &'a str>) -> f32 {
    params
        .filter_map(|param| param.trim().strip_prefix("q="))
        .find_map(|value| value.trim().parse::<f32>().ok())
        .unwrap_or(1.0)
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;
    use axum::http::header::COOKIE;

    use super::*;

    #[test]
    fn reads_regional_tags() {
        assert_eq!(preferred_lang("en-US,en;q=0.9"), Some(Lang::En));
        assert_eq!(preferred_lang("fr-CH, fr;q=0.9, en;q=0.8"), Some(Lang::Fr));
        assert_eq!(preferred_lang("EN-gb"), Some(Lang::En));
    }

    #[test]
    fn follows_the_weights() {
        assert_eq!(preferred_lang("fr;q=0.5, en;q=0.8"), Some(Lang::En));
        assert_eq!(preferred_lang("de, en;q=0.7, fr;q=0.9"), Some(Lang::Fr));
        assert_eq!(preferred_lang("en;q=0, fr;q=0.1"), Some(Lang::Fr));
    }

    #[test]
    fn ignores_unsupported_languages() {
        assert_eq!(preferred_lang("de-DE, it;q=0.8, *;q=0.5"), None);
        assert_eq!(preferred_lang(""), None);
        assert_eq!(preferred_lang(";;,"), None);
    }

    fn headers(pairs: &[(axum::http::HeaderName, &str)]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (name, value) in pairs {
            headers.append(name, HeaderValue::from_str(value).unwrap());
        }
        headers
    }

    #[test]
    fn saved_choice_beats_the_browser() {
        let headers = headers(&[(COOKIE, "lang=fr"), (ACCEPT_LANGUAGE, "en")]);
        assert_eq!(request_lang(&headers), Lang::Fr);
    }

    #[test]
    fn falls_back_to_the_browser_then_french() {
        let english = headers(&[(COOKIE, "lang=xx"), (ACCEPT_LANGUAGE, "en")]);
        assert_eq!(request_lang(&english), Lang::En);
        assert_eq!(request_lang(&HeaderMap::new()), Lang::Fr);
        let german = headers(&[(ACCEPT_LANGUAGE, "de")]);
        assert_eq!(request_lang(&german), Lang::Fr);
    }

    #[test]
    fn saved_value_reads_back() {
        let cookie = set_cookie(Lang::En);
        let pair = cookie.split(';').next().unwrap();
        assert_eq!(saved_lang(&headers(&[(COOKIE, pair)])), Some(Lang::En));
    }
}
