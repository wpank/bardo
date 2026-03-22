use ratatui::style::{Color, Modifier, Style};

use crate::agent::AgentRole;

/// ROSEDUST color palette — aligned to prd2 00-design-system.md
pub struct Theme;

impl Theme {
    // Core palette
    pub const VOID: Color = Color::Rgb(6, 6, 8); // #060608 — true background
    pub const ROSE: Color = Color::Rgb(170, 112, 136); // #AA7088 — primary accent
    pub const ROSE_BRIGHT: Color = Color::Rgb(204, 144, 168); // #CC90A8
    pub const ROSE_DIM: Color = Color::Rgb(122, 80, 96); // #7A5060
    pub const BONE: Color = Color::Rgb(200, 184, 144); // #C8B890 — scarcest emphasis
    pub const BONE_DIM: Color = Color::Rgb(138, 122, 90); // #8A7A5A
    pub const TEXT: Color = Color::Rgb(152, 128, 144); // #988090
    pub const TEXT_DIM: Color = Color::Rgb(160, 136, 152); // #A08898 — readable but still subdued
    pub const TEXT_GHOST: Color = Color::Rgb(96, 80, 96); // #605060 — visible for future phases
    pub const DREAM: Color = Color::Rgb(88, 88, 120); // #585878
    pub const SAGE: Color = Color::Rgb(112, 136, 122); // #70887A — success
    pub const EMBER: Color = Color::Rgb(170, 100, 80); // #AA6450 — error
    pub const WARNING: Color = Color::Rgb(170, 136, 85); // #AA8855

    // Extended ROSEDUST palette (PRD2)
    pub const BG_RAISED: Color = Color::Rgb(12, 10, 14); // #0C0A0E
    pub const ROSE_DEEP: Color = Color::Rgb(58, 32, 48); // #3A2030
    pub const ROSE_EMBER: Color = Color::Rgb(72, 40, 56); // #482838
    pub const TEXT_PHANTOM: Color = Color::Rgb(32, 24, 32); // #201820

    // Derived aliases
    pub const BG: Color = Self::VOID;
    pub const BG_SECONDARY: Color = Color::Rgb(14, 12, 16); // #0E0C10 — subtle lift
    pub const BG_HIGHLIGHT: Color = Color::Rgb(34, 28, 36); // #221C24 — selection
    pub const FG: Color = Self::TEXT;
    pub const FG_DIM: Color = Self::TEXT_DIM;
    pub const FG_BRIGHT: Color = Self::BONE;

    pub const BG_BUBBLE_ALT: Color = Color::Rgb(18, 16, 22); // #121016 — alt bubble bg

    // Status colors
    pub const STATUS_OK: Color = Self::SAGE;
    pub const STATUS_ACTIVE: Color = Self::ROSE;
    pub const STATUS_WARN: Color = Self::WARNING;
    pub const STATUS_ERROR: Color = Self::EMBER;
    pub const STATUS_DIM: Color = Self::TEXT_DIM;

    // Semantic styles
    pub fn default_style() -> Style {
        Style::default().fg(Self::FG).bg(Self::BG)
    }

    pub fn block_style() -> Style {
        Style::default().fg(Self::FG_DIM).bg(Self::BG)
    }

    pub fn selected_style() -> Style {
        Style::default().fg(Self::FG_BRIGHT).bg(Self::BG_HIGHLIGHT)
    }

    pub fn active_style() -> Style {
        Style::default().fg(Self::ROSE).add_modifier(Modifier::BOLD)
    }

    pub fn title_style() -> Style {
        Style::default()
            .fg(Self::BONE_DIM)
            .add_modifier(Modifier::BOLD)
    }

    pub fn tab_active_style() -> Style {
        Style::default().fg(Self::BG).bg(Self::ROSE)
    }

    pub fn tab_inactive_style() -> Style {
        Style::default().fg(Self::FG_DIM).bg(Self::BG_SECONDARY)
    }

    pub fn error_style() -> Style {
        Style::default().fg(Self::STATUS_ERROR)
    }

    pub fn success_style() -> Style {
        Style::default().fg(Self::STATUS_OK)
    }

    pub fn dim_style() -> Style {
        Style::default().fg(Self::FG_DIM)
    }

    /// Focused panel border: bright rose, bold
    pub fn focused_border_style() -> Style {
        Style::default()
            .fg(Self::ROSE_BRIGHT)
            .add_modifier(Modifier::BOLD)
    }

    /// Focused panel title: bone, bold
    pub fn focused_title_style() -> Style {
        Style::default().fg(Self::BONE).add_modifier(Modifier::BOLD)
    }

    /// Unfocused panel border: nearly invisible, recedes into background
    pub fn unfocused_border_style() -> Style {
        Style::default().fg(Self::TEXT_PHANTOM)
    }

    /// Unfocused panel title: ghost text
    pub fn unfocused_title_style() -> Style {
        Style::default().fg(Self::TEXT_GHOST)
    }

    /// Per-role accent color for agent output
    pub fn role_accent(role: AgentRole) -> Color {
        match role {
            AgentRole::Conductor => Self::EMBER,
            AgentRole::Strategist => Self::BONE_DIM,
            AgentRole::Implementer => Self::ROSE,
            AgentRole::Architect => Self::SAGE,
            AgentRole::Auditor => Self::WARNING,
            AgentRole::Scribe => Self::DREAM,
            AgentRole::Critic => Self::TEXT,
            AgentRole::Refactorer => Self::SAGE,
            AgentRole::PrePlanner => Self::BONE_DIM,
            AgentRole::DocVerifier => Self::DREAM,
            AgentRole::IntegrationTester => Self::WARNING,
            AgentRole::MergeResolver => Self::EMBER,
            AgentRole::TerminalValidator => Self::SAGE,
            AgentRole::GolemLifecycleTester => Self::DREAM,
            AgentRole::SpecDriftDetector => Self::WARNING,
            AgentRole::RegressionDetector => Self::ROSE,
            AgentRole::PerformanceSentinel => Self::EMBER,
            AgentRole::CoverageTracker => Self::BONE_DIM,
            AgentRole::PlanLifecycleManager => Self::TEXT,
            AgentRole::CrossSystemTester => Self::SAGE,
            AgentRole::ErrorDiagnoser => Self::ROSE,
            AgentRole::Researcher => Self::DREAM,
            AgentRole::DependencyValidator => Self::WARNING,
            AgentRole::PatternExtractor => Self::BONE_DIM,
            AgentRole::SnapshotComparator => Self::SAGE,
            AgentRole::AutoFixer => Self::WARNING,
            AgentRole::QuickReviewer => Self::SAGE,
            AgentRole::FullLoopValidator => Self::SAGE,
        }
    }

    /// Per-phase accent color for pipeline visualization
    pub fn phase_accent(phase: &str) -> Color {
        match phase {
            "preflight" => Self::TEXT_DIM,
            "strategist" => Self::BONE_DIM,
            "implementer" => Self::ROSE,
            "compile-gate" => Self::WARNING,
            "test-gate" => Self::WARNING,
            "reviewing" => Self::SAGE,
            "critic-review" => Self::DREAM,
            "verdict" => Self::BONE,
            "doc-revision" => Self::DREAM,
            "committing" => Self::SAGE,
            _ => Self::FG_DIM,
        }
    }
}

/// Brighten an RGB color by a factor (1.0 = no change, 1.5 = 50% brighter)
pub fn brighten(color: Color, factor: f64) -> Color {
    match color {
        Color::Rgb(r, g, b) => Color::Rgb(
            (r as f64 * factor).min(255.0) as u8,
            (g as f64 * factor).min(255.0) as u8,
            (b as f64 * factor).min(255.0) as u8,
        ),
        other => other,
    }
}

/// Gradient with multiple color stops for smooth interpolation.
pub struct Gradient {
    stops: Vec<(f64, Color)>,
}

impl Gradient {
    pub const fn new(_stops: &[(f64, Color)]) -> Self {
        // Can't use const fn with Vec, so use a builder pattern
        Self { stops: Vec::new() }
    }

    pub fn from_stops(stops: Vec<(f64, Color)>) -> Self {
        Self { stops }
    }

    /// Sample the gradient at position t (0.0..=1.0).
    pub fn sample(&self, t: f64) -> Color {
        let t = t.clamp(0.0, 1.0);
        if self.stops.is_empty() {
            return Color::White;
        }
        if self.stops.len() == 1 {
            return self.stops[0].1;
        }

        // Find the two stops to interpolate between
        let mut lower = &self.stops[0];
        let mut upper = &self.stops[self.stops.len() - 1];
        for i in 0..self.stops.len() - 1 {
            if t >= self.stops[i].0 && t <= self.stops[i + 1].0 {
                lower = &self.stops[i];
                upper = &self.stops[i + 1];
                break;
            }
        }

        let range = upper.0 - lower.0;
        if range <= 0.0 {
            return lower.1;
        }
        let local_t = (t - lower.0) / range;
        lerp_color(lower.1, upper.1, local_t)
    }
}

/// Linearly interpolate between two RGB colors.
fn lerp_color(from: Color, to: Color, t: f64) -> Color {
    match (from, to) {
        (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
            let t = t.clamp(0.0, 1.0);
            Color::Rgb(
                (r1 as f64 + (r2 as f64 - r1 as f64) * t) as u8,
                (g1 as f64 + (g2 as f64 - g1 as f64) * t) as u8,
                (b1 as f64 + (b2 as f64 - b1 as f64) * t) as u8,
            )
        }
        _ => to,
    }
}

// Named gradients — lazily constructed since const fn can't build Vecs.
// Use these via gradient_fire(), gradient_context(), gradient_ocean().

/// Fire gradient: dark red → amber → gold. For progress bars.
pub fn gradient_fire() -> Gradient {
    Gradient::from_stops(vec![
        (0.0, Color::Rgb(100, 30, 30)),
        (0.5, Color::Rgb(200, 100, 30)),
        (1.0, Color::Rgb(220, 180, 60)),
    ])
}

/// Context pressure gradient: sage → warning → ember. For context gauges.
pub fn gradient_context() -> Gradient {
    Gradient::from_stops(vec![
        (0.0, Theme::SAGE),
        (0.5, Theme::WARNING),
        (1.0, Theme::EMBER),
    ])
}

/// Ocean gradient: deep blue → teal → cyan. For wave bars.
pub fn gradient_ocean() -> Gradient {
    Gradient::from_stops(vec![
        (0.0, Color::Rgb(40, 60, 90)),
        (0.5, Color::Rgb(60, 120, 160)),
        (1.0, Color::Rgb(80, 180, 200)),
    ])
}
