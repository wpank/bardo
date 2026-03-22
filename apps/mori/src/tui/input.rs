use crate::state::{AgentPaneGroup, ConfirmAction, FocusZone, InputMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Actions the TUI can request from the event loop
#[derive(Debug, Clone)]
pub enum TuiAction {
    Quit,
    SwitchTab(usize),
    SelectPlanUp,
    SelectPlanDown,
    ScrollLogUp,
    ScrollLogDown,
    SwitchAgentTab(usize),
    ApproveCommand,
    /// E4: Approve all pending approvals at once
    ApproveAll,
    RejectCommand,
    StartInject,
    SubmitInject(String),
    CancelInject,
    InputChar(char),
    InputBackspace,
    ShowHelp,
    FocusNext,
    FocusPrev,
    ScrollFocusedUp,
    ScrollFocusedDown,
    ExpandCollapse,
    ShowPlanDetail,
    ClosePlanDetail,
    ScrollDetailUp,
    ScrollDetailDown,
    ScrollDetailPageUp,
    ScrollDetailPageDown,
    ScrollAgentUp,
    ScrollAgentDown,
    ScrollAgentEnd,
    ScrollDiffUp,
    ScrollDiffDown,
    RestartPhase,
    RestartPlan,
    SwitchDetailTab,
    ToggleAgentPaneGroup,
    DismissNotification,
    ConfigUp,
    ConfigDown,
    ConfigLeft,
    ConfigRight,
    ConfigSelect,
    ForceAdvance,
    ResetPlanState,
    ReverifyPlan,
    /// Request confirmation for a destructive action
    RequestConfirm(ConfirmAction),
    /// User confirmed the pending action
    ConfirmYes,
    /// User cancelled the pending action
    ConfirmNo,
    /// Toggle pause/resume
    TogglePause,
    VerifyTabNext,
    VerifyTabPrev,
    ShowWaveOverview,
    ShowAgentPoolModal,
    /// Switch the detail panel sub-tab (0=Agents, 1=Output, 2=Diff, 3=Errors, 4=Git)
    SwitchDetailSubTab(usize),
    /// Enter filter mode for plan tree
    StartFilter,
    /// Accept current filter and return to normal mode
    AcceptFilter,
    /// Cancel filter and clear text
    CancelFilter,
    /// Collapse/expand wave in plan tree
    CollapseExpand,
    /// Show task detail modal
    ShowTaskDetail,
    /// Close task detail modal
    CloseTaskDetail,
    /// Scroll task detail modal up
    ScrollTaskDetailUp,
    /// Scroll task detail modal down
    ScrollTaskDetailDown,
    /// Open the task picker modal
    OpenTaskPicker,
    /// Close the task picker modal
    CloseTaskPicker,
    /// Move cursor up in task picker
    TaskPickerUp,
    /// Move cursor down in task picker
    TaskPickerDown,
    /// Confirm selected task in picker
    TaskPickerConfirm,
    /// Prepare a merge-batch-to-main confirm dialog (gathers info from RunState first)
    PrepareMergeBatchToMain,
    /// Generic cursor up (used in Plans/Agents/Git views)
    NavigateUp,
    /// Generic cursor down
    NavigateDown,
    /// Page-jump up (10 items) in plan tree
    NavigatePageUp,
    /// Page-jump down (10 items) in plan tree
    NavigatePageDown,
    /// Move to next wave (Plans tab)
    WaveNext,
    /// Move to previous wave (Plans tab)
    WavePrev,
    /// Expand or drill into selected item
    DrillIn,
    /// Collapse or go back to parent level
    DrillOut,
    None,
}

pub fn handle_key(
    key: KeyEvent,
    input_mode: &InputMode,
    message_input: &str,
    focus: &FocusZone,
    show_plan_detail: bool,
    active_tab: usize,
    agent_pane_group: &AgentPaneGroup,
    show_task_detail: bool,
    selected_plan_name: &str,
    show_task_picker: bool,
) -> TuiAction {
    match input_mode {
        InputMode::Normal => {
            // Task picker modal intercepts keys when visible
            if show_task_picker {
                return match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => TuiAction::CloseTaskPicker,
                    KeyCode::Up | KeyCode::Char('k') => TuiAction::TaskPickerUp,
                    KeyCode::Down | KeyCode::Char('j') => TuiAction::TaskPickerDown,
                    KeyCode::Enter => TuiAction::TaskPickerConfirm,
                    _ => TuiAction::None,
                };
            }
            // Task detail modal intercepts keys when visible
            if show_task_detail {
                return match key.code {
                    KeyCode::Esc => TuiAction::CloseTaskDetail,
                    KeyCode::Up | KeyCode::Char('k') => TuiAction::ScrollTaskDetailUp,
                    KeyCode::Down | KeyCode::Char('j') => TuiAction::ScrollTaskDetailDown,
                    KeyCode::Char('q') => TuiAction::Quit,
                    _ => TuiAction::None,
                };
            }
            handle_normal_key(
                key,
                focus,
                show_plan_detail,
                active_tab,
                agent_pane_group,
                selected_plan_name,
            )
        }
        InputMode::Inject => handle_inject_key(key, message_input),
        InputMode::Filter => handle_filter_key(key),
        InputMode::Confirm => handle_confirm_key(key),
    }
}

fn handle_normal_key(
    key: KeyEvent,
    focus: &FocusZone,
    show_plan_detail: bool,
    active_tab: usize,
    agent_pane_group: &AgentPaneGroup,
    selected_plan_name: &str,
) -> TuiAction {
    // Ctrl-C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return TuiAction::Quit;
    }

    // Global actions available on all tabs (ctrl+ for destructive ops → confirm first)
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        let plan = selected_plan_name.to_string();
        match key.code {
            KeyCode::Char('r') => return TuiAction::RequestConfirm(ConfirmAction::RestartAllPlans),
            KeyCode::Char('x') => {
                return TuiAction::RequestConfirm(ConfirmAction::ForceAdvance(plan))
            }
            KeyCode::Char('d') => {
                return TuiAction::RequestConfirm(ConfirmAction::ResetSelectedPlan(plan))
            }
            KeyCode::Char('g') => return TuiAction::RequestConfirm(ConfirmAction::GitReconcile),
            KeyCode::Char('a') => return TuiAction::ApproveAll,
            KeyCode::Char('t') => return TuiAction::OpenTaskPicker,
            _ => {}
        }
    }

    // Plans tab (1): hierarchical navigation
    if active_tab == 1 {
        return match key.code {
            KeyCode::Up | KeyCode::Char('k') => TuiAction::NavigateUp,
            KeyCode::Down | KeyCode::Char('j') => TuiAction::NavigateDown,
            KeyCode::PageUp => TuiAction::NavigatePageUp,
            KeyCode::PageDown => TuiAction::NavigatePageDown,
            KeyCode::Right | KeyCode::Char('l') => TuiAction::WaveNext,
            KeyCode::Left | KeyCode::Char('h') => TuiAction::WavePrev,
            KeyCode::Enter => TuiAction::DrillIn,
            KeyCode::Esc => TuiAction::DrillOut,
            KeyCode::Tab => TuiAction::FocusNext,
            KeyCode::BackTab => TuiAction::FocusPrev,
            KeyCode::Char('/') => TuiAction::StartFilter,
            KeyCode::Char('m') => TuiAction::PrepareMergeBatchToMain,
            KeyCode::Char('r') => TuiAction::RequestConfirm(ConfirmAction::RestartPhase),
            KeyCode::Char('q') => TuiAction::Quit,
            KeyCode::Char('?') => TuiAction::ShowHelp,
            KeyCode::F(n) if (1..=6).contains(&n) => TuiAction::SwitchTab((n - 1) as usize),
            KeyCode::Char(c)
                if ('1'..='6').contains(&c) && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                TuiAction::SwitchTab((c as usize) - ('1' as usize))
            }
            _ => TuiAction::None,
        };
    }

    // Agents tab (2): agent list navigation
    if active_tab == 2 {
        return match key.code {
            KeyCode::Up | KeyCode::Char('k') => TuiAction::NavigateUp,
            KeyCode::Down | KeyCode::Char('j') => TuiAction::NavigateDown,
            KeyCode::Tab => TuiAction::FocusNext,
            KeyCode::BackTab => TuiAction::FocusPrev,
            KeyCode::End => TuiAction::ScrollAgentEnd,
            KeyCode::Char('`') => TuiAction::SwitchAgentTab(usize::MAX),
            KeyCode::Char('r') => TuiAction::RequestConfirm(ConfirmAction::RestartPhase),
            KeyCode::Char('q') => TuiAction::Quit,
            KeyCode::Char('?') => TuiAction::ShowHelp,
            KeyCode::F(n) if (1..=6).contains(&n) => TuiAction::SwitchTab((n - 1) as usize),
            KeyCode::Char(c)
                if ('1'..='6').contains(&c) && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                TuiAction::SwitchTab((c as usize) - ('1' as usize))
            }
            _ => TuiAction::None,
        };
    }

    // Git tab (3): branch tree navigation
    if active_tab == 3 {
        return match key.code {
            KeyCode::Up | KeyCode::Char('k') => TuiAction::NavigateUp,
            KeyCode::Down | KeyCode::Char('j') => TuiAction::NavigateDown,
            KeyCode::Enter => TuiAction::DrillIn,
            KeyCode::Char('r') => TuiAction::RequestConfirm(ConfirmAction::RestartPhase),
            KeyCode::Char('q') => TuiAction::Quit,
            KeyCode::Char('?') => TuiAction::ShowHelp,
            KeyCode::F(n) if (1..=6).contains(&n) => TuiAction::SwitchTab((n - 1) as usize),
            KeyCode::Char(c)
                if ('1'..='6').contains(&c) && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                TuiAction::SwitchTab((c as usize) - ('1' as usize))
            }
            _ => TuiAction::None,
        };
    }

    // Config tab (5) gets its own input handling
    if active_tab == 5 {
        return match key.code {
            KeyCode::Up | KeyCode::Char('k') => TuiAction::ConfigUp,
            KeyCode::Down | KeyCode::Char('j') => TuiAction::ConfigDown,
            KeyCode::Left | KeyCode::Char('h') => TuiAction::ConfigLeft,
            KeyCode::Right | KeyCode::Char('l') => TuiAction::ConfigRight,
            KeyCode::Enter | KeyCode::Char(' ') => TuiAction::ConfigSelect,
            KeyCode::Char('r') => TuiAction::RequestConfirm(ConfirmAction::RestartPhase),
            KeyCode::Char('q') => TuiAction::Quit,
            KeyCode::Char('?') => TuiAction::ShowHelp,
            KeyCode::F(n) if (1..=6).contains(&n) => TuiAction::SwitchTab((n - 1) as usize),
            KeyCode::Char(c)
                if ('1'..='6').contains(&c) && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                TuiAction::SwitchTab((c as usize) - ('1' as usize))
            }
            _ => TuiAction::None,
        };
    }

    match key.code {
        KeyCode::Char('q') => TuiAction::Quit,

        // Tab switching via F-keys
        KeyCode::F(n) if (1..=6).contains(&n) => TuiAction::SwitchTab((n - 1) as usize),

        // Alt+1-7: direct agent tab selection (must be before plain number match)
        KeyCode::Char(c)
            if ('1'..='7').contains(&c) && key.modifiers.contains(KeyModifiers::ALT) =>
        {
            TuiAction::SwitchAgentTab((c as usize) - ('1' as usize))
        }

        // Tab switching via number keys
        KeyCode::Char(c) if ('1'..='6').contains(&c) => {
            TuiAction::SwitchTab((c as usize) - ('1' as usize))
        }

        // Detail sub-tab switching (dashboard mode)
        KeyCode::Char('a') => TuiAction::SwitchDetailSubTab(0), // Agents
        KeyCode::Char('o') => TuiAction::SwitchDetailSubTab(1), // Output
        KeyCode::Char('d') => TuiAction::SwitchDetailSubTab(2), // Diff
        KeyCode::Char('e') => TuiAction::SwitchDetailSubTab(3), // Errors
        KeyCode::Char('g') => TuiAction::SwitchDetailSubTab(4), // Git

        // Filter
        KeyCode::Char('/') => TuiAction::StartFilter,

        // Focus cycling / modal tab switching
        KeyCode::Tab => {
            if show_plan_detail {
                TuiAction::SwitchDetailTab
            } else {
                TuiAction::FocusNext
            }
        }
        KeyCode::BackTab => TuiAction::FocusPrev,

        // Backtick cycles agent sub-tabs (old Tab behavior)
        KeyCode::Char('`') => TuiAction::SwitchAgentTab(usize::MAX),

        // Up/Down: context-dependent on focus zone
        KeyCode::Up | KeyCode::Char('k') => {
            if show_plan_detail {
                TuiAction::ScrollDetailUp
            } else {
                match focus {
                    FocusZone::Plans => TuiAction::SelectPlanUp,
                    FocusZone::Tasks => TuiAction::ScrollFocusedUp,
                    FocusZone::AgentOutput => TuiAction::ScrollAgentUp,
                    FocusZone::CommandOutput => TuiAction::ScrollDiffUp,
                }
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if show_plan_detail {
                TuiAction::ScrollDetailDown
            } else {
                match focus {
                    FocusZone::Plans => TuiAction::SelectPlanDown,
                    FocusZone::Tasks => TuiAction::ScrollFocusedDown,
                    FocusZone::AgentOutput => TuiAction::ScrollAgentDown,
                    FocusZone::CommandOutput => TuiAction::ScrollDiffDown,
                }
            }
        }

        // Left/Right in plan tree: collapse/expand wave
        KeyCode::Left | KeyCode::Char('h') if *focus == FocusZone::Plans && !show_plan_detail => {
            TuiAction::CollapseExpand
        }
        KeyCode::Right | KeyCode::Char('l') if *focus == FocusZone::Plans && !show_plan_detail => {
            TuiAction::CollapseExpand
        }

        // Left/Right: switch detail tab in modal
        KeyCode::Left | KeyCode::Right if show_plan_detail => TuiAction::SwitchDetailTab,

        // Enter: context-dependent on focus zone
        KeyCode::Enter => match focus {
            FocusZone::Plans => TuiAction::ShowPlanDetail,
            FocusZone::Tasks => TuiAction::ShowTaskDetail,
            FocusZone::AgentOutput | FocusZone::CommandOutput => TuiAction::None,
        },

        // Esc: close modals
        KeyCode::Esc => TuiAction::ClosePlanDetail,

        // PageUp/PageDown: modal-aware, then focus-aware per tab
        KeyCode::PageUp => {
            if show_plan_detail {
                TuiAction::ScrollDetailPageUp
            } else {
                match active_tab {
                    0..=3 => match focus {
                        FocusZone::Plans => TuiAction::SelectPlanUp,
                        FocusZone::Tasks => TuiAction::ScrollFocusedUp,
                        FocusZone::AgentOutput => TuiAction::ScrollAgentUp,
                        FocusZone::CommandOutput => TuiAction::ScrollDiffUp,
                    },
                    4 => TuiAction::ScrollLogUp,
                    _ => TuiAction::None,
                }
            }
        }
        KeyCode::PageDown => {
            if show_plan_detail {
                TuiAction::ScrollDetailPageDown
            } else {
                match active_tab {
                    0..=3 => match focus {
                        FocusZone::Plans => TuiAction::SelectPlanDown,
                        FocusZone::Tasks => TuiAction::ScrollFocusedDown,
                        FocusZone::AgentOutput => TuiAction::ScrollAgentDown,
                        FocusZone::CommandOutput => TuiAction::ScrollDiffDown,
                    },
                    4 => TuiAction::ScrollLogDown,
                    _ => TuiAction::None,
                }
            }
        }

        // End key: resume auto-scroll on agent output
        KeyCode::End => TuiAction::ScrollAgentEnd,

        // Space: pause/resume auto-scroll in output
        KeyCode::Char(' ') if *focus == FocusZone::AgentOutput => TuiAction::ScrollAgentEnd,

        // Approval
        KeyCode::Char('y') => TuiAction::ApproveCommand,
        KeyCode::Char('n') => TuiAction::RejectCommand,

        // Inject
        KeyCode::Char('i') => TuiAction::StartInject,

        // Restart current phase / re-verify plan
        KeyCode::Char('r') => TuiAction::RequestConfirm(ConfirmAction::RestartPhase),
        KeyCode::Char('c') => {
            TuiAction::RequestConfirm(ConfirmAction::ReverifyPlan(selected_plan_name.to_string()))
        }

        // Pause/resume pipeline
        KeyCode::Char('p') => TuiAction::TogglePause,

        // Modal shortcuts
        KeyCode::Char('w') => TuiAction::ShowWaveOverview,

        // Agent pane group toggle
        KeyCode::Char('v') => TuiAction::ToggleAgentPaneGroup,

        // Verify tab navigation (left/right when in verify mode)
        KeyCode::Left if *agent_pane_group == AgentPaneGroup::Verification && !show_plan_detail => {
            TuiAction::VerifyTabPrev
        }
        KeyCode::Right
            if *agent_pane_group == AgentPaneGroup::Verification && !show_plan_detail =>
        {
            TuiAction::VerifyTabNext
        }

        // Help
        KeyCode::Char('?') => TuiAction::ShowHelp,

        _ => TuiAction::None,
    }
}

fn handle_inject_key(key: KeyEvent, message_input: &str) -> TuiAction {
    match key.code {
        KeyCode::Esc => TuiAction::CancelInject,
        KeyCode::Enter => TuiAction::SubmitInject(message_input.to_string()),
        KeyCode::Backspace => TuiAction::InputBackspace,
        KeyCode::Char(c) => TuiAction::InputChar(c),
        _ => TuiAction::None,
    }
}

fn handle_filter_key(key: KeyEvent) -> TuiAction {
    match key.code {
        KeyCode::Esc => TuiAction::CancelFilter,
        KeyCode::Enter => TuiAction::AcceptFilter,
        KeyCode::Backspace => TuiAction::InputBackspace,
        KeyCode::Char(c) => TuiAction::InputChar(c),
        _ => TuiAction::None,
    }
}

fn handle_confirm_key(key: KeyEvent) -> TuiAction {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => TuiAction::ConfirmYes,
        KeyCode::Char('n') | KeyCode::Esc => TuiAction::ConfirmNo,
        _ => TuiAction::None,
    }
}
