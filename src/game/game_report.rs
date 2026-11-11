//! Results of a game: miniatures used against the stock, and unique cards taken twice.

use indexmap::IndexMap;

use crate::fleet::FleetParseError;
use crate::fleet_url::FleetUrlError;
use crate::game::figure_kind::FigureKind;
use crate::game::fleet_roster::FleetRoster;
use crate::game::game::GameFleet;
use crate::game::model_catalogue::ModelCatalogue;
use crate::game::stock::Stock;

/// Results computed from the stock and the current content of every fleet.
#[derive(Debug)]
pub struct GameReport {
    /// Every fleet with what was read from it, in game order.
    pub fleets: Vec<FleetResult>,
    /// Every model in stock or in a fleet, ships first, then by name.
    pub models: Vec<ModelUsage>,
    /// Unique cards present more than once across the fleets, by name.
    pub duplicates: Vec<DuplicateCard>,
    /// Totals per kind, ships first.
    pub totals: Vec<KindTotal>,
}

/// A fleet of the game and its content, or why it could not be read.
#[derive(Debug)]
pub struct FleetResult {
    /// The fleet as saved in the game.
    pub fleet: GameFleet,
    /// Its content on the fleet builder.
    pub roster: Result<FleetRoster, FleetLoadError>,
}

/// Error returned when a fleet of the game cannot be read.
#[derive(Debug, thiserror::Error)]
pub enum FleetLoadError {
    /// The saved link is not a fleet builder link.
    #[error("invalid fleet link")]
    Url(#[from] FleetUrlError),
    /// The fleet page could not be downloaded.
    #[error("downloading fleet page failed")]
    Fetch(#[from] reqwest::Error),
    /// The fleet page has no readable fleet.
    #[error("reading fleet failed")]
    Parse(#[from] FleetParseError),
}

/// Stock and use of one model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelUsage {
    /// Builder `type`.
    pub model: String,
    /// Ship or squadron.
    pub kind: FigureKind,
    /// Name shown: the stock sheet names if any, else the builder card titles.
    pub name: String,
    /// Miniatures in stock.
    pub stock: u32,
    /// Miniatures used by the fleets.
    pub used: u32,
    /// Miniatures in stock per owner.
    pub owners: Vec<Share>,
    /// Miniatures used per fleet player.
    pub players: Vec<Share>,
}

/// Number of miniatures held or used by someone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Share {
    /// Owner or player name.
    pub who: String,
    /// Number of miniatures.
    pub count: u32,
}

/// A unique card present more than once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateCard {
    /// Card name, as first seen.
    pub name: String,
    /// Player of each fleet holding a copy, once per copy.
    pub players: Vec<String>,
}

/// Miniatures of one kind in stock and used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KindTotal {
    /// Ship or squadron.
    pub kind: FigureKind,
    /// Miniatures in stock.
    pub stock: u32,
    /// Miniatures used by the fleets.
    pub used: u32,
}

impl GameReport {
    /// Compares the fleets with the stock.
    ///
    /// # Arguments
    /// * `stock` - Miniatures of the players.
    /// * `fleets` - Every fleet of the game; unreadable ones are left out of the counts.
    /// * `catalogue` - Builder models, naming the models missing from the stock when available.
    pub fn build(
        stock: &Stock,
        fleets: Vec<FleetResult>,
        catalogue: Option<&ModelCatalogue>,
    ) -> Self {
        let models = model_usages(stock, &fleets, catalogue);
        let totals = [FigureKind::Ship, FigureKind::Squadron]
            .into_iter()
            .map(|kind| kind_total(kind, &models))
            .collect();
        Self {
            duplicates: duplicate_cards(&fleets),
            models,
            totals,
            fleets,
        }
    }

    /// Whether some model is used more than it is stocked.
    pub fn has_shortfall(&self) -> bool {
        self.models.iter().any(|usage| usage.shortfall() > 0)
    }
}

impl ModelUsage {
    /// Miniatures missing from the stock.
    pub fn shortfall(&self) -> u32 {
        self.used.saturating_sub(self.stock)
    }
}

/// Usage of every stocked or used model, ships first, then by name.
fn model_usages(
    stock: &Stock,
    fleets: &[FleetResult],
    catalogue: Option<&ModelCatalogue>,
) -> Vec<ModelUsage> {
    let mut usages: IndexMap<(FigureKind, String), ModelUsage> = IndexMap::new();
    let mut stock_names: IndexMap<(FigureKind, String), Vec<String>> = IndexMap::new();
    for line in &stock.lines {
        let Some(model) = &line.model else { continue };
        let key = (line.kind, model.clone());
        let usage = usages
            .entry(key.clone())
            .or_insert_with(|| empty_usage(line.kind, model));
        usage.stock += line.quantity;
        add_share(&mut usage.owners, &line.owner, line.quantity);
        let names = stock_names.entry(key).or_default();
        if !names.contains(&line.name) {
            names.push(line.name.clone());
        }
    }
    for result in fleets {
        let Ok(roster) = &result.roster else { continue };
        for figure in &roster.figures {
            let key = (figure.kind, figure.model.clone());
            let usage = usages
                .entry(key)
                .or_insert_with(|| empty_usage(figure.kind, &figure.model));
            usage.used += figure.count;
            add_share(&mut usage.players, &result.fleet.player, figure.count);
        }
    }
    let mut usages: Vec<ModelUsage> = usages
        .into_iter()
        .map(|(key, mut usage)| {
            usage.name = model_name(&key, &stock_names, catalogue);
            usage
        })
        .collect();
    usages.sort_by_cached_key(|usage| (usage.kind, usage.name.to_lowercase()));
    usages
}

fn empty_usage(kind: FigureKind, model: &str) -> ModelUsage {
    ModelUsage {
        model: model.to_owned(),
        kind,
        name: String::new(),
        stock: 0,
        used: 0,
        owners: Vec::new(),
        players: Vec::new(),
    }
}

fn add_share(shares: &mut Vec<Share>, who: &str, count: u32) {
    match shares.iter_mut().find(|share| share.who == who) {
        Some(share) => share.count += count,
        None => shares.push(Share {
            who: who.to_owned(),
            count,
        }),
    }
}

fn model_name(
    key: &(FigureKind, String),
    stock_names: &IndexMap<(FigureKind, String), Vec<String>>,
    catalogue: Option<&ModelCatalogue>,
) -> String {
    if let Some(names) = stock_names.get(key) {
        return names.join(" / ");
    }
    let (kind, model) = key;
    catalogue
        .and_then(|catalogue| catalogue.get(*kind, model))
        .map_or_else(|| model.clone(), |entry| entry.name.clone())
}

fn kind_total(kind: FigureKind, models: &[ModelUsage]) -> KindTotal {
    let of_kind = || models.iter().filter(move |usage| usage.kind == kind);
    KindTotal {
        kind,
        stock: of_kind().map(|usage| usage.stock).sum(),
        used: of_kind().map(|usage| usage.used).sum(),
    }
}

/// Unique cards held more than once, matched by name: "Darth Vader (TIE Defender)" and the
/// commander "Darth Vader" are the same character.
fn duplicate_cards(fleets: &[FleetResult]) -> Vec<DuplicateCard> {
    let mut cards: IndexMap<String, DuplicateCard> = IndexMap::new();
    for result in fleets {
        let Ok(roster) = &result.roster else { continue };
        for title in &roster.unique_cards {
            cards
                .entry(unique_name_key(title))
                .or_insert_with(|| DuplicateCard {
                    name: title.clone(),
                    players: Vec::new(),
                })
                .players
                .push(result.fleet.player.clone());
        }
    }
    let mut duplicates: Vec<DuplicateCard> = cards
        .into_values()
        .filter(|card| card.players.len() > 1)
        .collect();
    duplicates.sort_by_cached_key(|card| card.name.to_lowercase());
    duplicates
}

/// Name a unique card is compared by: no leading `•`, no trailing "(…)" variant, lowercase.
fn unique_name_key(title: &str) -> String {
    let title = title.trim().trim_start_matches('•').trim();
    let base = match title.rfind(" (") {
        Some(index) if title.ends_with(')') => &title[..index],
        _ => title,
    };
    base.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_names_ignore_variants_and_case() {
        assert_eq!(unique_name_key("Darth Vader (TIE Defender)"), "darth vader");
        assert_eq!(unique_name_key("• Darth Vader"), "darth vader");
        assert_eq!(unique_name_key("Dodonna's Pride"), "dodonna's pride");
    }
}
