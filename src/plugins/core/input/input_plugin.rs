use std::f32::consts::E;

use glam::Vec2;
use hashbrown::HashMap;
use winit::keyboard::PhysicalKey::Code;

use crate::{app::Plugin, ecs::world::World};

use super::keycode::KeyCode;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]

pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]

pub enum ButtonState {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy)]
pub struct MouseButtonEvent {
    pub button: MouseButton,
    pub state: ButtonState,
}

#[derive(Debug)]
pub struct MouseMotionEvent {
    pub delta: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    pub button: KeyCode,
    pub state: ButtonState,
}

#[derive(Debug)]
pub enum MouseEvent {
    MouseButtonEvent(MouseButtonEvent),
    MouseMotionEvent,
    MouseWheelEvent,
}

#[derive(Debug)]
pub enum KeyboardEvent {
    KeyEvent(KeyEvent),
}

#[derive(Debug)]
pub enum InputEvent {
    MouseEvent(MouseEvent),
    KeyboardEvent(KeyboardEvent),
}

#[derive(Default, Debug)]
pub struct Input {
    mouse_input: HashMap<MouseButton, MouseButtonEvent>,
    keyboard_input: HashMap<KeyCode, KeyEvent>,
    mouse_position: Vec2,
    pub last_event: Option<InputEvent>,
}

impl Input {
    pub fn on_mouse_input(&mut self, input: MouseButtonEvent) {
        self.last_event = Some(InputEvent::MouseEvent(MouseEvent::MouseButtonEvent(input)));

        self.mouse_input.insert(input.button, input);
    }

    pub fn on_curser_moved(&mut self, cursor_pos: Vec2) {
        self.last_event = Some(InputEvent::MouseEvent(MouseEvent::MouseMotionEvent));

        self.mouse_position = cursor_pos;
    }

    pub fn on_keyboard_input(&mut self, input: KeyEvent) {
        self.last_event = Some(InputEvent::KeyboardEvent(KeyboardEvent::KeyEvent(input)));
        self.keyboard_input.insert(input.button, input);
    }

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        let a = self.mouse_input.get(&button);

        let result = if let Some(input) = a {
            input.state == ButtonState::Pressed
        } else {
            false
        };

        result
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        let a = self.keyboard_input.get(&key);

        let result = if let Some(input) = a {
            input.state == ButtonState::Pressed
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
    }
}
