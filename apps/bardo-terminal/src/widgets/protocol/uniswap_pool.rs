//! Uniswap V3/V4 pool visualization widget.

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::mock::protocol_data::MockPoolState;
use crate::palette::{
    BONE, BORDER_ACTIVE, ROSE, ROSE_DEEP, SUCCESS, TEXT_DIM, TEXT_PRIMARY, WARNING,
};
use crate::widgets::sparkline::BrailleSparkline;

#[derive(Debug, Clone)]
pub(crate) struct UniswapPoolWidgetConfig {
    pub(crate) show_depth: bool,
    pub(crate) price_range_pct: f32,
}

impl Default for UniswapPoolWidgetConfig {
    fn default() -> Self {
        Self {
            show_depth: true,
            price_range_pct: 0.25,
        }
    }
}

pub(crate) struct UniswapPoolWidget<'a> {
    pub(crate) pool: &'a MockPoolState,
    pub(crate) config: UniswapPoolWidgetConfig,
}

impl<'a> UniswapPoolWidget<'a> {
    pub(crate) fn new(pool: &'a MockPoolState) -> Self {
        Self {
            pool,
            config: UniswapPoolWidgetConfig::default(),
        }
    }

    pub(crate) fn with_config(mut self, config: UniswapPoolWidgetConfig) -> Self {
        self.config = config;
        self
    }
}

pub(crate) fn format_fee_tier(bps: u16) -> String {
    format!("{:.2}%", bps as f64 / 100.0)
}

pub(crate) fn format_usd_compact(value: f64) -> String {
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

impl<'a> Widget for UniswapPoolWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 8 || area.height < 3 {
            return;
        }

        let pool = self.pool;
        let in_range = pool.tick_range.is_in_range();

        let border_color = if in_range { BORDER_ACTIVE } else { WARNING };
        let title = format!(
            " {}/{} {} {} ",
            pool.token0_symbol,
            pool.token1_symbol,
            format_fee_tier(pool.fee_tier_bps),
            pool.chain,
        );

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(title, Style::default().fg(BONE)));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height == 0 || inner.width < 4 {
            return;
        }

        // If very short, only show price.
        if inner.height < 3 {
            let price_str = format!("{:.2} {}", pool.current_price, pool.token1_symbol);
            buf.set_stringn(
                inner.x,
                inner.y,
                &price_str,
                inner.width as usize,
                Style::default().fg(BONE),
            );
            return;
        }

        // Layout: price (1), tick bar (1), sparkline (2 if room), footer (1)
        let show_sparkline =
            self.config.show_depth && inner.height >= 5 && !pool.depth_samples.is_empty();
        let constraints = if show_sparkline {
            vec![
                Constraint::Length(1), // price
                Constraint::Length(1), // tick bar
                Constraint::Length(2), // sparkline
                Constraint::Length(1), // footer
            ]
        } else {
            vec![
                Constraint::Length(1), // price
                Constraint::Length(1), // tick bar
                Constraint::Length(1), // footer
            ]
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        // -- Price row --
        let price_str = format!("{:.2} {}", pool.current_price, pool.token1_symbol);
        let price_line = Line::from(vec![Span::styled(
            price_str,
            Style::default().fg(BONE).add_modifier(Modifier::BOLD),
        )]);
        Paragraph::new(price_line).render(chunks[0], buf);

        // -- Tick range bar --
        render_tick_bar(chunks[1], buf, pool, in_range);

        // -- Sparkline (optional) --
        let footer_idx = if show_sparkline {
            let spark = BrailleSparkline {
                data: pool.depth_samples.clone(),
                max_value: 0.0, // auto-scale
                color: ROSE,
                label: None,
            };
            spark.render(chunks[2], buf);
            3
        } else {
            2
        };

        // -- Footer: TVL + Vol 24h --
        let tvl = format_usd_compact(pool.tvl_usd);
        let vol = format_usd_compact(pool.volume_24h_usd);
        let footer = Line::from(vec![
            Span::styled("TVL ", Style::default().fg(TEXT_DIM)),
            Span::styled(tvl, Style::default().fg(BONE)),
            Span::styled("  Vol ", Style::default().fg(TEXT_DIM)),
            Span::styled(vol, Style::default().fg(TEXT_PRIMARY)),
        ]);
        Paragraph::new(footer).render(chunks[footer_idx], buf);
    }
}

fn render_tick_bar(area: Rect, buf: &mut Buffer, pool: &MockPoolState, in_range: bool) {
    let width = area.width as usize;
    if width < 4 {
        return;
    }
    let bar_width = width.saturating_sub(4);
    if bar_width == 0 {
        return;
    }
    let frac = pool.tick_range.position_fraction().unwrap_or(0.5);
    let cursor_pos = ((frac * bar_width as f64) as usize).min(bar_width.saturating_sub(1));

    for i in 0..bar_width {
        let (ch, style) = if i == cursor_pos {
            (
                '\u{25C6}', // ◆
                Style::default().fg(BONE),
            )
        } else if in_range {
            (
                '\u{2591}', // ░
                Style::default().fg(SUCCESS).add_modifier(Modifier::DIM),
            )
        } else {
            (
                '\u{2591}', // ░
                Style::default().fg(ROSE_DEEP),
            )
        };

        let x = area.x + 2 + i as u16;
        if x < area.right() {
            let cell = buf.get_mut(x, area.y);
            cell.set_char(ch);
            cell.set_style(style);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::protocol_data::{MockPoolState, MockTickRange};
    use ratatui::{Terminal, backend::TestBackend};

    fn buffer_text(buf: &Buffer) -> String {
        let a = buf.area();
        (a.top()..a.bottom())
            .flat_map(|y| (a.left()..a.right()).map(move |x| (x, y)))
            .map(|(x, y)| buf.get(x, y).symbol().to_string())
            .collect()
    }

    #[test]
    fn renders_with_mock_data() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let pool = MockPoolState::mock_default();
        terminal
            .draw(|frame| {
                frame.render_widget(UniswapPoolWidget::new(&pool), frame.size());
            })
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("ETH"), "should contain token0 symbol");
        assert!(text.contains("USDC"), "should contain token1 symbol");
    }

    #[test]
    fn out_of_range_no_panic() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut pool = MockPoolState::mock_default();
        pool.tick_range.current_tick = pool.tick_range.upper_tick + 500;
        terminal
            .draw(|frame| {
                frame.render_widget(UniswapPoolWidget::new(&pool), frame.size());
            })
            .unwrap();
    }

    #[test]
    fn fee_tier_formatting() {
        assert_eq!(format_fee_tier(5), "0.05%");
        assert_eq!(format_fee_tier(30), "0.30%");
        assert_eq!(format_fee_tier(100), "1.00%");
    }

    #[test]
    fn usd_compact_formatting() {
        assert_eq!(format_usd_compact(48_200_000.0), "$48.2M");
        assert_eq!(format_usd_compact(3_100.0), "$3.1K");
        assert_eq!(format_usd_compact(0.99), "$0.99");
    }

    #[test]
    fn out_of_range_border_uses_warning() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut pool = MockPoolState::mock_default();
        pool.tick_range.current_tick = pool.tick_range.upper_tick + 500;
        terminal
            .draw(|frame| {
                frame.render_widget(UniswapPoolWidget::new(&pool), frame.size());
            })
            .unwrap();
        // Top-left corner cell should have WARNING color
        let cell = terminal.backend().buffer().get(0, 0);
        assert_eq!(cell.fg, WARNING);
    }

    #[test]
    fn tiny_area_no_panic() {
        let backend = TestBackend::new(5, 2);
        let mut terminal = Terminal::new(backend).unwrap();
        let pool = MockPoolState::mock_default();
        terminal
            .draw(|frame| {
                frame.render_widget(UniswapPoolWidget::new(&pool), frame.size());
            })
            .unwrap();
    }

    #[test]
    fn sparkline_disabled_via_config() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let pool = MockPoolState::mock_default();
        let config = UniswapPoolWidgetConfig {
            show_depth: false,
            price_range_pct: 0.25,
        };
        terminal
            .draw(|frame| {
                frame.render_widget(
                    UniswapPoolWidget::new(&pool).with_config(config),
                    frame.size(),
                );
            })
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("ETH"));
        assert!(text.contains("TVL"));
    }
}
