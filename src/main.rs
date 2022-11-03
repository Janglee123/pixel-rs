mod ball;
mod brick;

use brick::*;
use ball::*;
use raylib::prelude::*;

fn main() {
    let (mut rl, thread) = raylib::init().size(640, 480).title("DVD").build();

    let mut ball = Ball {
        position: Vector2 { x: 320.0, y: 240.0 },
        speed: Vector2 { x: 100.0, y: 100.0 },
        color: Color::BLUE,
        radius: 50.0,
        hue: 0.0,
    };

    while !rl.window_should_close() {
        let mut canvas = rl.begin_drawing(&thread);

        canvas.clear_background(Color::WHITE);

        ball.update(canvas.get_frame_time());
        ball.draw(canvas);
    }
}
