use ratatui::{buffer::Buffer, layout::Rect};

use super::effects_config::EffectsConfig;
use super::postfx;
use super::tabs::Tab;

/// Apply per-tab postfx composition pipeline.
pub fn apply_pipeline(
    tab: &Tab,
    area: Rect,
    buf: &mut Buffer,
    elapsed: f64,
    _frame: u64,
    fx: &EffectsConfig,
) {
    if !fx.screen_postfx {
        return;
    }

    match tab {
        Tab::Dashboard => {
            if fx.vignette {
                postfx::vignette(area, buf, 0.3);
            }
            if fx.ambient_orbs {
                postfx::ambient_orbs(area, buf, elapsed, 8, 0.25);
            }
            if fx.bloom {
                postfx::bloom(area, buf, 180, 1, 0.15);
            }
            if fx.amber_color_grade {
                postfx::amber_color_grade(area, buf, 0.25);
            }
        }
        Tab::Agents => {
            if fx.bloom {
                postfx::bloom(area, buf, 200, 1, 0.12);
            }
            if fx.vignette {
                postfx::vignette(area, buf, 0.2);
            }
        }
        Tab::Git => {
            if fx.vignette {
                postfx::vignette(area, buf, 0.2);
            }
            if fx.bloom {
                postfx::bloom(area, buf, 200, 1, 0.1);
            }
        }
        Tab::Plans => {
            if fx.bloom {
                postfx::bloom(area, buf, 180, 1, 0.12);
            }
            if fx.vignette {
                postfx::vignette(area, buf, 0.25);
            }
        }
        Tab::Logs => {
            if fx.bloom {
                postfx::bloom(area, buf, 200, 1, 0.08);
            }
        }
        Tab::Config => {
            if fx.bloom {
                postfx::bloom(area, buf, 180, 1, 0.1);
            }
        }
    }
}
