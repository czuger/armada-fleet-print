//! Placement of fleet cards on printed pages.

mod document_plan;
mod fleet_layout;
mod shelf;

pub use document_plan::{DocumentPlan, Element, PAGE_HEIGHT, PAGE_WIDTH, Page, TextAlign};
pub use fleet_layout::{LayoutError, lay_out};
