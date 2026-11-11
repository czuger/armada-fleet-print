//! Fleet layout: ships with their upgrades beside them, then squadrons and objectives, packed
//! left to right in lines that keep the builder's order.

use std::collections::HashMap;
use std::mem;

use crate::card_name::CardName;
use crate::fleet::{Fleet, Ship, SquadronGroup};
use crate::lang::Lang;
use crate::layout::document_plan::{DocumentPlan, Element, Page, TextAlign};
use crate::layout::orientation::Orientation;
use crate::layout::shelf::Shelf;
use crate::layout::upgrade_grid::{UPGRADE_GAP, UpgradeGrid};

/// Margin on every side of the page (Prawn's default).
const MARGIN: f32 = 36.0;

const COLUMN_GAP: f32 = 3.0;
const ROW_GAP: f32 = 10.0;

/// Width of ship cards in portrait.
const SHIP_WIDTH: f32 = 200.0;
/// Width of upgrade cards beside a [`SHIP_WIDTH`] ship; they scale with the ship.
const UPGRADE_WIDTH: f32 = 100.0;

/// Width of squadron and objective cards.
const CARD_WIDTH: f32 = 180.0;

const BADGE_OFFSET_X: f32 = 160.0;
const BADGE_OFFSET_Y: f32 = 22.0;
const BADGE_RADIUS: f32 = 15.0;
const BADGE_FONT_SIZE: f32 = 24.0;

const FOOTER_FONT_SIZE: f32 = 12.0;
/// Prawn put the top of the footer text on the bottom margin, so the baseline sits one
/// Helvetica ascent (0.718 em) below it.
const FOOTER_BASELINE: f32 = MARGIN - 0.718 * FOOTER_FONT_SIZE;
const FOOTER_SITE: &str = "https://apps.ieroe.com/armada/";

/// Error returned when the layout needs the size of an image it was not given.
#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    /// No aspect ratio was provided for this card.
    #[error("no image size known for card {0}")]
    UnknownCard(CardName),
}

/// Size of the area inside the margins and how many small cards fit on one of its lines.
#[derive(Debug, Clone, Copy)]
struct Geometry {
    orientation: Orientation,
    content_width: f32,
    content_height: f32,
    /// Squadron or objective cards per line.
    cards_per_line: usize,
}

impl Geometry {
    fn new(orientation: Orientation) -> Self {
        let (width, height) = orientation.page_size();
        let content_width = width - 2.0 * MARGIN;
        // Allow a few points of the right margin, as in the original, where three cards ran
        // 6 pt into it in portrait; four cards run 9 pt into it in landscape.
        let cards_per_line =
            ((content_width + MARGIN / 2.0 + COLUMN_GAP) / (CARD_WIDTH + COLUMN_GAP)) as usize;
        Self {
            orientation,
            content_width,
            content_height: height - 2.0 * MARGIN,
            cards_per_line,
        }
    }

    /// Width of a line of the shelf: exactly `cards_per_line` small cards.
    fn shelf_width(self) -> f32 {
        let cards = self.cards_per_line as f32;
        cards * CARD_WIDTH + (cards - 1.0) * COLUMN_GAP
    }

    /// Width and height of a ship card with height-to-width `ratio`: a fixed width in
    /// portrait, the whole content height in landscape.
    fn ship_size(self, ratio: f32) -> (f32, f32) {
        match self.orientation {
            Orientation::Portrait => (SHIP_WIDTH, SHIP_WIDTH * ratio),
            Orientation::Landscape => (self.content_height / ratio, self.content_height),
        }
    }
}

/// Lays out every card of `fleet`, with a footer on each page.
///
/// # Arguments
/// * `fleet` - The fleet to print.
/// * `aspect_ratios` - Height-to-width ratio of each card image of the fleet.
/// * `orientation` - Orientation of the pages.
/// * `lang` - Language of the footer.
///
/// # Errors
/// Returns [`LayoutError::UnknownCard`] if a card has no entry in `aspect_ratios`.
pub fn lay_out(
    fleet: &Fleet,
    aspect_ratios: &HashMap<CardName, f32>,
    orientation: Orientation,
    lang: Lang,
) -> Result<DocumentPlan, LayoutError> {
    let mut builder = PlanBuilder::new(aspect_ratios, Geometry::new(orientation));
    for ship in &fleet.ships {
        builder.ship(ship)?;
    }
    for group in &fleet.squadrons {
        builder.squadron(group)?;
    }
    // Objectives get a page of their own; `finish` drops the page left empty when there are none.
    builder.page_break();
    for card in &fleet.objectives {
        builder.objective(card)?;
    }
    Ok(builder.finish(&footer_label(fleet), lang))
}

/// Accumulates pages while walking through the fleet.
struct PlanBuilder<'a> {
    aspect_ratios: &'a HashMap<CardName, f32>,
    geometry: Geometry,
    done: Vec<Page>,
    current: Page,
    shelf: Shelf,
}

impl<'a> PlanBuilder<'a> {
    fn new(aspect_ratios: &'a HashMap<CardName, f32>, geometry: Geometry) -> Self {
        Self {
            aspect_ratios,
            geometry,
            done: Vec::new(),
            current: Page::default(),
            shelf: Shelf::new(
                geometry.shelf_width(),
                geometry.content_height,
                COLUMN_GAP,
                ROW_GAP,
            ),
        }
    }

    fn new_page(&mut self) {
        self.done.push(mem::take(&mut self.current));
    }

    /// Ends the current page, so the next block goes at the top of a new one.
    fn page_break(&mut self) {
        self.shelf.break_page();
        self.new_page();
    }

    /// Reserves room for a `width` × `height` block and returns its top-left corner.
    fn slot(&mut self, width: f32, height: f32) -> (f32, f32) {
        let slot = self.shelf.fit(width, height);
        if slot.new_page {
            self.new_page();
        }
        (slot.left, slot.top)
    }

    /// Lays out a ship with its upgrades in rows to its right. A ship with upgrades takes a
    /// whole line; its upgrades scale with it, and shrink only when they would not fit.
    fn ship(&mut self, ship: &Ship) -> Result<(), LayoutError> {
        let (width, height) = self.geometry.ship_size(self.aspect_ratio(&ship.card)?);
        let upgrades_left = width + UPGRADE_GAP;
        let shelf_width = self.geometry.shelf_width();
        let grid = UpgradeGrid::fit(
            ship.upgrades.len(),
            UPGRADE_WIDTH * width / SHIP_WIDTH,
            shelf_width - upgrades_left,
            self.geometry.content_height,
        );
        let block_width = if ship.upgrades.is_empty() {
            width
        } else {
            shelf_width
        };
        let (left, top) = self.slot(block_width, height.max(grid.height()));
        self.place(&ship.card, left, top, width, height);
        for (index, upgrade) in ship.upgrades.iter().enumerate() {
            let (dx, dy) = grid.offset(index);
            self.place(
                upgrade,
                left + upgrades_left + dx,
                top - dy,
                grid.card_width,
                grid.card_height,
            );
        }
        Ok(())
    }

    /// Lays out a squadron card with its count badge.
    fn squadron(&mut self, group: &SquadronGroup) -> Result<(), LayoutError> {
        let (left, top) = self.small_card(&group.card)?;
        self.current.elements.push(Element::CountBadge {
            count: group.count,
            center_x: MARGIN + left + BADGE_OFFSET_X,
            center_y: MARGIN + top - BADGE_OFFSET_Y,
            radius: BADGE_RADIUS,
            font_size: BADGE_FONT_SIZE,
        });
        Ok(())
    }

    fn objective(&mut self, card: &CardName) -> Result<(), LayoutError> {
        self.small_card(card).map(|_| ())
    }

    /// Places a squadron or objective card and returns its top-left corner.
    fn small_card(&mut self, card: &CardName) -> Result<(f32, f32), LayoutError> {
        let height = self.height_at(card, CARD_WIDTH)?;
        let (left, top) = self.slot(CARD_WIDTH, height);
        self.place(card, left, top, CARD_WIDTH, height);
        Ok((left, top))
    }

    fn height_at(&self, card: &CardName, width: f32) -> Result<f32, LayoutError> {
        self.aspect_ratio(card).map(|ratio| width * ratio)
    }

    fn aspect_ratio(&self, card: &CardName) -> Result<f32, LayoutError> {
        self.aspect_ratios
            .get(card)
            .copied()
            .ok_or_else(|| LayoutError::UnknownCard(card.clone()))
    }

    /// Adds a card whose top-left corner is at (`left`, `top`) inside the margins.
    fn place(&mut self, card: &CardName, left: f32, top: f32, width: f32, height: f32) {
        self.current.elements.push(Element::Card {
            card: card.clone(),
            x: MARGIN + left,
            y: MARGIN + top - height,
            width,
            height,
        });
    }

    /// Drops empty pages and adds the footer to the remaining ones.
    fn finish(mut self, label: &str, lang: Lang) -> DocumentPlan {
        self.new_page();
        let mut pages = self.done;
        pages.retain(|page| !page.elements.is_empty());
        if pages.is_empty() {
            pages.push(Page::default());
        }
        let count = pages.len();
        let right = MARGIN + self.geometry.content_width;
        for (index, page) in pages.iter_mut().enumerate() {
            page.elements
                .extend(footer(label, page_number(lang, index + 1, count), right));
        }
        DocumentPlan {
            orientation: self.geometry.orientation,
            pages,
        }
    }
}

/// Fleet summary shown at the bottom left of every page.
fn footer_label(fleet: &Fleet) -> String {
    format!(
        "{} ({}/{}) - {FOOTER_SITE}",
        fleet.name, fleet.squadron_points, fleet.total_points
    )
}

/// "page 2 sur 3", in `lang`.
fn page_number(lang: Lang, number: usize, count: usize) -> String {
    match lang {
        Lang::Fr => format!("page {number} sur {count}"),
        Lang::En => format!("page {number} of {count}"),
    }
}

/// Footer elements: `label` on the left, `page_number` ending at `right`.
fn footer(label: &str, page_number: String, right: f32) -> [Element; 2] {
    let text = |text: String, x: f32, align: TextAlign| Element::Text {
        text,
        x,
        baseline: FOOTER_BASELINE,
        size: FOOTER_FONT_SIZE,
        align,
    };
    [
        text(label.to_owned(), MARGIN, TextAlign::Left),
        text(page_number, right, TextAlign::Right),
    ]
}
