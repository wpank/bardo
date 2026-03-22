use std::f64::consts::PI;

#[derive(Clone, Copy, Debug, Default)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalized(&self) -> Self {
        let len = self.length();
        if len < 1e-10 {
            return Self::default();
        }
        Self {
            x: self.x / len,
            y: self.y / len,
        }
    }

    pub fn from_polar(angle: f64, magnitude: f64) -> Self {
        Self {
            x: angle.cos() * magnitude,
            y: angle.sin() * magnitude,
        }
    }
}

impl std::ops::Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Mul<f64> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

pub fn remap(value: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
    let t = (value - in_min) / (in_max - in_min);
    lerp(out_min, out_max, t.clamp(0.0, 1.0))
}

// Easing functions

pub fn ease_out_cubic(t: f64) -> f64 {
    1.0 - (1.0 - t).powi(3)
}

pub fn ease_out_quad(t: f64) -> f64 {
    1.0 - (1.0 - t) * (1.0 - t)
}

pub fn ease_in_out_quad(t: f64) -> f64 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

pub fn ease_in_cubic(t: f64) -> f64 {
    t * t * t
}

pub fn ease_out_elastic(t: f64) -> f64 {
    if t <= 0.0 || t >= 1.0 {
        return t;
    }
    let c4 = (2.0 * PI) / 3.0;
    (2.0_f64).powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
}

// Wave combinators

pub fn wave_combine(t: f64, freqs: &[(f64, f64, f64)]) -> f64 {
    let mut sum = 0.0;
    let mut weight = 0.0;
    for &(freq, phase, w) in freqs {
        sum += (t * freq + phase).sin() * w;
        weight += w;
    }
    if weight > 0.0 {
        sum / weight
    } else {
        0.0
    }
}

pub fn sin01(t: f64) -> f64 {
    t.sin() * 0.5 + 0.5
}

pub fn triangle(t: f64) -> f64 {
    let t = t.rem_euclid(1.0);
    if t < 0.5 {
        t * 2.0
    } else {
        2.0 - t * 2.0
    }
}
