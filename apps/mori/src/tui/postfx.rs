use ratatui::{buffer::Buffer, layout::Rect, style::Color};

use super::color::hsv_to_rgb;
use super::vfx;

fn lerp_u8(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t).clamp(0.0, 255.0) as u8
}

/// Bloom: bright cells spread light to neighbors via box blur + screen blend.
pub fn bloom(area: Rect, buf: &mut Buffer, threshold: u8, radius: u16, intensity: f64) {
    if intensity < 0.01 || area.width < 4 || area.height < 2 {
        return;
    }

    let w = area.width as usize;
    let h = area.height as usize;

    // Extract brightness (skip spaces)
    let mut bright: Vec<(u8, u8, u8)> = Vec::with_capacity(w * h);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = &buf[(x, y)];
            let is_space = cell.symbol() == " ";
            match cell.fg {
                Color::Rgb(r, g, b) if !is_space => {
                    let lum = (r as u16 + g as u16 + b as u16) / 3;
                    if lum > threshold as u16 {
                        bright.push((r, g, b));
                    } else {
                        bright.push((0, 0, 0));
                    }
                }
                _ => bright.push((0, 0, 0)),
            }
        }
    }

    // Box blur on bright pixels
    let mut blurred = vec![(0u16, 0u16, 0u16); w * h];
    let r = radius as i16;
    for iy in 0..h {
        for ix in 0..w {
            let mut sr = 0u32;
            let mut sg = 0u32;
            let mut sb = 0u32;
            let mut count = 0u32;
            for dy in -r..=r {
                for dx in -r..=r {
                    let ny = iy as i16 + dy;
                    let nx = ix as i16 + dx;
                    if ny >= 0 && ny < h as i16 && nx >= 0 && nx < w as i16 {
                        let idx = ny as usize * w + nx as usize;
                        let (br, bg, bb) = bright[idx];
                        sr += br as u32;
                        sg += bg as u32;
                        sb += bb as u32;
                        count += 1;
                    }
                }
            }
            if count > 0 {
                blurred[iy * w + ix] = (
                    (sr / count) as u16,
                    (sg / count) as u16,
                    (sb / count) as u16,
                );
            }
        }
    }

    // Screen-blend blurred layer back
    for iy in 0..h {
        for ix in 0..w {
            let idx = iy * w + ix;
            let (br, bg, bb) = blurred[idx];
            if br == 0 && bg == 0 && bb == 0 {
                continue;
            }
            let x = area.left() + ix as u16;
            let y = area.top() + iy as u16;
            let cell = &mut buf[(x, y)];
            if let Color::Rgb(cr, cg, cb) = cell.fg {
                let bloom_r = (br as f64 * intensity / 255.0).min(1.0);
                let bloom_g = (bg as f64 * intensity / 255.0).min(1.0);
                let bloom_b = (bb as f64 * intensity / 255.0).min(1.0);
                let nr = 255.0 - (255.0 - cr as f64) * (1.0 - bloom_r);
                let ng = 255.0 - (255.0 - cg as f64) * (1.0 - bloom_g);
                let nb = 255.0 - (255.0 - cb as f64) * (1.0 - bloom_b);
                cell.set_fg(Color::Rgb(
                    nr.clamp(0.0, 255.0) as u8,
                    ng.clamp(0.0, 255.0) as u8,
                    nb.clamp(0.0, 255.0) as u8,
                ));
            }
        }
    }
}

/// Radial edge darkening.
pub fn vignette(area: Rect, buf: &mut Buffer, intensity: f64) {
    if intensity < 0.01 || area.width < 2 || area.height < 2 {
        return;
    }
    let w = area.width as f64;
    let h = area.height as f64;
    let cx = w / 2.0;
    let cy = h / 2.0;
    let max_dist = (cx * cx + cy * cy).sqrt();

    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let dx = (x - area.left()) as f64 - cx;
            let dy = (y - area.top()) as f64 - cy;
            let dist = (dx * dx + dy * dy).sqrt() / max_dist;
            let darken = 1.0 - dist * dist * intensity;
            if darken >= 0.99 {
                continue;
            }
            let darken = darken.max(0.3);
            let cell = &mut buf[(x, y)];
            if let Color::Rgb(r, g, b) = cell.fg {
                cell.set_fg(Color::Rgb(
                    (r as f64 * darken) as u8,
                    (g as f64 * darken) as u8,
                    (b as f64 * darken) as u8,
                ));
            }
            if let Color::Rgb(r, g, b) = cell.bg {
                cell.set_bg(Color::Rgb(
                    (r as f64 * darken) as u8,
                    (g as f64 * darken) as u8,
                    (b as f64 * darken) as u8,
                ));
            }
        }
    }
}

/// Dim the entire buffer by a factor. Used for modal overlays.
/// factor: 0.0 = full black, 1.0 = no change. Typical: 0.4-0.6.
pub fn dim_overlay(area: Rect, buf: &mut Buffer, factor: f64) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = &mut buf[(x, y)];
            if let Color::Rgb(r, g, b) = cell.fg {
                cell.set_fg(Color::Rgb(
                    (r as f64 * factor) as u8,
                    (g as f64 * factor) as u8,
                    (b as f64 * factor) as u8,
                ));
            }
            if let Color::Rgb(r, g, b) = cell.bg {
                cell.set_bg(Color::Rgb(
                    (r as f64 * factor) as u8,
                    (g as f64 * factor) as u8,
                    (b as f64 * factor) as u8,
                ));
            }
        }
    }
}

/// Add a subtle glow/shadow around a rect. Tints adjacent cells with a color.
pub fn modal_glow(
    area: Rect,
    buf: &mut Buffer,
    full_area: Rect,
    glow_color: Color,
    intensity: f64,
) {
    let glow_r = 2u16; // glow radius in cells
    if let Color::Rgb(gr, gg, gb) = glow_color {
        for y in area.top().saturating_sub(glow_r)
            ..=(area.bottom() + glow_r).min(full_area.bottom().saturating_sub(1))
        {
            for x in area.left().saturating_sub(glow_r)
                ..=(area.right() + glow_r).min(full_area.right().saturating_sub(1))
            {
                // Skip cells inside the modal
                if x >= area.left() && x < area.right() && y >= area.top() && y < area.bottom() {
                    continue;
                }
                // Skip cells outside the full area
                if x < full_area.left() || y < full_area.top() {
                    continue;
                }
                // Distance from nearest edge of the modal
                let dx = if x < area.left() {
                    (area.left() - x) as f64
                } else if x >= area.right() {
                    (x - area.right() + 1) as f64
                } else {
                    0.0
                };
                let dy = if y < area.top() {
                    (area.top() - y) as f64
                } else if y >= area.bottom() {
                    (y - area.bottom() + 1) as f64
                } else {
                    0.0
                };
                let dist = (dx * dx + dy * dy).sqrt();
                let t = ((1.0 - dist / glow_r as f64) * intensity).max(0.0);
                if t < 0.01 {
                    continue;
                }

                let cell = &mut buf[(x, y)];
                if let Color::Rgb(r, g, b) = cell.bg {
                    cell.set_bg(Color::Rgb(
                        lerp_u8(r, gr, t),
                        lerp_u8(g, gg, t),
                        lerp_u8(b, gb, t),
                    ));
                }
            }
        }
    }
}

/// Stateless drifting motes.
pub fn ambient_orbs(area: Rect, buf: &mut Buffer, elapsed: f64, count: usize, brightness: f64) {
    if area.width < 4 || area.height < 4 {
        return;
    }
    let w = area.width as f64;
    let h = area.height as f64;
    let orb_chars = [
        '\u{00B7}', '\u{00B0}', '\u{2022}', '\u{2727}', '\u{2218}', '\u{22C6}',
    ];

    for i in 0..count {
        let seed = i as f64;
        let ox = ((seed * 7.31 + elapsed * 0.13 + (seed * 0.37).cos() * elapsed * 0.07).sin()
            * 0.5
            + 0.5)
            * w;
        let oy = ((seed * 13.73 + elapsed * 0.11 + (seed * 0.53).sin() * elapsed * 0.05).sin()
            * 0.5
            + 0.5)
            * h;

        let pulse = ((elapsed * 0.6 + seed * 2.17).sin() * 0.5 + 0.5).powf(1.5);
        let val = brightness * pulse;
        if val < 0.02 {
            continue;
        }

        let hue = ((seed * 47.3 + elapsed * 3.0) % 360.0 + 360.0) % 360.0;
        let (or, og, ob) = hsv_to_rgb(hue, 0.25 + pulse * 0.15, val);

        let px = area.x + (ox as u16).min(area.width.saturating_sub(1));
        let py = area.y + (oy as u16).min(area.height.saturating_sub(1));

        if px >= area.right() || py >= area.bottom() {
            continue;
        }

        let ch = orb_chars[i % orb_chars.len()];
        let cell = &mut buf[(px, py)];
        if let Color::Rgb(r, g, b) = cell.fg {
            cell.set_char(ch).set_fg(Color::Rgb(
                r.saturating_add(or),
                g.saturating_add(og),
                b.saturating_add(ob),
            ));
        } else {
            cell.set_char(ch).set_fg(Color::Rgb(or, og, ob));
        }

        // Dim glow on adjacent cells
        let dr = (or as f64 * 0.3) as u8;
        let dg = (og as f64 * 0.3) as u8;
        let db = (ob as f64 * 0.3) as u8;
        if dr > 0 || dg > 0 || db > 0 {
            for &(nx, ny) in &[
                (px.wrapping_add(1), py),
                (px.wrapping_sub(1), py),
                (px, py.wrapping_add(1)),
                (px, py.wrapping_sub(1)),
            ] {
                if nx >= area.left() && nx < area.right() && ny >= area.top() && ny < area.bottom()
                {
                    let ncell = &mut buf[(nx, ny)];
                    if let Color::Rgb(r, g, b) = ncell.fg {
                        ncell.set_fg(Color::Rgb(
                            r.saturating_add(dr),
                            g.saturating_add(dg),
                            b.saturating_add(db),
                        ));
                    }
                }
            }
        }
    }
}

/// Vignette + film grain + breathing in a single pass.
pub fn dream_atmosphere(area: Rect, buf: &mut Buffer, elapsed: f64, frame_seed: u64) {
    if area.width < 2 || area.height < 2 {
        return;
    }
    let w = area.width as f64;
    let h = area.height as f64;
    let cx = w / 2.0;
    let cy = h / 2.0;
    let max_dist = (cx * cx + cy * cy).sqrt();

    let breath = 1.0 + (elapsed * 0.35).sin() * 0.08 + (elapsed * 0.17).sin() * 0.04;

    let mut rng = fastrand::Rng::with_seed(frame_seed);

    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let dx = (x - area.left()) as f64 - cx;
            let dy = (y - area.top()) as f64 - cy;
            let dist = (dx * dx + dy * dy).sqrt() / max_dist;

            let vig = 1.0 - dist * dist * 0.5;
            let rose_tint = (1.0 - vig) * 16.0;
            let grain = ((rng.u8(..) as f64 - 128.0) / 128.0) * 4.0;

            let cell = &mut buf[(x, y)];
            if let Color::Rgb(r, g, b) = cell.fg {
                let rf = r as f64 * vig * breath + grain + rose_tint * 0.7;
                let gf = g as f64 * vig * breath + grain + rose_tint * 0.2;
                let bf = b as f64 * vig * breath + grain + rose_tint * 0.5;
                cell.set_fg(Color::Rgb(
                    rf.clamp(0.0, 255.0) as u8,
                    gf.clamp(0.0, 255.0) as u8,
                    bf.clamp(0.0, 255.0) as u8,
                ));
            }
            if let Color::Rgb(r, g, b) = cell.bg {
                let rf = r as f64 * vig * breath + rose_tint * 0.25;
                let gf = g as f64 * vig * breath + rose_tint * 0.08;
                let bf = b as f64 * vig * breath + rose_tint * 0.18;
                cell.set_bg(Color::Rgb(
                    rf.clamp(0.0, 255.0) as u8,
                    gf.clamp(0.0, 255.0) as u8,
                    bf.clamp(0.0, 255.0) as u8,
                ));
            }
        }
    }
}

/// Rose-tinted final color grade pass. Boosts red and blue, reduces green.
pub fn amber_color_grade(area: Rect, buf: &mut Buffer, intensity: f64) {
    if intensity < 0.01 {
        return;
    }
    let boost_r = (6.0 * intensity) as u8;
    let cut_g = (8.0 * intensity) as u8;
    let boost_b = (4.0 * intensity) as u8;
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = &mut buf[(x, y)];
            if let Color::Rgb(r, g, b) = cell.fg {
                let lum = (r as u16 + g as u16 + b as u16) / 3;
                let scale = (lum as f64 / 255.0).min(1.0);
                let scaled_r = (boost_r as f64 * scale) as u8;
                let scaled_g = (cut_g as f64 * scale) as u8;
                let scaled_b = (boost_b as f64 * scale) as u8;
                cell.set_fg(Color::Rgb(
                    r.saturating_add(scaled_r),
                    g.saturating_sub(scaled_g),
                    b.saturating_add(scaled_b),
                ));
            }
            if let Color::Rgb(r, g, b) = cell.bg {
                let lum = (r as u16 + g as u16 + b as u16) / 3;
                let scale = (lum as f64 / 255.0).min(1.0);
                let scaled_r = (boost_r as f64 * scale) as u8;
                let scaled_g = (cut_g as f64 * scale) as u8;
                let scaled_b = (boost_b as f64 * scale) as u8;
                cell.set_bg(Color::Rgb(
                    r.saturating_add(scaled_r),
                    g.saturating_sub(scaled_g),
                    b.saturating_add(scaled_b),
                ));
            }
        }
    }
}

/// Fill empty cells with sparse shimmering dust.
pub fn ambient_fill(area: Rect, buf: &mut Buffer, elapsed: f64) {
    let ethereal = vfx::ETHEREAL;
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = &buf[(x, y)];
            if cell.symbol() != " " {
                continue;
            }
            // ~2% density via deterministic noise
            let n = vfx::noise(x as f64, y as f64, elapsed * 0.5);
            if n > 0.02 {
                continue;
            }
            // Faint shimmer
            let shimmer = (elapsed * 0.8 + x as f64 * 0.3 + y as f64 * 0.7).sin() * 0.5 + 0.5;
            let brightness = 15.0 + shimmer * 20.0;
            let ch = ethereal[((x as usize).wrapping_mul(7) + y as usize) % ethereal.len()];
            let cell = &mut buf[(x, y)];
            cell.set_char(ch);
            cell.set_fg(Color::Rgb(
                brightness as u8,
                (brightness * 0.8) as u8,
                (brightness * 0.9) as u8,
            ));
        }
    }
}
