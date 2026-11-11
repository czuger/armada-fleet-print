//! Formatting of nested errors for log lines.

use std::error::Error;

/// Joins `err` and all its sources into one line, outermost first.
///
/// # Arguments
/// * `err` - The error to describe.
pub fn error_chain(err: &dyn Error) -> String {
    let mut message = err.to_string();
    let mut source = err.source();
    while let Some(cause) = source {
        message.push_str(": ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }
    message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, thiserror::Error)]
    #[error("outer")]
    struct Outer(#[source] std::io::Error);

    #[test]
    fn joins_sources() {
        let err = Outer(std::io::Error::other("inner"));
        assert_eq!(error_chain(&err), "outer: inner");
    }
}
