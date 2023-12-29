#[derive(Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub const WHITE: Color = Color {r: 1.0, g: 1.0, b: 1.0};

    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }
}
impl Into<[f32; 3]> for Color {
    fn into(self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }
}
