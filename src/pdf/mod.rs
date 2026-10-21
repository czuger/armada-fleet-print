//! PDF output of a [`crate::layout::DocumentPlan`].

mod helvetica;
mod pdf_renderer;

pub use pdf_renderer::{RenderError, render};
