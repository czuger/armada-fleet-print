//! Command-line interface. Help texts are user-facing, hence in French.

use std::io;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use crate::armada_data::{ArmadaDatabase, DataSync, DataSyncError};
use crate::fleet_printer::{FleetPrinter, PrintError};
use crate::fleet_url::{FleetUrl, FleetUrlError};
use crate::game::GameService;
use crate::http_client::build_http_client;
use crate::web::{AppState, serve};

/// Command-line arguments.
#[derive(Debug, Parser)]
#[command(
    version,
    about = "Impression de flottes Star Wars: Armada créées sur armada.ryankingston.com"
)]
pub struct Cli {
    /// Directory where downloaded card images are cached.
    #[arg(
        long,
        env = "AFP_CARDS_DIR",
        default_value = "pics",
        global = true,
        help = "Dossier de cache des images de cartes"
    )]
    pub cards_dir: PathBuf,
    /// Action to run.
    #[command(subcommand)]
    pub command: Command,
}

/// Available actions.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Runs the web server.
    #[command(about = "Lance le serveur web")]
    Serve {
        /// Address and port to listen on.
        #[arg(
            long,
            env = "AFP_BIND",
            default_value = "127.0.0.1:4567",
            help = "Adresse et port d'écoute"
        )]
        bind: SocketAddr,
        /// Unix socket to listen on instead of `bind`.
        #[arg(
            long,
            env = "AFP_SOCKET",
            help = "Socket Unix d'écoute (remplace --bind)"
        )]
        socket: Option<PathBuf>,
        /// Directory where multi-player games are saved.
        #[arg(
            long,
            env = "AFP_GAMES_DIR",
            default_value = "games",
            help = "Dossier des parties à plusieurs joueurs"
        )]
        games_dir: PathBuf,
    },
    /// Prints one fleet to a PDF file without starting the server.
    #[command(about = "Imprime une flotte dans un fichier PDF, sans serveur")]
    Render {
        /// Link to the saved fleet.
        #[arg(help = "Lien de la flotte sauvegardée")]
        url: String,
        /// Output file.
        #[arg(
            short,
            long,
            default_value = "flotte.pdf",
            help = "Fichier PDF à écrire"
        )]
        output: PathBuf,
    },
    /// Manages the local card database.
    #[command(about = "Gère la base locale des cartes")]
    Data {
        /// Database action to run.
        #[command(subcommand)]
        command: DataCommand,
    },
}

/// Actions on the local card database.
#[derive(Debug, Subcommand)]
pub enum DataCommand {
    /// Builds the database from the BSData repository.
    #[command(about = "Construit la base JSON des cartes à partir du dépôt BSData")]
    Sync {
        /// Git reference to read: tag, branch or commit. Defaults to the latest release.
        #[arg(
            long = "ref",
            value_name = "REF",
            help = "Référence git à lire : tag, branche ou commit (par défaut, la dernière release)"
        )]
        git_ref: Option<String>,
        /// Output file.
        #[arg(
            short,
            long,
            default_value = "data/armada.json",
            help = "Fichier JSON à écrire"
        )]
        output: PathBuf,
    },
}

/// Error returned when a command fails.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// The HTTP client could not be initialized.
    #[error("creating HTTP client failed")]
    Client(#[source] reqwest::Error),
    /// The fleet link is invalid.
    #[error("invalid fleet link")]
    Url(#[from] FleetUrlError),
    /// Printing the fleet failed.
    #[error("printing fleet failed")]
    Print(#[from] PrintError),
    /// Building the card database failed.
    #[error("building card database failed")]
    DataSync(#[from] DataSyncError),
    /// Binding the server or writing the PDF failed.
    #[error("I/O failed")]
    Io(#[from] io::Error),
}

impl Cli {
    /// Runs the selected command.
    ///
    /// # Errors
    /// Returns [`CliError`] if the command fails.
    pub async fn run(self) -> Result<(), CliError> {
        match self.command {
            Command::Serve {
                bind,
                socket,
                games_dir,
            } => {
                let state = AppState {
                    printer: FleetPrinter::new(self.cards_dir).map_err(CliError::Client)?,
                    games: GameService::new(games_dir).map_err(CliError::Client)?,
                };
                serve(bind, socket.as_deref(), state).await?;
            }
            Command::Render { url, output } => {
                let printer = FleetPrinter::new(self.cards_dir).map_err(CliError::Client)?;
                let pdf = printer.print(&FleetUrl::parse(&url)?).await?;
                tokio::fs::write(&output, pdf).await?;
                println!("PDF écrit dans {}", output.display());
            }
            Command::Data {
                command: DataCommand::Sync { git_ref, output },
            } => {
                let client = build_http_client().map_err(CliError::Client)?;
                let database = DataSync::new(client)
                    .run(git_ref.as_deref(), &output)
                    .await?;
                print_summary(&database, &output);
            }
        }
        Ok(())
    }
}

/// Prints what `data sync` wrote.
fn print_summary(database: &ArmadaDatabase, output: &Path) {
    let source = &database.source;
    println!(
        "Base écrite dans {} (BSData {}, commit {})",
        output.display(),
        source.git_ref,
        source.commit
    );
    println!(
        "{} vaisseaux, {} escadrons, {} améliorations, {} objectifs",
        database.ships.len(),
        database.squadrons.len(),
        database.upgrades.len(),
        database.objectives.len()
    );
}
