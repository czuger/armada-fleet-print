//! Serde mirror of the fleet JSON embedded in fleet builder pages. Only the fields used for
//! printing are declared.

use indexmap::IndexMap;
use serde::Deserialize;

use crate::card_name::CardName;

#[derive(Debug, Deserialize)]
pub(crate) struct RawFleet {
    pub name: Option<String>,
    #[serde(default)]
    pub ships: Vec<RawShip>,
    #[serde(default)]
    pub squadrons: Vec<RawSquadron>,
    pub assault: Option<RawCard>,
    pub defense: Option<RawCard>,
    pub navigation: Option<RawCard>,
    #[serde(default)]
    pub points: RawPoints,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct RawPoints {
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub squadron: u32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawShip {
    pub image: CardName,
    #[serde(default)]
    pub title: String,
    /// Hull shared by the ship's variants, e.g. `corvette` for both CR90 cards.
    #[serde(rename = "type", default)]
    pub model: String,
    #[serde(default)]
    pub unique: bool,
    #[serde(default)]
    pub upgrades: RawUpgrades,
}

/// Upgrade slots in the builder's order; an empty slot is `null`. Some encoders emit `[]`
/// instead of `{}` for an empty object, hence the list variant.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum RawUpgrades {
    Slots(IndexMap<String, Option<RawCard>>),
    List(Vec<Option<RawCard>>),
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawSquadron {
    pub image: CardName,
    #[serde(default)]
    pub title: String,
    /// Craft shared by generic and named squadrons, e.g. `x-wing` for Luke Skywalker.
    #[serde(rename = "type", default)]
    pub model: String,
    #[serde(default)]
    pub unique: bool,
    pub quantity: RawQuantity,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum RawQuantity {
    Number(u32),
    Text(String),
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawCard {
    pub image: CardName,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub unique: bool,
}

impl Default for RawUpgrades {
    fn default() -> Self {
        Self::List(Vec::new())
    }
}

impl RawUpgrades {
    /// Equipped upgrade cards, in slot order.
    pub fn into_cards(self) -> Vec<CardName> {
        let slots: Vec<Option<RawCard>> = match self {
            Self::Slots(slots) => slots.into_values().collect(),
            Self::List(slots) => slots,
        };
        slots.into_iter().flatten().map(|card| card.image).collect()
    }

    /// Equipped upgrade cards, in slot order, without consuming the slots.
    pub fn iter(&self) -> impl Iterator<Item = &RawCard> {
        let slots: Vec<&Option<RawCard>> = match self {
            Self::Slots(slots) => slots.values().collect(),
            Self::List(slots) => slots.iter().collect(),
        };
        slots.into_iter().flatten()
    }
}

impl RawQuantity {
    /// Number of copies; the builder may store it as a number or as text.
    pub fn count(&self) -> Result<u32, std::num::ParseIntError> {
        match self {
            Self::Number(count) => Ok(*count),
            Self::Text(text) => text.trim().parse(),
        }
    }
}
