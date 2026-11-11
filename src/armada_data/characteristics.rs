//! Non-empty characteristics of a profile, consumed field by field.

use std::collections::BTreeMap;

use indexmap::IndexMap;

use crate::armada_data::dice::Dice;
use crate::armada_data::raw_entry::RawProfile;

/// The non-empty characteristics of a profile. Each `take_*` call removes what it reads, so
/// whatever is left at the end is unknown or unreadable and goes to an `other` map.
#[derive(Debug, Clone, Default)]
pub(crate) struct Characteristics(IndexMap<String, String>);

impl Characteristics {
    /// Collects the non-empty characteristics of `profile`.
    pub(crate) fn from_profile(profile: &RawProfile) -> Self {
        let values = profile
            .characteristics
            .iter()
            .filter(|(_, value)| !value.is_empty())
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect();
        Self(values)
    }

    /// Removes and returns the value of `name`.
    pub(crate) fn take(&mut self, name: &str) -> Option<String> {
        self.0.shift_remove(name)
    }

    /// Removes and returns the value of the first characteristic whose name ends with `suffix`.
    pub(crate) fn take_ending_with(&mut self, suffix: &str) -> Option<String> {
        let name = self.names_ending_with(suffix).into_iter().next()?;
        self.take(&name)
    }

    /// Removes and returns `name` as a number. An unreadable value stays in place.
    pub(crate) fn take_number(&mut self, name: &str) -> Option<u32> {
        let number = self.0.get(name)?.parse().ok()?;
        self.0.shift_remove(name);
        Some(number)
    }

    /// Removes and returns `name` as a comma-separated list.
    pub(crate) fn take_list(&mut self, name: &str) -> Vec<String> {
        self.take(name)
            .map(|value| split_list(&value).map(str::to_owned).collect())
            .unwrap_or_default()
    }

    /// Removes and returns `name` as defense tokens, expanding counts: `2 Brace` gives two
    /// `Brace` tokens.
    pub(crate) fn take_defense_tokens(&mut self, name: &str) -> Vec<String> {
        let Some(value) = self.take(name) else {
            return Vec::new();
        };
        split_list(&value)
            .flat_map(|item| {
                let (count, token) = counted(item);
                std::iter::repeat_n(token.to_owned(), count)
            })
            .collect()
    }

    /// Removes and returns `name` as a dice pool.
    pub(crate) fn take_dice(&mut self, name: &str) -> Option<Dice> {
        self.take(name).map(|value| Dice::parse(&value))
    }

    /// Names of the remaining characteristics ending with `suffix`, in XML order.
    pub(crate) fn names_ending_with(&self, suffix: &str) -> Vec<String> {
        self.0
            .keys()
            .filter(|name| name.ends_with(suffix))
            .cloned()
            .collect()
    }

    /// Returns the characteristics not taken yet.
    pub(crate) fn into_other(self) -> BTreeMap<String, String> {
        self.0.into_iter().collect()
    }
}

/// Splits an optional leading count off `item`: `2 Brace` gives `(2, "Brace")`.
fn counted(item: &str) -> (usize, &str) {
    item.split_once(' ')
        .and_then(|(count, token)| Some((count.parse().ok()?, token.trim())))
        .unwrap_or((1, item))
}

/// Trimmed, non-empty items of a comma-separated list.
fn split_list(value: &str) -> impl Iterator<Item = &str> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn characteristics(values: &[(&str, &str)]) -> Characteristics {
        Characteristics::from_profile(&RawProfile {
            name: String::new(),
            type_name: String::new(),
            characteristics: values
                .iter()
                .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
                .collect(),
        })
    }

    #[test]
    fn expands_counted_defense_tokens() {
        let mut values = characteristics(&[("Defense Tokens", "2 Brace, Scatter")]);
        assert_eq!(
            values.take_defense_tokens("Defense Tokens"),
            ["Brace", "Brace", "Scatter"]
        );
    }

    #[test]
    fn unreadable_number_stays_in_other() {
        let mut values = characteristics(&[("Hull", "?"), ("Keywords", "")]);
        assert_eq!(values.take_number("Hull"), None);
        assert_eq!(values.take_list("Keywords"), Vec::<String>::new());
        let other = values.into_other();
        assert_eq!(other.get("Hull").map(String::as_str), Some("?"));
        assert!(!other.contains_key("Keywords"));
    }
}
