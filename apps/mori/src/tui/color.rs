use ratatui::style::Color;

pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let h = h.rem_euclid(360.0);
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (
        ((r + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((g + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((b + m) * 255.0).clamp(0.0, 255.0) as u8,
    )
}

pub fn hsv_color(h: f64, s: f64, v: f64) -> Color {
    let (r, g, b) = hsv_to_rgb(h, s, v);
    Color::Rgb(r, g, b)
}

pub fn screen_blend(a: (u8, u8, u8), b: (u8, u8, u8)) -> (u8, u8, u8) {
    let blend = |a: u8, b: u8| -> u8 {
        let af = a as f64 / 255.0;
        let bf = b as f64 / 255.0;
        ((1.0 - (1.0 - af) * (1.0 - bf)) * 255.0) as u8
    };
    (blend(a.0, b.0), blend(a.1, b.1), blend(a.2, b.2))
}

pub fn additive_blend(a: (u8, u8, u8), b: (u8, u8, u8)) -> (u8, u8, u8) {
    (
        a.0.saturating_add(b.0),
        a.1.saturating_add(b.1),
        a.2.saturating_add(b.2),
    )
}

/// Pre-computed gradient lookup table for O(1) sampling.
pub struct Gradient {
    lut: Vec<(u8, u8, u8)>,
}

impl Gradient {
    pub fn from_hsv_stops(stops: &[(f64, f64, f64, f64)], size: usize) -> Self {
        let size = size.max(2);
        let mut lut = Vec::with_capacity(size);
        for i in 0..size {
            let t = i as f64 / (size - 1) as f64;
            let mut left = &stops[0];
            let mut right = &stops[stops.len() - 1];
            for w in stops.windows(2) {
                if w[0].0 <= t && t <= w[1].0 {
                    left = &w[0];
                    right = &w[1];
                    break;
                }
            }
            let span = right.0 - left.0;
            let local_t = if span > 0.0 { (t - left.0) / span } else { 0.0 };
            let h = lerp_hue(left.1, right.1, local_t);
            let s = left.2 + (right.2 - left.2) * local_t;
            let v = left.3 + (right.3 - left.3) * local_t;
            lut.push(hsv_to_rgb(h, s, v));
        }
        Self { lut }
    }

    pub fn sample(&self, t: f64) -> (u8, u8, u8) {
        let idx = (t.clamp(0.0, 1.0) * (self.lut.len() - 1) as f64) as usize;
        self.lut[idx.min(self.lut.len() - 1)]
    }

    pub fn sample_color(&self, t: f64) -> Color {
        let (r, g, b) = self.sample(t);
        Color::Rgb(r, g, b)
    }
}

pub fn lerp_hue(a: f64, b: f64, t: f64) -> f64 {
    let diff = b - a;
    let d = if diff > 180.0 {
        diff - 360.0
    } else if diff < -180.0 {
        diff + 360.0
    } else {
        diff
    };
    (a + d * t).rem_euclid(360.0)
}

// Named gradients

pub fn fire_gradient() -> Gradient {
    // Rose-themed: 320->350 hue range matching rosedust palette
    Gradient::from_hsv_stops(
        &[
            (0.0, 320.0, 1.0, 0.2),
            (0.3, 330.0, 1.0, 0.6),
            (0.6, 340.0, 0.9, 0.9),
            (0.8, 345.0, 0.8, 1.0),
            (1.0, 350.0, 0.3, 1.0),
        ],
        256,
    )
}

pub fn context_gradient() -> Gradient {
    // Sage -> warning -> ember pressure gradient
    // Sage ~150deg, Warning ~38deg, Ember ~14deg
    Gradient::from_hsv_stops(
        &[
            (0.0, 150.0, 0.45, 0.53), // sage
            (0.5, 38.0, 0.50, 0.67),  // warning
            (1.0, 14.0, 0.53, 0.67),  // ember
        ],
        256,
    )
}

pub fn ocean_gradient() -> Gradient {
    // Deep blue -> teal -> cyan
    Gradient::from_hsv_stops(
        &[
            (0.0, 220.0, 0.9, 0.15),
            (0.3, 210.0, 0.8, 0.35),
            (0.6, 195.0, 0.7, 0.55),
            (0.8, 185.0, 0.6, 0.75),
            (1.0, 180.0, 0.5, 0.85),
        ],
        256,
    )
}

pub fn ember_gradient() -> Gradient {
    // Red -> orange: for 0-30% completion
    // Deep red -> burnt orange
    Gradient::from_hsv_stops(
        &[
            (0.0, 0.0, 1.0, 0.25),  // deep red
            (0.5, 15.0, 1.0, 0.55), // red-orange
            (1.0, 30.0, 0.9, 0.75), // burnt orange
        ],
        256,
    )
}

pub fn amber_gradient() -> Gradient {
    // Amber -> gold: for 30-70% completion
    // Amber -> warm yellow
    Gradient::from_hsv_stops(
        &[
            (0.0, 38.0, 0.8, 0.55),  // amber
            (0.5, 45.0, 0.75, 0.75), // gold
            (1.0, 50.0, 0.6, 0.85),  // warm yellow
        ],
        256,
    )
}

pub fn sage_gradient() -> Gradient {
    // Yellow-green -> sage -> bright green: for 70-100% completion
    Gradient::from_hsv_stops(
        &[
            (0.0, 80.0, 0.4, 0.65),   // yellow-green
            (0.5, 120.0, 0.45, 0.75), // sage
            (1.0, 130.0, 0.55, 0.85), // bright green
        ],
        256,
    )
}
