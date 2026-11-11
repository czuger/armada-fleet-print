//! Orientation of the printed US Letter pages.

use std::fmt;
use std::str::FromStr;

use serde::Deserialize;

/// Short side of a US Letter page, in points.
const LETTER_SHORT_SIDE: f32 = 612.0;
/// Long side of a US Letter page, in points.
const LETTER_LONG_SIDE: f32 = 792.0;

/// Whether pages are printed upright or sideways.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    /// 612 × 792 pt pages.
    #[default]
    Portrait,
    /// 792 × 612 pt pages.
    Landscape,
}

/// Error returned when parsing a name that is not `portrait` or `landscape`.
#[derive(Debug, thiserror::Error)]
#[error("unknown orientation {0:?}, expected portrait or landscape")]
pub struct OrientationParseError(String);

impl Orientation {
    /// Width and height of a page, in points.
    pub fn page_size(self) -> (f32, f32) {
        match self {
            Self::Portrait => (LETTER_SHORT_SIDE, LETTER_LONG_SIDE),
            Self::Landscape => (LETTER_LONG_SIDE, LETTER_SHORT_SIDE),
        }
    }

    /// Name used in forms, cookies and on the command line.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Portrait => "portrait",
            Self::Landscape => "landscape",
        }
    }
}

impl fmt::Display for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Orientation {
    type Err = OrientationParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "portrait" => Ok(Self::Portrait),
            "landscape" => Ok(Self::Landscape),
            _ => Err(OrientationParseError(s.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_its_own_names() {
        for orientation in [Orientation::Portrait, Orientation::Landscape] {
            assert_eq!(
                orientation.as_str().parse::<Orientation>().unwrap(),
                orientation
            );
        }
        assert!("paysage".parse::<Orientation>().is_err());
    }

    #[test]
    fn landscape_swaps_the_sides() {
        assert_eq!(Orientation::Portrait.page_size(), (612.0, 792.0));
        assert_eq!(Orientation::Landscape.page_size(), (792.0, 612.0));
    }
}
