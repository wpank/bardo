//! Home screen with creature silhouette, vitality gauge, connection status, and tick counter.

use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    palette::{
        BONE, BONE_DIM, BORDER, BORDER_ACTIVE, DANGER, DREAM, DREAM_DIM, ROSE, ROSE_DIM, SUCCESS,
        TEXT_DIM, TEXT_GHOST, WARNING,
    },
    screen::{Screen, ScreenId},
    state::{AppAction, AppState, ConnectionStatus},
    widgets::{BrailleSparkline, VitalityGauge, vitality_to_phase},
};

/// Home screen with creature silhouette, pipeline progress, and per-task ETA.
pub(crate) struct HomeScreen {
    focused: bool,
}

impl HomeScreen {
    pub(crate) fn new() -> Self {
        Self { focused: false }
    }
}

impl Screen for HomeScreen {
    fn id(&self) -> ScreenId {
        ScreenId::HearthOverview
    }

    fn title(&self) -> &str {
        "HEARTH"
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        // ── Left: creature panel ─────────────────────────────────
        let creature_border = if self.focused && state.tick_count % 8 < 4 {
            BORDER_ACTIVE
        } else {
            BORDER
        };

        let creature_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(creature_border))
            .title(Span::styled(" SPECTRE ", Style::default().fg(ROSE)));
        let creature_inner = creature_block.inner(chunks[0]);
        frame.render_widget(creature_block, chunks[0]);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled("  ◉ ◉  ", Style::default().fg(ROSE))),
                Line::from(Span::styled(" ░░░░░ ", Style::default().fg(ROSE_DIM))),
                Line::from(Span::styled(" ░   ░ ", Style::default().fg(ROSE_DIM))),
                Line::from(Span::styled("  ─ ─  ", Style::default().fg(ROSE_DIM))),
            ])
            .alignment(Alignment::Center),
            creature_inner,
        );

        // ── Right: data panel ─────────────────────────────────────
        let data_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // vitality gauge
                Constraint::Length(3), // connection status
                Constraint::Min(3),    // tick counter + help
            ])
            .split(chunks[1]);

        // ── Vitality gauge (top) ─────────────────────────────────
        let vitality_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER));
        let vitality_inner = vitality_block.inner(data_chunks[0]);
        frame.render_widget(vitality_block, data_chunks[0]);
        frame.render_widget(
            VitalityGauge {
                value: state.vitality.value,
                label: "VITALITY".to_string(),
                phase: vitality_to_phase(state.vitality.value),
            },
            vitality_inner,
        );

        // ── Connection status (middle) ───────────────────────────
        let (status_text, status_color) = match state.connection_status {
            ConnectionStatus::Connected => ("● CONNECTED", SUCCESS),
            ConnectionStatus::Connecting => ("◌ CONNECTING…", WARNING),
            ConnectionStatus::Disconnected => ("○ DISCONNECTED", DANGER),
        };
        frame.render_widget(
            Paragraph::new(status_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(BORDER)),
                )
                .style(Style::default().fg(status_color))
                .alignment(Alignment::Center),
            data_chunks[1],
        );

        // ── Tick counter + help (bottom) ──────────────────────────
        let tick_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER));
        let tick_inner = tick_block.inner(data_chunks[2]);
        frame.render_widget(tick_block, data_chunks[2]);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled(" tick: ", Style::default().fg(TEXT_DIM)),
                    Span::styled(state.tick_count.to_string(), Style::default().fg(ROSE)),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    " q=quit  Tab=next  Shift+Tab=prev",
                    Style::default().fg(TEXT_GHOST),
                )),
            ])
            .alignment(Alignment::Left),
            tick_inner,
        );
    }

    fn handle_key(&mut self, _key: KeyEvent) -> Option<AppAction> {
        None
    }

    fn on_focus(&mut self) {
        self.focused = true;
    }

    fn on_blur(&mut self) {
        self.focused = false;
    }
}

/// Renders the 4-column system resources panel: CPU | MEM | NET | DISK.
fn render_sys_panel(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let sys_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(" SYSTEM ", Style::default().fg(BONE)))
        .border_style(Style::default().fg(BORDER));
    let inner = sys_block.inner(area);
    frame.render_widget(sys_block, area);

    if inner.width < 8 || inner.height < 2 {
        return;
    }

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(inner);

    let rows = |col: Rect| {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(col)
    };

    let sys = &state.sys;

    // ── CPU ──────────────────────────────────────────────────────
    let cpu_frac = sys.cpu_pct as f64 / 100.0;
    let cpu_color = resource_color(cpu_frac);
    let r = rows(cols[0]);
    frame.render_widget(
        Paragraph::new(format!(" CPU  {:.1}%", sys.cpu_pct))
            .style(Style::default().fg(cpu_color).add_modifier(Modifier::BOLD)),
        r[0],
    );
    frame.render_widget(
        BrailleSparkline {
            data: sys.cpu_history.clone(),
            max_value: 100.0,
            color: cpu_color,
            label: None,
        },
        r[1],
    );
    frame.render_widget(
        Paragraph::new(format!(
            " {}",
            compact_bar(cpu_frac, cols[0].width.saturating_sub(2) as usize)
        ))
        .style(Style::default().fg(cpu_color)),
        r[2],
    );
    frame.render_widget(
        Paragraph::new(format!(
            " {}",
            if sys.mem_total_bytes > 0 {
                format!("{} cores active", num_cores_label(sys.cpu_pct))
            } else {
                String::new()
            }
        ))
        .style(Style::default().fg(TEXT_DIM)),
        r[3],
    );

    // ── MEM ──────────────────────────────────────────────────────
    let mem_frac = if sys.mem_total_bytes > 0 {
        sys.mem_used_bytes as f64 / sys.mem_total_bytes as f64
    } else {
        0.0
    };
    let mem_color = resource_color(mem_frac);
    let r = rows(cols[1]);
    frame.render_widget(
        Paragraph::new(format!(" MEM  {}", fmt_bytes(sys.mem_used_bytes)))
            .style(Style::default().fg(mem_color).add_modifier(Modifier::BOLD)),
        r[0],
    );
    frame.render_widget(
        BrailleSparkline {
            data: sys.mem_history.clone(),
            max_value: 1.0,
            color: mem_color,
            label: None,
        },
        r[1],
    );
    frame.render_widget(
        Paragraph::new(format!(
            " {}",
            compact_bar(mem_frac, cols[1].width.saturating_sub(2) as usize)
        ))
        .style(Style::default().fg(mem_color)),
        r[2],
    );
    frame.render_widget(
        Paragraph::new(format!(" / {}", fmt_bytes(sys.mem_total_bytes)))
            .style(Style::default().fg(TEXT_DIM)),
        r[3],
    );

    // ── NET ──────────────────────────────────────────────────────
    let r = rows(cols[2]);
    frame.render_widget(
        Paragraph::new(format!(" NET  ↑ {}", fmt_rate(sys.net_tx_bps)))
            .style(Style::default().fg(DREAM).add_modifier(Modifier::BOLD)),
        r[0],
    );
    frame.render_widget(
        BrailleSparkline {
            data: sys.net_tx_history.clone(),
            max_value: 0.0,
            color: DREAM,
            label: None,
        },
        r[1],
    );
    frame.render_widget(
        Paragraph::new(format!(" ↓ {}", fmt_rate(sys.net_rx_bps)))
            .style(Style::default().fg(DREAM_DIM)),
        r[2],
    );
    frame.render_widget(
        BrailleSparkline {
            data: sys.net_rx_history.clone(),
            max_value: 0.0,
            color: DREAM_DIM,
            label: None,
        },
        r[3],
    );

    // ── DISK ─────────────────────────────────────────────────────
    let disk_frac = if sys.disk_total_bytes > 0 {
        sys.disk_used_bytes as f64 / sys.disk_total_bytes as f64
    } else {
        0.0
    };
    let r = rows(cols[3]);
    frame.render_widget(
        Paragraph::new(format!(" DISK  R {}", fmt_rate(sys.disk_read_bps)))
            .style(Style::default().fg(BONE_DIM).add_modifier(Modifier::BOLD)),
        r[0],
    );
    frame.render_widget(
        BrailleSparkline {
            data: sys.disk_read_history.clone(),
            max_value: 0.0,
            color: BONE_DIM,
            label: None,
        },
        r[1],
    );
    frame.render_widget(
        Paragraph::new(format!(
            " W {}  {}",
            fmt_rate(sys.disk_write_bps),
            fmt_bytes(sys.disk_used_bytes)
        ))
        .style(Style::default().fg(TEXT_DIM)),
        r[2],
    );
    frame.render_widget(
        BrailleSparkline {
            data: sys.disk_write_history.clone(),
            max_value: 0.0,
            color: resource_color(disk_frac),
            label: None,
        },
        r[3],
    );
}

/// Color based on utilisation fraction: green → yellow → red.
fn resource_color(fraction: f64) -> Color {
    if fraction >= 0.8 {
        DANGER
    } else if fraction >= 0.5 {
        WARNING
    } else {
        SUCCESS
    }
}

/// Rough active-core count label from global CPU %.
fn num_cores_label(cpu_pct: f32) -> String {
    // Just show rounded utilisation; real core count needs sysinfo CPUs list.
    format!("{:.0}% util", cpu_pct)
}

/// Human-readable byte count (SI prefix, single decimal).
fn fmt_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1}G", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1}M", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1_024 {
        format!("{:.1}K", bytes as f64 / 1_024.0)
    } else {
        format!("{bytes}B")
    }
}

/// Human-readable bytes-per-second rate.
fn fmt_rate(bps: f64) -> String {
    format!("{}/s", fmt_bytes(bps as u64))
}

/// Horizontal fill bar using block chars.
fn compact_bar(fraction: f64, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let fill = ((fraction.clamp(0.0, 1.0) * width as f64) as usize).min(width);
    let empty = width - fill;
    format!("{}{}", "█".repeat(fill), "░".repeat(empty))
}

/// Modulate a color's brightness with the heartbeat oscillator.
fn pulse_color(base: Color, heartbeat: f64) -> Color {
    match base {
        Color::Rgb(r, g, b) => {
            let scale = heartbeat.clamp(0.9, 1.1);
            Color::Rgb(
                (r as f64 * scale).min(255.0) as u8,
                (g as f64 * scale).min(255.0) as u8,
                (b as f64 * scale).min(255.0) as u8,
            )
        }
        other => other,
    }
}

fn tick_waveform(tick_count: u64, width: usize) -> String {
    const SYMBOLS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let phase = tick_count as f64 * 0.17;

    (0..width)
        .map(|index| {
            let sample = (phase + index as f64 * 0.42).sin();
            let normalized = ((sample + 1.0) * 0.5) * (SYMBOLS.len() - 1) as f64;
            SYMBOLS[normalized.round() as usize]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{ConnectionStatus, MockVitality};

    #[test]
    fn home_screen_defaults_to_unfocused() {
        assert!(!HomeScreen::new().focused);
    }

    #[test]
    fn home_screen_defers_global_keys_to_the_app() {
        let mut screen = HomeScreen::new();

        assert_eq!(
            screen.handle_key(KeyEvent::from(crossterm::event::KeyCode::Char('q'))),
            None
        );
        assert_eq!(
            screen.handle_key(KeyEvent::from(crossterm::event::KeyCode::Tab)),
            None
        );
        assert_eq!(
            screen.handle_key(KeyEvent::from(crossterm::event::KeyCode::BackTab)),
            None
        );
    }

    #[test]
    fn tick_waveform_has_requested_width() {
        assert_eq!(tick_waveform(123, 18).chars().count(), 18);
    }

    #[test]
    fn connection_status_labels_cover_all_variants() {
        assert_eq!(ConnectionStatus::Connected.label(), "CONNECTED");
        assert_eq!(ConnectionStatus::Connecting.label(), "CONNECTING…");
        assert_eq!(ConnectionStatus::Disconnected.label(), "DISCONNECTED");
        assert_eq!(MockVitality::default(), MockVitality { value: 0.75 });
    }
}
