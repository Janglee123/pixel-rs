use winit::dpi::PhysicalSize;

use crate::{
    ecs::world::{Schedular, World},
    plugins::core::render_plugin::Renderer,
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
    pub world: World,
    pub schedular: Schedular<SystemStage, World>,
    pub renderers: Vec<Box<dyn Renderer>>,
    pub runner: fn(App),
    pub render_function: fn(&mut World, &Vec<Box<dyn Renderer>>),
}

impl App {
    pub fn new() -> Self {
        Self {
            world: World::new(),
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
        self.schedular.run(SystemStage::PreUpdate, &mut self.world);
        self.schedular.run(SystemStage::Update, &mut self.world);

        self.schedular.run(SystemStage::PreRender, &mut self.world);
        let fun = self.render_function;
        let world = &mut self.world;

        (fun)(world, &self.renderers);
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

    pub fn on_curser_moved(&mut self) {
        // Nothing to do??
    }

    pub fn register_plugin<T: Plugin>(&mut self) {
        T::build(self);
    }

    pub fn set_runner(&mut self, fun: fn(App)) {
        self.runner = fun;
    }

    pub fn set_renderer(&mut self, fun: fn(&mut World, &Vec<Box<dyn Renderer>>)) {
        self.render_function = fun;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
