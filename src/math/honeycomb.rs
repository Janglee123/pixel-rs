// well well just make a struct

use std::ops::{Add, Mul};

static SQRT_THREE: f32 = 1.7320508075688772;

static SQRT_THREE_HALF: f32 = 0.8660254037844386;

static DIRECTION_VECTORS: [Hexter; 6] = [
    Hexter::new(1, 0),
    Hexter::new(1, -1),
    Hexter::new(0, -1),
    Hexter::new(-1, 0),
    Hexter::new(-1, 1),
    Hexter::new(0, 1),
];

pub static HEXAGON: [[f32; 2]; 6] = [
    [0.0, 1.0],
    [0.866025404, 0.5],
    [0.866025404, -0.5],
    [0.0, -1.0],
    [-0.866025404, -0.5],
    [-0.866025404, 0.5],
];

pub static HEXAGON_INDICES: &[u16] = &[0, 1, 2, 2, 3, 4, 4, 5, 0, 0, 2, 4];

#[derive(Debug, Clone, Copy, Default)]
pub struct Hexter {
    pub q: i32,
    pub r: i32,
}

impl Hexter {
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    pub fn from_vector(x: f32, y: f32, tile_size: f32) -> Self {
        let float_q = (SQRT_THREE * x - y) / (3.0 * tile_size);
        let float_r = (2.0 * y) / (tile_size * 3.0);
        let float_s = -float_q - float_r;

        let mut q = float_q.round();
        let mut r = float_r.round();
        let mut s = float_s.round();

        let q_frac = float_q - q;
        let r_frac = float_r - r;
        let s_frac = float_s - s;

        if q_frac > r_frac && q_frac > s_frac {
            q = -r - s;
        } else if r_frac > s_frac {
            r = -q - s;
        }

        Self {
            q: q as i32,
            r: r as i32,
        }
    }

    // Radius means distance between center to vertex
    pub fn to_vector(&self, radius: f32) -> [f32; 2] {
        
        let x = SQRT_THREE * radius * (self.q as f32 + self.r as f32 * 0.5);
        let y = 1.5 * radius * (self.r as f32);

        [x, y]
    }

    pub fn rotate(&self, clock_wise: bool) -> Self {
        if clock_wise {
            Self {
                q: -self.r,
                r: self.q + self.r,
            }
        } else {
            Self {
                q: self.q + self.r,
                r: -self.q,
            }
        }
    }
}

impl Add for Hexter {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
        }
    }
}

impl Mul<i32> for Hexter {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self {
            q: self.q * rhs,
            r: self.r * rhs,
        }
    }
}

impl Mul<u32> for Hexter {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        Self {
            q: self.q * rhs as i32,
            r: self.r * rhs as i32,
        }
    }
}

pub struct SpiralLoop {
    pub center_pos: Hexter,
    pub range: u32,

    current_radius: u32,
    current_direction: u32,
    current_point: u32,
}

impl SpiralLoop {
    pub fn new(center_pos: Hexter, range: u32) -> Self {
        Self {
            center_pos,
            range,
            current_radius: 0,
            current_direction: 0,
            current_point: 0,
        }
    }
}

impl Iterator for SpiralLoop {
    type Item = Hexter;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_radius == 0 && self.current_direction == 0 {
            self.current_radius += 1;
            return Some(self.center_pos);
        }

        if self.current_radius > self.range {
            return None;
        }

        let point = DIRECTION_VECTORS[self.current_direction as usize] * self.current_radius;
        let dir = DIRECTION_VECTORS[((self.current_direction + 2) % 6) as usize];

        let result = self.center_pos + point + dir * self.current_point;

        self.current_point += 1;

        if self.current_point == self.current_radius {
            self.current_point = 0;
            self.current_direction += 1;
        }

        if self.current_direction == 6 {
            self.current_direction = 0;
            self.current_radius += 1;
        }

        // Now return here
        Some(result)
    }
}
