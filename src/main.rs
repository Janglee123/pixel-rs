#![allow(warnings)]

use std::{
    any::{Any, TypeId},
    borrow::BorrowMut,
    cell::{Cell, RefCell},
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    rc::Rc,
    time::{Instant, SystemTime, UNIX_EPOCH}, hint::black_box,
};

use game::GamePlugin;
use hashbrown::HashMap;

use app::App;
use ecs::{world::{self, *}, entity::EntityId};
use plugins::{
    core::{camera_plugin::CameraPlugin, CorePlugins},
    other::tweener::TweenerPlugin,
    renderer_plugins::Renderer2dPlugin,
};
use winit::event::MouseButton;
use zerocopy::{AsBytes, FromBytes};

use crate::{plugins::core::asset_storage::AssetStorage, ecs::component::Component};

mod app;
mod ecs;
mod game;
mod math;
mod plugins;
mod storage;

use std::path::Iter;

#[derive(Debug)]
struct Foo {
    foo: u8,
}
impl Component for Foo{}

#[derive(Debug, Clone, Copy)]
struct Bar {
    bar: u8,
}
impl Component for Bar{}

// fn system(storage: &mut Storage) {
//     for bar in query_mut!(world, Bar) {
//         println!("{}", bar.bar);
//     }
// }

fn main() {
    // env_logger::builder()
    // // setting this to None disables the timestamp
    // .format_timestamp(Some(env_logger::TimestampPrecision::Seconds))
    // .init();


    // let mut app = App::new();

    // // Core plugins
    // app.register_plugin::<CorePlugins>();

    // app.register_plugin::<CameraPlugin>();
    // app.register_plugin::<TweenerPlugin>();

    // app.register_plugin::<Renderer2dPlugin>();

    // // Game related plugins are added into [game/mod.rs]
    // app.register_plugin::<GamePlugin>();

    // app.run();

    let n = 100_000u32;

    let mut world = World::new();

    world.register_component::<Foo>();


    let now = Instant::now();
    for i in 0..n {
        world.insert_entity((Foo{foo: black_box(0)},));
    }
    println!("Map {}", now.elapsed().as_micros());

    let mut world = World::new();
    world.register_component::<Foo>();

    let now = Instant::now();
    for i in 0..n {
        world.insert_entity_two((Foo{foo: black_box(0)},));
    }
    println!("Dyn {}", now.elapsed().as_micros());
}
