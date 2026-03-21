//! Shared application state for the terminal scaffold.

use std::time::Instant;

use crate::{layout::LayoutBreakpoint, screen::ScreenId};

const DEFAULT_VITALITY_VALUE: f64 = 0.75;
const MAX_SCAFFOLD_TICK_DT_SECS: f64 = 0.1;

fn clamp_scaffold_dt(dt: f64) -> f64 {
    dt.clamp(0.0, MAX_SCAFFOLD_TICK_DT_SECS)
}

// ── System metrics ───────────────────────────────────────────────────

/// Number of samples kept in each history ring for sparklines.
pub(crate) const SYS_HISTORY_LEN: usize = 60;

/// Snapshot of live system resource metrics, updated once per second.
///
/// History vecs are ordered oldest-to-newest and hold at most
/// [`SYS_HISTORY_LEN`] entries.
#[derive(Debug, Clone)]
pub(crate) struct SysMetrics {
    /// CPU utilisation 0..100.
    pub(crate) cpu_pct: f32,
    /// Rolling CPU utilisation history (fraction 0.0..100.0).
    pub(crate) cpu_history: Vec<f64>,
    /// Bytes of RAM currently in use.
    pub(crate) mem_used_bytes: u64,
    /// Total installed RAM in bytes.
    pub(crate) mem_total_bytes: u64,
    /// Rolling memory utilisation history (fraction 0.0..1.0).
    pub(crate) mem_history: Vec<f64>,
    /// Network bytes received per second (all interfaces summed).
    pub(crate) net_rx_bps: f64,
    /// Network bytes transmitted per second (all interfaces summed).
    pub(crate) net_tx_bps: f64,
    /// Rolling rx rate history (bytes/sec).
    pub(crate) net_rx_history: Vec<f64>,
    /// Rolling tx rate history (bytes/sec).
    pub(crate) net_tx_history: Vec<f64>,
    /// Disk bytes read per second (all disks summed).
    pub(crate) disk_read_bps: f64,
    /// Disk bytes written per second (all disks summed).
    pub(crate) disk_write_bps: f64,
    /// Rolling disk-read rate history (bytes/sec).
    pub(crate) disk_read_history: Vec<f64>,
    /// Rolling disk-write rate history (bytes/sec).
    pub(crate) disk_write_history: Vec<f64>,
    /// Total bytes used across all mounted disks.
    pub(crate) disk_used_bytes: u64,
    /// Total bytes across all mounted disks.
    pub(crate) disk_total_bytes: u64,
}

impl Default for SysMetrics {
    fn default() -> Self {
        Self {
            cpu_pct: 0.0,
            cpu_history: Vec::new(),
            mem_used_bytes: 0,
            mem_total_bytes: 0,
            mem_history: Vec::new(),
            net_rx_bps: 0.0,
            net_tx_bps: 0.0,
            net_rx_history: Vec::new(),
            net_tx_history: Vec::new(),
            disk_read_bps: 0.0,
            disk_write_bps: 0.0,
            disk_read_history: Vec::new(),
            disk_write_history: Vec::new(),
            disk_used_bytes: 0,
            disk_total_bytes: 0,
        }
    }
}

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
        Self {
            value: DEFAULT_VITALITY_VALUE,
        }
    }
}

// ── Atmosphere ──────────────────────────────────────────────────────

const BRAILLE_SPINNER: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// Smooth animation state ticked every frame at 60fps.
/// Provides heartbeat pulse, breathing brightness, and spinner rotation.
#[derive(Debug, Clone)]
pub(crate) struct Atmosphere {
    /// Total seconds since app start.
    pub(crate) elapsed_secs: f64,
    /// Delta time for the current frame.
    pub(crate) dt: f64,
    heartbeat_phase: f64,
    breathing_phase: f64,
    frame_count: u64,
}

impl Default for Atmosphere {
    fn default() -> Self {
        Self {
            elapsed_secs: 0.0,
            dt: 0.0,
            heartbeat_phase: 0.0,
            breathing_phase: 0.0,
            frame_count: 0,
        }
    }
}

impl Atmosphere {
    /// Oscillates 0.95..1.05 -- multiply into color channels for a pulse.
    pub(crate) fn heartbeat(&self) -> f64 {
        1.0 + 0.05 * self.heartbeat_phase.sin()
    }

    /// Rotating braille spinner character (~20 transitions/sec at 60fps).
    pub(crate) fn spinner(&self) -> char {
        BRAILLE_SPINNER[(self.frame_count / 3) as usize % BRAILLE_SPINNER.len()]
    }

    /// Slow ambient brightness oscillation (0.88..1.0, ~5.2s period).
    #[allow(dead_code)]
    pub(crate) fn breathing_brightness(&self) -> f64 {
        0.94 + 0.06 * self.breathing_phase.sin()
    }

    /// Current frame number.
    #[allow(dead_code)]
    pub(crate) fn frame(&self) -> u64 {
        self.frame_count
    }

    /// Advance animation state by dt seconds.
    pub(crate) fn tick(&mut self, dt: f64) {
        let dt = clamp_scaffold_dt(dt);
        self.dt = dt;
        self.elapsed_secs += dt;
        self.frame_count = self.frame_count.wrapping_add(1);
        self.heartbeat_phase = self.elapsed_secs * std::f64::consts::TAU;
        self.breathing_phase = self.elapsed_secs * (std::f64::consts::TAU / 5.2);
    }
}

// ── Task / Progress ─────────────────────────────────────────────────

/// Status of an individual task in the pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TaskStatus {
    Pending,
    Active,
    Done,
}

/// A single task (phase) in the pipeline.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TaskEntry {
    pub(crate) name: String,
    pub(crate) status: TaskStatus,
    /// How long this task is expected to take.
    pub(crate) estimated_secs: f64,
    /// How long this task has been running (or ran, if done).
    pub(crate) elapsed_secs: f64,
}

/// Tracks overall pipeline progress, per-task timing, and ETA.
#[derive(Debug, Clone)]
pub(crate) struct ProgressState {
    pub(crate) tasks: Vec<TaskEntry>,
    pub(crate) start_time: Instant,
}

impl Default for ProgressState {
    fn default() -> Self {
        Self {
            tasks: vec![
                TaskEntry {
                    name: "preflight".into(),
                    status: TaskStatus::Active,
                    estimated_secs: 8.0,
                    elapsed_secs: 0.0,
                },
                TaskEntry {
                    name: "strategist".into(),
                    status: TaskStatus::Pending,
                    estimated_secs: 25.0,
                    elapsed_secs: 0.0,
                },
                TaskEntry {
                    name: "implementer".into(),
                    status: TaskStatus::Pending,
                    estimated_secs: 45.0,
                    elapsed_secs: 0.0,
                },
                TaskEntry {
                    name: "compile-gate".into(),
                    status: TaskStatus::Pending,
                    estimated_secs: 15.0,
                    elapsed_secs: 0.0,
                },
                TaskEntry {
                    name: "test-gate".into(),
                    status: TaskStatus::Pending,
                    estimated_secs: 20.0,
                    elapsed_secs: 0.0,
                },
                TaskEntry {
                    name: "reviewing".into(),
                    status: TaskStatus::Pending,
                    estimated_secs: 18.0,
                    elapsed_secs: 0.0,
                },
                TaskEntry {
                    name: "committing".into(),
                    status: TaskStatus::Pending,
                    estimated_secs: 5.0,
                    elapsed_secs: 0.0,
                },
            ],
            start_time: Instant::now(),
        }
    }
}

impl ProgressState {
    /// Advance the active task by dt seconds. Auto-promotes to next task on completion.
    pub(crate) fn tick(&mut self, dt: f64) {
        let dt = clamp_scaffold_dt(dt);
        if let Some(idx) = self
            .tasks
            .iter()
            .position(|t| t.status == TaskStatus::Active)
        {
            self.tasks[idx].elapsed_secs += dt;
            if self.tasks[idx].elapsed_secs >= self.tasks[idx].estimated_secs {
                self.tasks[idx].elapsed_secs = self.tasks[idx].estimated_secs;
                self.tasks[idx].status = TaskStatus::Done;
                if idx + 1 < self.tasks.len() {
                    self.tasks[idx + 1].status = TaskStatus::Active;
                }
            }
        }
    }

    /// Sum of all task estimates.
    pub(crate) fn total_estimated_secs(&self) -> f64 {
        self.tasks.iter().map(|t| t.estimated_secs).sum()
    }

    /// Sum of all task elapsed times.
    pub(crate) fn total_elapsed_secs(&self) -> f64 {
        self.tasks.iter().map(|t| t.elapsed_secs).sum()
    }

    /// Overall progress as 0.0..1.0.
    pub(crate) fn progress_fraction(&self) -> f64 {
        let total = self.total_estimated_secs();
        if total <= 0.0 {
            return 0.0;
        }
        (self.total_elapsed_secs() / total).clamp(0.0, 1.0)
    }

    /// Estimated seconds remaining across all incomplete tasks.
    pub(crate) fn eta_remaining_secs(&self) -> f64 {
        self.tasks
            .iter()
            .filter(|t| t.status != TaskStatus::Done)
            .map(|t| (t.estimated_secs - t.elapsed_secs).max(0.0))
            .sum()
    }

    /// Wall-clock seconds since the run started.
    pub(crate) fn wall_elapsed_secs(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    /// True when every task is Done.
    pub(crate) fn is_complete(&self) -> bool {
        self.tasks.iter().all(|t| t.status == TaskStatus::Done)
    }

    /// The currently active task, if any.
    #[allow(dead_code)]
    pub(crate) fn current_task(&self) -> Option<&TaskEntry> {
        self.tasks.iter().find(|t| t.status == TaskStatus::Active)
    }
}

/// Format seconds as a human-readable duration string.
pub(crate) fn format_duration(total_secs: f64) -> String {
    let secs = if total_secs.is_finite() {
        total_secs.max(0.0).ceil() as u64
    } else {
        0
    };
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    if hours > 0 {
        format!("{hours}h{minutes:02}m{seconds:02}s")
    } else if minutes > 0 {
        format!("{minutes}m{seconds:02}s")
    } else {
        format!("{seconds}s")
    }
}

// ── AppState ────────────────────────────────────────────────────────

/// Global application state shared by all screens.
///
/// The terminal scaffold tracks at minimum: `tick_count`, `connection_status`,
/// `vitality`, and `layout`. Additional fields support animation, progress, and
/// system metrics.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct AppState {
    /// Frame tick counter.
    pub(crate) tick_count: u64,
    /// Current connection status.
    pub(crate) connection_status: ConnectionStatus,
    /// Placeholder vitality snapshot.
    pub(crate) vitality: MockVitality,
    /// Current responsive layout breakpoint.
    pub(crate) layout: LayoutBreakpoint,
    /// Smooth animation state (ticked each frame).
    pub(crate) atmosphere: Atmosphere,
    /// Pipeline progress and ETA tracking.
    pub(crate) progress: ProgressState,
    /// Live system resource metrics (updated once per second).
    pub(crate) sys: SysMetrics,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            tick_count: 0,
            connection_status: ConnectionStatus::default(),
            vitality: MockVitality::default(),
            layout: LayoutBreakpoint::Standard,
            atmosphere: Atmosphere::default(),
            progress: ProgressState::default(),
            sys: SysMetrics::default(),
        }
    }
}

/// Action emitted by screens and consumed by the app loop.
///
/// Core scaffold actions are `Quit`, `NextScreen`, `PrevScreen`, and `Resize`.
/// Additional variants support command palette, modals, vim-style navigation, and scrolling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AppAction {
    /// Exit the application.
    Quit,
    /// Move to the next screen.
    NextScreen,
    /// Move to the previous screen.
    PrevScreen,
    /// Update layout state after a resize (terminal columns, rows).
    Resize(u16, u16),
    /// Jump directly to a screen by identifier.
    GotoScreen(ScreenId),
    /// Jump to the first tab of a named window.
    GotoWindow(WindowId),
    /// Open the command palette.
    OpenCommandPalette,
    /// Close the command palette without executing.
    CloseCommandPalette,
    /// Execute a palette entry by filtered index.
    ExecuteCommand(usize),
    /// Forward a character to the palette query.
    PaletteInput(char),
    /// Delete the previous character from the palette query.
    PaletteBackspace,
    /// Move the palette selection downward.
    PaletteSelectNext,
    /// Move the palette selection upward.
    PaletteSelectPrev,
    /// Show the contextual help overlay.
    ShowHelp,
    /// Hide the contextual help overlay.
    HideHelp,
    /// Close the top-most modal.
    CloseModal,
    /// Confirm the top-most modal.
    ConfirmModal,
    /// Forward a character to the active input modal.
    ModalInput(char),
    /// Delete the previous character from the active input modal.
    ModalBackspace,
    /// Enter vim normal mode.
    EnterVimMode,
    /// Exit vim mode and return to standard input handling.
    ExitVimMode,
    /// Navigate using vim-style directional input.
    VimNavigate(VimDirection),
    /// Execute a colon command from vim mode.
    VimCommand(String),
    /// Scroll the active pane upward.
    ScrollUp,
    /// Scroll the active pane downward.
    ScrollDown,
    /// Jump to the top of the active pane.
    ScrollTop,
    /// Jump to the bottom of the active pane.
    ScrollBottom,
}

/// Named window identifiers for top-level terminal navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum WindowId {
    Hearth,
    Mind,
    Soma,
    World,
    Fate,
    Command,
    /// DeFi protocol overview (matches `ScreenId::ProtocolViews` / PROTOCOL chrome).
    Protocol,
}

/// Vim directional navigation commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VimDirection {
    Up,
    Down,
    Left,
    Right,
}

#[cfg(test)]
mod tests {
    use super::{
        AppAction, AppState, Atmosphere, ConnectionStatus, DEFAULT_VITALITY_VALUE,
        MAX_SCAFFOLD_TICK_DT_SECS, MockVitality, SysMetrics, TaskStatus, VimDirection, WindowId,
        format_duration,
    };
    use crate::{layout::LayoutBreakpoint, screen::ScreenId};

    /// Plan 04 T5: scaffold `AppState` fields, defaults, `ConnectionStatus`, `MockVitality`,
    /// and core `AppAction` variants.
    #[test]
    fn plan04_t5_scaffold_state_and_actions() {
        let state = AppState::default();

        assert_eq!(state.tick_count, 0);
        assert_eq!(state.connection_status, ConnectionStatus::Disconnected);
        assert_eq!(state.vitality.value, DEFAULT_VITALITY_VALUE);
        assert!((0.0..=1.0).contains(&state.vitality.value));
        assert_eq!(state.layout, LayoutBreakpoint::Standard);

        let _ = ConnectionStatus::Connected;
        let _ = ConnectionStatus::Disconnected;
        let _ = ConnectionStatus::Connecting;

        assert_eq!(AppAction::Quit, AppAction::Quit);
        assert_eq!(AppAction::NextScreen, AppAction::NextScreen);
        assert_eq!(AppAction::PrevScreen, AppAction::PrevScreen);
        assert_eq!(AppAction::Resize(80, 24), AppAction::Resize(80, 24));
    }

    #[test]
    fn default_state_uses_expected_placeholders() {
        let state = AppState::default();

        assert_eq!(state.tick_count, 0);
        assert_eq!(state.connection_status, ConnectionStatus::Disconnected);
        assert_eq!(
            state.vitality,
            MockVitality {
                value: DEFAULT_VITALITY_VALUE,
            }
        );
        assert_eq!(state.layout, LayoutBreakpoint::Standard);
        assert_eq!(state.atmosphere.elapsed_secs, 0.0);
        assert_eq!(state.atmosphere.dt, 0.0);
        assert!(!state.progress.tasks.is_empty());
        assert!(state.sys.cpu_history.is_empty());
        assert!(state.sys.mem_history.is_empty());
        assert_eq!(state.sys.cpu_pct, 0.0);
        assert_eq!(state.sys.mem_total_bytes, 0);
    }

    #[test]
    fn action_variants_compile() {
        assert_eq!(AppAction::Quit, AppAction::Quit);
        assert_eq!(
            AppAction::GotoScreen(ScreenId::HearthOverview),
            AppAction::GotoScreen(ScreenId::HearthOverview)
        );
        assert_eq!(
            AppAction::GotoWindow(WindowId::Hearth),
            AppAction::GotoWindow(WindowId::Hearth)
        );
        assert_eq!(
            AppAction::VimNavigate(VimDirection::Up),
            AppAction::VimNavigate(VimDirection::Up)
        );
        assert_eq!(
            AppAction::VimCommand("quit".to_string()),
            AppAction::VimCommand("quit".to_string())
        );
        assert_eq!(ConnectionStatus::Connected.label(), "CONNECTED");
    }

    #[test]
    fn navigation_enums_expose_expected_variants() {
        let windows = [
            WindowId::Hearth,
            WindowId::Mind,
            WindowId::Soma,
            WindowId::World,
            WindowId::Fate,
            WindowId::Command,
            WindowId::Protocol,
        ];
        let directions = [
            VimDirection::Up,
            VimDirection::Down,
            VimDirection::Left,
            VimDirection::Right,
        ];

        assert_eq!(windows.len(), 7);
        assert_eq!(directions.len(), 4);
        assert_eq!(WindowId::Hearth, WindowId::Hearth);
        assert_eq!(VimDirection::Left, VimDirection::Left);
    }

    #[test]
    fn app_action_covers_navigation_surface() {
        let actions = vec![
            AppAction::Quit,
            AppAction::NextScreen,
            AppAction::PrevScreen,
            AppAction::Resize(120, 40),
            AppAction::GotoScreen(ScreenId::CommandSteer),
            AppAction::GotoWindow(WindowId::Command),
            AppAction::OpenCommandPalette,
            AppAction::CloseCommandPalette,
            AppAction::ExecuteCommand(3),
            AppAction::PaletteInput('/'),
            AppAction::PaletteBackspace,
            AppAction::PaletteSelectNext,
            AppAction::PaletteSelectPrev,
            AppAction::ShowHelp,
            AppAction::HideHelp,
            AppAction::CloseModal,
            AppAction::ConfirmModal,
            AppAction::ModalInput('y'),
            AppAction::ModalBackspace,
            AppAction::EnterVimMode,
            AppAction::ExitVimMode,
            AppAction::VimNavigate(VimDirection::Down),
            AppAction::VimCommand("wq".to_string()),
            AppAction::ScrollUp,
            AppAction::ScrollDown,
            AppAction::ScrollTop,
            AppAction::ScrollBottom,
        ];

        assert_eq!(actions.len(), 27);
    }

    #[test]
    fn sys_metrics_default_starts_empty() {
        let metrics = SysMetrics::default();

        assert_eq!(metrics.cpu_pct, 0.0);
        assert!(metrics.cpu_history.is_empty());
        assert!(metrics.mem_history.is_empty());
        assert!(metrics.net_rx_history.is_empty());
        assert!(metrics.net_tx_history.is_empty());
        assert!(metrics.disk_read_history.is_empty());
        assert!(metrics.disk_write_history.is_empty());
        assert_eq!(metrics.disk_total_bytes, 0);
    }

    #[test]
    fn progress_state_defaults_with_tasks() {
        let state = AppState::default();
        assert!(!state.progress.tasks.is_empty());
        assert_eq!(state.progress.tasks[0].status, TaskStatus::Active);
        assert!(state.progress.eta_remaining_secs() > 0.0);
    }

    #[test]
    fn progress_tick_advances_active_task() {
        let mut state = AppState::default();
        let initial_elapsed = state.progress.tasks[0].elapsed_secs;
        state.progress.tick(1.0);
        assert!(state.progress.tasks[0].elapsed_secs > initial_elapsed);
        assert_eq!(
            state.progress.tasks[0].elapsed_secs,
            MAX_SCAFFOLD_TICK_DT_SECS
        );
    }

    #[test]
    fn progress_task_completes_and_advances() {
        let mut state = AppState::default();
        // tick() clamps dt to 0.1s, so loop enough to exceed the 8s estimate
        for _ in 0..100 {
            state.progress.tick(1.0);
        }
        assert_eq!(state.progress.tasks[0].status, TaskStatus::Done);
        assert_eq!(state.progress.tasks[1].status, TaskStatus::Active);
    }

    #[test]
    fn atmosphere_tick_clamps_large_dt_without_rewinding() {
        let mut atmosphere = Atmosphere::default();

        atmosphere.tick(0.5);
        assert_eq!(atmosphere.dt, MAX_SCAFFOLD_TICK_DT_SECS);
        assert_eq!(atmosphere.elapsed_secs, MAX_SCAFFOLD_TICK_DT_SECS);
        assert_eq!(atmosphere.frame(), 1);

        atmosphere.tick(-1.0);
        assert_eq!(atmosphere.dt, 0.0);
        assert_eq!(atmosphere.elapsed_secs, MAX_SCAFFOLD_TICK_DT_SECS);
        assert_eq!(atmosphere.frame(), 2);
    }

    #[test]
    fn format_duration_produces_expected_output() {
        assert_eq!(format_duration(42.0), "42s");
        assert_eq!(format_duration(125.0), "2m05s");
        assert_eq!(format_duration(3723.0), "1h02m03s");
        assert_eq!(format_duration(60.01), "1m01s");
        assert_eq!(format_duration(-5.0), "0s");
        assert_eq!(format_duration(f64::NAN), "0s");
    }
}
