//! The `data sync` command: BSData files → local JSON database.

use std::path::Path;

use futures_util::future::try_join_all;
use reqwest::Client;

use crate::armada_data::armada_database::{ArmadaDatabase, DataSource};
use crate::armada_data::bs_catalogue::{BsCatalogue, BsCatalogueError};
use crate::armada_data::github_source::{BSDATA_REPO, GithubError, GithubSource, SOURCE_FILES};
use crate::atomic_file::{AtomicWriteError, write_atomic};

/// Builds the local card database from the BSData repository.
#[derive(Debug, Clone)]
pub struct DataSync {
    source: GithubSource,
}

/// Error returned when the database cannot be built.
#[derive(Debug, thiserror::Error)]
pub enum DataSyncError {
    /// The repository could not be read.
    #[error("reading BSData repository failed")]
    Github(#[from] GithubError),
    /// A downloaded file could not be parsed.
    #[error("parsing {file} failed")]
    Parse {
        /// Repository file.
        file: &'static str,
        /// Underlying error.
        #[source]
        source: BsCatalogueError,
    },
    /// The database could not be serialized.
    #[error("serializing database failed")]
    Serialize(#[from] serde_json::Error),
    /// The output file could not be written.
    #[error("writing database failed")]
    Write(#[from] AtomicWriteError),
}

impl DataSync {
    /// Creates the command.
    ///
    /// # Arguments
    /// * `client` - HTTP client, which must send a User-Agent.
    pub fn new(client: Client) -> Self {
        Self {
            source: GithubSource::new(client),
        }
    }

    /// Downloads the BSData files at `git_ref`, builds the database and writes it to `output`
    /// as pretty-printed JSON, atomically.
    ///
    /// # Arguments
    /// * `git_ref` - Tag, branch or commit; the latest release when `None`.
    /// * `output` - JSON file to write; its directory is created if needed.
    ///
    /// # Returns
    /// The database written.
    ///
    /// # Errors
    /// Returns [`DataSyncError`] describing the failed step.
    pub async fn run(
        &self,
        git_ref: Option<&str>,
        output: &Path,
    ) -> Result<ArmadaDatabase, DataSyncError> {
        let resolved = self.source.resolve(git_ref).await?;
        tracing::info!(
            git_ref = resolved.git_ref,
            commit = resolved.commit,
            "downloading BSData files"
        );
        let downloads = SOURCE_FILES
            .iter()
            .map(|file| self.source.download(&resolved.commit, file));
        let files = try_join_all(downloads).await?;
        let catalogues = SOURCE_FILES
            .iter()
            .zip(&files)
            .map(|(file, xml)| {
                BsCatalogue::parse(xml).map_err(|source| DataSyncError::Parse { file, source })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let source = DataSource {
            repo: BSDATA_REPO.to_owned(),
            git_ref: resolved.git_ref,
            commit: resolved.commit,
            generated_at: jiff::Timestamp::now()
                .strftime("%Y-%m-%dT%H:%M:%SZ")
                .to_string(),
        };
        let database = ArmadaDatabase::build(source, &catalogues);
        let mut json = serde_json::to_vec_pretty(&database)?;
        json.push(b'\n');
        write_atomic(output, &json).await?;
        Ok(database)
    }
}
