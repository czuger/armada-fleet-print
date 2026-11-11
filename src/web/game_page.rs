//! Page of a game: results, fleets and stock.

use askama::Template;

use crate::game::{
    FigureKind, FleetLoadError, Game, GameFleet, GameId, GameReport, Share, StockLine,
};
use crate::lang::Lang;
use crate::web::game_form_error::fleet_error;
use crate::web::text::{GameText, PageText};

/// Text typed in the page forms, shown again when a submission is rejected.
#[derive(Debug, Default)]
pub(crate) struct Drafts {
    /// Pasted stock; when empty, the saved stock is shown instead.
    pub stock: String,
    /// Player of the fleet to add.
    pub player: String,
    /// Link of the fleet to add.
    pub url: String,
    /// Fleet the form replaces instead of adding one. When `player` and `url` are empty,
    /// the form starts from that fleet.
    pub replacing: Option<u32>,
}

#[derive(Template)]
#[template(path = "game.html")]
struct GamePage<'a> {
    lang: Lang,
    page: &'static PageText,
    t: &'static GameText,
    id: &'a GameId,
    game: &'a Game,
    report: &'a GameReport,
    unmatched: Vec<&'a StockLine>,
    error: Option<&'a str>,
    stock_draft: String,
    /// Fleet being replaced, if it is still in the game.
    replacing: Option<&'a GameFleet>,
    player_draft: &'a str,
    url_draft: &'a str,
}

/// Renders the page of `game` in `lang`.
///
/// # Arguments
/// * `id` - Identifier of the game, for the language switch.
/// * `game` - The game.
/// * `report` - Its results.
/// * `drafts` - Text to put back in the forms.
/// * `error` - Message explaining why the last submission was rejected.
/// * `lang` - Language of the page.
pub(crate) fn render_game_page(
    id: &GameId,
    game: &Game,
    report: &GameReport,
    drafts: &Drafts,
    error: Option<&str>,
    lang: Lang,
) -> Result<String, askama::Error> {
    let stock_draft = if drafts.stock.is_empty() && !game.stock.lines.is_empty() {
        game.stock.to_sheet()
    } else {
        drafts.stock.clone()
    };
    let replacing = drafts
        .replacing
        .and_then(|id| game.fleets.iter().find(|fleet| fleet.id == id));
    let (player_draft, url_draft) = match replacing {
        Some(fleet) if drafts.player.is_empty() && drafts.url.is_empty() => {
            (fleet.player.as_str(), fleet.url.as_str())
        }
        _ => (drafts.player.as_str(), drafts.url.as_str()),
    };
    GamePage {
        lang,
        page: PageText::of(lang),
        t: GameText::of(lang),
        id,
        game,
        report,
        unmatched: game.stock.unmatched().collect(),
        error,
        stock_draft,
        replacing,
        player_draft,
        url_draft,
    }
    .render()
}

/// Helpers called by the template, which resolves bare calls as methods.
impl GamePage<'_> {
    /// "Cyrille ×4, Manu ×1".
    fn shares(&self, shares: &[Share]) -> String {
        shares
            .iter()
            .map(|share| format!("{} ×{}", share.who, share.count))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn fleet_error(&self, err: &FleetLoadError) -> &'static str {
        fleet_error(err).text(self.lang)
    }

    fn is_replacing(&self, fleet: &GameFleet) -> bool {
        self.replacing
            .is_some_and(|replacing| replacing.id == fleet.id)
    }

    fn kind_label(&self, kind: &FigureKind) -> &'static str {
        match kind {
            FigureKind::Ship => self.t.ships,
            FigureKind::Squadron => self.t.squadrons,
        }
    }

    /// Link of the language switch: through the app root, then back to this game. The page
    /// is served at `games/<id>/` or, after a rejected form, at a sibling such as
    /// `games/<id>/stock`, so `../../` is the app root either way.
    fn switch_link(&self, to: Lang) -> String {
        format!("../../lang?to={to}&game={}", self.id)
    }
}
