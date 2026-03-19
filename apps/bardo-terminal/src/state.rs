//! Shared application state for the terminal scaffold.

use crate::layout::LayoutBreakpoint;

/// Current connection status for the scaffold.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConnectionStatus {
    /// Connected to the upstream surface.
    Connected,
    /// Not connected to any upstream surface.
    Disconnected,
    /// Connection is in progress.
    Connecting,
}

impl ConnectionStatus {
    /// Returns the display label used in status areas.
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Connected => "CONNECTED",
            Self::Disconnected => "DISCONNECTED",
            Self::Connecting => "CONNECTING…",
        }
    }
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        Self::Disconnected
    }
}

/// Placeholder vitality data until the real mortality model is wired in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MockVitality {
    /// Normalized vitality value between 0.0 and 1.0.
    pub(crate) value: f64,
}

impl Default for MockVitality {
    fn default() -> Self {
        Self { value: 0.75 }
    }
}

/// Global application state shared by all screens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct AppState {
    /// Frame tick counter.
    pub(crate) tick_count: u64,
    /// Current connection status.
    pub(crate) connection_status: ConnectionStatus,
    /// Placeholder vitality snapshot.
    pub(crate) vitality: MockVitality,
    /// Current responsive layout breakpoint.
    pub(crate) layout: LayoutBreakpoint,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            tick_count: 0,
            connection_status: ConnectionStatus::default(),
            vitality: MockVitality::default(),
            layout: LayoutBreakpoint::Standard,
        }
    }
}

/// Action emitted by screens and consumed by the app loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppAction {
    /// Exit the application.
    Quit,
    /// Move to the next screen.
    NextScreen,
    /// Move to the previous screen.
    PrevScreen,
    /// Update layout state after a resize.
    Resize(u16, u16),
}

#[cfg(test)]
mod tests {
    use super::{AppAction, AppState, ConnectionStatus, MockVitality};
    use crate::layout::LayoutBreakpoint;

    #[test]
    fn default_state_uses_expected_placeholders() {
        let state = AppState::default();

        assert_eq!(state.tick_count, 0);
        assert_eq!(state.connection_status, ConnectionStatus::Disconnected);
        assert_eq!(state.vitality, MockVitality { value: 0.75 });
        assert_eq!(state.layout, LayoutBreakpoint::Standard);
    }

    #[test]
    fn action_variants_compile() {
        assert_eq!(AppAction::Quit, AppAction::Quit);
        assert_eq!(ConnectionStatus::Connected.label(), "CONNECTED");
    }
}
