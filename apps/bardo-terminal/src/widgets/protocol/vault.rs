//! ERC-4626 vault widget (Beefy, Yearn, or generic).

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};

use crate::mock::protocol_data::MockVaultState;
use crate::palette::{BONE, BORDER, BORDER_ACTIVE, DANGER, ROSE, SUCCESS, TEXT_DIM, TEXT_PRIMARY};

/// Format a percentage change value as a signed string with color.
///
/// Positive values get a `+` prefix and SUCCESS color. Negative values keep
/// their minus sign and use DANGER. Zero gets TEXT_DIM.
pub(crate) fn format_pct_change(change: f64) -> (String, Color) {
    let color = if change > 0.0 {
        SUCCESS
    } else if change < 0.0 {
        DANGER
    } else {
        TEXT_DIM
    };
    let sign = if change > 0.0 { "+" } else { "" };
    let text = format!("{}{:.2}%", sign, change * 100.0);
    (text, color)
}

/// Format NAV per share: 4 decimals normally, 2 if >1000, 6 if <0.01.
fn format_nav(nav: f64, symbol: &str) -> String {
    if nav > 1000.0 {
        format!("{:.2} {}", nav, symbol)
    } else if nav < 0.01 {
        format!("{:.6} {}", nav, symbol)
    } else {
        format!("{:.4} {}", nav, symbol)
    }
}

/// Format USD value in compact form ($1.2M, $840K, $12.5B, etc.).
fn format_usd_compact(value: f64) -> String {
    if value >= 1_000_000_000.0 {
        format!("${:.1}B", value / 1_000_000_000.0)
    } else if value >= 1_000_000.0 {
        format!("${:.1}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("${:.0}K", value / 1_000.0)
    } else {
        format!("${:.2}", value)
    }
}

/// Ratatui widget for an ERC-4626 vault (Beefy, Yearn, or generic).
#[derive(Debug, Clone)]
pub(crate) struct VaultWidget<'a> {
    pub(crate) vault: &'a MockVaultState,
}

impl<'a> VaultWidget<'a> {
    pub(crate) fn new(vault: &'a MockVaultState) -> Self {
        Self { vault }
    }
}

impl<'a> Widget for VaultWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 4 || area.height < 4 {
            return;
        }

        let title = if area.width >= 24 {
            format!(" {} · {} ", self.vault.vault_name, self.vault.chain)
        } else {
            format!(
                " {} · {} ",
                self.vault.asset_symbol, self.vault.protocol_name
            )
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER))
            .title(title)
            .title_style(Style::default().fg(BORDER_ACTIVE));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let mut row = inner.y;
        let label_w = 11_u16; // "NAV/share  " width
        let val_x = inner.x + label_w.min(inner.width);

        // Row 1: NAV/share
        if row < inner.bottom() {
            buf.set_stringn(
                inner.x,
                row,
                "NAV/share",
                inner.width as usize,
                Style::default().fg(TEXT_DIM),
            );
            let nav_str = format_nav(self.vault.nav_per_share, &self.vault.asset_symbol);
            if val_x < inner.right() {
                buf.set_stringn(
                    val_x,
                    row,
                    &nav_str,
                    (inner.right() - val_x) as usize,
                    Style::default().fg(BONE),
                );
            }
            row += 1;
        }

        // Row 2: TVL
        if row < inner.bottom() {
            buf.set_stringn(
                inner.x,
                row,
                "TVL",
                inner.width as usize,
                Style::default().fg(TEXT_DIM),
            );
            let tvl_str = format_usd_compact(self.vault.tvl_usd);
            if val_x < inner.right() {
                buf.set_stringn(
                    val_x,
                    row,
                    &tvl_str,
                    (inner.right() - val_x) as usize,
                    Style::default().fg(ROSE),
                );
            }
            row += 1;
        }

        // Row 3: APY
        if row < inner.bottom() {
            buf.set_stringn(
                inner.x,
                row,
                "APY",
                inner.width as usize,
                Style::default().fg(TEXT_DIM),
            );
            let apy_str = format!("{:.2}%", self.vault.apy * 100.0);
            if val_x < inner.right() {
                buf.set_stringn(
                    val_x,
                    row,
                    &apy_str,
                    (inner.right() - val_x) as usize,
                    Style::default().fg(SUCCESS),
                );
            }
            row += 1;
        }

        // Row 4: 24h change
        if row < inner.bottom() {
            buf.set_stringn(
                inner.x,
                row,
                "24h",
                inner.width as usize,
                Style::default().fg(TEXT_DIM),
            );
            let (change_str, change_color) = format_pct_change(self.vault.share_price_24h_change);
            if val_x < inner.right() {
                buf.set_stringn(
                    val_x,
                    row,
                    &change_str,
                    (inner.right() - val_x) as usize,
                    Style::default().fg(change_color),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::protocol_data::MockVaultState;
    use ratatui::{Terminal, backend::TestBackend};

    fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
        let buf = terminal.backend().buffer().clone();
        (0..buf.area.height)
            .flat_map(|y| (0..buf.area.width).map(move |x| (x, y)))
            .map(|(x, y)| buf.get(x, y).symbol().to_string())
            .collect()
    }

    #[test]
    fn test_vault_widget_renders_with_mock_data() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let vault = MockVaultState::mock_default();
        terminal
            .draw(|frame| {
                frame.render_widget(VaultWidget::new(&vault), frame.size());
            })
            .unwrap();
        let text = buffer_text(&terminal);
        assert!(text.contains("Beefy"), "should contain protocol name");
        assert!(text.contains("USDC"), "should contain asset symbol");
    }

    #[test]
    fn test_format_pct_change_sign() {
        let (text_pos, color_pos) = format_pct_change(0.0031);
        assert!(
            text_pos.starts_with('+'),
            "positive change should have + prefix"
        );
        assert_eq!(color_pos, SUCCESS);

        let (text_neg, color_neg) = format_pct_change(-0.0050);
        assert!(
            text_neg.starts_with('-'),
            "negative change should have - prefix"
        );
        assert_eq!(color_neg, DANGER);

        let (text_zero, color_zero) = format_pct_change(0.0);
        assert_eq!(color_zero, TEXT_DIM);
        assert!(text_zero.contains("0.00%"));
    }

    #[test]
    fn test_vault_widget_negative_change_no_panic() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut vault = MockVaultState::mock_default();
        vault.share_price_24h_change = -0.025; // -2.5%
        terminal
            .draw(|frame| {
                frame.render_widget(VaultWidget::new(&vault), frame.size());
            })
            .unwrap();
        let text = buffer_text(&terminal);
        assert!(text.contains("-2.50%"), "should show negative percentage");
    }

    #[test]
    fn test_format_nav_decimal_places() {
        assert_eq!(format_nav(1.0842, "USDC"), "1.0842 USDC");
        assert_eq!(format_nav(1500.25, "ETH"), "1500.25 ETH");
        assert_eq!(format_nav(0.005, "BTC"), "0.005000 BTC");
    }

    #[test]
    fn test_format_usd_compact_ranges() {
        assert_eq!(format_usd_compact(8_400_000.0), "$8.4M");
        assert_eq!(format_usd_compact(1_200_000_000.0), "$1.2B");
        assert_eq!(format_usd_compact(50_000.0), "$50K");
        assert_eq!(format_usd_compact(500.0), "$500.00");
    }

    #[test]
    fn test_vault_widget_narrow_title() {
        let backend = TestBackend::new(20, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let vault = MockVaultState::mock_default();
        terminal
            .draw(|frame| {
                frame.render_widget(VaultWidget::new(&vault), frame.size());
            })
            .unwrap();
        let text = buffer_text(&terminal);
        // Narrow width should show "USDC · Beefy" not the full vault name
        assert!(
            text.contains("Beefy"),
            "narrow title should contain protocol"
        );
    }

    #[test]
    fn test_vault_widget_min_height_guard() {
        let backend = TestBackend::new(40, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let vault = MockVaultState::mock_default();
        // Should not panic on area too small
        terminal
            .draw(|frame| {
                frame.render_widget(VaultWidget::new(&vault), frame.size());
            })
            .unwrap();
    }
}
