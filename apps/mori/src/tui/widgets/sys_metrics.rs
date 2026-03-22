use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::state::RunState;
use crate::tui::theme::Theme;
use crate::tui::widgets::braille;

/// Render system metrics with braille sparklines in a bordered block.
///
/// Layout (4 rows inside border):
/// ```text
///  CPU  36.5%  ⠁⠃⠇⡇⣇⣿⣿⡇⣇⡇
///  MEM  50.3G  ⡇⡇⡇⣇⣇⣿⣿⣿⣿⣿
///  NET  ↑55K   ⠁⠁⠃⠁⠁⠁⠃⠁⠁⠁
///  DSK  R2.5M  ⠁⠁⠁⠁⠃⣿⠃⠁⠁⠁
/// ```
pub fn render(f: &mut Frame, area: Rect, state: &RunState) {
    let snap = &state.sys;

    let block = Block::default()
        .borders(Borders::ALL)
        .title("System")
        .style(Theme::block_style())
        .border_style(Style::default().fg(Theme::TEXT_GHOST))
        .title_style(Theme::title_style());
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width < 12 || inner.height < 2 {
        return;
    }

    let w = inner.width as usize;
    // label(4) + value(7) + space(1) = 12, rest is sparkline
    let spark_w = w.saturating_sub(12) / 2; // braille = 2 samples per cell

    let mut lines = Vec::new();

    // CPU
    {
        let val = format!("{:>5.1}%", snap.cpu_pct);
        let color = pct_color(snap.cpu_pct as f64 / 100.0);
        let data: Vec<f32> = snap.cpu_history.iter().copied().collect();
        let mut spans = vec![
            Span::styled("CPU ", Style::default().fg(Theme::TEXT_DIM)),
            Span::styled(val, Style::default().fg(color)),
            Span::styled(" ", Style::default()),
        ];
        spans.extend(braille::braille_spans_f32(&data, 100.0, spark_w, color));
        lines.push(Line::from(spans));
    }

    // MEM
    if inner.height >= 2 {
        let mem_frac = if snap.mem_total_bytes > 0 {
            snap.mem_used_bytes as f64 / snap.mem_total_bytes as f64
        } else {
            0.0
        };
        let val = format!("{:>6}", fmt_bytes(snap.mem_used_bytes));
        let color = pct_color(mem_frac);
        let data: Vec<f32> = snap.mem_history.iter().copied().collect();
        let mut spans = vec![
            Span::styled("MEM ", Style::default().fg(Theme::TEXT_DIM)),
            Span::styled(val, Style::default().fg(color)),
            Span::styled(" ", Style::default()),
        ];
        spans.extend(braille::braille_spans_f32(&data, 1.0, spark_w, color));
        lines.push(Line::from(spans));
    }

    // NET
    if inner.height >= 3 {
        let val = format!("↑{:>4}", fmt_rate(snap.net_rx_bytes_sec));
        let data: Vec<f64> = snap.net_rx_history.iter().copied().collect();
        let mut spans = vec![
            Span::styled("NET ", Style::default().fg(Theme::TEXT_DIM)),
            Span::styled(val, Style::default().fg(Theme::DREAM)),
            Span::styled(" ", Style::default()),
        ];
        spans.extend(braille::braille_spans_f64(
            &data,
            0.0,
            spark_w,
            Theme::DREAM,
        ));
        lines.push(Line::from(spans));
    }

    // DSK
    if inner.height >= 4 {
        let val = format!("R{:>4}", fmt_rate(snap.disk_read_bytes_sec));
        let data: Vec<f64> = snap.disk_r_history.iter().copied().collect();
        let mut spans = vec![
            Span::styled("DSK ", Style::default().fg(Theme::TEXT_DIM)),
            Span::styled(val, Style::default().fg(Theme::BONE_DIM)),
            Span::styled(" ", Style::default()),
        ];
        spans.extend(braille::braille_spans_f64(
            &data,
            0.0,
            spark_w,
            Theme::BONE_DIM,
        ));
        lines.push(Line::from(spans));
    }

    let para = Paragraph::new(lines);
    f.render_widget(para, inner);
}

fn pct_color(pct: f64) -> ratatui::style::Color {
    if pct >= 0.8 {
        Theme::EMBER
    } else if pct >= 0.5 {
        Theme::WARNING
    } else {
        Theme::SAGE
    }
}

fn fmt_bytes(b: u64) -> String {
    const GIB: u64 = 1 << 30;
    const MIB: u64 = 1 << 20;
    if b >= GIB {
        format!("{:.1}G", b as f64 / GIB as f64)
    } else if b >= MIB {
        format!("{:.0}M", b as f64 / MIB as f64)
    } else {
        format!("{}K", b / 1024)
    }
}

fn fmt_rate(bps: f64) -> String {
    const GIB: f64 = (1u64 << 30) as f64;
    const MIB: f64 = (1u64 << 20) as f64;
    const KIB: f64 = 1024.0;
    if bps >= GIB {
        format!("{:.1}G", bps / GIB)
    } else if bps >= MIB {
        format!("{:.1}M", bps / MIB)
    } else if bps >= KIB {
        format!("{:.1}K", bps / KIB)
    } else if bps > 0.5 {
        format!("{:.0}B", bps)
    } else {
        "0B".to_string()
    }
}
