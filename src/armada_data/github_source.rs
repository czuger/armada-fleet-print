//! Download of the BSData Star Wars: Armada files from GitHub, over HTTPS, without cloning.

use std::string::FromUtf8Error;

use reqwest::{Client, Response, StatusCode};
use serde::Deserialize;
use url::Url;

/// BSData repository, as `owner/name`.
pub const BSDATA_REPO: &str = "BSData/star-wars-armada";

/// Files read from the repository: the game system first, then the faction catalogues in
/// alphabetical order. This order decides which id is kept for upgrades listed in several files.
pub const SOURCE_FILES: [&str; 5] = [
    "Star_Wars_Armada.gst",
    "Galactic Republic.cat",
    "Imperial Navy.cat",
    "Rebel Fleet.cat",
    "Separatist Alliance.cat",
];

/// GitHub REST API root of the repository.
const API_URL: &str = "https://api.github.com/repos/BSData/star-wars-armada/";
/// Raw file root of the repository.
const RAW_URL: &str = "https://raw.githubusercontent.com/BSData/star-wars-armada/";
/// Branch used when the repository has no release.
const DEFAULT_BRANCH: &str = "master";
/// Media type of GitHub JSON responses.
const GITHUB_JSON: &str = "application/vnd.github+json";
/// Media type asking the commits endpoint for the bare SHA.
const GITHUB_SHA: &str = "application/vnd.github.sha";
/// GitHub REST API version the requests are written for.
const GITHUB_API_VERSION: &str = "2022-11-28";

/// Client of the BSData repository on GitHub.
#[derive(Debug, Clone)]
pub struct GithubSource {
    client: Client,
}

/// A git reference and the commit it points to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRef {
    /// Reference asked for, or the latest release tag, or the default branch.
    pub git_ref: String,
    /// Full commit SHA.
    pub commit: String,
}

/// Error returned when the repository cannot be read.
#[derive(Debug, thiserror::Error)]
pub enum GithubError {
    /// A request failed or returned an error status.
    #[error("requesting {url} failed")]
    Request {
        /// Requested URL.
        url: Url,
        /// Underlying error.
        #[source]
        source: reqwest::Error,
    },
    /// The latest release response is not the expected JSON.
    #[error("reading latest release failed")]
    Release(#[source] serde_json::Error),
    /// The commits endpoint returned something other than a SHA.
    #[error("ref {git_ref} resolved to unexpected commit {commit:?}")]
    InvalidCommit {
        /// Reference being resolved.
        git_ref: String,
        /// Response body.
        commit: String,
    },
    /// A downloaded file is not UTF-8.
    #[error("file {file} is not UTF-8")]
    Encoding {
        /// Repository file.
        file: String,
        /// Underlying error.
        #[source]
        source: FromUtf8Error,
    },
}

/// The fields read from the latest release.
#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
}

impl GithubSource {
    /// Creates a source using `client` for all requests.
    ///
    /// # Arguments
    /// * `client` - HTTP client, which must send a User-Agent as GitHub requires one.
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Resolves `git_ref` to a commit. Without a reference, the latest release tag is used, or
    /// the default branch if the repository has no release.
    ///
    /// # Errors
    /// Returns [`GithubError`] if a request fails or a response is unexpected.
    pub async fn resolve(&self, git_ref: Option<&str>) -> Result<ResolvedRef, GithubError> {
        let git_ref = match git_ref {
            Some(git_ref) => git_ref.to_owned(),
            None => self
                .latest_release()
                .await?
                .unwrap_or_else(|| DEFAULT_BRANCH.to_owned()),
        };
        let commit = self.commit_sha(&git_ref).await?;
        Ok(ResolvedRef { git_ref, commit })
    }

    /// Downloads `file` as of `commit`.
    ///
    /// # Arguments
    /// * `commit` - Commit SHA, so that all files come from the same tree.
    /// * `file` - File path in the repository.
    ///
    /// # Errors
    /// Returns [`GithubError`] if the download fails or the file is not UTF-8.
    pub async fn download(&self, commit: &str, file: &str) -> Result<String, GithubError> {
        let url = repo_url(RAW_URL, &[commit, file]);
        let body = self.fetch(url, None).await?;
        String::from_utf8(body).map_err(|source| GithubError::Encoding {
            file: file.to_owned(),
            source,
        })
    }

    /// Tag of the latest release, `None` if the repository has none.
    async fn latest_release(&self) -> Result<Option<String>, GithubError> {
        let url = repo_url(API_URL, &["releases", "latest"]);
        let response = self.send(&url, Some(GITHUB_JSON)).await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let body = read_body(url, response).await?;
        let release: Release = serde_json::from_slice(&body).map_err(GithubError::Release)?;
        Ok(Some(release.tag_name))
    }

    /// Full SHA of the commit `git_ref` points to.
    async fn commit_sha(&self, git_ref: &str) -> Result<String, GithubError> {
        let url = repo_url(API_URL, &["commits", git_ref]);
        let body = self.fetch(url, Some(GITHUB_SHA)).await?;
        let commit = String::from_utf8_lossy(&body).trim().to_owned();
        if commit.len() != 40 || !commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(GithubError::InvalidCommit {
                git_ref: git_ref.to_owned(),
                commit,
            });
        }
        Ok(commit)
    }

    /// Downloads `url`, failing on error statuses.
    async fn fetch(&self, url: Url, accept: Option<&str>) -> Result<Vec<u8>, GithubError> {
        let response = self.send(&url, accept).await?;
        read_body(url, response).await
    }

    /// Sends a GET request to `url`.
    async fn send(&self, url: &Url, accept: Option<&str>) -> Result<Response, GithubError> {
        let mut request = self.client.get(url.clone());
        if let Some(accept) = accept {
            request = request
                .header(reqwest::header::ACCEPT, accept)
                .header("X-GitHub-Api-Version", GITHUB_API_VERSION);
        }
        request.send().await.map_err(|source| GithubError::Request {
            url: url.clone(),
            source,
        })
    }
}

/// Body of `response`, failing on error statuses.
async fn read_body(url: Url, response: Response) -> Result<Vec<u8>, GithubError> {
    let to_error = |source| GithubError::Request {
        url: url.clone(),
        source,
    };
    let response = response.error_for_status().map_err(to_error)?;
    Ok(response.bytes().await.map_err(to_error)?.to_vec())
}

/// `base` followed by `segments`, each percent-encoded (spaces in file names, `/` in refs).
fn repo_url(base: &str, segments: &[&str]) -> Url {
    // safe: `base` is one of the absolute https URL constants above
    let mut url = Url::parse(base).expect("constant base URL is valid");
    url.path_segments_mut()
        // safe: https URLs always have a path
        .expect("https URL has path segments")
        .pop_if_empty()
        .extend(segments);
    url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_file_names_and_refs() {
        assert_eq!(
            repo_url(RAW_URL, &["abc", "Imperial Navy.cat"]).as_str(),
            "https://raw.githubusercontent.com/BSData/star-wars-armada/abc/Imperial%20Navy.cat"
        );
        assert_eq!(
            repo_url(API_URL, &["commits", "feature/x"]).as_str(),
            "https://api.github.com/repos/BSData/star-wars-armada/commits/feature%2Fx"
        );
    }
}
