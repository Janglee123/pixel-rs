use ggez::conf::*;
use ggez::event::{self, EventHandler};
use ggez::glam::Vec2;
use ggez::graphics::{self, Color, DrawParam, Drawable};
use ggez::{Context, ContextBuilder, GameResult};


struct Foo {
    value: i32
}

fn main() {
    
    let window_setup = WindowSetup {
        title: "DvD".to_owned(),
        samples: NumSamples::Four,
        vsync: false,
        srgb: true,
        icon: "".to_owned(),
    };

    let window_mode = WindowMode{
        resizable: true,
        ..Default::default()
    };

    let (mut ctx, event_loop) = ContextBuilder::new("my_game", "Cool Game Author")
        .window_setup(window_setup)
        .window_mode(window_mode)
        .backend(Backend::Vulkan)
        .build()
        .expect("Cant make context");
    
    let my_game = MyGame::new(&mut ctx);

    event::run(ctx, event_loop, my_game);
}


struct MyGame {
    pub position: Vec2,
    pub velocity: Vec2,
    circle: graphics::Mesh,
}

impl MyGame {
    pub fn new(_ctx: &mut Context) -> MyGame {
        return  MyGame {
            position: Vec2::ZERO,
            velocity: Vec2::new(100.0, 100.0),
            circle: graphics::Mesh::new_circle(
                _ctx, 
                graphics::DrawMode::fill(), 
                Vec2::ZERO,
                50.0,
                0.1,
                Color::RED
            ).unwrap()
        };
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        
        self.position += self.velocity * _ctx.time.delta().as_secs_f32();
        return  Ok(());
    }

    fn draw(&mut self, _ctx: &mut Context) -> GameResult {
        
        let mut canvas = graphics::Canvas::from_frame(_ctx, Color::WHITE);
        
        // self.circle.
        canvas.draw(&self.circle, self.position);
        canvas.finish(_ctx).unwrap();
        return Ok(());
    }
}