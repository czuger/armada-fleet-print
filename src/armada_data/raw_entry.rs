//! Card data as read from BattleScribe XML, before interpretation.

/// Bullet marking unique cards in BattleScribe names.
const UNIQUE_BULLET: char = '•';

/// A `<selectionEntry>` holding at least one profile.
#[derive(Debug, Clone, PartialEq)]
pub struct RawEntry {
    /// BSData id of the entry.
    pub id: String,
    /// Entry name, possibly starting with the unique bullet.
    pub name: String,
    /// Cost in fleet points, when the entry has one.
    pub points: Option<u32>,
    /// Ids of the categories the entry links to.
    pub category_ids: Vec<String>,
    /// Profiles declared directly on the entry, in XML order.
    pub profiles: Vec<RawProfile>,
}

/// A `<profile>` and its characteristics.
#[derive(Debug, Clone, PartialEq)]
pub struct RawProfile {
    /// Profile name, possibly starting with the unique bullet.
    pub name: String,
    /// Profile type, such as `1.1 Ships` or `3.1 Officer`.
    pub type_name: String,
    /// Characteristic names and trimmed values in XML order, empty values included.
    pub characteristics: Vec<(String, String)>,
}

/// Name of a card without its bullet, and whether the card is unique.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardTitle {
    /// Name without the unique bullet.
    pub name: String,
    /// Whether the profile or entry name carries the unique bullet.
    pub unique: bool,
}

impl RawEntry {
    /// Returns the card name of `profile` and its uniqueness. Some entries carry the bullet
    /// while their profile does not (the Mon Karren title), so both names are checked.
    ///
    /// # Arguments
    /// * `profile` - One of this entry's profiles.
    pub fn title(&self, profile: &RawProfile) -> CardTitle {
        let (name, profile_unique) = strip_bullet(&profile.name);
        let (_, entry_unique) = strip_bullet(&self.name);
        CardTitle {
            name: name.to_owned(),
            unique: profile_unique || entry_unique,
        }
    }
}

/// Splits the unique bullet off `name`.
fn strip_bullet(name: &str) -> (&str, bool) {
    let name = name.trim();
    match name.strip_prefix(UNIQUE_BULLET) {
        Some(rest) => (rest.trim_start(), true),
        None => (name, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, profile_name: &str) -> (RawEntry, RawProfile) {
        let profile = RawProfile {
            name: profile_name.to_owned(),
            type_name: "2.1 Title".to_owned(),
            characteristics: Vec::new(),
        };
        let entry = RawEntry {
            id: "id".to_owned(),
            name: name.to_owned(),
            points: None,
            category_ids: Vec::new(),
            profiles: vec![profile.clone()],
        };
        (entry, profile)
    }

    #[test]
    fn unique_from_entry_name() {
        let (entry, profile) = entry("•Mon Karren", "Mon Karren");
        let title = entry.title(&profile);
        assert_eq!((title.name.as_str(), title.unique), ("Mon Karren", true));
    }

    #[test]
    fn unique_from_profile_name() {
        let (entry, profile) = entry("Admonition", "• Admonition");
        let title = entry.title(&profile);
        assert_eq!((title.name.as_str(), title.unique), ("Admonition", true));
    }

    #[test]
    fn not_unique_without_bullet() {
        let (entry, profile) = entry("Task Force Antilles", "Task Force Antilles");
        assert!(!entry.title(&profile).unique);
    }
}
