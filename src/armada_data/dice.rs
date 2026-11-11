//! Attack dice pools such as `2 Red, 1 Black`.

use serde::Serialize;

/// An attack dice pool, with the text it was read from.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Dice {
    /// Red dice.
    pub red: u8,
    /// Blue dice.
    pub blue: u8,
    /// Black dice.
    pub black: u8,
    /// Text as written in BSData.
    pub raw: String,
}

impl Dice {
    /// Parses a dice pool. Commas are optional, a color without a count means one die, and
    /// `-` or `None` mean no dice. Unknown words are logged and skipped.
    ///
    /// # Arguments
    /// * `raw` - Free text such as `2 Red, 1 Black`, `2 Red 1 Blue` or `Blue, Black`.
    pub fn parse(raw: &str) -> Self {
        let mut dice = Self {
            raw: raw.to_owned(),
            ..Self::default()
        };
        let mut count = None;
        let tokens = raw.split(|c: char| c == ',' || c.is_whitespace());
        for token in tokens.filter(|token| !token.is_empty()) {
            if let Ok(number) = token.parse::<u8>() {
                count = Some(number);
                continue;
            }
            let die = match token.to_ascii_lowercase().as_str() {
                "red" => &mut dice.red,
                "blue" => &mut dice.blue,
                "black" => &mut dice.black,
                "-" | "--" | "none" => continue,
                _ => {
                    tracing::warn!(raw, token, "unknown word in dice pool skipped");
                    continue;
                }
            };
            *die = die.saturating_add(count.take().unwrap_or(1));
        }
        dice
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(raw: &str) -> (u8, u8, u8) {
        let dice = Dice::parse(raw);
        assert_eq!(dice.raw, raw);
        (dice.red, dice.blue, dice.black)
    }

    #[test]
    fn parses_counted_colors() {
        assert_eq!(counts("2 Red, 1 Black"), (2, 0, 1));
        assert_eq!(counts("1 Red, 1 Blue, 3 Black"), (1, 1, 3));
    }

    #[test]
    fn parses_without_commas() {
        assert_eq!(counts("2 Red 1 Blue"), (2, 1, 0));
    }

    #[test]
    fn color_without_count_is_one_die() {
        assert_eq!(counts("Blue, Black"), (0, 1, 1));
    }

    #[test]
    fn dash_and_none_are_empty() {
        assert_eq!(counts("-"), (0, 0, 0));
        assert_eq!(counts("None"), (0, 0, 0));
    }
}
