//! Text shared by every page.

use crate::lang::Lang;

/// Text of the parts common to all pages.
#[derive(Debug)]
pub(crate) struct PageText {
    /// Heading of the home and error pages.
    pub heading: &'static str,
    /// Link back to the home page from the error page.
    pub back: &'static str,
    /// Hint shown in fleet link fields.
    pub fleet_url_placeholder: &'static str,
}

const FR: PageText = PageText {
    heading: "Impression de flotte Armada",
    back: "Retour",
    fleet_url_placeholder: "https://armada.ryankingston.com/fleet/<numéro>/",
};

const EN: PageText = PageText {
    heading: "Armada fleet printing",
    back: "Back",
    fleet_url_placeholder: "https://armada.ryankingston.com/fleet/<number>/",
};

impl PageText {
    /// The text in `lang`.
    pub(crate) fn of(lang: Lang) -> &'static Self {
        match lang {
            Lang::Fr => &FR,
            Lang::En => &EN,
        }
    }
}
