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
    core::{
        asset_storage::{self, Asset},
        // asset_storage::{self, Asset, AssetStorage},
        camera_plugin::CameraPlugin,
        input_plugin::InputPlugin,
        render_plugin::RenderPlugin,
        timer_plugin::TimerPlugin,
        window_plugin::WindowPlugin,
    },
    other::tweener::TweenerPlugin,
    renderer_plugins::{
        multi_instance_mesh_renderer::MultiInstanceMeshRenderer, sprite_renderer::SpritePlugin,
        tilemap_renderer::TileMapRenderer,
    },
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

impl Asset for Bar {
    fn from_binary(binary: Vec<u8>) -> Self {
        todo!()
    }
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
    app.register_plugin::<InputPlugin>();
    app.register_plugin::<WindowPlugin>();
    app.register_plugin::<RenderPlugin>();
    app.register_plugin::<CameraPlugin>();
    app.register_plugin::<TimerPlugin>();
    app.register_plugin::<TweenerPlugin>();

    // Rendering plugins
    app.register_plugin::<TileMapRenderer>();
    app.register_plugin::<SpritePlugin>();
    app.register_plugin::<MultiInstanceMeshRenderer>();

    // Game
    // Game related plugins are added into [game/mod.rs]
    app.register_plugin::<GamePlugin>();

    app.run();
}
