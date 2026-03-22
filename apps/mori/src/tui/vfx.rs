/// Stateless VFX field library. Pure math, no allocations.

// Character palettes

pub const DENSITY: [char; 10] = [' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];
pub const ORBS: [char; 6] = [
    '\u{00B7}', '\u{00B0}', '\u{2022}', '\u{25E6}', '\u{25CB}', '\u{25CF}',
];
pub const ETHEREAL: [char; 8] = [
    '\u{2727}', '\u{00B7}', '\u{00B0}', '\u{2726}', '\u{2218}', '\u{22C6}', '\u{2736}', '\u{274B}',
];
pub const DECAY_GLYPHS: [char; 6] = ['\u{2591}', '\u{2592}', '\u{00B7}', '.', ':', '\u{254C}'];

// Character mappers

pub fn density_char(v: f64) -> char {
    let idx = (v.clamp(0.0, 1.0) * 9.0) as usize;
    DENSITY[idx.min(9)]
}

pub fn orb_char(v: f64) -> char {
    let idx = (v.clamp(0.0, 1.0) * 5.0) as usize;
    ORBS[idx.min(5)]
}

pub fn ethereal_char(v: f64) -> char {
    let idx = (v.clamp(0.0, 1.0) * 7.0) as usize;
    ETHEREAL[idx.min(7)]
}

// Field generators

pub fn plasma(x: f64, y: f64, t: f64) -> f64 {
    let cx = x - 0.5;
    let cy = y - 0.5;
    ((x * 0.05 + t).sin()
        + (y * 0.05 + t * 1.2).sin()
        + ((x + y) * 0.03 + t * 0.8).sin()
        + ((cx * cx + cy * cy).sqrt() * 0.1 - t).sin())
        / 4.0
}

pub fn noise(x: f64, y: f64, seed: f64) -> f64 {
    ((x * 7.3 + y * 13.7 + seed).sin() * 43758.5453)
        .fract()
        .abs()
}

pub fn smooth_noise(x: f64, y: f64, seed: f64) -> f64 {
    let ix = x.floor();
    let iy = y.floor();
    let fx = x - ix;
    let fy = y - iy;
    let sx = fx * fx * (3.0 - 2.0 * fx);
    let sy = fy * fy * (3.0 - 2.0 * fy);
    let n00 = noise(ix, iy, seed);
    let n10 = noise(ix + 1.0, iy, seed);
    let n01 = noise(ix, iy + 1.0, seed);
    let n11 = noise(ix + 1.0, iy + 1.0, seed);
    let nx0 = n00 + (n10 - n00) * sx;
    let nx1 = n01 + (n11 - n01) * sx;
    nx0 + (nx1 - nx0) * sy
}

pub fn fbm(x: f64, y: f64, seed: f64, octaves: u32, lacunarity: f64, gain: f64) -> f64 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut total_amp = 0.0;
    for i in 0..octaves {
        value += smooth_noise(x * frequency, y * frequency, seed + i as f64 * 100.0) * amplitude;
        total_amp += amplitude;
        frequency *= lacunarity;
        amplitude *= gain;
    }
    value / total_amp
}

pub fn voronoi(x: f64, y: f64, seed: f64) -> f64 {
    let ix = x.floor();
    let iy = y.floor();
    let fx = x - ix;
    let fy = y - iy;
    let mut min_dist = 2.0f64;
    for dy in -1..=1 {
        for dx in -1..=1 {
            let nx = ix + dx as f64;
            let ny = iy + dy as f64;
            let px = noise(nx, ny, seed) + dx as f64 - fx;
            let py = noise(nx, ny, seed + 50.0) + dy as f64 - fy;
            let dist = (px * px + py * py).sqrt();
            min_dist = min_dist.min(dist);
        }
    }
    min_dist.clamp(0.0, 1.0)
}

pub fn ripple(x: f64, y: f64, cx: f64, cy: f64, t: f64) -> f64 {
    let dx = x - cx;
    let dy = y - cy;
    let dist = (dx * dx + dy * dy).sqrt();
    let ring = ((dist * 0.8 - t * 3.0).sin() * 0.5 + 0.5) * (-dist * 0.05).exp();
    ring.clamp(0.0, 1.0)
}
