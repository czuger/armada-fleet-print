//! Validated card image file names.

use std::fmt;

use serde::Deserialize;

/// File name of a card image on the fleet builder, safe to use as a local path component.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
#[serde(try_from = "String")]
pub struct CardName(String);

/// Error returned when a card image name is not a plain file name.
#[derive(Debug, thiserror::Error)]
#[error("invalid card image name {0:?}")]
pub struct CardNameError(String);

impl CardName {
    /// Validates `name` as a card image file name.
    ///
    /// # Errors
    /// Returns [`CardNameError`] if `name` is empty, starts with a dot, or contains anything
    /// other than ASCII letters, digits, `-`, `_` and `.`.
    pub fn new(name: &str) -> Result<Self, CardNameError> {
        let allowed = |c: char| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.');
        if name.is_empty() || name.starts_with('.') || !name.chars().all(allowed) {
            return Err(CardNameError(name.to_owned()));
        }
        Ok(Self(name.to_owned()))
    }

    /// Returns the file name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for CardName {
    type Error = CardNameError;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        Self::new(&name)
    }
}

impl fmt::Display for CardName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_plain_file_names() {
        let name = CardName::new("swm19-gr-75-combat-retrofits.png").unwrap();
        assert_eq!(name.as_str(), "swm19-gr-75-combat-retrofits.png");
    }

    #[test]
    fn rejects_paths_and_hidden_files() {
        for name in ["", "../secret", "a/b.png", ".hidden", "a\\b.png", "é.png"] {
            assert!(CardName::new(name).is_err(), "{name:?} should be rejected");
        }
    }
}
