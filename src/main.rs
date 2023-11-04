use std::{
    any::{Any, TypeId},
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    rc::Rc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use hashbrown::HashMap;

use app::App;
use ecs::world::{self, *};
use plugins::{
    core::{
        camera_plugin::CameraPlugin, render_plugin::RenderPlugin, timer_plugin::TimerPlugin,
        window_plugin::WindowPlugin, input_plugin::InputPlugin,
    },
    renderer_plugins::tilemap_renderer::TileMapRenderer,
    triangle_plugin::TrianglePlugin,
};
use winit::event::MouseButton;

mod app;
mod ecs;
mod math;
mod plugins;

#[derive(Debug)]
struct Foo {
    foo: u8,
}

#[derive(Debug)]
struct Bar {
    bar: u8,
}

fn system(world: &mut World) {
    for bar in query!(world, Bar) {
        println!("{}", bar.bar);
    }
}

fn main() {
    env_logger::init();

    let mut app = App::new();

    // Core plugins
    app.register_plugin::<InputPlugin>();
    app.register_plugin::<WindowPlugin>();
    app.register_plugin::<RenderPlugin>();
    app.register_plugin::<CameraPlugin>();
    app.register_plugin::<TimerPlugin>();

    // Rendering plugins
    app.register_plugin::<TileMapRenderer>();

    app.run();
}
