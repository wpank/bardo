//! TUI integration for sonification rack visualization.
//!
//! Stub: will render rack state, signal levels, and patch cables
//! in the terminal UI.

/// Controller for the sonification TUI panel.
pub struct TuiController;

impl TuiController {
    /// Create a new TUI controller.
    pub fn new() -> Self {
        Self
    }

    /// Run the TUI render loop (stub).
    pub fn run(&self) {
        // Will integrate with ratatui in a future plan.
    }
}

impl Default for TuiController {
    fn default() -> Self {
        Self::new()
    }
}
