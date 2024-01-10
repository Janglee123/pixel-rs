use super::vector2::{self, Vector2};

#[derive(Debug, Default)]
pub struct Rect {
    pub size: Vector2<f32>,
    pub position: Vector2<f32>,
}

impl Rect {
    pub fn new(size: Vector2<f32>, position: Vector2<f32>) -> Self {
        Self { size, position }
    }

    pub fn from_numbers(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            size: Vector2::new(w, h),
            position: Vector2::new(x, y),
        }
    }

    pub fn from_center(size: Vector2<f32>, center: Vector2<f32>) -> Self {
        Self {
            size,
            position: center - size * 0.5,
        }
    }

    pub fn get_center(&self) -> Vector2<f32> {
        self.position + self.size * 0.5
    }

    pub fn get_end(&self) -> Vector2<f32> {
        self.position + self.size
    }

    pub fn map_uniform_position(&self, input: Vector2<f32>) -> Vector2<f32> {
        self.position + self.size * input
    }
}
