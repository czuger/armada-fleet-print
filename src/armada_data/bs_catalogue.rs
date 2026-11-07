//! Reading of BattleScribe game system (`.gst`) and catalogue (`.cat`) files.
//!
//! Conditional `<modifiers>` (for example the Gladiator upgrade bar changing with a selection)
//! are ignored: only base values are read.

use std::collections::HashMap;

use roxmltree::{Document, Node};

use crate::armada_data::raw_entry::{RawEntry, RawProfile};

/// Root element of a catalogue file.
const CATALOGUE_ROOT: &str = "catalogue";
/// Root element of the game system file.
const GAME_SYSTEM_ROOT: &str = "gameSystem";
/// Cost type holding fleet points.
const POINTS_COST_TYPE: &str = "points";

/// The card data of one BattleScribe file.
#[derive(Debug, Clone, PartialEq)]
pub struct BsCatalogue {
    /// Faction name for a catalogue, `None` for the game system file.
    pub faction: Option<String>,
    /// Category names by id, declared in this file.
    pub categories: HashMap<String, String>,
    /// Every selection entry holding a profile, nested ones included, in document order.
    pub entries: Vec<RawEntry>,
}

/// Error returned when a BattleScribe file cannot be read.
#[derive(Debug, thiserror::Error)]
pub enum BsCatalogueError {
    /// The file is not well-formed XML.
    #[error("invalid XML")]
    Xml(#[from] roxmltree::Error),
    /// The root element is neither `catalogue` nor `gameSystem`.
    #[error("unexpected root element {0}")]
    UnexpectedRoot(String),
    /// An element lacks a required attribute.
    #[error("{element} element at byte {position} has no {attribute} attribute")]
    MissingAttribute {
        /// Element name.
        element: String,
        /// Missing attribute.
        attribute: &'static str,
        /// Byte offset of the element in the file.
        position: usize,
    },
    /// A points cost is not a non-negative number.
    #[error("entry {entry} has invalid cost {value:?}")]
    InvalidCost {
        /// Id of the entry.
        entry: String,
        /// Cost as written in the file.
        value: String,
    },
}

impl BsCatalogue {
    /// Parses a `.cat` or `.gst` file. Namespaces are ignored.
    ///
    /// # Arguments
    /// * `xml` - File contents.
    ///
    /// # Errors
    /// Returns [`BsCatalogueError`] if the XML is malformed, the root is unexpected, or an
    /// entry lacks its id or name or has an invalid cost.
    pub fn parse(xml: &str) -> Result<Self, BsCatalogueError> {
        let document = Document::parse(xml)?;
        let root = document.root_element();
        let faction = match root.tag_name().name() {
            CATALOGUE_ROOT => Some(attribute(root, "name")?.to_owned()),
            GAME_SYSTEM_ROOT => None,
            other => return Err(BsCatalogueError::UnexpectedRoot(other.to_owned())),
        };
        let mut categories = HashMap::new();
        for node in elements(root, "categoryEntry") {
            categories.insert(
                attribute(node, "id")?.to_owned(),
                attribute(node, "name")?.to_owned(),
            );
        }
        let mut entries = Vec::new();
        for node in elements(root, "selectionEntry") {
            let profiles = read_profiles(node)?;
            if !profiles.is_empty() {
                entries.push(read_entry(node, profiles)?);
            }
        }
        Ok(Self {
            faction,
            categories,
            entries,
        })
    }
}

/// Builds the entry of `node`, whose profiles are already read.
fn read_entry(node: Node, profiles: Vec<RawProfile>) -> Result<RawEntry, BsCatalogueError> {
    let id = attribute(node, "id")?.to_owned();
    let mut category_ids = Vec::new();
    for link in children(node, "categoryLinks").flat_map(|links| children(links, "categoryLink")) {
        category_ids.push(attribute(link, "targetId")?.to_owned());
    }
    let points = read_points(node, &id)?;
    Ok(RawEntry {
        name: attribute(node, "name")?.to_owned(),
        id,
        points,
        category_ids,
        profiles,
    })
}

/// Reads the points cost declared directly on the entry `node`.
fn read_points(node: Node, id: &str) -> Result<Option<u32>, BsCatalogueError> {
    let cost = children(node, "costs")
        .flat_map(|costs| children(costs, "cost"))
        .find(|cost| cost.attribute("typeId") == Some(POINTS_COST_TYPE));
    let Some(cost) = cost else {
        return Ok(None);
    };
    let value = attribute(cost, "value")?;
    let invalid = || BsCatalogueError::InvalidCost {
        entry: id.to_owned(),
        value: value.to_owned(),
    };
    let points: f64 = value.trim().parse().map_err(|_| invalid())?;
    if !points.is_finite() || points < 0.0 || points > f64::from(u32::MAX) {
        return Err(invalid());
    }
    if points.fract() != 0.0 {
        tracing::warn!(entry = id, value, "fractional points cost rounded");
    }
    // safe: range checked above
    Ok(Some(points.round() as u32))
}

/// Reads the profiles declared directly on the entry `node`, not those of nested entries.
fn read_profiles(node: Node) -> Result<Vec<RawProfile>, BsCatalogueError> {
    children(node, "profiles")
        .flat_map(|profiles| children(profiles, "profile"))
        .map(|profile| {
            Ok(RawProfile {
                name: attribute(profile, "name")?.to_owned(),
                type_name: attribute(profile, "typeName")?.to_owned(),
                characteristics: read_characteristics(profile)?,
            })
        })
        .collect()
}

/// Reads the characteristics of `profile`, ignoring its modifiers.
fn read_characteristics(profile: Node) -> Result<Vec<(String, String)>, BsCatalogueError> {
    children(profile, "characteristics")
        .flat_map(|list| children(list, "characteristic"))
        .map(|characteristic| {
            let name = attribute(characteristic, "name")?.to_owned();
            let value = characteristic.text().unwrap_or_default().trim().to_owned();
            Ok((name, value))
        })
        .collect()
}

/// Descendant elements of `node` with the local name `name`.
fn elements<'a, 'input>(
    node: Node<'a, 'input>,
    name: &'static str,
) -> impl Iterator<Item = Node<'a, 'input>> {
    node.descendants()
        .filter(move |child| child.is_element() && child.tag_name().name() == name)
}

/// Child elements of `node` with the local name `name`.
fn children<'a, 'input>(
    node: Node<'a, 'input>,
    name: &'static str,
) -> impl Iterator<Item = Node<'a, 'input>> {
    node.children()
        .filter(move |child| child.is_element() && child.tag_name().name() == name)
}

/// Value of the required attribute `name` of `node`.
fn attribute<'a>(node: Node<'a, '_>, name: &'static str) -> Result<&'a str, BsCatalogueError> {
    node.attribute(name)
        .ok_or_else(|| BsCatalogueError::MissingAttribute {
            element: node.tag_name().name().to_owned(),
            attribute: name,
            position: node.range().start,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CATALOGUE: &str = r#"<?xml version="1.0"?>
<catalogue name="Rebel Fleet" xmlns="http://www.battlescribe.net/schema/catalogueSchema">
  <categoryEntries><categoryEntry id="c1" name="Small Ship"/></categoryEntries>
  <selectionEntries>
    <selectionEntry id="e1" name="Container">
      <selectionEntries>
        <selectionEntry id="e2" name="•Card">
          <profiles>
            <profile name="Card" typeName="2.1 Title">
              <modifiers><modifier type="set" field="x" value="ignored"/></modifiers>
              <characteristics>
                <characteristic name="Title Ability"> Text </characteristic>
                <characteristic name="Setup"/>
              </characteristics>
            </profile>
          </profiles>
          <categoryLinks><categoryLink targetId="c1"/></categoryLinks>
          <costs><cost name="pts" typeId="points" value="6.0"/></costs>
        </selectionEntry>
      </selectionEntries>
    </selectionEntry>
  </selectionEntries>
</catalogue>"#;

    #[test]
    fn reads_nested_entries_ignoring_namespaces() {
        let catalogue = BsCatalogue::parse(CATALOGUE).unwrap();
        assert_eq!(catalogue.faction.as_deref(), Some("Rebel Fleet"));
        assert_eq!(catalogue.categories["c1"], "Small Ship");
        let [entry] = catalogue.entries.as_slice() else {
            panic!("expected one entry, got {:?}", catalogue.entries);
        };
        assert_eq!((entry.id.as_str(), entry.points), ("e2", Some(6)));
        assert_eq!(entry.category_ids, ["c1"]);
        assert_eq!(
            entry.profiles[0].characteristics,
            [
                ("Title Ability".to_owned(), "Text".to_owned()),
                ("Setup".to_owned(), String::new())
            ]
        );
    }

    #[test]
    fn rejects_unknown_root() {
        let err = BsCatalogue::parse("<roster/>").unwrap_err();
        assert!(matches!(err, BsCatalogueError::UnexpectedRoot(root) if root == "roster"));
    }

    #[test]
    fn rejects_invalid_cost() {
        let xml = CATALOGUE.replace("6.0", "-1");
        let err = BsCatalogue::parse(&xml).unwrap_err();
        assert!(matches!(err, BsCatalogueError::InvalidCost { .. }));
    }
}
