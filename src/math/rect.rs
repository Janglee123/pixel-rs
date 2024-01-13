use glam::Vec2;

#[derive(Debug, Default)]
pub struct Rect {
    pub size: Vec2,
    pub position: Vec2,
}

impl Rect {
    pub fn new(size: Vec2, position: Vec2) -> Self {
        Self { size, position }
    }

    pub fn from_numbers(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            size: Vec2::new(w, h),
            position: Vec2::new(x, y),
        }
    }

    pub fn from_center(size: Vec2, center: Vec2) -> Self {
        Self {
            size,
            position: center - size * 0.5,
        }
    }

    pub fn get_center(&self) -> Vec2 {
        self.position + self.size * 0.5
    }

    pub fn get_end(&self) -> Vec2 {
        self.position + self.size
    }

    pub fn map_uniform_position(&self, input: Vec2) -> Vec2 {
        self.position + self.size * input
    }
}
