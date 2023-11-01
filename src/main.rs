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
    core::{render_plugin::RenderPlugin, window_plugin::WindowPlugin, camera_plugin::CameraPlugin},
    triangle_plugin::TrianglePlugin, renderer_plugins::tilemap_renderer::TileMapRenderer,
};

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

    // app.world.insert_entity(Bar { bar: 1 });
    // app.world.insert_entity(Bar { bar: 2 });
    // app.world.insert_entity(Bar { bar: 3 });
    // app.schedular.add_system(0, system);

    // Core plugins
    app.register_plugin::<WindowPlugin>();
    app.register_plugin::<RenderPlugin>();
    app.register_plugin::<CameraPlugin>();

    // Rendering plugins
    app.register_plugin::<TileMapRenderer>();

    // let mut target_bitset = BitSet::new();

    // target_bitset.insert_id(app.world.component_id_map.get(&TypeId::of::<Foo>()).unwrap().clone());
    // target_bitset.insert_id(app.world.component_id_map.get(&TypeId::of::<Bar>()).unwrap().clone());

    // let mut archetype_map = app.world.archetype_id_map;

    // let a = archetype_map.iter_mut().filter(|(bitset, archetype)| {
    //     bitset.contains(&target_bitset)
    // }).map(|(bitset, archetype)| {
    //     let a = archetype.get_mut::<Bar>();
    //     let b = archetype.get_mut::<Foo>();

    //     a.iter().zip(b.iter())
    // }).flatten().map(|x| {});

    app.run();
}