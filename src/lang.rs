//! Language of the interface and of the generated PDF text.

use std::fmt;
use std::str::FromStr;

/// Language of the text shown to users. Card images are never translated.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Lang {
    /// French, the default.
    #[default]
    Fr,
    /// English.
    En,
}

/// Error returned when parsing a code that is not `fr` or `en`.
#[derive(Debug, thiserror::Error)]
#[error("unknown language {0:?}, expected fr or en")]
pub struct LangParseError(String);

impl Lang {
    /// Every supported language, in the order of the language switch.
    pub const ALL: [Self; 2] = [Self::Fr, Self::En];

    /// ISO 639-1 code, used in cookies, links, the `lang` HTML attribute and on the command line.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fr => "fr",
            Self::En => "en",
        }
    }
}

impl fmt::Display for Lang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Lang {
    type Err = LangParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fr" => Ok(Self::Fr),
            "en" => Ok(Self::En),
            _ => Err(LangParseError(s.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_its_own_codes() {
        for lang in Lang::ALL {
            assert_eq!(lang.as_str().parse::<Lang>().unwrap(), lang);
        }
        assert!("de".parse::<Lang>().is_err());
        assert!("EN".parse::<Lang>().is_err());
    }
}
