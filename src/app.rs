use std::{any::Any, any::TypeId, collections::HashMap};

use winit::{
    dpi::PhysicalSize,
    event::{Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, Window, WindowBuilder},
};

use crate::{
    ecs::world::{self, Schedular, World},
    plugins::core::input_plugin::MouseButtonInput,
};

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum SystemStage {
    Update,
    Resize,
    Start,
    PreInput,
    Input,
    PreUpdate,
}

pub trait Plugin {
    fn build(app: &mut App);
}

pub struct App {
    pub world: World,
    pub schedular: Schedular<SystemStage>,
    pub runner: fn(App),
}

impl App {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            schedular: Schedular::new(),
            runner: |app: App| {},
        }
    }

    pub fn run(self) {
        (self.runner)(self);
    }

    pub fn update(&mut self) {
        self.schedular.run(SystemStage::PreUpdate, &mut self.world);
        self.schedular.run(SystemStage::Update, &mut self.world);
    }

    pub fn on_resize(&mut self, physical_size: PhysicalSize<u32>) {
        self.schedular.run(SystemStage::Resize, &mut self.world);
    }

    pub fn on_keyboard_input(&mut self) {
        // self.schedular.run(SystemStage::PreInput, &mut self.world);
        self.schedular.run(SystemStage::Input, &mut self.world);
    }

    pub fn on_mouse_input(&mut self) {
        // self.schedular.run(SystemStage::PreInput, &mut self.world);
        self.schedular.run(SystemStage::Input, &mut self.world);
    }

    pub fn register_plugin<T: Plugin>(&mut self) {
        T::build(self);
    }

    pub fn set_runner(&mut self, fun: fn(App)) {
        self.runner = fun;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
