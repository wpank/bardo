//! ERC-4626 vault panel for the Protocol Views screen (plan 07, unit 4).

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::{
    mock::protocol_data::MockVaultState,
    palette::{BONE, DANGER, ROSE, ROSE_DIM, SUCCESS, TEXT_DIM},
};

/// Ratatui widget for an ERC-4626 vault (Beefy, Yearn, or generic).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct VaultWidget<'a> {
    pub(crate) vault: &'a MockVaultState,
}

impl<'a> VaultWidget<'a> {
    pub(crate) fn new(vault: &'a MockVaultState) -> Self {
        Self { vault }
    }
}

fn paint_prefix_value(
    buf: &mut Buffer,
    area: Rect,
    row: u16,
    prefix: &str,
    value: &str,
    prefix_style: Style,
    value_style: Style,
) {
    if row >= area.height {
        return;
    }
    let w = area.width as usize;
    let px = area.x;
    let py = area.y + row;
    let p_len = prefix.chars().count().min(w);
    buf.set_stringn(px, py, prefix, p_len, prefix_style);
    let vx = px + p_len as u16;
    let vw = w.saturating_sub(p_len);
    buf.set_stringn(vx, py, value, vw, value_style);
}

/// NAV per share formatting: 4 dp by default; 2 if ≥ 1000; 6 if below 0.01.
pub(crate) fn format_nav_per_share(nav: f64, symbol: &str) -> String {
    let places = if nav >= 1000.0 {
        2usize
    } else if nav >= 0.01 {
        4usize
    } else {
        6usize
    };
    format!("{nav:.*} {symbol}", places)
}

/// 24h share-price change as display percent and semantic color.
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

fn vault_title(vault: &MockVaultState, width: u16) -> String {
    let w = width as usize;
    let s = if width >= 24 {
        format!("{} · {}", vault.vault_name, vault.chain)
    } else {
        format!("{} · {}", vault.asset_symbol, vault.protocol_name)
    };
    s.chars().take(w.max(1)).collect()
}

impl Widget for VaultWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 8 || area.height < 2 {
            return;
        }

        let w = area.width as usize;
        let title = vault_title(self.vault, area.width);
        buf.set_stringn(
            area.x,
            area.y,
            &title,
            w,
            Style::default().fg(ROSE_DIM).add_modifier(Modifier::BOLD),
        );

        if area.height < 3 {
            return;
        }

        let nav_str = format_nav_per_share(self.vault.nav_per_share, &self.vault.asset_symbol);
        let nav_line = format!("NAV/share  {nav_str}");
        let nav_trim: String = nav_line.chars().take(w).collect();
        buf.set_stringn(
            area.x,
            area.y + 1,
            &nav_trim,
            w,
            Style::default().fg(BONE).add_modifier(Modifier::BOLD),
        );

        let tvl_compact = crate::widgets::protocol_panels::format_compact_usd(self.vault.tvl_usd);
        let apy_pct = format!("{:.2}%", self.vault.apy * 100.0);

        if area.height >= 6 {
            paint_prefix_value(
                buf,
                area,
                2,
                "TVL        ",
                &tvl_compact,
                Style::default().fg(TEXT_DIM),
                Style::default().fg(ROSE),
            );
            paint_prefix_value(
                buf,
                area,
                3,
                "APY        ",
                &apy_pct,
                Style::default().fg(TEXT_DIM),
                Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD),
            );
            let (chg, chg_color) = format_pct_change(self.vault.share_price_24h_change);
            paint_prefix_value(
                buf,
                area,
                4,
                "24h        ",
                &chg,
                Style::default().fg(TEXT_DIM),
                Style::default().fg(chg_color).add_modifier(Modifier::BOLD),
            );
            return;
        }

        if area.height >= 4 {
            let combo = format!("TVL {tvl_compact}   APY {apy_pct}");
            let combo_trim: String = combo.chars().take(w).collect();
            buf.set_stringn(
                area.x,
                area.y + 2,
                &combo_trim,
                w,
                Style::default().fg(TEXT_DIM),
            );

            let (chg, chg_color) = format_pct_change(self.vault.share_price_24h_change);
            paint_prefix_value(
                buf,
                area,
                3,
                "24h ",
                &chg,
                Style::default().fg(TEXT_DIM),
                Style::default().fg(chg_color).add_modifier(Modifier::BOLD),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn nav_precision_tiers() {
        assert_eq!(format_nav_per_share(1500.0, "TKN"), "1500.00 TKN");
        assert_eq!(format_nav_per_share(1.0842, "USDC"), "1.0842 USDC");
        assert_eq!(format_nav_per_share(0.005, "X"), "0.005000 X");
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
    }

    #[test]
    fn test_vault_widget_renders_with_mock_data() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let vault = MockVaultState::mock_default();
        terminal
            .draw(|frame| {
                frame.render_widget(VaultWidget::new(&vault), frame.size());
            })
            .expect("draw");
        let output = terminal.backend().buffer().clone();
        let text: String = (0..output.area.height)
            .flat_map(|y| (0..output.area.width).map(move |x| (x, y)))
            .map(|(x, y)| output.get(x, y).symbol().to_string())
            .collect();
        assert!(text.contains("Beefy"), "should contain protocol name");
        assert!(text.contains("USDC"), "should contain asset symbol");
    }

    #[test]
    fn test_vault_widget_negative_change_no_panic() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let mut vault = MockVaultState::mock_default();
        vault.share_price_24h_change = -0.025;
        terminal
            .draw(|frame| {
                frame.render_widget(VaultWidget::new(&vault), frame.size());
            })
            .expect("draw");
    }

    #[test]
    fn narrow_title_uses_asset_and_protocol() {
        let mut vault = MockVaultState::mock_default();
        vault.vault_name = "Very Long Vault Name That Will Not Fit".to_string();
        let title = vault_title(&vault, 20);
        assert!(title.contains("USDC"));
        assert!(title.contains("Beefy"));
    }
}
