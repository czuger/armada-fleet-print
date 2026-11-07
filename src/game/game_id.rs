//! Secret identifiers of games.

use std::fmt;
use std::str::FromStr;

/// Random bytes in an identifier: 128 bits, too many to guess.
const ID_BYTES: usize = 16;

/// Identifier of a game, and the secret granting access to it: 32 lowercase hex digits.
///
/// Parsing only accepts that shape, so an identifier is always a safe file name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameId(String);

/// Error returned when text is not a game identifier.
#[derive(Debug, thiserror::Error)]
#[error("invalid game identifier")]
pub struct GameIdError;

impl GameId {
    /// Draws a new identifier from the operating system's random generator.
    ///
    /// # Errors
    /// Returns [`getrandom::Error`] if the generator is unavailable.
    pub fn generate() -> Result<Self, getrandom::Error> {
        let mut bytes = [0_u8; ID_BYTES];
        getrandom::fill(&mut bytes)?;
        Ok(Self(
            bytes.iter().map(|byte| format!("{byte:02x}")).collect(),
        ))
    }

    /// Returns the identifier as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for GameId {
    type Err = GameIdError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let valid = text.len() == ID_BYTES * 2
            && text
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
        if valid {
            Ok(Self(text.to_owned()))
        } else {
            Err(GameIdError)
        }
    }
}

impl fmt::Display for GameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_ids_parse_back() {
        let id = GameId::generate().unwrap();
        assert_eq!(id.as_str().parse::<GameId>().unwrap(), id);
        assert_ne!(GameId::generate().unwrap(), id);
    }

    #[test]
    fn rejects_anything_but_32_lowercase_hex_digits() {
        for text in [
            "",
            "../../etc/passwd",
            "0123456789ABCDEF0123456789abcdef",
            "0123456789abcdef0123456789abcde",
            "0123456789abcdef0123456789abcdef0",
            "0123456789abcdef0123456789abcde/",
        ] {
            assert!(text.parse::<GameId>().is_err(), "{text:?} accepted");
        }
    }
}
