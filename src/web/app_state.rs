//! State shared by the request handlers.

use axum::extract::FromRef;

use crate::fleet_printer::FleetPrinter;
use crate::game::GameService;

/// Services the handlers draw from, each extracted on its own with `State`.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Prints fleets to PDF.
    pub printer: FleetPrinter,
    /// Manages multi-player games.
    pub games: GameService,
}

impl FromRef<AppState> for FleetPrinter {
    fn from_ref(state: &AppState) -> Self {
        state.printer.clone()
    }
}

impl FromRef<AppState> for GameService {
    fn from_ref(state: &AppState) -> Self {
        state.games.clone()
    }
}
