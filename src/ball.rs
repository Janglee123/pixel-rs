use raylib::prelude::*;

pub struct Ball {
    pub position: Vector2,
    pub speed: Vector2,
    pub color: Color,
    pub radius: f32,
    pub hue: f32,
}

impl Ball {
    pub fn draw(&self, mut canvas: RaylibDrawHandle) {
        canvas.draw_circle_v(self.position, self.radius, self.color);
    }

    pub fn update(&mut self, delta: f32) {
        self.position += self.speed * delta;

        if self.position.x + self.radius > 640.0 && self.speed.x > 0.0
            || self.position.x - self.radius < 0.0 && self.speed.x < 0.0
        {
            self.speed.x *= -1.0;
        }

        if self.position.y + self.radius > 480.0 && self.speed.y > 0.0
            || self.position.y - self.radius < 0.0 && self.speed.y < 0.0
        {
            self.speed.y *= -1.0;
        }

        self.hue += delta * 30.0;
        self.color = Color::color_from_hsv(self.hue, 0.8, 0.8);
    }
}
