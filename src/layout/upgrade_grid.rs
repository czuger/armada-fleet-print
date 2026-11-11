//! Grid of the upgrade cards printed to the right of their ship.

/// Height-to-width ratio at which upgrade cards are drawn.
const UPGRADE_ASPECT: f32 = 1.5;
/// Space between two upgrades, and between the ship and its first upgrade column.
pub(crate) const UPGRADE_GAP: f32 = 10.0;

/// Card size and number of columns of the upgrades of one ship.
#[derive(Debug, PartialEq)]
pub(crate) struct UpgradeGrid {
    count: usize,
    columns: usize,
    /// Width of each upgrade card.
    pub card_width: f32,
    /// Height of each upgrade card.
    pub card_height: f32,
}

impl UpgradeGrid {
    /// Picks the column count that draws `count` upgrades largest, no wider than `max_width`,
    /// inside an `area_width` × `area_height` area. Ties go to more columns, so rows stay full.
    pub(crate) fn fit(count: usize, max_width: f32, area_width: f32, area_height: f32) -> Self {
        let mut best = Self {
            count,
            columns: 1,
            card_width: 0.0,
            card_height: 0.0,
        };
        for columns in 1..=count {
            let rows = count.div_ceil(columns);
            let card_width = max_width
                .min(span(area_width, columns))
                .min(span(area_height, rows) / UPGRADE_ASPECT);
            if card_width >= best.card_width {
                best = Self {
                    count,
                    columns,
                    card_width,
                    card_height: card_width * UPGRADE_ASPECT,
                };
            }
        }
        best
    }

    /// Height of all the rows, 0 without upgrades.
    pub(crate) fn height(&self) -> f32 {
        let rows = self.count.div_ceil(self.columns) as f32;
        (rows * (self.card_height + UPGRADE_GAP) - UPGRADE_GAP).max(0.0)
    }

    /// Offset of the top-left corner of upgrade `index` from the top-left corner of the grid,
    /// rightwards and downwards.
    pub(crate) fn offset(&self, index: usize) -> (f32, f32) {
        let column = (index % self.columns) as f32;
        let row = (index / self.columns) as f32;
        (
            column * (self.card_width + UPGRADE_GAP),
            row * (self.card_height + UPGRADE_GAP),
        )
    }
}

/// Size of each of `count` cells splitting `length` with gaps between them.
fn span(length: f32, count: usize) -> f32 {
    let count = count as f32;
    (length - (count - 1.0) * UPGRADE_GAP) / count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_largest_size_with_full_rows() {
        let grid = UpgradeGrid::fit(4, 100.0, 336.0, 720.0);
        assert_eq!((grid.columns, grid.card_width), (3, 100.0));
        assert_eq!(grid.offset(3), (0.0, 160.0));
        assert_eq!(grid.height(), 310.0);
    }

    #[test]
    fn shrinks_cards_that_would_not_fit() {
        let grid = UpgradeGrid::fit(6, 157.0, 405.0, 540.0);
        assert_eq!(grid.columns, 3);
        assert!((grid.card_width - 385.0 / 3.0).abs() < 1e-3);
    }

    #[test]
    fn is_empty_without_upgrades() {
        assert_eq!(UpgradeGrid::fit(0, 100.0, 336.0, 720.0).height(), 0.0);
    }
}
