//! Card kind derived from a BattleScribe profile type such as `1.1 Ships` or `3.1 Officer`.

/// Profile types describing ships.
const SHIP_KINDS: [&str; 4] = [
    "Ships",
    "Ships With Special Battery",
    "Huge Ships",
    "Stations",
];
/// Prefix of the dual upgrade type label, dropped from the stored type.
const DUAL_TYPE_PREFIX: &str = "Dual Type Upgrade - ";
/// Suffix of objective type labels, dropped from the stored kind.
const OBJECTIVE_SUFFIX: &str = " Objective";

/// What a profile describes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardKind {
    /// A ship or station; holds the kind label, such as `Huge Ships`.
    Ship(String),
    /// A squadron.
    Squadron,
    /// One row of a ship speed chart.
    SpeedChart,
    /// An upgrade; holds its type, such as `Officer` or `Weapons Team & Offensive Retrofit`.
    Upgrade(String),
    /// An objective; holds its kind, such as `Assault`.
    Objective(String),
    /// Any other profile type.
    Other,
}

impl CardKind {
    /// Classifies a profile type.
    ///
    /// # Arguments
    /// * `type_name` - Profile `typeName`, a section number followed by a label.
    pub fn from_type_name(type_name: &str) -> Self {
        let Some((number, label)) = type_name.trim().split_once(' ') else {
            return Self::Other;
        };
        let section = number.split('.').next().unwrap_or_default();
        match section {
            "1" if label == "Squadrons" => Self::Squadron,
            "1" if label == "Speed Chart" => Self::SpeedChart,
            "1" if SHIP_KINDS.contains(&label) => Self::Ship(label.to_owned()),
            "2" | "3" | "4" | "5" | "6" | "7" => {
                let label = label.strip_prefix(DUAL_TYPE_PREFIX).unwrap_or(label);
                Self::Upgrade(label.to_owned())
            }
            "8" => {
                let label = label.strip_suffix(OBJECTIVE_SUFFIX).unwrap_or(label);
                Self::Objective(label.to_owned())
            }
            _ => Self::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_profile_types() {
        let cases = [
            ("1.1 Ships", CardKind::Ship("Ships".to_owned())),
            ("1.3 Huge Ships", CardKind::Ship("Huge Ships".to_owned())),
            ("1.5 Stations", CardKind::Ship("Stations".to_owned())),
            ("1.4 Squadrons", CardKind::Squadron),
            ("1.9 Speed Chart", CardKind::SpeedChart),
            ("2.0 Commander", CardKind::Upgrade("Commander".to_owned())),
            (
                "5.1 Offensive Retrofit",
                CardKind::Upgrade("Offensive Retrofit".to_owned()),
            ),
            (
                "7.1 Dual Type Upgrade - Weapons Team & Offensive Retrofit",
                CardKind::Upgrade("Weapons Team & Offensive Retrofit".to_owned()),
            ),
            (
                "8.4 Campaign Objective",
                CardKind::Objective("Campaign".to_owned()),
            ),
            ("9.1 Something", CardKind::Other),
            ("1.7 Unknown", CardKind::Other),
        ];
        for (type_name, expected) in cases {
            assert_eq!(CardKind::from_type_name(type_name), expected, "{type_name}");
        }
    }
}
