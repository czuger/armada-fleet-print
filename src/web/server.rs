//! HTTP server lifecycle.

use std::fs::{self, Permissions};
use std::io;
use std::net::SocketAddr;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::path::Path;

use axum::Router;
use tokio::net::{TcpListener, UnixListener};

use crate::web::app_state::AppState;
use crate::web::router::router;

/// nginx runs in a container under another uid and needs write access to connect.
const SOCKET_MODE: u32 = 0o666;

/// Serves the application until Ctrl-C or SIGTERM, then drains open requests.
///
/// # Arguments
/// * `addr` - TCP address and port to listen on when no socket is given.
/// * `socket` - Unix socket path to listen on instead of `addr`.
/// * `state` - Services shared by all requests.
///
/// # Errors
/// Returns [`io::Error`] if the listener cannot be set up, if the socket path is taken
/// by something that is not a socket, or if the server fails.
pub async fn serve(addr: SocketAddr, socket: Option<&Path>, state: AppState) -> io::Result<()> {
    let app = router(state);
    match socket {
        Some(path) => serve_unix(app, path).await,
        None => serve_tcp(app, addr).await,
    }
}

async fn serve_tcp(app: Router, addr: SocketAddr) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    tracing::info!(%addr, "listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
}

async fn serve_unix(app: Router, path: &Path) -> io::Result<()> {
    remove_stale_socket(path)?;
    let listener = UnixListener::bind(path)?;
    fs::set_permissions(path, Permissions::from_mode(SOCKET_MODE))?;
    tracing::info!(socket = %path.display(), "listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    fs::remove_file(path)
}

/// A socket left by a previous run (crash, kill -9) would make `bind` fail; anything
/// else at that path is not ours to delete.
fn remove_stale_socket(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_socket() => fs::remove_file(path),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("{} exists and is not a socket", path.display()),
        )),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

/// Resolves when the process is asked to stop.
async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            tracing::error!("listening for Ctrl-C failed: {err}");
            std::future::pending::<()>().await;
        }
    };
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};
        match signal(SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(err) => {
                tracing::error!("listening for SIGTERM failed: {err}");
                std::future::pending::<()>().await;
            }
        }
    };

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
    tracing::info!("shutting down");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regular_file_at_socket_path_is_kept() {
        let path = std::env::temp_dir().join(format!(
            "armada_fleet_print_{}.not_a_socket",
            std::process::id()
        ));
        fs::write(&path, "keep me").unwrap();
        let err = remove_stale_socket(&path).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read_to_string(&path).unwrap(), "keep me");
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn stale_socket_is_removed_and_missing_path_is_fine() {
        let path =
            std::env::temp_dir().join(format!("armada_fleet_print_{}.sock", std::process::id()));
        let listener = std::os::unix::net::UnixListener::bind(&path).unwrap();
        drop(listener);
        remove_stale_socket(&path).unwrap();
        assert!(!path.exists());
        remove_stale_socket(&path).unwrap();
    }
}
