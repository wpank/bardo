//! Screen abstraction and screen registry for the terminal scaffold.

use std::collections::HashMap;

use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    palette::{BORDER, ROSE_DIM},
    state::{AppAction, AppState},
};

/// Pluggable screen contract implemented by all screens in the terminal.
pub(crate) trait Screen: Send + Sync {
    /// Returns the screen identifier.
    fn id(&self) -> ScreenId;

    /// Returns the screen title shown in chrome and placeholder content.
    fn title(&self) -> &str;

    /// Renders the screen contents into the provided area.
    fn render(&self, frame: &mut Frame<'_>, area: Rect, state: &AppState);

    /// Handles a key event and optionally emits an application action.
    fn handle_key(&mut self, key: KeyEvent) -> Option<AppAction>;

    /// Called when the screen becomes active.
    fn on_focus(&mut self) {}

    /// Called when the screen loses focus.
    fn on_blur(&mut self) {}
}

/// Identifiers for every screen in the current catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ScreenId {
    HearthOverview,
    HearthSignals,
    HearthOperations,
    HearthStatus,
    MindPipeline,
    MindGrimoire,
    MindPlaybook,
    MindDreams,
    MindInference,
    MindChainIntelligence,
    MindTechnicalAnalysis,
    SomaPortfolio,
    SomaTrades,
    SomaCustody,
    SomaBudget,
    SomaSanctum,
    WorldSolaris,
    WorldClade,
    WorldLethe,
    WorldBloodstains,
    WorldBazaar,
    FateMortality,
    FateLineage,
    FateAchievements,
    FateGraveyard,
    CommandSteer,
    CommandConfig,
    CommandEffects,
    CommandHermes,
    /// DeFi protocol overview (Uniswap, lending, vault, bridge widgets).
    ProtocolViews,
}

impl ScreenId {
    /// Ordered screen catalog used for tab cycling.
    pub(crate) fn all() -> &'static [Self] {
        &SCREEN_CATALOG
    }

    /// Returns the window name for this screen.
    pub(crate) const fn window_name(self) -> &'static str {
        match self {
            Self::HearthOverview
            | Self::HearthSignals
            | Self::HearthOperations
            | Self::HearthStatus => "HEARTH",
            Self::MindPipeline
            | Self::MindGrimoire
            | Self::MindPlaybook
            | Self::MindDreams
            | Self::MindInference
            | Self::MindChainIntelligence
            | Self::MindTechnicalAnalysis => "MIND",
            Self::SomaPortfolio
            | Self::SomaTrades
            | Self::SomaCustody
            | Self::SomaBudget
            | Self::SomaSanctum => "SOMA",
            Self::WorldSolaris
            | Self::WorldClade
            | Self::WorldLethe
            | Self::WorldBloodstains
            | Self::WorldBazaar => "WORLD",
            Self::FateMortality
            | Self::FateLineage
            | Self::FateAchievements
            | Self::FateGraveyard => "FATE",
            Self::CommandSteer
            | Self::CommandConfig
            | Self::CommandEffects
            | Self::CommandHermes => "COMMAND",
            Self::ProtocolViews => "PROTOCOLS",
        }
    }

    /// Returns the tab name for this screen.
    pub(crate) const fn tab_name(self) -> &'static str {
        match self {
            Self::HearthOverview => "Overview",
            Self::HearthSignals => "Signals",
            Self::HearthOperations => "Operations",
            Self::HearthStatus => "Status",
            Self::MindPipeline => "Pipeline",
            Self::MindGrimoire => "Grimoire",
            Self::MindPlaybook => "Playbook",
            Self::MindDreams => "Dreams",
            Self::MindInference => "Inference",
            Self::MindChainIntelligence => "Chain Intelligence",
            Self::MindTechnicalAnalysis => "Technical Analysis",
            Self::SomaPortfolio => "Portfolio",
            Self::SomaTrades => "Trades",
            Self::SomaCustody => "Custody",
            Self::SomaBudget => "Budget",
            Self::SomaSanctum => "Sanctum",
            Self::WorldSolaris => "Solaris",
            Self::WorldClade => "Clade",
            Self::WorldLethe => "Lethe",
            Self::WorldBloodstains => "Bloodstains",
            Self::WorldBazaar => "Bazaar",
            Self::FateMortality => "Mortality",
            Self::FateLineage => "Lineage",
            Self::FateAchievements => "Achievements",
            Self::FateGraveyard => "Graveyard",
            Self::CommandSteer => "Steer",
            Self::CommandConfig => "Config",
            Self::CommandEffects => "Effects",
            Self::CommandHermes => "Hermes",
            Self::ProtocolViews => "Protocols",
        }
    }
}

const SCREEN_CATALOG: [ScreenId; 30] = [
    ScreenId::HearthOverview,
    ScreenId::HearthSignals,
    ScreenId::HearthOperations,
    ScreenId::HearthStatus,
    ScreenId::MindPipeline,
    ScreenId::MindGrimoire,
    ScreenId::MindPlaybook,
    ScreenId::MindDreams,
    ScreenId::MindInference,
    ScreenId::MindChainIntelligence,
    ScreenId::MindTechnicalAnalysis,
    ScreenId::SomaPortfolio,
    ScreenId::SomaTrades,
    ScreenId::SomaCustody,
    ScreenId::SomaBudget,
    ScreenId::SomaSanctum,
    ScreenId::WorldSolaris,
    ScreenId::WorldClade,
    ScreenId::WorldLethe,
    ScreenId::WorldBloodstains,
    ScreenId::WorldBazaar,
    ScreenId::FateMortality,
    ScreenId::FateLineage,
    ScreenId::FateAchievements,
    ScreenId::FateGraveyard,
    ScreenId::CommandSteer,
    ScreenId::CommandConfig,
    ScreenId::CommandEffects,
    ScreenId::CommandHermes,
    ScreenId::ProtocolViews,
];

/// Generic placeholder for screens not yet implemented.
pub(crate) struct StubScreen {
    id: ScreenId,
    title: String,
}

impl StubScreen {
    /// Creates a new stub screen.
    pub(crate) fn new(id: ScreenId, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
        }
    }
}

impl Screen for StubScreen {
    fn id(&self) -> ScreenId {
        self.id
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect, _state: &AppState) {
        let message = format!("[ {} - not yet implemented ]", self.title);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(
            Paragraph::new(message)
                .style(Style::default().fg(ROSE_DIM).add_modifier(Modifier::BOLD))
                .alignment(ratatui::layout::Alignment::Center),
            inner,
        );
    }

    fn handle_key(&mut self, _key: KeyEvent) -> Option<AppAction> {
        None
    }
}

/// Registry of loaded screens.
pub(crate) struct ScreenRegistry {
    screens: HashMap<ScreenId, Box<dyn Screen>>,
}

impl ScreenRegistry {
    /// Creates an empty registry.
    pub(crate) fn new() -> Self {
        Self {
            screens: HashMap::new(),
        }
    }

    /// Registers or replaces a screen by its identifier.
    pub(crate) fn register(&mut self, screen: Box<dyn Screen>) {
        self.screens.insert(screen.id(), screen);
    }

    /// Returns an immutable screen reference.
    pub(crate) fn get(&self, id: &ScreenId) -> Option<&dyn Screen> {
        self.screens.get(id).map(Box::as_ref)
    }

    /// Returns a mutable screen reference.
    pub(crate) fn get_mut(&mut self, id: &ScreenId) -> Option<&mut dyn Screen> {
        if let Some(screen) = self.screens.get_mut(id) {
            Some(screen.as_mut())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ratatui::{Terminal, backend::TestBackend};

    struct TestScreen;

    struct ReplacementScreen;

    impl Screen for TestScreen {
        fn id(&self) -> ScreenId {
            ScreenId::HearthOverview
        }

        fn title(&self) -> &str {
            "HEARTH"
        }

        fn render(&self, _frame: &mut Frame<'_>, _area: Rect, _state: &AppState) {}

        fn handle_key(&mut self, _key: KeyEvent) -> Option<AppAction> {
            None
        }
    }

    impl Screen for ReplacementScreen {
        fn id(&self) -> ScreenId {
            ScreenId::HearthOverview
        }

        fn title(&self) -> &str {
            "REPLACED"
        }

        fn render(&self, _frame: &mut Frame<'_>, _area: Rect, _state: &AppState) {}

        fn handle_key(&mut self, _key: KeyEvent) -> Option<AppAction> {
            None
        }
    }

    fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
        let buffer = terminal.backend().buffer();
        let area = buffer.area();
        let mut text = String::new();

        for y in 0..area.height {
            for x in 0..area.width {
                text.push_str(buffer.get(x, y).symbol());
            }
            text.push('\n');
        }

        text
    }

    #[test]
    fn screen_trait_is_send_sync_and_default_lifecycle_is_no_op() {
        let mut screen = TestScreen;
        screen.on_focus();
        screen.on_blur();

        let _boxed: Box<dyn Screen> = Box::new(TestScreen);
    }

    #[test]
    fn screen_id_supports_hashing_as_map_key() {
        use std::collections::HashMap;

        let mut m = HashMap::new();
        m.insert(ScreenId::HearthOverview, 1u8);
        m.insert(ScreenId::CommandHermes, 2u8);
        assert_eq!(m.get(&ScreenId::HearthOverview), Some(&1));
        assert_eq!(m.get(&ScreenId::CommandHermes), Some(&2));
    }

    #[test]
    fn screen_catalog_has_the_expected_size_and_order() {
        assert_eq!(ScreenId::all().len(), 30);
        let expected = [
            ScreenId::HearthOverview,
            ScreenId::HearthSignals,
            ScreenId::HearthOperations,
            ScreenId::HearthStatus,
            ScreenId::MindPipeline,
            ScreenId::MindGrimoire,
            ScreenId::MindPlaybook,
            ScreenId::MindDreams,
            ScreenId::MindInference,
            ScreenId::MindChainIntelligence,
            ScreenId::MindTechnicalAnalysis,
            ScreenId::SomaPortfolio,
            ScreenId::SomaTrades,
            ScreenId::SomaCustody,
            ScreenId::SomaBudget,
            ScreenId::SomaSanctum,
            ScreenId::WorldSolaris,
            ScreenId::WorldClade,
            ScreenId::WorldLethe,
            ScreenId::WorldBloodstains,
            ScreenId::WorldBazaar,
            ScreenId::FateMortality,
            ScreenId::FateLineage,
            ScreenId::FateAchievements,
            ScreenId::FateGraveyard,
            ScreenId::CommandSteer,
            ScreenId::CommandConfig,
            ScreenId::CommandEffects,
            ScreenId::CommandHermes,
            ScreenId::ProtocolViews,
        ];

        assert_eq!(ScreenId::all(), &expected);
        assert_eq!(ScreenId::MindTechnicalAnalysis.window_name(), "MIND");
        assert_eq!(
            ScreenId::MindTechnicalAnalysis.tab_name(),
            "Technical Analysis"
        );
    }

    #[test]
    fn registry_stores_replaces_and_returns_screens() {
        let mut registry = ScreenRegistry::new();
        registry.register(Box::new(TestScreen));
        registry.register(Box::new(ReplacementScreen));

        assert_eq!(
            registry.get(&ScreenId::HearthOverview).map(Screen::title),
            Some("REPLACED")
        );
        assert!(registry.get_mut(&ScreenId::HearthOverview).is_some());
        assert!(registry.get(&ScreenId::CommandHermes).is_none());
    }

    #[test]
    fn stub_screen_renders_fallback_message_and_defers_global_navigation_keys() {
        let screen = StubScreen::new(ScreenId::MindPipeline, "MIND / Pipeline");
        let backend = TestBackend::new(48, 5);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        terminal
            .draw(|frame| screen.render(frame, frame.size(), &AppState::default()))
            .expect("stub screen should render into a test backend");

        let rendered = buffer_text(&terminal);
        assert!(rendered.contains("MIND / Pipeline - not yet implemented"));

        let mut screen = screen;
        assert_eq!(
            screen.handle_key(KeyEvent::from(crossterm::event::KeyCode::Tab)),
            None
        );
        assert_eq!(
            screen.handle_key(KeyEvent::from(crossterm::event::KeyCode::BackTab)),
            None
        );
        assert_eq!(
            screen.handle_key(KeyEvent::from(crossterm::event::KeyCode::Char('q'))),
            None
        );
    }
}
