use std::{
    any::{Any, TypeId},
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    rc::Rc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use app::App;
use ecs::world::*;
use plugins::{core::{window_plugin::WindowPlugin, render_plugin::RenderPlugin}, triangle_plugin::TrianglePlugin};

mod plugins;
mod app;
mod ecs;
mod math;

#[derive(Debug)]
struct Foo {
    foo: u8,
}
impl Component for Foo {}

#[derive(Debug)]
struct Bar {
    bar: u8,
}
impl Component for Bar {}

fn system(world: &mut World) {
    for bar in query!(world, Bar) {
        println!("{}", bar.bar);
    }
}

fn main() {
    env_logger::init();

    let mut app = App::new();

    // app.world.insert_entity(Bar { bar: 1 });
    // app.world.insert_entity(Bar { bar: 2 });
    // app.world.insert_entity(Bar { bar: 3 });
    // app.schedular.add_system(0, system);


    // Core plugins
    app.register_plugin::<WindowPlugin>();
    app.register_plugin::<RenderPlugin>();


    // Rendering plugins
    app.register_plugin::<TrianglePlugin>();
    


    app.run();
}
