//! Rich protocol panels for the Chain Intelligence (protocol views) screen.
//!
//! Renders mock-backed Uniswap pool, lending, vault, and bridge summaries.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::mock::protocol_data::{
    MockBridgeRoute, MockBridgeStatus, MockLendingMarket, MockPoolState,
};
use crate::palette::{
    BLOCK_FULL, BLOCK_LIGHT, BONE, BORDER, DANGER, ROSE, ROSE_DIM, SUCCESS, TEXT_DIM, TEXT_PRIMARY,
    WARNING,
};
use crate::state::format_duration;
use crate::widgets::BrailleSparkline;

/// Compact USD per protocol-views spec (B / M / K / plain).
pub(crate) fn format_compact_usd(value: f64) -> String {
    if !value.is_finite() || value < 0.0 {
        return "$—".to_string();
    }
    if value >= 1_000_000_000.0 {
        format!("${:.1}B", value / 1_000_000_000.0)
    } else if value >= 1_000_000.0 {
        format!("${:.1}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("${:.1}K", value / 1_000.0)
    } else {
        format!("${value:.2}")
    }
}

/// Fee tier as a percentage with two decimals (basis points → percent).
pub(crate) fn format_fee_tier_pct(fee_bps: u16) -> String {
    let pct = f64::from(fee_bps) / 100.0;
    format!("{pct:.2}%")
}

/// Rate in decimal form (e.g. `0.042`) → display percent.
pub(crate) fn format_rate_as_percent(rate: f64) -> String {
    format!("{:.2}%", rate * 100.0)
}

fn format_int_grouped(n: i64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

fn format_price_display(price: f64) -> String {
    if !price.is_finite() {
        return "—".to_string();
    }
    let int = price.floor() as i64;
    let frac = ((price - price.floor()) * 100.0).round().clamp(0.0, 99.0) as i32;
    format!("{}.{}", format_int_grouped(int), format!("{frac:02}"))
}

fn utilization_bar_color(utilization: f64) -> Color {
    let u = utilization.clamp(0.0, 1.0);
    if u < 0.80 {
        SUCCESS
    } else if u < 0.95 {
        WARNING
    } else {
        DANGER
    }
}

fn borrow_apy_style(borrow_apy: f64) -> Style {
    if borrow_apy > 0.10 {
        Style::default().fg(WARNING)
    } else {
        Style::default().fg(ROSE_DIM)
    }
}

fn bridge_progress_fraction(route: &MockBridgeRoute) -> Option<f64> {
    if route.status != MockBridgeStatus::InFlight {
        return None;
    }
    let est = route.estimated_seconds? as f64;
    let el = route.elapsed_seconds? as f64;
    if est <= 0.0 {
        return Some(0.0);
    }
    Some((el / est).clamp(0.0, 1.0))
}

fn bridge_status_line(status: MockBridgeStatus) -> (&'static str, Color) {
    match status {
        MockBridgeStatus::Quoted => ("◌ QUOTED", TEXT_DIM),
        MockBridgeStatus::Pending => ("◌ PENDING", WARNING),
        MockBridgeStatus::InFlight => ("◈ IN FLIGHT", ROSE),
        MockBridgeStatus::Complete => ("● COMPLETE", SUCCESS),
        MockBridgeStatus::Failed => ("✗ FAILED", DANGER),
    }
}

fn progress_bar_color(t: f64) -> Color {
    let t = t.clamp(0.0, 1.0);
    let r = (170.0 + (112.0 - 170.0) * t) as u8;
    let g = (112.0 + (136.0 - 112.0) * t) as u8;
    let b = (136.0 + (122.0 - 136.0) * t) as u8;
    Color::Rgb(r, g, b)
}

fn set_row(buf: &mut Buffer, area: Rect, y_off: u16, text: &str, style: Style) {
    if y_off >= area.height {
        return;
    }
    buf.set_stringn(area.x, area.y + y_off, text, area.width as usize, style);
}

fn tick_range_line(width: usize, pool: &MockPoolState) -> String {
    let w = width.max(3);
    let inner = w.saturating_sub(2);
    let in_range = pool.tick_range.is_in_range();
    let pos = pool.tick_range.position_fraction().unwrap_or(0.5);
    let max_ci = inner.saturating_sub(1);
    let cursor = (max_ci as f64 * pos).round() as usize;
    let cursor = cursor.min(max_ci);
    let mut s = String::with_capacity(w);
    s.push(' ');
    for i in 0..inner {
        let ch = if i == cursor {
            '◆'
        } else if in_range {
            '█'
        } else {
            '░'
        };
        s.push(ch);
    }
    s.push(' ');
    s
}

/// Uniswap-style pool summary (mock data).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct UniswapPoolWidget<'a> {
    pub(crate) state: &'a MockPoolState,
}

impl Widget for UniswapPoolWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let fee = format_fee_tier_pct(self.state.fee_bps);
        let header = format!(
            "{} / {} · {} · {}",
            self.state.base_symbol, self.state.quote_symbol, fee, self.state.chain
        );
        let header_trim: String = header.chars().take(area.width as usize).collect();
        set_row(buf, area, 0, &header_trim, Style::default().fg(ROSE_DIM));

        if area.height < 2 {
            return;
        }

        let price_line = format!(
            "{} {}",
            format_price_display(self.state.price_quote),
            self.state.quote_symbol
        );
        let price_pad = area.width.saturating_sub(price_line.len() as u16) as usize;
        let price_row = format!(
            "{:>width$}",
            price_line,
            width = price_pad + price_line.len()
        );
        let price_trim: String = price_row
            .chars()
            .rev()
            .take(area.width as usize)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        set_row(
            buf,
            area,
            1,
            &price_trim,
            Style::default().fg(BONE).add_modifier(Modifier::BOLD),
        );

        let mut row = 2;
        if area.height > row && area.width > 4 {
            let bar_w = area.width as usize;
            let bar = tick_range_line(bar_w, self.state);
            let bar_style = if self.state.tick_range.is_in_range() {
                Style::default().fg(SUCCESS)
            } else {
                Style::default().fg(WARNING)
            };
            set_row(buf, area, row, &bar, bar_style);
            row += 1;
        }

        if area.height > row {
            let spark_h = if area.height - row >= 2 { 2 } else { 1 };
            let spark_area = Rect::new(area.x, area.y + row, area.width, spark_h);
            BrailleSparkline {
                data: self.state.depth_samples.clone(),
                max_value: 0.0,
                color: ROSE,
                label: None,
            }
            .render(spark_area, buf);
            row += spark_h;
        }

        if area.height > row {
            let footer = format!(
                "TVL {}  ·  Vol {}",
                format_compact_usd(self.state.tvl_usd),
                format_compact_usd(self.state.volume_24h_usd)
            );
            let footer_trim: String = footer.chars().take(area.width as usize).collect();
            set_row(buf, area, row, &footer_trim, Style::default().fg(TEXT_DIM));
        }
    }
}

/// Lending market summary (mock data).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LendingMarketWidget<'a> {
    pub(crate) state: &'a MockLendingMarket,
}

impl Widget for LendingMarketWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let header = format!(
            "{} · {} · {}",
            self.state.protocol, self.state.asset_symbol, self.state.chain
        );
        let header_trim: String = header.chars().take(area.width as usize).collect();
        set_row(buf, area, 0, &header_trim, Style::default().fg(ROSE_DIM));

        if area.height < 2 {
            return;
        }

        let u_pct = format_rate_as_percent(self.state.utilization);
        let util_label = format!("Utilization  {u_pct}");
        let util_trim: String = util_label.chars().take(area.width as usize).collect();
        set_row(buf, area, 1, &util_trim, Style::default().fg(TEXT_PRIMARY));

        let mut row = 2;
        if area.height > row && area.width > 4 {
            let bar_y = area.y + row;
            let inner_w = area.width.saturating_sub(2);
            let fill = (f64::from(inner_w) * self.state.utilization.clamp(0.0, 1.0)).round() as u16;
            let fill_color = utilization_bar_color(self.state.utilization);
            for offset in 0..inner_w {
                let gx = area.x + 1 + offset;
                let filled = offset < fill;
                let cell = buf.get_mut(gx, bar_y);
                cell.set_char(if filled { BLOCK_FULL } else { BLOCK_LIGHT });
                cell.set_style(Style::default().fg(if filled { fill_color } else { BORDER }));
            }
            row += 1;
        }

        if area.height > row {
            let y = area.y + row;
            let supply_txt = format!(
                "Supply {:>6}",
                format_rate_as_percent(self.state.supply_apy)
            );
            let mid = "  │  ";
            let borrow_txt = format!(
                "Borrow {:>6}",
                format_rate_as_percent(self.state.borrow_apy)
            );
            let max = area.width as usize;
            let supply_len = supply_txt.chars().count().min(max);
            buf.set_stringn(
                area.x,
                y,
                &supply_txt,
                supply_len,
                Style::default().fg(SUCCESS),
            );
            let mut used = supply_txt.chars().count().min(max);
            if used < max {
                let mid_len = mid.chars().count().min(max - used);
                buf.set_stringn(
                    area.x + used as u16,
                    y,
                    mid,
                    mid_len,
                    Style::default().fg(TEXT_DIM),
                );
                used += mid_len;
            }
            if used < max {
                let borrow_len = borrow_txt.chars().count().min(max - used);
                buf.set_stringn(
                    area.x + used as u16,
                    y,
                    &borrow_txt,
                    borrow_len,
                    borrow_apy_style(self.state.borrow_apy),
                );
            }
            row += 1;
        }

        if area.height > row {
            let foot = format!(
                "Sup {}  ·  Borr {}",
                format_compact_usd(self.state.total_supplied_usd),
                format_compact_usd(self.state.total_borrowed_usd)
            );
            let foot_trim: String = foot.chars().take(area.width as usize).collect();
            set_row(buf, area, row, &foot_trim, Style::default().fg(TEXT_DIM));
        }
    }
}

/// Bridge route / transfer summary (mock data).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct BridgeStatusWidget<'a> {
    pub(crate) route: &'a MockBridgeRoute,
}

impl Widget for BridgeStatusWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let header = format!(
            "{} · {}→{}",
            self.route.bridge_name, self.route.source_chain, self.route.dest_chain
        );
        let header_trim: String = header.chars().take(area.width as usize).collect();
        set_row(buf, area, 0, &header_trim, Style::default().fg(ROSE_DIM));

        if area.height < 2 {
            return;
        }

        let amt = format!("{} {}", self.route.amount, self.route.token_symbol);
        let amt_pad = area.width.saturating_sub(amt.len() as u16) as usize;
        let amt_row = format!("{:>width$}", amt, width = amt_pad + amt.len());
        let amt_trim: String = amt_row
            .chars()
            .rev()
            .take(area.width as usize)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        set_row(
            buf,
            area,
            1,
            &amt_trim,
            Style::default().fg(BONE).add_modifier(Modifier::BOLD),
        );

        let mut row = 2;
        if area.height > row {
            let fee_eta = format!(
                "Fee {}  ·  ETA {}",
                format_compact_usd(self.route.fee_usd),
                format_duration(self.route.eta_seconds as f64)
            );
            let line: String = fee_eta.chars().take(area.width as usize).collect();
            set_row(buf, area, row, &line, Style::default().fg(TEXT_DIM));
            row += 1;
        }

        let (badge, badge_color) = bridge_status_line(self.route.status);
        if area.height > row {
            let badge_trim: String = badge.chars().take(area.width as usize).collect();
            set_row(
                buf,
                area,
                row,
                &badge_trim,
                Style::default()
                    .fg(badge_color)
                    .add_modifier(Modifier::BOLD),
            );
            row += 1;
        }

        if area.height > row {
            if let Some(p) = bridge_progress_fraction(self.route) {
                let bar_y = area.y + row;
                let inner_w = area.width.saturating_sub(2);
                let fill = (inner_w as f64 * p).round() as u16;
                for x in 0..area.width {
                    let gx = area.x + x;
                    let edge = x == 0 || x + 1 == area.width;
                    let (ch, fg) = if edge {
                        (BLOCK_LIGHT, BORDER)
                    } else {
                        let ix = x - 1;
                        if ix < fill {
                            (BLOCK_FULL, progress_bar_color(p))
                        } else {
                            (BLOCK_LIGHT, BORDER)
                        }
                    };
                    let cell = buf.get_mut(gx, bar_y);
                    cell.set_char(ch);
                    cell.set_style(Style::default().fg(fg));
                }
                row += 1;
            } else if self.route.status == MockBridgeStatus::Quoted {
                let t = "Quoted — awaiting broadcast";
                let trim: String = t.chars().take(area.width as usize).collect();
                set_row(buf, area, row, &trim, Style::default().fg(TEXT_DIM));
                row += 1;
            }
        }

        if area.height > row {
            let arrow = format!(
                "{} ──────→ {}",
                self.route.source_chain, self.route.dest_chain
            );
            let trim: String = arrow.chars().take(area.width as usize).collect();
            // Arrow segment in rose dim: whole line uses mixed — keep simple
            set_row(buf, area, row, &trim, Style::default().fg(TEXT_PRIMARY));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::protocol_data::MockVaultState;
    use crate::widgets::VaultWidget;

    #[test]
    fn compact_usd_scales() {
        assert_eq!(format_compact_usd(1_200_000_000.0), "$1.2B");
        assert_eq!(format_compact_usd(2_500_000.0), "$2.5M");
        assert_eq!(format_compact_usd(3_400.0), "$3.4K");
        assert_eq!(format_compact_usd(42.12), "$42.12");
    }

    #[test]
    fn fee_tier_from_bps() {
        assert_eq!(format_fee_tier_pct(1), "0.01%");
        assert_eq!(format_fee_tier_pct(5), "0.05%");
        assert_eq!(format_fee_tier_pct(30), "0.30%");
        assert_eq!(format_fee_tier_pct(100), "1.00%");
    }

    #[test]
    fn bridge_progress_clamps() {
        let mut r = MockBridgeRoute::mock_default();
        r.elapsed_seconds = Some(300);
        r.estimated_seconds = Some(180);
        assert_eq!(bridge_progress_fraction(&r), Some(1.0));
    }

    #[test]
    fn widgets_render_without_panic() {
        let area = Rect::new(0, 0, 40, 12);
        let mut buf = Buffer::empty(area);
        let pool = MockPoolState::mock_default();
        UniswapPoolWidget { state: &pool }.render(area, &mut buf);

        let lend = MockLendingMarket::mock_default();
        LendingMarketWidget { state: &lend }.render(area, &mut buf);

        let vault = MockVaultState::mock_default();
        VaultWidget::new(&vault).render(area, &mut buf);

        let bridge = MockBridgeRoute::mock_default();
        BridgeStatusWidget { route: &bridge }.render(area, &mut buf);
    }
}
