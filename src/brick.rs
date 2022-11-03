use raylib::prelude::*;


pub struct Brick {
    pub position: Vector2,
    pub size: Vector2,
    pub is_falling: bool,
    pub is_dead: bool,
    pub hue: f32,
    pub life: i32,
    pub color: Color,
}


impl Brick {
    pub fn draw(&self, mut canvas: RaylibDrawHandle) {
        canvas.draw_rectangle_v(self.position, self.size, self.color);
    }
    
    pub fn update(&mut self, delta: f32) {
        // i dont know what to do for now
    }
}