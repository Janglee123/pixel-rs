use std::f32::consts::E;

use glam::Vec2;
use hashbrown::HashMap;
use winit::{
    event::{ElementState, KeyEvent, MouseButton},
    keyboard::KeyCode,
    keyboard::PhysicalKey::Code,
};

use crate::{app::Plugin, ecs::world::World};

pub enum ButtonState {
    JustPressed, // Will implement
    Pressed,
    Released,
}

#[derive(Default, Debug)]
pub struct MouseMotion {
    pub delta: Vec2,
}

#[derive(Debug)]
pub struct MouseButtonInput {
    pub button: MouseButton,
    pub state: ElementState,
}

#[derive(Default, Debug)]
pub struct Input {
    mouse_input: HashMap<MouseButton, MouseButtonInput>,
    keyboard_input: HashMap<KeyCode, KeyEvent>,
    mouse_motion: MouseMotion,
    mouse_position: Vec2,
}

impl Input {
    pub fn on_mouse_input(&mut self, input: MouseButtonInput) {
        self.mouse_input.insert(input.button, input);
    }

    pub fn on_curser_moved(&mut self, cursor_pos: Vec2) {
        self.mouse_position = cursor_pos;
    }

    pub fn on_keyboard_input(&mut self, input: KeyEvent) {
        if let Code(keycode) = input.physical_key {
            self.keyboard_input.insert(keycode, input);
        }
    }

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        let a = self.mouse_input.get(&button);

        let result = if let Some(input) = a {
            input.state == ElementState::Pressed
        } else {
            false
        };

        result
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        let a = self.keyboard_input.get(&key);

        let result = if let Some(input) = a {
            input.state == ElementState::Pressed
        } else {
            false
        };

        result
    }

    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(app: &mut crate::app::App) {
        let input = Input::default();
        app.world.singletons.insert(input);

        app.schedular
            .add_system(crate::app::SystemStage::Input, print_input_keys);
    }
}

pub fn print_input_keys(world: &mut World) {
    let input = world.singletons.get_mut::<Input>().unwrap();

    println!(
        "input: {:?}",
        input.is_mouse_button_pressed(MouseButton::Left)
    );
}
