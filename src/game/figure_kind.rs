//! Kinds of miniatures counted in a game.

use serde::{Deserialize, Serialize};

/// Kind of miniature: a ship or a squadron stand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FigureKind {
    /// A ship, one miniature per ship card.
    Ship,
    /// A squadron, one miniature per squadron in the fleet.
    Squadron,
}

impl FigureKind {
    /// Reads the "Type" column of a stock sheet, in English or French, ignoring case.
    pub fn from_sheet(label: &str) -> Option<Self> {
        match label.trim().to_lowercase().as_str() {
            "ship" | "ships" | "vaisseau" | "vaisseaux" => Some(Self::Ship),
            "squadron" | "squadrons" | "escadron" | "escadrons" | "escadrille" | "escadrilles" => {
                Some(Self::Squadron)
            }
            _ => None,
        }
    }

    /// Returns the label written back into a stock sheet.
    pub fn sheet_label(self) -> &'static str {
        match self {
            Self::Ship => "Ship",
            Self::Squadron => "Squadron",
        }
    }

    /// Returns the French plural name shown to users.
    pub fn plural_label(self) -> &'static str {
        match self {
            Self::Ship => "Vaisseaux",
            Self::Squadron => "Escadrons",
        }
    }
}
