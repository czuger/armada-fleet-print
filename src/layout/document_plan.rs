//! What to draw on each page. Coordinates are PDF points from the bottom-left corner of the page.

use crate::card_name::CardName;
use crate::layout::orientation::Orientation;

/// Every page of the printed document, in order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DocumentPlan {
    /// Orientation of every page, which gives their size.
    pub orientation: Orientation,
    /// The pages, never empty once produced by [`crate::layout::lay_out`].
    pub pages: Vec<Page>,
}

/// One printed page.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Page {
    /// Elements in drawing order.
    pub elements: Vec<Element>,
}

/// Something drawn on a page.
#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    /// A card image scaled to `width` × `height`, with its bottom-left corner at (`x`, `y`).
    Card {
        /// Image to draw.
        card: CardName,
        /// Left edge.
        x: f32,
        /// Bottom edge.
        y: f32,
        /// Drawn width.
        width: f32,
        /// Drawn height.
        height: f32,
    },
    /// A white disc showing how many copies of a squadron the fleet has.
    CountBadge {
        /// Number shown.
        count: u32,
        /// Horizontal center of the disc.
        center_x: f32,
        /// Vertical center of the disc.
        center_y: f32,
        /// Disc radius.
        radius: f32,
        /// Font size of the number.
        font_size: f32,
    },
    /// A single line of Helvetica text.
    Text {
        /// Text to show.
        text: String,
        /// Left edge for [`TextAlign::Left`], right edge for [`TextAlign::Right`].
        x: f32,
        /// Baseline position.
        baseline: f32,
        /// Font size.
        size: f32,
        /// Which edge `x` refers to.
        align: TextAlign,
    },
}

/// Horizontal anchoring of a text line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    /// The text starts at its `x`.
    Left,
    /// The text ends at its `x`.
    Right,
}
