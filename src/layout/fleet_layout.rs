//! Fleet layout: ships with their upgrades beside them, then squadrons and objectives, packed
//! left to right in lines that keep the builder's order.

use std::collections::HashMap;
use std::mem;

use crate::card_name::CardName;
use crate::fleet::{Fleet, Ship, SquadronGroup};
use crate::layout::document_plan::{
    DocumentPlan, Element, PAGE_HEIGHT, PAGE_WIDTH, Page, TextAlign,
};
use crate::layout::shelf::Shelf;

/// Margin on every side of the page (Prawn's default).
const MARGIN: f32 = 36.0;
/// Width of the area inside the margins.
const CONTENT_WIDTH: f32 = PAGE_WIDTH - 2.0 * MARGIN;
/// Height of the area inside the margins.
const CONTENT_HEIGHT: f32 = PAGE_HEIGHT - 2.0 * MARGIN;

const COLUMN_GAP: f32 = 3.0;
const ROW_GAP: f32 = 10.0;
/// As in the original, a line of three small cards runs 6 pt into the right margin.
const SHELF_WIDTH: f32 = 3.0 * CARD_WIDTH + 2.0 * COLUMN_GAP;

const SHIP_WIDTH: f32 = 200.0;
const UPGRADE_LEFT: f32 = 210.0;
const UPGRADE_WIDTH: f32 = 100.0;
const UPGRADE_HEIGHT: f32 = 150.0;
const UPGRADE_STEP_X: f32 = 110.0;
const UPGRADE_STEP_Y: f32 = 160.0;
const UPGRADES_PER_ROW: usize = 3;

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

/// Lays out every card of `fleet`, with a footer on each page.
///
/// # Arguments
/// * `fleet` - The fleet to print.
/// * `aspect_ratios` - Height-to-width ratio of each card image of the fleet.
///
/// # Errors
/// Returns [`LayoutError::UnknownCard`] if a card has no entry in `aspect_ratios`.
pub fn lay_out(
    fleet: &Fleet,
    aspect_ratios: &HashMap<CardName, f32>,
) -> Result<DocumentPlan, LayoutError> {
    let mut builder = PlanBuilder::new(aspect_ratios);
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
    Ok(builder.finish(&footer_label(fleet)))
}

/// Accumulates pages while walking through the fleet.
struct PlanBuilder<'a> {
    aspect_ratios: &'a HashMap<CardName, f32>,
    done: Vec<Page>,
    current: Page,
    shelf: Shelf,
}

impl<'a> PlanBuilder<'a> {
    fn new(aspect_ratios: &'a HashMap<CardName, f32>) -> Self {
        Self {
            aspect_ratios,
            done: Vec::new(),
            current: Page::default(),
            shelf: Shelf::new(SHELF_WIDTH, CONTENT_HEIGHT, COLUMN_GAP, ROW_GAP),
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

    /// Lays out a ship with its upgrades in rows of three to its right.
    fn ship(&mut self, ship: &Ship) -> Result<(), LayoutError> {
        let height = self.height_at(&ship.card, SHIP_WIDTH)?;
        let (block_width, upgrades_height) = upgrades_size(ship.upgrades.len());
        let (left, top) = self.slot(block_width, height.max(upgrades_height));
        self.place(&ship.card, left, top, SHIP_WIDTH, height);
        for (index, upgrade) in ship.upgrades.iter().enumerate() {
            let column = (index % UPGRADES_PER_ROW) as f32;
            let row = (index / UPGRADES_PER_ROW) as f32;
            let upgrade_left = left + UPGRADE_LEFT + column * UPGRADE_STEP_X;
            let upgrade_top = top - row * UPGRADE_STEP_Y;
            self.place(
                upgrade,
                upgrade_left,
                upgrade_top,
                UPGRADE_WIDTH,
                UPGRADE_HEIGHT,
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
        self.aspect_ratios
            .get(card)
            .map(|ratio| width * ratio)
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
    fn finish(mut self, label: &str) -> DocumentPlan {
        self.new_page();
        let mut pages = self.done;
        pages.retain(|page| !page.elements.is_empty());
        if pages.is_empty() {
            pages.push(Page::default());
        }
        let count = pages.len();
        for (index, page) in pages.iter_mut().enumerate() {
            page.elements.extend(footer(label, index + 1, count));
        }
        DocumentPlan { pages }
    }
}

/// Width of a ship block with `count` upgrades, and the height of its upgrade rows. A ship with
/// upgrades takes a whole line.
fn upgrades_size(count: usize) -> (f32, f32) {
    if count == 0 {
        return (SHIP_WIDTH, 0.0);
    }
    let rows = count.div_ceil(UPGRADES_PER_ROW) as f32;
    (
        SHELF_WIDTH,
        rows * UPGRADE_STEP_Y - (UPGRADE_STEP_Y - UPGRADE_HEIGHT),
    )
}

/// Fleet summary shown at the bottom left of every page.
fn footer_label(fleet: &Fleet) -> String {
    format!(
        "{} ({}/{}) - {FOOTER_SITE}",
        fleet.name, fleet.squadron_points, fleet.total_points
    )
}

/// Footer elements of page `number` out of `count`.
fn footer(label: &str, number: usize, count: usize) -> [Element; 2] {
    let text = |text: String, x: f32, align: TextAlign| Element::Text {
        text,
        x,
        baseline: FOOTER_BASELINE,
        size: FOOTER_FONT_SIZE,
        align,
    };
    [
        text(label.to_owned(), MARGIN, TextAlign::Left),
        text(
            format!("page {number} sur {count}"),
            MARGIN + CONTENT_WIDTH,
            TextAlign::Right,
        ),
    ]
}
