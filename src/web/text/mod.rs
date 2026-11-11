//! Interface text in each [`Lang`](crate::lang::Lang). Each page has a struct of fixed
//! strings with one constant per language, so a missing translation does not compile.

mod error_message;
mod game_text;
mod index_text;
mod page_text;

pub(crate) use error_message::ErrorMessage;
pub(crate) use game_text::GameText;
pub(crate) use index_text::IndexText;
pub(crate) use page_text::PageText;
