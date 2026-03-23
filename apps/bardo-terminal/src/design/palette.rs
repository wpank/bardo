//! Full ROSEDUST palette: 30+ color constants from `prd2/18-interfaces/rendering/00-design-system.md` section 2.
//!
//! Re-exports all Plan 04 constants from `crate::palette` and adds the remaining
//! tokens (BG_MID, BG_WARM, DREAM_DIM, DREAM_DEEP, BLEED_ROSE, HALFTONE_BG).

use ratatui::style::Color;

// Re-export Plan 04 palette constants so callers can import from one place.
pub use crate::palette::{
    BG_MID, BG_RAISED, BG_VOID, BG_WARM, BONE, BONE_DIM, BORDER, BORDER_ACTIVE, BORDER_DREAM,
    DANGER, DREAM, DREAM_DIM, DREAM_DEEP, NOISE_COOL, NOISE_WARM, PHOSPHOR_RES, ROSE, ROSE_BRIGHT,
    ROSE_DIM, ROSE_DEEP, ROSE_EMBER, SCANLINE_DARK, SUCCESS, TEXT_DIM, TEXT_GHOST, TEXT_PHANTOM,
    TEXT_PRIMARY, WARNING,
};

// ── New tokens not in Plan 04 ───────────────────────────────────────

/// Rose phosphor bleed at 9% apparent opacity, for low-density background tinting.
pub const BLEED_ROSE: Color = Color::Rgb(170, 112, 136);

/// Halftone background pattern fill.
pub const HALFTONE_BG: Color = Color::Rgb(14, 10, 16);

/// Extract (r, g, b) components from a `Color::Rgb`. Returns (0,0,0) for non-RGB variants.
pub fn rgb_components(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (0, 0, 0),
    }
}

/// Linearly interpolate between two RGB colors. `t` clamped to [0.0, 1.0].
pub fn lerp_color_linear(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let (ar, ag, ab) = rgb_components(a);
    let (br, bg, bb) = rgb_components(b);
    Color::Rgb(
        (ar as f32 + (br as f32 - ar as f32) * t) as u8,
        (ag as f32 + (bg as f32 - ag as f32) * t) as u8,
        (ab as f32 + (bb as f32 - ab as f32) * t) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rosedust_all_colors_no_pure_black() {
        let bgs = [BG_VOID, BG_RAISED, BG_MID, BG_WARM];
        for bg in bgs {
            let (r, g, b) = rgb_components(bg);
            assert!(
                (r as u16 + g as u16 + b as u16) > 0,
                "Background color {:?} must not be pure black",
                bg
            );
        }
    }

    #[test]
    fn test_bleed_rose_matches_spec() {
        assert_eq!(BLEED_ROSE, Color::Rgb(170, 112, 136));
    }

    #[test]
    fn test_halftone_bg_matches_spec() {
        assert_eq!(HALFTONE_BG, Color::Rgb(14, 10, 16));
    }

    #[test]
    fn test_lerp_color_endpoints() {
        let a = Color::Rgb(0, 0, 0);
        let b = Color::Rgb(100, 200, 50);
        assert_eq!(lerp_color_linear(a, b, 0.0), a);
        assert_eq!(lerp_color_linear(a, b, 1.0), b);
    }

    #[test]
    fn test_lerp_color_midpoint() {
        let a = Color::Rgb(0, 0, 0);
        let b = Color::Rgb(100, 200, 50);
        let mid = lerp_color_linear(a, b, 0.5);
        assert_eq!(mid, Color::Rgb(50, 100, 25));
    }

    #[test]
    fn test_rgb_components_non_rgb_returns_zeros() {
        assert_eq!(rgb_components(Color::Yellow), (0, 0, 0));
    }
}
