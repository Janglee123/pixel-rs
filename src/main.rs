#![allow(warnings)]

use std::{
    any::{Any, TypeId},
    borrow::BorrowMut,
    cell::{Cell, RefCell},
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    hint::black_box,
    rc::Rc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use game::GamePlugin;
use hashbrown::HashMap;

use app::App;
use ecs::{
    entity::EntityId,
    world::{self, *},
};
use plugins::{
    core::{camera_plugin::CameraPlugin, CorePlugins},
    other::tweener::TweenerPlugin,
    renderer_plugins::Renderer2dPlugin,
};
use winit::event::MouseButton;
use zerocopy::{AsBytes, FromBytes};

use crate::{ecs::component::Component, plugins::core::asset_storage::AssetStorage};

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
impl Component for Foo {}

#[derive(Debug, Clone, Copy)]
struct Bar {
    bar: u8,
}
impl Component for Bar {}

fn main() {
    env_logger::builder()
        // setting this to None disables the timestamp
        .format_timestamp(Some(env_logger::TimestampPrecision::Seconds))
        .init();

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
