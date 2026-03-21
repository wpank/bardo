//! Bridge route and transfer status widget.

use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

use crate::mock::protocol_data::{MockBridgeRoute, MockBridgeStatus};
use crate::palette;

/// Display configuration for BridgeStatusWidget.
#[derive(Debug, Clone)]
pub(crate) struct BridgeStatusWidgetConfig {
    /// Show the estimated time remaining as a progress bar.
    pub(crate) show_progress: bool,
}

impl Default for BridgeStatusWidgetConfig {
    fn default() -> Self {
        Self {
            show_progress: true,
        }
    }
}

/// Ratatui widget for a bridge route or active bridge transfer.
pub(crate) struct BridgeStatusWidget<'a> {
    pub(crate) route: &'a MockBridgeRoute,
    pub(crate) config: BridgeStatusWidgetConfig,
    /// Current time in seconds (unix epoch), used to compute progress for active transfers.
    /// Pass 0 for Quoted status (no progress bar logic needed).
    pub(crate) now_secs: u64,
    /// For InFlight transfers: unix timestamp when the transfer was submitted.
    /// Pass 0 if not applicable.
    pub(crate) submitted_at_secs: u64,
}

impl<'a> BridgeStatusWidget<'a> {
    pub(crate) fn new(route: &'a MockBridgeRoute) -> Self {
        Self {
            route,
            config: BridgeStatusWidgetConfig::default(),
            now_secs: 0,
            submitted_at_secs: 0,
        }
    }

    pub(crate) fn with_timing(mut self, now_secs: u64, submitted_at_secs: u64) -> Self {
        self.now_secs = now_secs;
        self.submitted_at_secs = submitted_at_secs;
        self
    }
}

/// Compute flight progress as a percentage (0-100), clamped.
pub(crate) fn flight_progress_pct(
    now_secs: u64,
    submitted_at_secs: u64,
    estimated_time_secs: u64,
) -> u16 {
    if submitted_at_secs == 0 || estimated_time_secs == 0 {
        return 0;
    }
    let elapsed = now_secs.saturating_sub(submitted_at_secs);
    ((elapsed as f64 / estimated_time_secs as f64) * 100.0).clamp(0.0, 100.0) as u16
}

/// Status badge symbol and label for each bridge status.
fn status_badge(status: &MockBridgeStatus) -> (&'static str, ratatui::style::Color) {
    match status {
        MockBridgeStatus::Quoted => ("\u{25cc} QUOTED", palette::TEXT_DIM),
        MockBridgeStatus::Pending => ("\u{25cc} PENDING", palette::WARNING),
        MockBridgeStatus::InFlight => ("\u{25c8} IN FLIGHT", palette::ROSE),
        MockBridgeStatus::Complete => ("\u{25cf} COMPLETE", palette::SUCCESS),
        MockBridgeStatus::Failed => ("\u{2717} FAILED", palette::DANGER),
    }
}

/// Write a string into the buffer at (x, y), returning the number of columns consumed.
fn write_str(buf: &mut Buffer, x: u16, y: u16, s: &str, style: Style) -> u16 {
    let area = *buf.area();
    let mut col = 0u16;
    for ch in s.chars() {
        let cx = x + col;
        if cx >= area.right() || y >= area.bottom() || y < area.top() || cx < area.left() {
            col += 1;
            continue;
        }
        buf.get_mut(cx, y).set_char(ch).set_style(style);
        col += 1;
    }
    col
}

/// Format a number with thousands separators.
fn format_amount(v: f64) -> String {
    let s = format!("{:.2}", v);
    let parts: Vec<&str> = s.split('.').collect();
    let int_part = parts[0];
    let dec_part = parts.get(1).unwrap_or(&"00");
    let bytes: Vec<u8> = int_part.bytes().collect();
    let mut result = String::new();
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(b as char);
    }
    result.push('.');
    result.push_str(dec_part);
    result
}

impl<'a> Widget for BridgeStatusWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let r = self.route;

        // Collapsed mode: if height < 4, just render status badge + amount on two lines.
        if area.height < 4 {
            let (badge_text, badge_color) = status_badge(&r.status);
            write_str(
                buf,
                area.x,
                area.y,
                badge_text,
                Style::default().fg(badge_color),
            );
            if area.height >= 2 {
                let amount_str = format!("{:.2} {}", r.amount, r.token_symbol);
                write_str(
                    buf,
                    area.x,
                    area.y + 1,
                    &amount_str,
                    Style::default().fg(palette::BONE),
                );
            }
            return;
        }

        // Full render with border.
        // Border title: " Across · ETH→Base " (omit route if width < 18)
        let title = if area.width >= 18 {
            format!(
                " {} \u{00b7} {}\u{2192}{} ",
                r.bridge_name, r.from_chain, r.to_chain
            )
        } else {
            format!(" {} ", r.bridge_name)
        };

        // Draw border
        let border_style = Style::default().fg(palette::BORDER);
        let active_style = Style::default().fg(palette::BORDER_ACTIVE);

        // Top border with title
        let top_y = area.y;
        buf.get_mut(area.x, top_y)
            .set_char(palette::BOX_TOP_LEFT)
            .set_style(border_style);
        let mut cx = area.x + 1;
        for ch in title.chars() {
            if cx >= area.right() - 1 {
                break;
            }
            buf.get_mut(cx, top_y).set_char(ch).set_style(active_style);
            cx += 1;
        }
        while cx < area.right() - 1 {
            buf.get_mut(cx, top_y)
                .set_char(palette::BOX_HORIZONTAL)
                .set_style(border_style);
            cx += 1;
        }
        if area.width > 1 {
            buf.get_mut(area.right() - 1, top_y)
                .set_char(palette::BOX_TOP_RIGHT)
                .set_style(border_style);
        }

        // Bottom border
        let bot_y = area.bottom() - 1;
        if bot_y > top_y {
            buf.get_mut(area.x, bot_y)
                .set_char(palette::BOX_BOTTOM_LEFT)
                .set_style(border_style);
            for bx in (area.x + 1)..(area.right() - 1) {
                buf.get_mut(bx, bot_y)
                    .set_char(palette::BOX_HORIZONTAL)
                    .set_style(border_style);
            }
            if area.width > 1 {
                buf.get_mut(area.right() - 1, bot_y)
                    .set_char(palette::BOX_BOTTOM_RIGHT)
                    .set_style(border_style);
            }
        }

        // Side borders
        for sy in (top_y + 1)..bot_y {
            buf.get_mut(area.x, sy)
                .set_char(palette::BOX_VERTICAL)
                .set_style(border_style);
            if area.width > 1 {
                buf.get_mut(area.right() - 1, sy)
                    .set_char(palette::BOX_VERTICAL)
                    .set_style(border_style);
            }
        }

        // Interior area (inside border)
        let inner_x = area.x + 1;
        let inner_w = area.width.saturating_sub(2);
        let inner_y = area.y + 1;
        let inner_h = area.height.saturating_sub(2);

        if inner_w == 0 || inner_h == 0 {
            return;
        }

        let mut row = 0u16;

        // Row 0: Amount line "5,000.00 USDC"
        if row < inner_h {
            let amount_str = format!("{} {}", format_amount(r.amount), r.token_symbol);
            write_str(
                buf,
                inner_x,
                inner_y + row,
                &amount_str,
                Style::default().fg(palette::BONE),
            );
            row += 1;
        }

        // Row 1: Fee + ETA "Fee: $1.85  |  ETA: ~18s"
        if row < inner_h {
            let fee_eta = format!(
                "Fee: ${:.2}  |  ETA: ~{}s",
                r.fee_usd, r.estimated_time_secs
            );
            write_str(
                buf,
                inner_x,
                inner_y + row,
                &fee_eta,
                Style::default().fg(palette::TEXT_DIM),
            );
            row += 1;
        }

        // Row 2: Status badge + progress bar
        if row < inner_h {
            let (badge_text, badge_color) = status_badge(&r.status);
            let badge_len = write_str(
                buf,
                inner_x,
                inner_y + row,
                badge_text,
                Style::default().fg(badge_color),
            );

            // Progress bar for InFlight only
            if self.config.show_progress && r.status == MockBridgeStatus::InFlight {
                let bar_width = (inner_w / 2).max(1);
                let bar_start = inner_x + inner_w - bar_width;
                let pct = flight_progress_pct(
                    self.now_secs,
                    self.submitted_at_secs,
                    r.estimated_time_secs,
                );
                let filled = ((pct as u32 * bar_width as u32) / 100) as u16;

                for bx in 0..bar_width {
                    let cx = bar_start + bx;
                    if cx >= area.right() - 1 || cx <= inner_x + badge_len {
                        continue;
                    }
                    if bx < filled {
                        buf.get_mut(cx, inner_y + row)
                            .set_char(palette::BLOCK_FULL)
                            .set_style(Style::default().fg(palette::ROSE));
                    } else {
                        buf.get_mut(cx, inner_y + row)
                            .set_char(palette::BLOCK_LIGHT)
                            .set_style(Style::default().fg(palette::TEXT_DIM));
                    }
                }
            }
            row += 1;
        }

        // Row 3: Route arrow "Ethereum ──→ Base"
        if row < inner_h {
            render_route_arrow(
                Rect::new(inner_x, inner_y + row, inner_w, 1),
                buf,
                &r.from_chain,
                &r.to_chain,
            );
        }
    }
}

/// Render the route arrow centered in the given area.
fn render_route_arrow(area: Rect, buf: &mut Buffer, from: &str, to: &str) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let arrow_str = " \u{2500}\u{2500}\u{2192} ";
    let total_len = from.len() + arrow_str.len() + to.len();
    let start_x = if (total_len as u16) < area.width {
        area.x + (area.width - total_len as u16) / 2
    } else {
        area.x
    };

    let mut cx = start_x;
    let y = area.y;

    // From chain name
    for ch in from.chars() {
        if cx >= area.right() {
            return;
        }
        buf.get_mut(cx, y)
            .set_char(ch)
            .set_style(Style::default().fg(palette::TEXT_PRIMARY));
        cx += 1;
    }

    // Arrow
    for ch in arrow_str.chars() {
        if cx >= area.right() {
            return;
        }
        buf.get_mut(cx, y)
            .set_char(ch)
            .set_style(Style::default().fg(palette::ROSE_DIM));
        cx += 1;
    }

    // To chain name
    for ch in to.chars() {
        if cx >= area.right() {
            return;
        }
        buf.get_mut(cx, y)
            .set_char(ch)
            .set_style(Style::default().fg(palette::TEXT_PRIMARY));
        cx += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn buffer_text(buf: &Buffer) -> String {
        (0..buf.area.height)
            .flat_map(|y| (0..buf.area.width).map(move |x| (x, y)))
            .map(|(x, y)| buf.get(x, y).symbol().to_string())
            .collect()
    }

    #[test]
    fn bridge_widget_renders_quoted_state() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let route = MockBridgeRoute::mock_default();
        terminal
            .draw(|frame| {
                frame.render_widget(BridgeStatusWidget::new(&route), frame.size());
            })
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Across"), "should contain bridge name");
        assert!(text.contains("USDC"), "should contain token symbol");
    }

    #[test]
    fn bridge_widget_inflight_no_panic() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut route = MockBridgeRoute::mock_default();
        route.status = MockBridgeStatus::InFlight;
        terminal
            .draw(|frame| {
                frame.render_widget(
                    BridgeStatusWidget::new(&route).with_timing(1_000_010, 1_000_000),
                    frame.size(),
                );
            })
            .unwrap();
    }

    #[test]
    fn bridge_widget_complete_no_panic() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut route = MockBridgeRoute::mock_default();
        route.status = MockBridgeStatus::Complete;
        terminal
            .draw(|frame| {
                frame.render_widget(BridgeStatusWidget::new(&route), frame.size());
            })
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("COMPLETE"), "should contain COMPLETE badge");
    }

    #[test]
    fn flight_progress_pct_clamped() {
        assert_eq!(flight_progress_pct(1_000_100, 1_000_000, 18), 100); // overrun -> 100
        assert_eq!(flight_progress_pct(1_000_009, 1_000_000, 18), 50); // 9s of 18s = 50%
        assert_eq!(flight_progress_pct(0, 0, 18), 0); // no timing data
    }

    #[test]
    fn flight_progress_zero_estimate() {
        assert_eq!(flight_progress_pct(100, 50, 0), 0);
    }

    #[test]
    fn collapsed_render_below_min_height() {
        let backend = TestBackend::new(30, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let route = MockBridgeRoute::mock_default();
        terminal
            .draw(|frame| {
                frame.render_widget(BridgeStatusWidget::new(&route), frame.size());
            })
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(
            text.contains("QUOTED"),
            "collapsed should show status badge"
        );
        assert!(text.contains("USDC"), "collapsed should show amount");
    }

    #[test]
    fn narrow_width_omits_route_from_title() {
        let backend = TestBackend::new(16, 6);
        let mut terminal = Terminal::new(backend).unwrap();
        let route = MockBridgeRoute::mock_default();
        terminal
            .draw(|frame| {
                frame.render_widget(BridgeStatusWidget::new(&route), frame.size());
            })
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(
            text.contains("Across"),
            "narrow should still show bridge name"
        );
    }

    #[test]
    fn route_arrow_contains_chain_names() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let route = MockBridgeRoute::mock_default();
        terminal
            .draw(|frame| {
                frame.render_widget(BridgeStatusWidget::new(&route), frame.size());
            })
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Ethereum"), "should show from_chain");
        assert!(text.contains("Base"), "should show to_chain");
    }
}
