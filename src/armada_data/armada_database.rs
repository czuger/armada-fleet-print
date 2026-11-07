//! The local card database built from BSData files.

use std::collections::HashMap;

use indexmap::IndexMap;
use indexmap::map::Entry;
use serde::Serialize;

use crate::armada_data::bs_catalogue::BsCatalogue;
use crate::armada_data::card_kind::CardKind;
use crate::armada_data::objective::Objective;
use crate::armada_data::raw_entry::RawEntry;
use crate::armada_data::ship::Ship;
use crate::armada_data::squadron::Squadron;
use crate::armada_data::upgrade::Upgrade;

/// Version of the JSON layout, bumped on incompatible changes.
pub const SCHEMA_VERSION: u32 = 1;

/// Ship size categories, most specific first: flotillas also link to `Small Ship`.
const SIZE_CATEGORIES: [&str; 7] = [
    "Flotilla",
    "Huge Ship",
    "Armed Station",
    "Unarmed Station",
    "Large Ship",
    "Medium Ship",
    "Small Ship",
];

/// Every card of the game, sorted for stable diffs.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArmadaDatabase {
    /// Version of the JSON layout.
    pub schema_version: u32,
    /// Where the data comes from.
    pub source: DataSource,
    /// Ships and stations, by faction then name.
    pub ships: Vec<Ship>,
    /// Squadrons, by faction then name.
    pub squadrons: Vec<Squadron>,
    /// Upgrades merged across factions, by name then type.
    pub upgrades: Vec<Upgrade>,
    /// Objectives, by kind then name.
    pub objectives: Vec<Objective>,
}

/// Origin of the data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DataSource {
    /// GitHub repository, as `owner/name`.
    pub repo: String,
    /// Git reference asked for or resolved: a release tag or a branch.
    #[serde(rename = "ref")]
    pub git_ref: String,
    /// Commit SHA the files were read at.
    pub commit: String,
    /// Generation time, RFC 3339 in UTC.
    pub generated_at: String,
}

impl ArmadaDatabase {
    /// Builds the database from parsed BattleScribe files.
    ///
    /// Upgrades listed in several files are merged when their name (ignoring case), type and
    /// points match; their factions are accumulated. Cards whose points differ stay distinct,
    /// as for the Republic and Rebel Ahsoka Tano officers. Ships and squadrons are not merged.
    ///
    /// # Arguments
    /// * `source` - Origin of the files.
    /// * `catalogues` - Parsed files; when merging, the first copy's id is kept, so pass them
    ///   in a fixed order.
    pub fn build(source: DataSource, catalogues: &[BsCatalogue]) -> Self {
        let categories: HashMap<&str, &str> = catalogues
            .iter()
            .flat_map(|catalogue| &catalogue.categories)
            .map(|(id, name)| (id.as_str(), name.as_str()))
            .collect();
        let mut collector = CardCollector::default();
        for catalogue in catalogues {
            for entry in &catalogue.entries {
                collector.add_entry(catalogue.faction.as_deref(), entry, &categories);
            }
        }
        collector.into_database(source)
    }
}

/// Merge key of upgrades: lowercase name, type and points.
type UpgradeKey = (String, String, Option<u32>);
/// Merge key of objectives: lowercase name and kind.
type ObjectiveKey = (String, String);

/// Cards gathered entry by entry.
#[derive(Debug, Default)]
struct CardCollector {
    ships: Vec<Ship>,
    squadrons: Vec<Squadron>,
    upgrades: IndexMap<UpgradeKey, Upgrade>,
    objectives: IndexMap<ObjectiveKey, Objective>,
}

impl CardCollector {
    /// Adds the cards described by the profiles of `entry`.
    fn add_entry(
        &mut self,
        faction: Option<&str>,
        entry: &RawEntry,
        categories: &HashMap<&str, &str>,
    ) {
        for profile in &entry.profiles {
            match CardKind::from_type_name(&profile.type_name) {
                CardKind::Ship(kind) => {
                    let size = ship_size(entry, categories);
                    self.ships
                        .push(Ship::from_entry(faction, entry, profile, kind, size));
                }
                CardKind::Squadron => {
                    self.squadrons
                        .push(Squadron::from_entry(faction, entry, profile));
                }
                CardKind::Upgrade(upgrade_type) => {
                    self.add_upgrade(Upgrade::from_entry(faction, entry, profile, upgrade_type));
                }
                CardKind::Objective(kind) => {
                    let objective = Objective::from_entry(entry, profile, kind);
                    let key = (objective.name.to_lowercase(), objective.kind.clone());
                    self.objectives.entry(key).or_insert(objective);
                }
                // Speed charts are read with their ship.
                CardKind::SpeedChart => {}
                CardKind::Other => {
                    tracing::debug!(entry = entry.id, profile.type_name, "profile type skipped");
                }
            }
        }
    }

    /// Adds `upgrade`, or merges it into the copy already met.
    fn add_upgrade(&mut self, upgrade: Upgrade) {
        let key = (
            upgrade.name.to_lowercase(),
            upgrade.upgrade_type.clone(),
            upgrade.points,
        );
        match self.upgrades.entry(key) {
            Entry::Occupied(mut known) => known.get_mut().merge(upgrade),
            Entry::Vacant(slot) => {
                slot.insert(upgrade);
            }
        }
    }

    /// Sorts the cards and wraps them with `source`.
    fn into_database(self, source: DataSource) -> ArmadaDatabase {
        let mut ships = self.ships;
        ships.sort_by(|a, b| (&a.faction, &a.name, &a.id).cmp(&(&b.faction, &b.name, &b.id)));
        let mut squadrons = self.squadrons;
        squadrons.sort_by(|a, b| (&a.faction, &a.name, &a.id).cmp(&(&b.faction, &b.name, &b.id)));
        let mut upgrades: Vec<Upgrade> = self.upgrades.into_values().collect();
        upgrades.sort_by_cached_key(|upgrade| {
            (
                upgrade.name.to_lowercase(),
                upgrade.upgrade_type.clone(),
                upgrade.points,
                upgrade.id.clone(),
            )
        });
        let mut objectives: Vec<Objective> = self.objectives.into_values().collect();
        objectives.sort_by(|a, b| (&a.kind, &a.name).cmp(&(&b.kind, &b.name)));
        ArmadaDatabase {
            schema_version: SCHEMA_VERSION,
            source,
            ships,
            squadrons,
            upgrades,
            objectives,
        }
    }
}

/// Size category of a ship, from the categories its entry links to.
fn ship_size(entry: &RawEntry, categories: &HashMap<&str, &str>) -> Option<String> {
    let linked: Vec<&str> = entry
        .category_ids
        .iter()
        .filter_map(|id| categories.get(id.as_str()).copied())
        .collect();
    SIZE_CATEGORIES
        .into_iter()
        .find(|size| linked.contains(size))
        .map(str::to_owned)
}
