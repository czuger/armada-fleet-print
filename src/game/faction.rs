//! Playable factions.

/// A faction of the fleet builder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Faction {
    /// Rebel Alliance.
    Rebel,
    /// Galactic Empire.
    Imperial,
    /// Galactic Republic.
    Republic,
    /// Separatist Alliance.
    Separatist,
}

/// Prefixes of the names a stock sheet may use for each faction, in English or French.
const SHEET_PREFIXES: [(Faction, &[&str]); 4] = [
    (Faction::Rebel, &["rebel", "rébel"]),
    (
        Faction::Imperial,
        &["empire", "imperial", "impérial", "galactic empire"],
    ),
    (
        Faction::Republic,
        &["republic", "républi", "republi", "galactic republic"],
    ),
    (
        Faction::Separatist,
        &["separatist", "séparatist", "cis", "confedera"],
    ),
];

impl Faction {
    /// Reads the `faction` field of the fleet builder data (`rebel`, `imperial`, ...).
    pub fn from_builder(code: &str) -> Option<Self> {
        match code {
            "rebel" => Some(Self::Rebel),
            "imperial" => Some(Self::Imperial),
            "republic" => Some(Self::Republic),
            "separatist" => Some(Self::Separatist),
            _ => None,
        }
    }

    /// Reads the "Faction" column of a stock sheet, such as `Rebellion` or `Empire`.
    pub fn from_sheet(label: &str) -> Option<Self> {
        let label = label.trim().to_lowercase();
        SHEET_PREFIXES
            .iter()
            .find(|(_, prefixes)| prefixes.iter().any(|prefix| label.starts_with(prefix)))
            .map(|(faction, _)| *faction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_sheet_labels() {
        assert_eq!(Faction::from_sheet("Rebellion"), Some(Faction::Rebel));
        assert_eq!(Faction::from_sheet("Empire"), Some(Faction::Imperial));
        assert_eq!(Faction::from_sheet("République"), Some(Faction::Republic));
        assert_eq!(
            Faction::from_sheet("Séparatistes"),
            Some(Faction::Separatist)
        );
        assert_eq!(Faction::from_sheet("Pirates"), None);
    }
}
