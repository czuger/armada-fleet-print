//! Validated links to saved fleets.

use std::fmt;

use url::Url;

/// Host of the fleet builder, the only site fleets are fetched from.
pub const FLEET_BUILDER_HOST: &str = "armada.ryankingston.com";

/// Link to a fleet page on the fleet builder, always fetched over HTTPS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FleetUrl(Url);

/// Error returned when a link does not point to the fleet builder.
#[derive(Debug, thiserror::Error)]
pub enum FleetUrlError {
    /// The text is not a URL.
    #[error("malformed fleet URL")]
    Malformed(#[from] url::ParseError),
    /// The URL uses a scheme other than HTTP(S) or another host than the fleet builder.
    #[error("URL does not point to https://{FLEET_BUILDER_HOST}/")]
    NotFleetBuilder,
}

impl FleetUrl {
    /// Parses a link pasted by a user. A missing scheme is accepted and HTTP is upgraded to HTTPS.
    ///
    /// # Errors
    /// Returns [`FleetUrlError`] if `input` is not a URL or does not point to the fleet builder.
    pub fn parse(input: &str) -> Result<Self, FleetUrlError> {
        let input = input.trim();
        let mut url = if input.contains("://") {
            Url::parse(input)?
        } else {
            Url::parse(&format!("https://{input}"))?
        };
        if !matches!(url.scheme(), "http" | "https") || url.host_str() != Some(FLEET_BUILDER_HOST) {
            return Err(FleetUrlError::NotFleetBuilder);
        }
        url.set_scheme("https")
            .map_err(|()| FleetUrlError::NotFleetBuilder)?;
        Ok(Self(url))
    }

    /// Returns the URL to fetch.
    pub fn as_url(&self) -> &Url {
        &self.0
    }
}

impl fmt::Display for FleetUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_fleet_builder_links() {
        for input in [
            "https://armada.ryankingston.com/fleet/158694/",
            "  http://armada.ryankingston.com/fleet/158694/ ",
            "armada.ryankingston.com/fleet/158694/",
        ] {
            let url = FleetUrl::parse(input).unwrap();
            assert_eq!(
                url.to_string(),
                "https://armada.ryankingston.com/fleet/158694/"
            );
        }
    }

    #[test]
    fn rejects_other_targets() {
        for input in [
            "",
            "https://example.com/fleet/1/",
            "file:///etc/passwd",
            "https://armada.ryankingston.com.evil.com/fleet/1/",
        ] {
            assert!(
                FleetUrl::parse(input).is_err(),
                "{input:?} should be rejected"
            );
        }
    }
}
