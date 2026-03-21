//! Lending market widget for protocol views.

use crate::mock::protocol_data::MockLendingMarket;
use crate::palette::{
    BG_RAISED, BONE, BORDER_ACTIVE, DANGER, ROSE_DIM, SUCCESS, TEXT_DIM, TEXT_PRIMARY, WARNING,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Widget},
};

/// Ratatui widget for a single lending market (Aave V3, Morpho, Compound).
pub(crate) struct LendingMarketWidget<'a> {
    pub(crate) market: &'a MockLendingMarket,
}

impl<'a> LendingMarketWidget<'a> {
    pub(crate) fn new(market: &'a MockLendingMarket) -> Self {
        Self { market }
    }
}

/// Color for the utilization gauge based on threshold bands.
///
/// - `< 80%` — SUCCESS (sage green)
/// - `80%–95%` — WARNING (amber)
/// - `>= 95%` — DANGER (rose bright)
pub(crate) fn utilization_color(util: f64) -> Color {
    if util >= 0.95 {
        DANGER
    } else if util >= 0.80 {
        WARNING
    } else {
        SUCCESS
    }
}

fn format_usd_compact(value: f64) -> String {
    if value >= 1_000_000_000.0 {
        format!("${:.1}B", value / 1_000_000_000.0)
    } else if value >= 1_000_000.0 {
        format!("${:.1}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("${:.1}K", value / 1_000.0)
    } else {
        format!("${:.2}", value)
    }
}

impl<'a> Widget for LendingMarketWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let m = self.market;

        if area.height < 4 || area.width < 10 {
            return;
        }

        // Title: " Aave V3 · USDC " (abbreviate for narrow terminals)
        let title = if area.width >= 20 {
            format!(" {} · {} ", m.protocol_name, m.asset_symbol)
        } else {
            format!(" {} ", m.asset_symbol)
        };

        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(BORDER_ACTIVE))
            .title(Line::styled(title, Style::default().fg(BONE)));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        let util_color = utilization_color(m.utilization);
        let util_pct = (m.utilization * 100.0).clamp(0.0, 100.0) as u16;

        // Row 0: Utilization label + percentage
        let util_line = Line::from(vec![
            Span::styled("Utilization  ", Style::default().fg(TEXT_DIM)),
            Span::styled(format!("{}%", util_pct), Style::default().fg(util_color)),
        ]);
        buf.set_line(inner.x, inner.y, &util_line, inner.width);

        // Row 1: Utilization gauge bar
        if inner.height >= 2 {
            let gauge_area = Rect::new(inner.x, inner.y + 1, inner.width, 1);
            Gauge::default()
                .gauge_style(Style::default().fg(util_color).bg(BG_RAISED))
                .percent(util_pct)
                .render(gauge_area, buf);
        }

        // Row 2: Supply/Borrow APY
        if inner.height >= 3 {
            let supply_str = format!("{:.2}%", m.supply_apy * 100.0);
            let borrow_str = format!("{:.2}%", m.borrow_apy * 100.0);
            let borrow_color = if m.borrow_apy > 0.10 {
                WARNING
            } else {
                ROSE_DIM
            };

            let apy_line = Line::from(vec![
                Span::styled("Supply ", Style::default().fg(TEXT_DIM)),
                Span::styled(supply_str, Style::default().fg(SUCCESS)),
                Span::styled("  Borrow ", Style::default().fg(TEXT_DIM)),
                Span::styled(borrow_str, Style::default().fg(borrow_color)),
            ]);
            buf.set_line(inner.x, inner.y + 2, &apy_line, inner.width);
        }

        // Row 3: Liquidity totals (collapsed when height < 5)
        if area.height >= 5 && inner.height >= 4 {
            let supplied = format_usd_compact(m.total_liquidity_usd);
            let borrowed = format_usd_compact(m.total_borrow_usd);
            let liq_line = Line::from(vec![
                Span::styled("Supplied ", Style::default().fg(TEXT_DIM)),
                Span::styled(supplied, Style::default().fg(TEXT_PRIMARY)),
                Span::styled("  Borrowed ", Style::default().fg(TEXT_DIM)),
                Span::styled(borrowed, Style::default().fg(TEXT_PRIMARY)),
            ]);
            buf.set_line(inner.x, inner.y + 3, &liq_line, inner.width);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::protocol_data::MockLendingMarket;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn renders_with_mock_data() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let market = MockLendingMarket::mock_default();
        terminal
            .draw(|frame| {
                let area = frame.size();
                frame.render_widget(LendingMarketWidget::new(&market), area);
            })
            .unwrap();
        let output = terminal.backend().buffer().clone();
        let text: String = (0..output.area.height)
            .flat_map(|y| (0..output.area.width).map(move |x| (x, y)))
            .map(|(x, y)| output.get(x, y).symbol().to_string())
            .collect();
        assert!(text.contains("Aave"), "should contain protocol name");
        assert!(text.contains("USDC"), "should contain asset symbol");
    }

    #[test]
    fn utilization_color_thresholds() {
        assert_eq!(utilization_color(0.50), SUCCESS);
        assert_eq!(utilization_color(0.79), SUCCESS);
        assert_eq!(utilization_color(0.80), WARNING);
        assert_eq!(utilization_color(0.94), WARNING);
        assert_eq!(utilization_color(0.95), DANGER);
    }

    #[test]
    fn collapsed_render_no_panic() {
        let backend = TestBackend::new(40, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        let market = MockLendingMarket::mock_default();
        terminal
            .draw(|frame| {
                let area = frame.size();
                frame.render_widget(LendingMarketWidget::new(&market), area);
            })
            .unwrap();
    }

    #[test]
    fn too_small_no_panic() {
        let backend = TestBackend::new(5, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let market = MockLendingMarket::mock_default();
        terminal
            .draw(|frame| {
                let area = frame.size();
                frame.render_widget(LendingMarketWidget::new(&market), area);
            })
            .unwrap();
    }

    #[test]
    fn format_usd_compact_ranges() {
        assert_eq!(format_usd_compact(125_000_000.0), "$125.0M");
        assert_eq!(format_usd_compact(97_500_000.0), "$97.5M");
        assert_eq!(format_usd_compact(1_500_000_000.0), "$1.5B");
        assert_eq!(format_usd_compact(3_100.0), "$3.1K");
        assert_eq!(format_usd_compact(0.99), "$0.99");
    }
}
