//! Command-line interface. Help texts are user-facing, hence in French.

use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::fleet_printer::{FleetPrinter, PrintError};
use crate::fleet_url::{FleetUrl, FleetUrlError};
use crate::web::serve;

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
        let printer = FleetPrinter::new(self.cards_dir).map_err(CliError::Client)?;
        match self.command {
            Command::Serve { bind, socket } => serve(bind, socket.as_deref(), printer).await?,
            Command::Render { url, output } => {
                let pdf = printer.print(&FleetUrl::parse(&url)?).await?;
                tokio::fs::write(&output, pdf).await?;
                println!("PDF écrit dans {}", output.display());
            }
        }
        Ok(())
    }
}
