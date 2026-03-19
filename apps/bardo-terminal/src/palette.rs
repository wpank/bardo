#![allow(dead_code)]

//! ROSEDUST palette tokens and chrome glyphs.

use ratatui::style::{Color, Modifier};

/// Namespace marker for the terminal palette.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ColorPalette;

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

    #[test]
    fn key_tokens_match_the_spec_values() {
        let _palette = ColorPalette;

        assert_eq!(ROSE, Color::Rgb(170, 112, 136));
        assert_eq!(ROSE_BRIGHT, Color::Rgb(204, 144, 168));
        assert_eq!(BONE, Color::Rgb(200, 184, 144));
        assert_eq!(WARNING, Color::Rgb(170, 136, 85));
        assert_eq!(SUCCESS, Color::Rgb(112, 136, 122));
        assert_eq!(DANGER, ROSE_BRIGHT);
        assert_eq!(FRAME_OPEN, '⌈');
        assert_eq!(BLOCK_FULL, '█');
    }
}
