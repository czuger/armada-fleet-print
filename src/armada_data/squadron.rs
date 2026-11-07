//! Squadron cards.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::armada_data::characteristics::Characteristics;
use crate::armada_data::dice::Dice;
use crate::armada_data::raw_entry::{RawEntry, RawProfile};

/// A squadron card.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Squadron {
    /// BSData id of the selection entry.
    pub id: String,
    /// Faction catalogue, `None` for cards of the game system file.
    pub faction: Option<String>,
    /// Card name without the unique bullet.
    pub name: String,
    /// Whether the card is unique.
    pub unique: bool,
    /// Cost in fleet points.
    pub points: Option<u32>,
    /// Speed.
    pub speed: Option<u32>,
    /// Hull value.
    pub hull: Option<u32>,
    /// Armament against squadrons.
    pub anti_squadron: Option<Dice>,
    /// Armament against ships.
    pub battery: Option<Dice>,
    /// Keywords, such as `Bomber` or `Counter 2`.
    pub keywords: Vec<String>,
    /// Defense tokens, one item per token.
    pub defense_tokens: Vec<String>,
    /// Ability text of unique squadrons.
    pub ability: Option<String>,
    /// Other non-empty characteristics.
    pub other: BTreeMap<String, String>,
}

impl Squadron {
    /// Builds a squadron from its entry and card profile.
    ///
    /// # Arguments
    /// * `faction` - Faction of the file holding the entry.
    /// * `entry` - Selection entry.
    /// * `profile` - Squadron profile of the entry.
    pub fn from_entry(faction: Option<&str>, entry: &RawEntry, profile: &RawProfile) -> Self {
        let title = entry.title(profile);
        let mut values = Characteristics::from_profile(profile);
        Self {
            id: entry.id.clone(),
            faction: faction.map(str::to_owned),
            name: title.name,
            unique: title.unique,
            points: entry.points,
            speed: values.take_number("Speed"),
            hull: values.take_number("Hull Value"),
            anti_squadron: values.take_dice("Anti-Squadron"),
            battery: values.take_dice("Battery"),
            keywords: values.take_list("Keywords"),
            defense_tokens: values.take_defense_tokens("Defense Tokens"),
            ability: values.take("Squadron Ability"),
            other: values.into_other(),
        }
    }
}
