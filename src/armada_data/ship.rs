//! Ship and station cards.

use std::collections::BTreeMap;

use indexmap::IndexMap;
use serde::Serialize;

use crate::armada_data::card_kind::CardKind;
use crate::armada_data::characteristics::Characteristics;
use crate::armada_data::dice::Dice;
use crate::armada_data::raw_entry::{RawEntry, RawProfile};
use crate::armada_data::speed_chart::SpeedChart;

/// Suffix of the battery characteristics, one per hull zone.
const FIRING_ARC_SUFFIX: &str = " Firing Arc";
/// Suffix of the shield characteristics, one per hull zone.
const SHIELD_SUFFIX: &str = " Shield Value";

/// A ship or station card, with base values (conditional modifiers are not applied).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Ship {
    /// BSData id of the selection entry.
    pub id: String,
    /// Faction catalogue, `None` for cards of the game system file (the objective station).
    pub faction: Option<String>,
    /// Card name without the unique bullet.
    pub name: String,
    /// Whether the card is unique.
    pub unique: bool,
    /// Profile kind: `Ships`, `Ships With Special Battery`, `Huge Ships` or `Stations`.
    pub kind: String,
    /// Size category, such as `Small Ship` or `Flotilla`.
    pub size: Option<String>,
    /// Cost in fleet points.
    pub points: Option<u32>,
    /// Hull value.
    pub hull: Option<u32>,
    /// Command value.
    pub command: Option<u32>,
    /// Squadron value.
    pub squadron: Option<u32>,
    /// Engineering value.
    pub engineering: Option<u32>,
    /// Defense tokens, one item per token.
    pub defense_tokens: Vec<String>,
    /// Anti-squadron armament.
    pub anti_squadron: Option<Dice>,
    /// Battery armament by hull zone (`front`, `left`, `left_auxiliary`…), in card order.
    pub batteries: IndexMap<String, Dice>,
    /// Shield value by hull zone, in card order.
    pub shields: IndexMap<String, u32>,
    /// Upgrade slots, repeated when a slot appears twice.
    pub upgrade_bar: Vec<String>,
    /// Keywords.
    pub keywords: Vec<String>,
    /// Maximum speed.
    pub max_speed: Option<u32>,
    /// Yaw values by speed.
    pub speed_chart: SpeedChart,
    /// Other non-empty characteristics, such as `Special Battery`.
    pub other: BTreeMap<String, String>,
}

impl Ship {
    /// Builds a ship from its entry and card profile.
    ///
    /// # Arguments
    /// * `faction` - Faction of the file holding the entry.
    /// * `entry` - Selection entry, which also holds the speed chart profiles.
    /// * `profile` - Ship profile of the entry.
    /// * `kind` - Kind label from the profile type.
    /// * `size` - Size category resolved from the entry's category links.
    pub fn from_entry(
        faction: Option<&str>,
        entry: &RawEntry,
        profile: &RawProfile,
        kind: String,
        size: Option<String>,
    ) -> Self {
        let title = entry.title(profile);
        let mut values = Characteristics::from_profile(profile);
        Self {
            id: entry.id.clone(),
            faction: faction.map(str::to_owned),
            name: title.name,
            unique: title.unique,
            kind,
            size,
            points: entry.points,
            // Stations use the squadron names of these two characteristics.
            hull: values
                .take_number("Hull")
                .or_else(|| values.take_number("Hull Value")),
            command: values.take_number("Command"),
            squadron: values.take_number("Squadron"),
            engineering: values.take_number("Engineering"),
            defense_tokens: values.take_defense_tokens("Defense Tokens"),
            anti_squadron: values
                .take_dice("Anti-Squadron Value")
                .or_else(|| values.take_dice("Anti-Squadron")),
            batteries: take_batteries(&mut values),
            shields: take_shields(&mut values),
            upgrade_bar: values.take_list("Upgrade Bar"),
            keywords: values.take_list("Keywords"),
            max_speed: values.take_number("Max Speed"),
            speed_chart: speed_chart(entry),
            other: values.into_other(),
        }
    }
}

/// Takes the `<Zone> Firing Arc` characteristics.
fn take_batteries(values: &mut Characteristics) -> IndexMap<String, Dice> {
    values
        .names_ending_with(FIRING_ARC_SUFFIX)
        .into_iter()
        .filter_map(|name| {
            let dice = values.take_dice(&name)?;
            Some((zone_key(&name, FIRING_ARC_SUFFIX), dice))
        })
        .collect()
}

/// Takes the readable `<Zone> Shield Value` characteristics.
fn take_shields(values: &mut Characteristics) -> IndexMap<String, u32> {
    values
        .names_ending_with(SHIELD_SUFFIX)
        .into_iter()
        .filter_map(|name| {
            let shield = values.take_number(&name)?;
            Some((zone_key(&name, SHIELD_SUFFIX), shield))
        })
        .collect()
}

/// Hull zone key of a characteristic name: `Left-Auxiliary Firing Arc` gives `left_auxiliary`.
fn zone_key(name: &str, suffix: &str) -> String {
    name.strip_suffix(suffix)
        .unwrap_or(name)
        .to_lowercase()
        .replace(['-', ' '], "_")
}

/// Speed chart from the `1.9 Speed Chart` profiles of `entry`.
fn speed_chart(entry: &RawEntry) -> SpeedChart {
    let joints: Vec<&RawProfile> = entry
        .profiles
        .iter()
        .filter(|profile| CardKind::from_type_name(&profile.type_name) == CardKind::SpeedChart)
        .collect();
    SpeedChart::from_profiles(&joints)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zone_keys() {
        assert_eq!(zone_key("Front Firing Arc", FIRING_ARC_SUFFIX), "front");
        assert_eq!(
            zone_key("Left-Auxiliary Shield Value", SHIELD_SUFFIX),
            "left_auxiliary"
        );
    }
}
