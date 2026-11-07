//! Miniature models known to the fleet builder, read from its card list.

use std::collections::BTreeSet;

use indexmap::IndexMap;
use serde::Deserialize;

use crate::game::faction::Faction;
use crate::game::figure_kind::FigureKind;
use crate::game::name_matcher::{best_match, tokens};

/// A miniature model: every ship or squadron card sharing one builder `type`, such as
/// `x-wing` for the X-wing Squadron and Luke Skywalker cards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogueModel {
    /// Builder `type`, the key fleets refer to.
    pub id: String,
    /// Ship or squadron.
    pub kind: FigureKind,
    /// Factions with at least one card of this model.
    pub factions: BTreeSet<Faction>,
    /// Display name built from the generic card titles.
    pub name: String,
    /// Titles compared with stock names: all ship titles, generic squadron titles only.
    pub(crate) titles: Vec<String>,
    /// Words of `id` and `titles`, lowercased.
    pub(crate) words: BTreeSet<String>,
}

/// The models of every ship and squadron card of the fleet builder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelCatalogue {
    models: Vec<CatalogueModel>,
}

/// Error returned when the builder card list cannot be read.
#[derive(Debug, thiserror::Error)]
pub enum CatalogueParseError {
    /// The script does not contain a JSON object.
    #[error("no card list found in script")]
    NoCardList,
    /// The card list does not have the expected shape.
    #[error("unreadable card list")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Deserialize)]
struct RawCardList {
    #[serde(default)]
    ship: Vec<RawListedCard>,
    #[serde(default)]
    squadron: Vec<RawListedCard>,
}

/// A card of the builder list; dividers between factions have none of these fields.
#[derive(Debug, Deserialize)]
struct RawListedCard {
    title: Option<String>,
    #[serde(rename = "type")]
    model: Option<String>,
    faction: Option<String>,
    #[serde(default)]
    unique: bool,
}

/// Cards of one model, gathered while reading the list.
#[derive(Debug, Default)]
struct ModelCards {
    factions: BTreeSet<Faction>,
    generic_titles: Vec<String>,
    unique_titles: Vec<String>,
}

impl ModelCatalogue {
    /// Reads the builder's `cards.js` script (`var cards = {...};`).
    ///
    /// # Errors
    /// Returns [`CatalogueParseError`] if the script holds no readable card list.
    pub fn parse_cards_js(script: &str) -> Result<Self, CatalogueParseError> {
        let start = script.find('{').ok_or(CatalogueParseError::NoCardList)?;
        // Without a call to `Deserializer::end`, the trailing `;` is never looked at.
        let mut deserializer = serde_json::Deserializer::from_str(&script[start..]);
        let list = RawCardList::deserialize(&mut deserializer)?;
        let mut models = group_models(FigureKind::Ship, &list.ship);
        models.extend(group_models(FigureKind::Squadron, &list.squadron));
        Ok(Self { models })
    }

    /// Returns the model with builder type `id`.
    pub fn get(&self, kind: FigureKind, id: &str) -> Option<&CatalogueModel> {
        self.models
            .iter()
            .find(|model| model.kind == kind && model.id == id)
    }

    /// Finds the model a stock sheet means by `name`, such as `Nebulon B Frigate`.
    ///
    /// # Arguments
    /// * `name` - Miniature name as written in the sheet.
    /// * `faction` - Faction of the sheet line, narrowing the search when known.
    /// * `kind` - Ship or squadron.
    ///
    /// # Returns
    /// The best-matching model, or `None` when nothing matches or two models match equally.
    pub fn resolve(
        &self,
        name: &str,
        faction: Option<Faction>,
        kind: FigureKind,
    ) -> Option<&CatalogueModel> {
        let candidates: Vec<&CatalogueModel> = self
            .models
            .iter()
            .filter(|model| model.kind == kind)
            .filter(|model| faction.is_none_or(|faction| model.factions.contains(&faction)))
            .collect();
        best_match(name, &candidates)
    }
}

/// Groups the cards of one kind by model, keeping the builder order.
fn group_models(kind: FigureKind, cards: &[RawListedCard]) -> Vec<CatalogueModel> {
    let mut grouped: IndexMap<&str, ModelCards> = IndexMap::new();
    for card in cards {
        let (Some(title), Some(model)) = (&card.title, &card.model) else {
            continue;
        };
        let entry = grouped.entry(model.as_str()).or_default();
        entry
            .factions
            .extend(card.faction.as_deref().and_then(Faction::from_builder));
        let titles = if card.unique {
            &mut entry.unique_titles
        } else {
            &mut entry.generic_titles
        };
        if !titles.contains(title) {
            titles.push(title.clone());
        }
    }
    grouped
        .into_iter()
        .map(|(id, cards)| build_model(kind, id, cards))
        .collect()
}

fn build_model(kind: FigureKind, id: &str, cards: ModelCards) -> CatalogueModel {
    let shown = if cards.generic_titles.is_empty() {
        &cards.unique_titles
    } else {
        &cards.generic_titles
    };
    let name = shown.join(" / ");
    // Named squadrons are pilots ("Luke Skywalker"), whose words would mislead the match;
    // unique ships ("Venator II") still name the hull.
    let titles = match kind {
        FigureKind::Ship => [cards.generic_titles, cards.unique_titles].concat(),
        FigureKind::Squadron => cards.generic_titles,
    };
    let words = tokens(id)
        .into_iter()
        .chain(titles.iter().flat_map(|title| tokens(title)))
        .collect();
    CatalogueModel {
        id: id.to_owned(),
        kind,
        factions: cards.factions,
        name,
        titles,
        words,
    }
}
