//! Objective cards.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::armada_data::characteristics::Characteristics;
use crate::armada_data::raw_entry::{RawEntry, RawProfile};

/// An objective card.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Objective {
    /// BSData id of the selection entry.
    pub id: String,
    /// Card name.
    pub name: String,
    /// Objective kind: `Assault`, `Defense`, `Navigation`, `Campaign` or `Scenario`.
    pub kind: String,
    /// Setup rule.
    pub setup: Option<String>,
    /// Special rule.
    pub special_rule: Option<String>,
    /// End of round rule.
    pub end_of_round: Option<String>,
    /// End of game rule.
    pub end_of_game: Option<String>,
    /// Victory token value, as written (`--` when none).
    pub victory_token_value: Option<String>,
    /// Other non-empty characteristics.
    pub other: BTreeMap<String, String>,
}

impl Objective {
    /// Builds an objective from its entry and card profile.
    ///
    /// # Arguments
    /// * `entry` - Selection entry.
    /// * `profile` - Objective profile of the entry.
    /// * `kind` - Kind label from the profile type.
    pub fn from_entry(entry: &RawEntry, profile: &RawProfile, kind: String) -> Self {
        let mut values = Characteristics::from_profile(profile);
        Self {
            id: entry.id.clone(),
            name: entry.title(profile).name,
            kind,
            setup: values.take("Setup"),
            special_rule: values.take("Special Rule"),
            end_of_round: values.take("End of Round"),
            end_of_game: values.take("End of Game"),
            victory_token_value: values.take("Victory Token Value"),
            other: values.into_other(),
        }
    }
}
