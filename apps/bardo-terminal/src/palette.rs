#![allow(dead_code)]

//! ROSEDUST palette tokens and chrome glyphs.
//!
//! `Color::Rgb` triples match `prd2/18-interfaces/04-design-system.md` §2 (solid RGB; where the
//! design doc lists borders with opacity, the TUI uses the same RGB components).

use ratatui::style::{Color, Modifier};

/// Namespace marker for the terminal palette.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ColorPalette;

impl ColorPalette {
    pub(crate) const BG_VOID: Color = crate::palette::BG_VOID;
    pub(crate) const BG_RAISED: Color = crate::palette::BG_RAISED;
    pub(crate) const BG_MID: Color = crate::palette::BG_MID;
    pub(crate) const BG_WARM: Color = crate::palette::BG_WARM;
    pub(crate) const BORDER: Color = crate::palette::BORDER;
    pub(crate) const BORDER_ACTIVE: Color = crate::palette::BORDER_ACTIVE;
    pub(crate) const BORDER_DREAM: Color = crate::palette::BORDER_DREAM;

    pub(crate) const ROSE: Color = crate::palette::ROSE;
    pub(crate) const ROSE_BRIGHT: Color = crate::palette::ROSE_BRIGHT;
    pub(crate) const ROSE_DIM: Color = crate::palette::ROSE_DIM;
    pub(crate) const ROSE_DEEP: Color = crate::palette::ROSE_DEEP;
    pub(crate) const ROSE_EMBER: Color = crate::palette::ROSE_EMBER;
    pub(crate) const BONE: Color = crate::palette::BONE;
    pub(crate) const BONE_DIM: Color = crate::palette::BONE_DIM;

    pub(crate) const TEXT_PRIMARY: Color = crate::palette::TEXT_PRIMARY;
    pub(crate) const TEXT_DIM: Color = crate::palette::TEXT_DIM;
    pub(crate) const TEXT_GHOST: Color = crate::palette::TEXT_GHOST;
    pub(crate) const TEXT_PHANTOM: Color = crate::palette::TEXT_PHANTOM;

    pub(crate) const DREAM: Color = crate::palette::DREAM;
    pub(crate) const DREAM_DIM: Color = crate::palette::DREAM_DIM;
    pub(crate) const DREAM_DEEP: Color = crate::palette::DREAM_DEEP;
    pub(crate) const WARNING: Color = crate::palette::WARNING;
    pub(crate) const SUCCESS: Color = crate::palette::SUCCESS;
    pub(crate) const DANGER: Color = crate::palette::DANGER;

    pub(crate) const SCANLINE_DARK: Color = crate::palette::SCANLINE_DARK;
    pub(crate) const PHOSPHOR_RES: Color = crate::palette::PHOSPHOR_RES;
    pub(crate) const NOISE_WARM: Color = crate::palette::NOISE_WARM;
    pub(crate) const NOISE_COOL: Color = crate::palette::NOISE_COOL;

    pub(crate) const STYLE_BOLD: Modifier = crate::palette::STYLE_BOLD;
    pub(crate) const STYLE_DIM: Modifier = crate::palette::STYLE_DIM;
    pub(crate) const STYLE_ITALIC: Modifier = crate::palette::STYLE_ITALIC;

    pub(crate) const BOX_TOP_LEFT: char = crate::palette::BOX_TOP_LEFT;
    pub(crate) const BOX_TOP_RIGHT: char = crate::palette::BOX_TOP_RIGHT;
    pub(crate) const BOX_BOTTOM_LEFT: char = crate::palette::BOX_BOTTOM_LEFT;
    pub(crate) const BOX_BOTTOM_RIGHT: char = crate::palette::BOX_BOTTOM_RIGHT;
    pub(crate) const BOX_HORIZONTAL: char = crate::palette::BOX_HORIZONTAL;
    pub(crate) const BOX_VERTICAL: char = crate::palette::BOX_VERTICAL;
    pub(crate) const BOX_T_DOWN: char = crate::palette::BOX_T_DOWN;
    pub(crate) const BOX_T_UP: char = crate::palette::BOX_T_UP;
    pub(crate) const BOX_T_RIGHT: char = crate::palette::BOX_T_RIGHT;
    pub(crate) const BOX_T_LEFT: char = crate::palette::BOX_T_LEFT;
    pub(crate) const BOX_CROSS: char = crate::palette::BOX_CROSS;
    pub(crate) const FRAME_OPEN: char = crate::palette::FRAME_OPEN;
    pub(crate) const FRAME_CLOSE: char = crate::palette::FRAME_CLOSE;
    pub(crate) const BLOCK_FULL: char = crate::palette::BLOCK_FULL;
    pub(crate) const BLOCK_DARK: char = crate::palette::BLOCK_DARK;
    pub(crate) const BLOCK_MED: char = crate::palette::BLOCK_MED;
    pub(crate) const BLOCK_LIGHT: char = crate::palette::BLOCK_LIGHT;
}

/// Deep background void color.
pub(crate) const BG_VOID: Color = Color::Rgb(6, 6, 8);
/// Raised panel background color.
pub(crate) const BG_RAISED: Color = Color::Rgb(12, 10, 14);
/// Mid-depth background color.
pub(crate) const BG_MID: Color = Color::Rgb(8, 8, 16);
/// Warm-shifted void background.
pub(crate) const BG_WARM: Color = Color::Rgb(10, 8, 8);
/// Standard border color.
pub(crate) const BORDER: Color = Color::Rgb(24, 20, 32);
/// Active border color.
pub(crate) const BORDER_ACTIVE: Color = Color::Rgb(170, 112, 136);
/// Dream-state border color.
pub(crate) const BORDER_DREAM: Color = Color::Rgb(88, 88, 120);
/// Primary rose text color.
pub(crate) const ROSE: Color = Color::Rgb(170, 112, 136);
/// Bright rose color for alerts.
pub(crate) const ROSE_BRIGHT: Color = Color::Rgb(204, 144, 168);
/// Dim rose color.
pub(crate) const ROSE_DIM: Color = Color::Rgb(122, 80, 96);
/// Deep rose color.
pub(crate) const ROSE_DEEP: Color = Color::Rgb(58, 32, 48);
/// Phosphor residue rose color.
pub(crate) const ROSE_EMBER: Color = Color::Rgb(72, 40, 56);
/// Bone highlight color.
pub(crate) const BONE: Color = Color::Rgb(200, 184, 144);
/// Dim bone color.
pub(crate) const BONE_DIM: Color = Color::Rgb(138, 122, 90);
/// Primary readable text color.
pub(crate) const TEXT_PRIMARY: Color = Color::Rgb(152, 128, 144);
/// Secondary text color.
pub(crate) const TEXT_DIM: Color = Color::Rgb(88, 72, 88);
/// Barely visible text color.
pub(crate) const TEXT_GHOST: Color = Color::Rgb(48, 40, 48);
/// Subliminal text color.
pub(crate) const TEXT_PHANTOM: Color = Color::Rgb(32, 24, 32);
/// Dream-state primary color.
pub(crate) const DREAM: Color = Color::Rgb(88, 88, 120);
/// Dim dream color.
pub(crate) const DREAM_DIM: Color = Color::Rgb(56, 56, 88);
/// Deep dream background noise color.
pub(crate) const DREAM_DEEP: Color = Color::Rgb(40, 40, 72);
/// Warning color.
pub(crate) const WARNING: Color = Color::Rgb(170, 136, 85);
/// Success color.
pub(crate) const SUCCESS: Color = Color::Rgb(112, 136, 122);
/// Danger alias.
pub(crate) const DANGER: Color = ROSE_BRIGHT;
/// Scanline dark color.
pub(crate) const SCANLINE_DARK: Color = Color::Rgb(5, 5, 7);
/// Phosphor residue color.
pub(crate) const PHOSPHOR_RES: Color = Color::Rgb(26, 16, 24);
/// Warm noise color.
pub(crate) const NOISE_WARM: Color = Color::Rgb(42, 24, 32);
/// Cool noise color.
pub(crate) const NOISE_COOL: Color = Color::Rgb(32, 24, 40);

/// Bold text modifier.
pub(crate) const STYLE_BOLD: Modifier = Modifier::BOLD;
/// Dim text modifier.
pub(crate) const STYLE_DIM: Modifier = Modifier::DIM;
/// Italic text modifier.
pub(crate) const STYLE_ITALIC: Modifier = Modifier::ITALIC;

/// Top-left box-drawing corner.
pub(crate) const BOX_TOP_LEFT: char = '┌';
/// Top-right box-drawing corner.
pub(crate) const BOX_TOP_RIGHT: char = '┐';
/// Bottom-left box-drawing corner.
pub(crate) const BOX_BOTTOM_LEFT: char = '└';
/// Bottom-right box-drawing corner.
pub(crate) const BOX_BOTTOM_RIGHT: char = '┘';
/// Horizontal box-drawing line.
pub(crate) const BOX_HORIZONTAL: char = '─';
/// Vertical box-drawing line.
pub(crate) const BOX_VERTICAL: char = '│';
/// Downward T junction.
pub(crate) const BOX_T_DOWN: char = '┬';
/// Upward T junction.
pub(crate) const BOX_T_UP: char = '┴';
/// Rightward T junction.
pub(crate) const BOX_T_RIGHT: char = '├';
/// Leftward T junction.
pub(crate) const BOX_T_LEFT: char = '┤';
/// Cross junction.
pub(crate) const BOX_CROSS: char = '┼';
/// Open frame bracket.
pub(crate) const FRAME_OPEN: char = '⌈';
/// Close frame bracket.
pub(crate) const FRAME_CLOSE: char = '⌋';
/// Full block glyph.
pub(crate) const BLOCK_FULL: char = '█';
/// Dark block glyph.
pub(crate) const BLOCK_DARK: char = '▓';
/// Medium block glyph.
pub(crate) const BLOCK_MED: char = '▒';
/// Light block glyph.
pub(crate) const BLOCK_LIGHT: char = '░';

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn void_background_is_not_pure_black() {
        let Color::Rgb(r, g, b) = BG_VOID else {
            panic!("BG_VOID must be an RGB color");
        };

        assert!(r > 0 || g > 0 || b > 0);
    }

    /// Every `Color::Rgb` value matches design-system §2 / plan 04 Quick Reference.
    #[test]
    fn rosedust_rgb_tokens_match_spec() {
        assert_eq!(BG_VOID, Color::Rgb(6, 6, 8));
        assert_eq!(BG_RAISED, Color::Rgb(12, 10, 14));
        assert_eq!(BG_MID, Color::Rgb(8, 8, 16));
        assert_eq!(BG_WARM, Color::Rgb(10, 8, 8));
        assert_eq!(BORDER, Color::Rgb(24, 20, 32));
        assert_eq!(BORDER_ACTIVE, Color::Rgb(170, 112, 136));
        assert_eq!(BORDER_DREAM, Color::Rgb(88, 88, 120));

        assert_eq!(ROSE, Color::Rgb(170, 112, 136));
        assert_eq!(ROSE_BRIGHT, Color::Rgb(204, 144, 168));
        assert_eq!(ROSE_DIM, Color::Rgb(122, 80, 96));
        assert_eq!(ROSE_DEEP, Color::Rgb(58, 32, 48));
        assert_eq!(ROSE_EMBER, Color::Rgb(72, 40, 56));

        assert_eq!(BONE, Color::Rgb(200, 184, 144));
        assert_eq!(BONE_DIM, Color::Rgb(138, 122, 90));

        assert_eq!(TEXT_PRIMARY, Color::Rgb(152, 128, 144));
        assert_eq!(TEXT_DIM, Color::Rgb(88, 72, 88));
        assert_eq!(TEXT_GHOST, Color::Rgb(48, 40, 48));
        assert_eq!(TEXT_PHANTOM, Color::Rgb(32, 24, 32));

        assert_eq!(DREAM, Color::Rgb(88, 88, 120));
        assert_eq!(DREAM_DIM, Color::Rgb(56, 56, 88));
        assert_eq!(DREAM_DEEP, Color::Rgb(40, 40, 72));
        assert_eq!(WARNING, Color::Rgb(170, 136, 85));
        assert_eq!(SUCCESS, Color::Rgb(112, 136, 122));
        assert_eq!(DANGER, ROSE_BRIGHT);

        assert_eq!(SCANLINE_DARK, Color::Rgb(5, 5, 7));
        assert_eq!(PHOSPHOR_RES, Color::Rgb(26, 16, 24));
        assert_eq!(NOISE_WARM, Color::Rgb(42, 24, 32));
        assert_eq!(NOISE_COOL, Color::Rgb(32, 24, 40));
    }

    #[test]
    fn box_drawing_and_block_glyphs_match_spec() {
        assert_eq!(BOX_TOP_LEFT, '┌');
        assert_eq!(BOX_TOP_RIGHT, '┐');
        assert_eq!(BOX_BOTTOM_LEFT, '└');
        assert_eq!(BOX_BOTTOM_RIGHT, '┘');
        assert_eq!(BOX_HORIZONTAL, '─');
        assert_eq!(BOX_VERTICAL, '│');
        assert_eq!(BOX_T_DOWN, '┬');
        assert_eq!(BOX_T_UP, '┴');
        assert_eq!(BOX_T_RIGHT, '├');
        assert_eq!(BOX_T_LEFT, '┤');
        assert_eq!(BOX_CROSS, '┼');
        assert_eq!(FRAME_OPEN, '⌈');
        assert_eq!(FRAME_CLOSE, '⌋');
        assert_eq!(BLOCK_FULL, '█');
        assert_eq!(BLOCK_DARK, '▓');
        assert_eq!(BLOCK_MED, '▒');
        assert_eq!(BLOCK_LIGHT, '░');
    }

    #[test]
    fn style_modifiers_match_ratatui_spec() {
        assert_eq!(STYLE_BOLD, Modifier::BOLD);
        assert_eq!(STYLE_DIM, Modifier::DIM);
        assert_eq!(STYLE_ITALIC, Modifier::ITALIC);
    }

    #[test]
    fn namespace_marker_re_exports_palette_tokens() {
        assert_eq!(ColorPalette::BG_VOID, BG_VOID);
        assert_eq!(ColorPalette::BORDER_DREAM, BORDER_DREAM);
        assert_eq!(ColorPalette::ROSE_EMBER, ROSE_EMBER);
        assert_eq!(ColorPalette::TEXT_PHANTOM, TEXT_PHANTOM);
        assert_eq!(ColorPalette::DREAM_DEEP, DREAM_DEEP);
        assert_eq!(ColorPalette::NOISE_COOL, NOISE_COOL);
        assert_eq!(ColorPalette::DANGER, ROSE_BRIGHT);
        assert_eq!(ColorPalette::STYLE_ITALIC, Modifier::ITALIC);
        assert_eq!(ColorPalette::BOX_CROSS, '┼');
        assert_eq!(ColorPalette::BLOCK_LIGHT, '░');
    }
}
