//! What a fleet takes from the shared stock: miniatures and unique cards.

use crate::fleet::FleetParseError;
use crate::fleet_json::RawFleet;
use crate::fleet_script::extract_fleet_json;
use crate::game::figure_kind::FigureKind;

/// The miniatures and unique cards of one fleet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FleetRoster {
    /// Name given to the fleet by its author.
    pub name: String,
    /// Total fleet points.
    pub total_points: u32,
    /// Miniatures per model, in order of first appearance.
    pub figures: Vec<FigureCount>,
    /// Titles of the unique cards: ships, squadrons and upgrades.
    pub unique_cards: Vec<String>,
}

/// Number of miniatures of one model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FigureCount {
    /// Builder `type` of the cards, such as `x-wing`.
    pub model: String,
    /// Ship or squadron.
    pub kind: FigureKind,
    /// Number of miniatures.
    pub count: u32,
}

impl FleetRoster {
    /// Reads the fleet embedded in a fleet builder page.
    ///
    /// # Errors
    /// Returns [`FleetParseError`] if the page has no fleet data or the data is malformed.
    pub fn from_page_html(html: &str) -> Result<Self, FleetParseError> {
        let json = extract_fleet_json(html).ok_or(FleetParseError::NoFleetData)?;
        let raw: RawFleet = serde_json::from_str(json)?;
        let mut roster = Self {
            name: raw.name.clone().unwrap_or_default(),
            total_points: raw.points.total,
            figures: Vec::new(),
            unique_cards: Vec::new(),
        };
        for ship in &raw.ships {
            roster.add_figures(&ship.model, &ship.title, FigureKind::Ship, 1);
            roster.add_unique(ship.unique, &ship.title);
            for upgrade in ship.upgrades.iter() {
                roster.add_unique(upgrade.unique, &upgrade.title);
            }
        }
        for squadron in &raw.squadrons {
            let count = squadron
                .quantity
                .count()
                .map_err(|err| FleetParseError::Quantity(squadron.image.clone(), err))?;
            roster.add_figures(
                &squadron.model,
                &squadron.title,
                FigureKind::Squadron,
                count,
            );
            if count > 0 {
                roster.add_unique(squadron.unique, &squadron.title);
            }
        }
        Ok(roster)
    }

    /// Number of miniatures of `kind` in the fleet.
    pub fn figure_total(&self, kind: FigureKind) -> u32 {
        self.figures
            .iter()
            .filter(|figure| figure.kind == kind)
            .map(|figure| figure.count)
            .sum()
    }

    /// Counts `count` miniatures of `model`, falling back to the card title for cards
    /// without a type.
    fn add_figures(&mut self, model: &str, title: &str, kind: FigureKind, count: u32) {
        if count == 0 {
            return;
        }
        let model = if model.is_empty() { title } else { model };
        match self
            .figures
            .iter_mut()
            .find(|figure| figure.kind == kind && figure.model == model)
        {
            Some(figure) => figure.count += count,
            None => self.figures.push(FigureCount {
                model: model.to_owned(),
                kind,
                count,
            }),
        }
    }

    fn add_unique(&mut self, unique: bool, title: &str) {
        if unique && !title.is_empty() {
            self.unique_cards.push(title.to_owned());
        }
    }
}
