//! Fleet contents, reduced to what gets printed.

use std::collections::BTreeSet;

use crate::card_name::CardName;
use crate::fleet_json::{RawFleet, RawSquadron};
use crate::fleet_script::extract_fleet_json;

/// A fleet read from a fleet builder page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fleet {
    /// Name given to the fleet by its author.
    pub name: String,
    /// Total fleet points.
    pub total_points: u32,
    /// Points spent on squadrons.
    pub squadron_points: u32,
    /// Ships in builder order.
    pub ships: Vec<Ship>,
    /// Squadrons, one entry per distinct card, in order of first appearance.
    pub squadrons: Vec<SquadronGroup>,
    /// Selected objective cards (assault, defense, navigation), skipping unselected ones.
    pub objectives: Vec<CardName>,
}

/// A ship card with its equipped upgrades.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ship {
    /// The ship card.
    pub card: CardName,
    /// Equipped upgrade cards, in slot order.
    pub upgrades: Vec<CardName>,
}

/// All copies of one squadron card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SquadronGroup {
    /// The squadron card.
    pub card: CardName,
    /// Number of copies in the fleet, at least 1.
    pub count: u32,
}

/// Error returned when a page does not contain a readable fleet.
#[derive(Debug, thiserror::Error)]
pub enum FleetParseError {
    /// The page has no embedded fleet, typically because the fleet was never saved.
    #[error("no fleet data found in page")]
    NoFleetData,
    /// The embedded fleet does not have the expected shape.
    #[error("unreadable fleet data")]
    Json(#[from] serde_json::Error),
    /// A squadron quantity is not a number.
    #[error("invalid quantity for squadron {0}")]
    Quantity(CardName, #[source] std::num::ParseIntError),
}

impl Fleet {
    /// Reads the fleet embedded in a fleet builder page.
    ///
    /// # Errors
    /// Returns [`FleetParseError`] if the page has no fleet data or the data is malformed.
    pub fn from_page_html(html: &str) -> Result<Self, FleetParseError> {
        let json = extract_fleet_json(html).ok_or(FleetParseError::NoFleetData)?;
        let raw: RawFleet = serde_json::from_str(json)?;
        let squadrons = group_squadrons(&raw.squadrons)?;
        let objectives = [raw.assault, raw.defense, raw.navigation]
            .into_iter()
            .flatten()
            .map(|card| card.image)
            .collect();
        let ships = raw
            .ships
            .into_iter()
            .map(|ship| Ship {
                card: ship.image,
                upgrades: ship.upgrades.into_cards(),
            })
            .collect();
        Ok(Self {
            name: raw.name.unwrap_or_default(),
            total_points: raw.points.total,
            squadron_points: raw.points.squadron,
            ships,
            squadrons,
            objectives,
        })
    }

    /// Returns every distinct card image the fleet needs, sorted by name.
    pub fn cards(&self) -> Vec<CardName> {
        let ships = self
            .ships
            .iter()
            .flat_map(|ship| std::iter::once(&ship.card).chain(&ship.upgrades));
        let squadrons = self.squadrons.iter().map(|group| &group.card);
        let unique: BTreeSet<&CardName> = ships.chain(squadrons).chain(&self.objectives).collect();
        unique.into_iter().cloned().collect()
    }
}

/// Merges squadron entries sharing a card, keeping first-appearance order and dropping empty ones.
fn group_squadrons(squadrons: &[RawSquadron]) -> Result<Vec<SquadronGroup>, FleetParseError> {
    let mut groups: Vec<SquadronGroup> = Vec::new();
    for squadron in squadrons {
        let count = squadron
            .quantity
            .count()
            .map_err(|err| FleetParseError::Quantity(squadron.image.clone(), err))?;
        match groups.iter_mut().find(|group| group.card == squadron.image) {
            Some(group) => group.count += count,
            None => groups.push(SquadronGroup {
                card: squadron.image.clone(),
                count,
            }),
        }
    }
    groups.retain(|group| group.count > 0);
    Ok(groups)
}
