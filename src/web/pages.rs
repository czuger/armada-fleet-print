//! Home and error pages.

use askama::Template;

use crate::lang::Lang;
use crate::layout::Orientation;
use crate::web::text::{ErrorMessage, IndexText, PageText};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexPage {
    lang: Lang,
    page: &'static PageText,
    t: &'static IndexText,
    landscape: bool,
}

/// Helpers called by the template, which resolves bare calls as methods.
impl IndexPage {
    /// Link of the language switch, relative to the app root.
    fn switch_link(&self, to: Lang) -> String {
        format!("lang?to={to}")
    }
}

/// Home page in `lang` with the fleet link form, `orientation` being the checked choice.
pub(crate) fn index_page(lang: Lang, orientation: Orientation) -> Result<String, askama::Error> {
    IndexPage {
        lang,
        page: PageText::of(lang),
        t: IndexText::of(lang),
        landscape: orientation == Orientation::Landscape,
    }
    .render()
}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorPage {
    lang: Lang,
    page: &'static PageText,
    message: &'static str,
}

/// Error page showing `message` in `lang`.
pub(crate) fn error_page(message: ErrorMessage, lang: Lang) -> Result<String, askama::Error> {
    ErrorPage {
        lang,
        page: PageText::of(lang),
        message: message.text(lang),
    }
    .render()
}
