#![allow(warnings)]

use std::{
    any::{Any, TypeId},
    borrow::BorrowMut,
    cell::{Cell, RefCell},
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    rc::Rc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use game::GamePlugin;
use hashbrown::HashMap;

use app::App;
use ecs::world::{self, *};
use plugins::{
    core::{camera_plugin::CameraPlugin, CorePlugins},
    other::tweener::TweenerPlugin,
    renderer_plugins::Renderer2dPlugin,
};
use winit::event::MouseButton;

use crate::plugins::core::asset_storage::AssetStorage;

mod app;
mod ecs;
mod game;
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
    for bar in query_mut!(world, Bar) {
        println!("{}", bar.bar);
    }
}

fn main() {
    env_logger::init();

    let mut app = App::new();

    // Core plugins
    app.register_plugin::<CorePlugins>();

    app.register_plugin::<CameraPlugin>();
    app.register_plugin::<TweenerPlugin>();

    app.register_plugin::<Renderer2dPlugin>();

    // Game related plugins are added into [game/mod.rs]
    app.register_plugin::<GamePlugin>();

    app.run();
}
