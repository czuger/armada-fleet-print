//! Extraction of the fleet JSON from the inline script of a fleet builder page.

use std::sync::LazyLock;

use regex::Regex;

// safe: both patterns are compile-time constants covered by tests.
static SCRIPT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<script[^>]*>(.*?)</script>").expect("valid regex"));
static FLEET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)\bfleet:\s*(\{.*\}),\s*stock:").expect("valid regex"));

/// Returns the JSON object assigned to `fleet` in the page's `window.fleetBuilder` script.
pub(crate) fn extract_fleet_json(html: &str) -> Option<&str> {
    SCRIPT
        .captures_iter(html)
        .filter_map(|script| script.get(1))
        .find_map(|body| FLEET.captures(body.as_str())?.get(1))
        .map(|json| json.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_fleet_object() {
        let html = "<script src=\"a.js\"></script>\n<script>\n\twindow.fleetBuilder = {\n\
            \t\tcards: cards,\n\t\tfleet: {\"name\":\"x\",\"ships\":[{}]},\n\t\tstock: null,\n\
            \t};\n</script>";
        assert_eq!(
            extract_fleet_json(html),
            Some("{\"name\":\"x\",\"ships\":[{}]}")
        );
    }

    #[test]
    fn ignores_pages_without_fleet() {
        assert_eq!(
            extract_fleet_json("<p>fleet: {}, stock: 1</p><script></script>"),
            None
        );
    }
}
