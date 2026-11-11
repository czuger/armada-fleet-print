//! Placement of fleet cards on printed pages.

mod document_plan;
mod fleet_layout;
mod orientation;
mod shelf;
mod upgrade_grid;

pub use document_plan::{DocumentPlan, Element, Page, TextAlign};
pub use fleet_layout::{LayoutError, lay_out};
pub use orientation::{Orientation, OrientationParseError};
