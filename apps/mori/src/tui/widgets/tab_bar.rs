use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::atmosphere::Atmosphere;
use crate::tui::tabs::Tab;
use crate::tui::theme::Theme;

pub fn render(f: &mut Frame, area: Rect, active: Tab, atmosphere: &Atmosphere) {
    let mut spans = Vec::new();

    for tab in Tab::ALL {
        let label = format!(" {}:{} ", tab.fkey_label(), tab.label());
        let style = if tab == active {
            // Modulate active tab background with heartbeat
            let hb = atmosphere.heartbeat();
            let r = (170.0 * hb).min(255.0) as u8;
            let g = (112.0 * hb).min(255.0) as u8;
            let b = (136.0 * hb).min(255.0) as u8;
            Style::default().fg(Theme::BG).bg(Color::Rgb(r, g, b))
        } else {
            Theme::tab_inactive_style()
        };
        spans.push(Span::styled(label, style));
        spans.push(Span::raw(" "));
    }

    let line = Line::from(spans);
    let p = Paragraph::new(line).style(Theme::default_style());
    f.render_widget(p, area);
}
