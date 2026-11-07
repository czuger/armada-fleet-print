//! Ship speed charts: the yaw allowed at each joint of the maneuver tool, per speed.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::armada_data::raw_entry::RawProfile;

/// Highest speed a chart can hold.
const MAX_SPEED: u8 = 4;

/// Yaw values by speed: `chart[speed][joint]` for joints `0..speed`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct SpeedChart(BTreeMap<u8, Vec<u8>>);

impl SpeedChart {
    /// Builds the chart from the `1.9 Speed Chart` profiles of a ship.
    ///
    /// Each profile is one joint; its name is made of spaces, four for the first joint down to
    /// one for the fourth. The XML order is not reliable (reversed for the Acclamator), so
    /// profiles are sorted by decreasing name length. Their characteristics `1` to `4` hold the
    /// yaw at each speed; `-` means 0 and an empty value means the speed is not available.
    ///
    /// # Arguments
    /// * `joints` - The speed chart profiles, in any order.
    pub fn from_profiles(joints: &[&RawProfile]) -> Self {
        let mut joints = joints.to_vec();
        joints.sort_by_key(|joint| std::cmp::Reverse(joint.name.len()));
        let chart = (1..=MAX_SPEED)
            .filter_map(|speed| speed_row(&joints, speed).map(|row| (speed, row)))
            .collect();
        Self(chart)
    }

    /// Returns the yaw at each joint for `speed`, if the ship can move at that speed.
    ///
    /// # Arguments
    /// * `speed` - Speed from 1 to 4.
    pub fn yaws(&self, speed: u8) -> Option<&[u8]> {
        self.0.get(&speed).map(Vec::as_slice)
    }

    /// Returns whether the chart has no speed at all.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Yaw values of the first `speed` joints at `speed`, or `None` if every cell is empty.
fn speed_row(joints: &[&RawProfile], speed: u8) -> Option<Vec<u8>> {
    let key = speed.to_string();
    let cells: Vec<Option<u8>> = (0..usize::from(speed))
        .map(|joint| {
            let value = joints.get(joint).and_then(|profile| {
                profile
                    .characteristics
                    .iter()
                    .find(|(name, _)| *name == key)
                    .map(|(_, value)| value.as_str())
            });
            yaw(value.unwrap_or_default())
        })
        .collect();
    if cells.iter().all(Option::is_none) {
        return None;
    }
    Some(cells.into_iter().map(Option::unwrap_or_default).collect())
}

/// Reads one chart cell: `None` when empty, 0 for `-`.
fn yaw(value: &str) -> Option<u8> {
    match value {
        "" => None,
        "-" => Some(0),
        _ => Some(value.parse().unwrap_or_else(|_| {
            tracing::warn!(value, "unreadable speed chart cell read as 0");
            0
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn joint(name: &str, cells: [&str; 4]) -> RawProfile {
        RawProfile {
            name: name.to_owned(),
            type_name: "1.9 Speed Chart".to_owned(),
            characteristics: ["1", "2", "3", "4"]
                .into_iter()
                .zip(cells)
                .map(|(speed, value)| (speed.to_owned(), value.to_owned()))
                .chain([("|".to_owned(), "|".to_owned())])
                .collect(),
        }
    }

    #[test]
    fn sorts_joints_by_name_length() {
        let first = joint("    ", ["2", "1", "-", ""]);
        let second = joint("   ", ["", "1", "1", ""]);
        let third = joint("  ", ["", "", "1", ""]);
        let fourth = joint(" ", ["", "", "", ""]);
        let chart = SpeedChart::from_profiles(&[&fourth, &third, &first, &second]);
        assert_eq!(chart.yaws(1), Some([2].as_slice()));
        assert_eq!(chart.yaws(2), Some([1, 1].as_slice()));
        assert_eq!(chart.yaws(3), Some([0, 1, 1].as_slice()));
        assert_eq!(chart.yaws(4), None);
    }

    #[test]
    fn dash_only_speed_is_kept() {
        let first = joint("    ", ["-", "-", "", ""]);
        let second = joint("   ", ["", "-", "", ""]);
        let chart = SpeedChart::from_profiles(&[&first, &second]);
        assert_eq!(chart.yaws(1), Some([0].as_slice()));
        assert_eq!(chart.yaws(2), Some([0, 0].as_slice()));
        assert_eq!(chart.yaws(3), None);
    }
}
