use winit::dpi::PhysicalSize;

use crate::{
    ecs::{world::{Schedular, World}, singletons::Singletons, event_bus::EventBus},
    plugins::core::render_plugin::Renderer, storage::Storage,
};

// Order is
// PreUpdate -> Update -> PreRender -> Render
// PreInput -> Input. Input is called by winit. Not sure in which order but most likely before PreUpdate
// Resize is called by winit
// Start is called only once on startup

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum SystemStage {
    Start,

    Resize,

    PreInput,
    Input,

    Update,
    PreUpdate,

    PreRender,
    Render,
}

pub trait Plugin {
    fn build(app: &mut App);
}

pub struct App {
    pub storage: Storage,
    pub schedular: Schedular<SystemStage, Storage>,
    pub renderers: Vec<Box<dyn Renderer>>,
    pub runner: fn(App),
    pub render_function: fn(&mut Storage, &Vec<Box<dyn Renderer>>), // SO I dont know why I had this anymore lol
}

impl App {
    pub fn new() -> Self {
        Self {
            storage: Storage::new(),
            schedular: Schedular::new(),
            runner: |_: App| {},
            render_function: |_, _| {},
            renderers: Vec::new(),
        }
    }

    pub fn run(self) {
        (self.runner)(self);
    }

    pub fn update(&mut self) {
        self.schedular.run(SystemStage::PreUpdate, &mut self.storage);
        self.schedular.run(SystemStage::Update, &mut self.storage);

        self.schedular.run(SystemStage::PreRender, &mut self.storage);
        let fun = self.render_function;
        let world = &mut self.storage;

        (fun)(world, &self.renderers);
    }

    pub fn on_resize(&mut self, physical_size: PhysicalSize<u32>) {
        self.schedular.run(SystemStage::Resize, &mut self.storage);
    }

    pub fn on_keyboard_input(&mut self) {
        // self.schedular.run(SystemStage::PreInput, &mut self.world);
        self.schedular.run(SystemStage::Input, &mut self.storage);
    }

    pub fn on_mouse_input(&mut self) {
        // self.schedular.run(SystemStage::PreInput, &mut self.world);
        self.schedular.run(SystemStage::Input, &mut self.storage);
    }

    pub fn on_curser_moved(&mut self) {
        // Nothing to do??
    }

    pub fn register_plugin<T: Plugin>(&mut self) {
        T::build(self);
    }

    pub fn set_runner(&mut self, fun: fn(App)) {
        self.runner = fun;
    }

    pub fn set_renderer(&mut self, fun: fn(&mut Storage, &Vec<Box<dyn Renderer>>)) {
        self.render_function = fun;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
