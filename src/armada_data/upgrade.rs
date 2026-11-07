//! Upgrade cards.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::armada_data::characteristics::Characteristics;
use crate::armada_data::raw_entry::{RawEntry, RawProfile};

/// Suffix of the characteristic holding the card text, such as `Officer Ability`.
const TEXT_SUFFIX: &str = "Ability";

/// An upgrade card, possibly shared by several factions.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Upgrade {
    /// BSData id of the first selection entry met.
    pub id: String,
    /// Card name without the unique bullet.
    pub name: String,
    /// Upgrade type, such as `Officer` or `Weapons Team & Offensive Retrofit`.
    #[serde(rename = "type")]
    pub upgrade_type: String,
    /// Whether the card is unique.
    pub unique: bool,
    /// Cost in fleet points.
    pub points: Option<u32>,
    /// Factions whose catalogue lists the card, sorted.
    pub factions: Vec<String>,
    /// Card text.
    pub text: String,
    /// Other non-empty characteristics, such as `Recur` or `Ready Cost`.
    pub details: BTreeMap<String, String>,
}

impl Upgrade {
    /// Builds an upgrade from its entry and card profile.
    ///
    /// # Arguments
    /// * `faction` - Faction of the file holding the entry, if any.
    /// * `entry` - Selection entry.
    /// * `profile` - Upgrade profile of the entry.
    /// * `upgrade_type` - Type label from the profile type.
    pub fn from_entry(
        faction: Option<&str>,
        entry: &RawEntry,
        profile: &RawProfile,
        upgrade_type: String,
    ) -> Self {
        let title = entry.title(profile);
        let mut values = Characteristics::from_profile(profile);
        Self {
            id: entry.id.clone(),
            name: title.name,
            upgrade_type,
            unique: title.unique,
            points: entry.points,
            factions: faction.map(str::to_owned).into_iter().collect(),
            text: values.take_ending_with(TEXT_SUFFIX).unwrap_or_default(),
            details: values.into_other(),
        }
    }

    /// Folds a duplicate of this card found in another file: adds its factions, and keeps the
    /// card unique if either copy says so (the bullet is missing in some catalogues).
    ///
    /// # Arguments
    /// * `other` - The same card read from another file.
    pub fn merge(&mut self, other: Self) {
        self.unique |= other.unique;
        for faction in other.factions {
            if !self.factions.contains(&faction) {
                self.factions.push(faction);
            }
        }
        self.factions.sort();
    }
}
