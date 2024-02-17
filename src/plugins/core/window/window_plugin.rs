use glam::Vec2;
use log::info;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::{
    app::{App, Plugin},
    plugins::core::input::{
        self,
        input_plugin::{self, ButtonState, Input, MouseButtonEvent},
    },
};

pub struct WindowPlugin;

fn runner(mut app: App) {
    let event_loop = app.storage.singletons.remove::<EventLoop<()>>().unwrap();
    let w_id = app.storage.singletons.get::<Window>().unwrap().id();

    event_loop.run(move |event, window_target| {
        // *r_control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { window_id, event } if window_id == w_id => match event {
                WindowEvent::CloseRequested => {
                    info!("bye bye");
                    window_target.exit();
                }

                WindowEvent::Resized(physical_size) => app.on_resize(physical_size),
                WindowEvent::KeyboardInput {
                    device_id: _,
                    is_synthetic,
                    event,
                } => on_keyboard_input(&mut app, event),
                WindowEvent::MouseInput {
                    device_id,
                    state,
                    button,
                    ..
                } => {
                    // let input = MouseButtonInput { button, state };
                    on_mouse_input(&mut app, button, state);
                }
                WindowEvent::CursorMoved {
                    device_id,
                    position,
                    ..
                } => {
                    on_curser_moved(&mut app, position.cast::<f32>());
                }
                WindowEvent::MouseWheel {
                    device_id,
                    delta,
                    phase,
                } => on_mouse_wheen_input(&mut app, delta),
                _ => (),
            },

            Event::AboutToWait => {
                app.update();
                let window = app.storage.singletons.get::<Window>().unwrap();
                window.request_redraw();
            }
            _ => (),
        }
    });
}

fn on_curser_moved(app: &mut App, cursor_pos: PhysicalPosition<f32>) {
    let (input, window) = app
        .storage
        .singletons
        .get_many_mut::<(Input, Window)>()
        .unwrap();

    let window_size = window.inner_size().cast::<f32>();

    let x_pos = (cursor_pos.x / window_size.width - 0.5) * 2.0;
    let y_pos = (0.5 - cursor_pos.y / window_size.height) * 2.0;

    let raw_screen_pos = cursor_pos;
    let screen_pos = Vec2::new(x_pos, y_pos);

    input.on_curser_moved(screen_pos);
}

fn on_mouse_wheen_input(app: &mut App, mouse_wheel_input: MouseScrollDelta) {
    let input = app.storage.singletons.get_mut::<Input>().unwrap();

    // Only If I had Enums for inputs to do it

    app.on_mouse_input();
}

fn on_mouse_input(app: &mut App, button: MouseButton, state: ElementState) {
    // Why accessing input here??

    let input = app.storage.singletons.get_mut::<Input>().unwrap();

    let mouse_button_event = MouseButtonEvent {
        button: convert_winit_mouse_button(button),
        state: convert_winit_state(state),
    };

    input.on_mouse_input(mouse_button_event);
    app.on_mouse_input();
}

fn on_keyboard_input(app: &mut App, key_input: KeyEvent) {
    let input = app.storage.singletons.get_mut::<Input>().unwrap();

    if let winit::keyboard::PhysicalKey::Code(key_code) = key_input.physical_key {
        let key_event = input::input_plugin::KeyEvent {
            button: convert_winit_key_code(key_code),
            state: convert_winit_state(key_input.state),
        };

        input.on_keyboard_input(key_event);
        app.on_keyboard_input();
    }
}

impl Plugin for WindowPlugin {
    fn build(app: &mut crate::app::App) {
        let event_loop = EventLoop::new().unwrap();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        app.storage.singletons.insert(window);
        app.storage.singletons.insert(event_loop);

        app.set_runner(runner);
    }
}

fn convert_winit_state(state: ElementState) -> ButtonState {
    match state {
        ElementState::Pressed => ButtonState::Pressed,
        ElementState::Released => ButtonState::Released,
    }
}

fn convert_winit_mouse_button(button: MouseButton) -> input::input_plugin::MouseButton {
    match button {
        MouseButton::Left => input::input_plugin::MouseButton::Left,
        MouseButton::Right => input::input_plugin::MouseButton::Right,
        MouseButton::Middle => input::input_plugin::MouseButton::Middle,
        MouseButton::Back => input::input_plugin::MouseButton::Back,
        MouseButton::Forward => input::input_plugin::MouseButton::Forward,
        MouseButton::Other(id) => input::input_plugin::MouseButton::Other(id),
    }
}

fn convert_winit_key_code(key_code: winit::keyboard::KeyCode) -> input::keycode::KeyCode {
    match key_code {
        winit::keyboard::KeyCode::Backquote => input::keycode::KeyCode::Backquote,
        winit::keyboard::KeyCode::Backslash => input::keycode::KeyCode::Backslash,
        winit::keyboard::KeyCode::BracketLeft => input::keycode::KeyCode::BracketLeft,
        winit::keyboard::KeyCode::BracketRight => input::keycode::KeyCode::BracketRight,
        winit::keyboard::KeyCode::Comma => input::keycode::KeyCode::Comma,
        winit::keyboard::KeyCode::Digit0 => input::keycode::KeyCode::Digit0,
        winit::keyboard::KeyCode::Digit1 => input::keycode::KeyCode::Digit1,
        winit::keyboard::KeyCode::Digit2 => input::keycode::KeyCode::Digit2,
        winit::keyboard::KeyCode::Digit3 => input::keycode::KeyCode::Digit3,
        winit::keyboard::KeyCode::Digit4 => input::keycode::KeyCode::Digit4,
        winit::keyboard::KeyCode::Digit5 => input::keycode::KeyCode::Digit5,
        winit::keyboard::KeyCode::Digit6 => input::keycode::KeyCode::Digit6,
        winit::keyboard::KeyCode::Digit7 => input::keycode::KeyCode::Digit7,
        winit::keyboard::KeyCode::Digit8 => input::keycode::KeyCode::Digit8,
        winit::keyboard::KeyCode::Digit9 => input::keycode::KeyCode::Digit9,
        winit::keyboard::KeyCode::Equal => input::keycode::KeyCode::Equal,
        winit::keyboard::KeyCode::IntlBackslash => input::keycode::KeyCode::IntlBackslash,
        winit::keyboard::KeyCode::IntlRo => input::keycode::KeyCode::IntlRo,
        winit::keyboard::KeyCode::IntlYen => input::keycode::KeyCode::IntlYen,
        winit::keyboard::KeyCode::KeyA => input::keycode::KeyCode::KeyA,
        winit::keyboard::KeyCode::KeyB => input::keycode::KeyCode::KeyB,
        winit::keyboard::KeyCode::KeyC => input::keycode::KeyCode::KeyC,
        winit::keyboard::KeyCode::KeyD => input::keycode::KeyCode::KeyD,
        winit::keyboard::KeyCode::KeyE => input::keycode::KeyCode::KeyE,
        winit::keyboard::KeyCode::KeyF => input::keycode::KeyCode::KeyF,
        winit::keyboard::KeyCode::KeyG => input::keycode::KeyCode::KeyG,
        winit::keyboard::KeyCode::KeyH => input::keycode::KeyCode::KeyH,
        winit::keyboard::KeyCode::KeyI => input::keycode::KeyCode::KeyI,
        winit::keyboard::KeyCode::KeyJ => input::keycode::KeyCode::KeyJ,
        winit::keyboard::KeyCode::KeyK => input::keycode::KeyCode::KeyK,
        winit::keyboard::KeyCode::KeyL => input::keycode::KeyCode::KeyL,
        winit::keyboard::KeyCode::KeyM => input::keycode::KeyCode::KeyM,
        winit::keyboard::KeyCode::KeyN => input::keycode::KeyCode::KeyN,
        winit::keyboard::KeyCode::KeyO => input::keycode::KeyCode::KeyO,
        winit::keyboard::KeyCode::KeyP => input::keycode::KeyCode::KeyP,
        winit::keyboard::KeyCode::KeyQ => input::keycode::KeyCode::KeyQ,
        winit::keyboard::KeyCode::KeyR => input::keycode::KeyCode::KeyR,
        winit::keyboard::KeyCode::KeyS => input::keycode::KeyCode::KeyS,
        winit::keyboard::KeyCode::KeyT => input::keycode::KeyCode::KeyT,
        winit::keyboard::KeyCode::KeyU => input::keycode::KeyCode::KeyU,
        winit::keyboard::KeyCode::KeyV => input::keycode::KeyCode::KeyV,
        winit::keyboard::KeyCode::KeyW => input::keycode::KeyCode::KeyW,
        winit::keyboard::KeyCode::KeyX => input::keycode::KeyCode::KeyX,
        winit::keyboard::KeyCode::KeyY => input::keycode::KeyCode::KeyY,
        winit::keyboard::KeyCode::KeyZ => input::keycode::KeyCode::KeyZ,
        winit::keyboard::KeyCode::Minus => input::keycode::KeyCode::Minus,
        winit::keyboard::KeyCode::Period => input::keycode::KeyCode::Period,
        winit::keyboard::KeyCode::Quote => input::keycode::KeyCode::Quote,
        winit::keyboard::KeyCode::Semicolon => input::keycode::KeyCode::Semicolon,
        winit::keyboard::KeyCode::Slash => input::keycode::KeyCode::Slash,
        winit::keyboard::KeyCode::AltLeft => input::keycode::KeyCode::AltLeft,
        winit::keyboard::KeyCode::AltRight => input::keycode::KeyCode::AltRight,
        winit::keyboard::KeyCode::Backspace => input::keycode::KeyCode::Backspace,
        winit::keyboard::KeyCode::CapsLock => input::keycode::KeyCode::CapsLock,
        winit::keyboard::KeyCode::ContextMenu => input::keycode::KeyCode::ContextMenu,
        winit::keyboard::KeyCode::ControlLeft => input::keycode::KeyCode::ControlLeft,
        winit::keyboard::KeyCode::ControlRight => input::keycode::KeyCode::ControlRight,
        winit::keyboard::KeyCode::Enter => input::keycode::KeyCode::Enter,
        winit::keyboard::KeyCode::SuperLeft => input::keycode::KeyCode::SuperLeft,
        winit::keyboard::KeyCode::SuperRight => input::keycode::KeyCode::SuperRight,
        winit::keyboard::KeyCode::ShiftLeft => input::keycode::KeyCode::ShiftLeft,
        winit::keyboard::KeyCode::ShiftRight => input::keycode::KeyCode::ShiftRight,
        winit::keyboard::KeyCode::Space => input::keycode::KeyCode::Space,
        winit::keyboard::KeyCode::Tab => input::keycode::KeyCode::Tab,
        winit::keyboard::KeyCode::Convert => input::keycode::KeyCode::Convert,
        winit::keyboard::KeyCode::KanaMode => input::keycode::KeyCode::KanaMode,
        winit::keyboard::KeyCode::Lang1 => input::keycode::KeyCode::Lang1,
        winit::keyboard::KeyCode::Lang2 => input::keycode::KeyCode::Lang2,
        winit::keyboard::KeyCode::Lang3 => input::keycode::KeyCode::Lang3,
        winit::keyboard::KeyCode::Lang4 => input::keycode::KeyCode::Lang4,
        winit::keyboard::KeyCode::Lang5 => input::keycode::KeyCode::Lang5,
        winit::keyboard::KeyCode::NonConvert => input::keycode::KeyCode::NonConvert,
        winit::keyboard::KeyCode::Delete => input::keycode::KeyCode::Delete,
        winit::keyboard::KeyCode::End => input::keycode::KeyCode::End,
        winit::keyboard::KeyCode::Help => input::keycode::KeyCode::Help,
        winit::keyboard::KeyCode::Home => input::keycode::KeyCode::Home,
        winit::keyboard::KeyCode::Insert => input::keycode::KeyCode::Insert,
        winit::keyboard::KeyCode::PageDown => input::keycode::KeyCode::PageDown,
        winit::keyboard::KeyCode::PageUp => input::keycode::KeyCode::PageUp,
        winit::keyboard::KeyCode::ArrowDown => input::keycode::KeyCode::ArrowDown,
        winit::keyboard::KeyCode::ArrowLeft => input::keycode::KeyCode::ArrowLeft,
        winit::keyboard::KeyCode::ArrowRight => input::keycode::KeyCode::ArrowRight,
        winit::keyboard::KeyCode::ArrowUp => input::keycode::KeyCode::ArrowUp,
        winit::keyboard::KeyCode::NumLock => input::keycode::KeyCode::NumLock,
        winit::keyboard::KeyCode::Numpad0 => input::keycode::KeyCode::Numpad0,
        winit::keyboard::KeyCode::Numpad1 => input::keycode::KeyCode::Numpad1,
        winit::keyboard::KeyCode::Numpad2 => input::keycode::KeyCode::Numpad2,
        winit::keyboard::KeyCode::Numpad3 => input::keycode::KeyCode::Numpad3,
        winit::keyboard::KeyCode::Numpad4 => input::keycode::KeyCode::Numpad4,
        winit::keyboard::KeyCode::Numpad5 => input::keycode::KeyCode::Numpad5,
        winit::keyboard::KeyCode::Numpad6 => input::keycode::KeyCode::Numpad6,
        winit::keyboard::KeyCode::Numpad7 => input::keycode::KeyCode::Numpad7,
        winit::keyboard::KeyCode::Numpad8 => input::keycode::KeyCode::Numpad8,
        winit::keyboard::KeyCode::Numpad9 => input::keycode::KeyCode::Numpad9,
        winit::keyboard::KeyCode::NumpadAdd => input::keycode::KeyCode::NumpadAdd,
        winit::keyboard::KeyCode::NumpadBackspace => input::keycode::KeyCode::NumpadBackspace,
        winit::keyboard::KeyCode::NumpadClear => input::keycode::KeyCode::NumpadClear,
        winit::keyboard::KeyCode::NumpadClearEntry => input::keycode::KeyCode::NumpadClearEntry,
        winit::keyboard::KeyCode::NumpadComma => input::keycode::KeyCode::NumpadComma,
        winit::keyboard::KeyCode::NumpadDecimal => input::keycode::KeyCode::NumpadDecimal,
        winit::keyboard::KeyCode::NumpadDivide => input::keycode::KeyCode::NumpadDivide,
        winit::keyboard::KeyCode::NumpadEnter => input::keycode::KeyCode::NumpadEnter,
        winit::keyboard::KeyCode::NumpadEqual => input::keycode::KeyCode::NumpadEqual,
        winit::keyboard::KeyCode::NumpadHash => input::keycode::KeyCode::NumpadHash,
        winit::keyboard::KeyCode::NumpadMemoryAdd => input::keycode::KeyCode::NumpadMemoryAdd,
        winit::keyboard::KeyCode::NumpadMemoryClear => input::keycode::KeyCode::NumpadMemoryClear,
        winit::keyboard::KeyCode::NumpadMemoryRecall => input::keycode::KeyCode::NumpadMemoryRecall,
        winit::keyboard::KeyCode::NumpadMemoryStore => input::keycode::KeyCode::NumpadMemoryStore,
        winit::keyboard::KeyCode::NumpadMemorySubtract => {
            input::keycode::KeyCode::NumpadMemorySubtract
        }
        winit::keyboard::KeyCode::NumpadMultiply => input::keycode::KeyCode::NumpadMultiply,
        winit::keyboard::KeyCode::NumpadParenLeft => input::keycode::KeyCode::NumpadParenLeft,
        winit::keyboard::KeyCode::NumpadParenRight => input::keycode::KeyCode::NumpadParenRight,
        winit::keyboard::KeyCode::NumpadStar => input::keycode::KeyCode::NumpadStar,
        winit::keyboard::KeyCode::NumpadSubtract => input::keycode::KeyCode::NumpadSubtract,
        winit::keyboard::KeyCode::Escape => input::keycode::KeyCode::Escape,
        winit::keyboard::KeyCode::Fn => input::keycode::KeyCode::Fn,
        winit::keyboard::KeyCode::FnLock => input::keycode::KeyCode::FnLock,
        winit::keyboard::KeyCode::PrintScreen => input::keycode::KeyCode::PrintScreen,
        winit::keyboard::KeyCode::ScrollLock => input::keycode::KeyCode::ScrollLock,
        winit::keyboard::KeyCode::Pause => input::keycode::KeyCode::Pause,
        winit::keyboard::KeyCode::BrowserBack => input::keycode::KeyCode::BrowserBack,
        winit::keyboard::KeyCode::BrowserFavorites => input::keycode::KeyCode::BrowserFavorites,
        winit::keyboard::KeyCode::BrowserForward => input::keycode::KeyCode::BrowserForward,
        winit::keyboard::KeyCode::BrowserHome => input::keycode::KeyCode::BrowserHome,
        winit::keyboard::KeyCode::BrowserRefresh => input::keycode::KeyCode::BrowserRefresh,
        winit::keyboard::KeyCode::BrowserSearch => input::keycode::KeyCode::BrowserSearch,
        winit::keyboard::KeyCode::BrowserStop => input::keycode::KeyCode::BrowserStop,
        winit::keyboard::KeyCode::Eject => input::keycode::KeyCode::Eject,
        winit::keyboard::KeyCode::LaunchApp1 => input::keycode::KeyCode::LaunchApp1,
        winit::keyboard::KeyCode::LaunchApp2 => input::keycode::KeyCode::LaunchApp2,
        winit::keyboard::KeyCode::LaunchMail => input::keycode::KeyCode::LaunchMail,
        winit::keyboard::KeyCode::MediaPlayPause => input::keycode::KeyCode::MediaPlayPause,
        winit::keyboard::KeyCode::MediaSelect => input::keycode::KeyCode::MediaSelect,
        winit::keyboard::KeyCode::MediaStop => input::keycode::KeyCode::MediaStop,
        winit::keyboard::KeyCode::MediaTrackNext => input::keycode::KeyCode::MediaTrackNext,
        winit::keyboard::KeyCode::MediaTrackPrevious => input::keycode::KeyCode::MediaTrackPrevious,
        winit::keyboard::KeyCode::Power => input::keycode::KeyCode::Power,
        winit::keyboard::KeyCode::Sleep => input::keycode::KeyCode::Sleep,
        winit::keyboard::KeyCode::AudioVolumeDown => input::keycode::KeyCode::AudioVolumeDown,
        winit::keyboard::KeyCode::AudioVolumeMute => input::keycode::KeyCode::AudioVolumeMute,
        winit::keyboard::KeyCode::AudioVolumeUp => input::keycode::KeyCode::AudioVolumeUp,
        winit::keyboard::KeyCode::WakeUp => input::keycode::KeyCode::WakeUp,
        winit::keyboard::KeyCode::Meta => input::keycode::KeyCode::Meta,
        winit::keyboard::KeyCode::Hyper => input::keycode::KeyCode::Hyper,
        winit::keyboard::KeyCode::Turbo => input::keycode::KeyCode::Turbo,
        winit::keyboard::KeyCode::Abort => input::keycode::KeyCode::Abort,
        winit::keyboard::KeyCode::Resume => input::keycode::KeyCode::Resume,
        winit::keyboard::KeyCode::Suspend => input::keycode::KeyCode::Suspend,
        winit::keyboard::KeyCode::Again => input::keycode::KeyCode::Again,
        winit::keyboard::KeyCode::Copy => input::keycode::KeyCode::Copy,
        winit::keyboard::KeyCode::Cut => input::keycode::KeyCode::Cut,
        winit::keyboard::KeyCode::Find => input::keycode::KeyCode::Find,
        winit::keyboard::KeyCode::Open => input::keycode::KeyCode::Open,
        winit::keyboard::KeyCode::Paste => input::keycode::KeyCode::Paste,
        winit::keyboard::KeyCode::Props => input::keycode::KeyCode::Props,
        winit::keyboard::KeyCode::Select => input::keycode::KeyCode::Select,
        winit::keyboard::KeyCode::Undo => input::keycode::KeyCode::Undo,
        winit::keyboard::KeyCode::Hiragana => input::keycode::KeyCode::Hiragana,
        winit::keyboard::KeyCode::Katakana => input::keycode::KeyCode::Katakana,
        winit::keyboard::KeyCode::F1 => input::keycode::KeyCode::F1,
        winit::keyboard::KeyCode::F2 => input::keycode::KeyCode::F2,
        winit::keyboard::KeyCode::F3 => input::keycode::KeyCode::F3,
        winit::keyboard::KeyCode::F4 => input::keycode::KeyCode::F4,
        winit::keyboard::KeyCode::F5 => input::keycode::KeyCode::F5,
        winit::keyboard::KeyCode::F6 => input::keycode::KeyCode::F6,
        winit::keyboard::KeyCode::F7 => input::keycode::KeyCode::F7,
        winit::keyboard::KeyCode::F8 => input::keycode::KeyCode::F8,
        winit::keyboard::KeyCode::F9 => input::keycode::KeyCode::F9,
        winit::keyboard::KeyCode::F10 => input::keycode::KeyCode::F10,
        winit::keyboard::KeyCode::F11 => input::keycode::KeyCode::F11,
        winit::keyboard::KeyCode::F12 => input::keycode::KeyCode::F12,
        winit::keyboard::KeyCode::F13 => input::keycode::KeyCode::F13,
        winit::keyboard::KeyCode::F14 => input::keycode::KeyCode::F14,
        winit::keyboard::KeyCode::F15 => input::keycode::KeyCode::F15,
        winit::keyboard::KeyCode::F16 => input::keycode::KeyCode::F16,
        winit::keyboard::KeyCode::F17 => input::keycode::KeyCode::F17,
        winit::keyboard::KeyCode::F18 => input::keycode::KeyCode::F18,
        winit::keyboard::KeyCode::F19 => input::keycode::KeyCode::F19,
        winit::keyboard::KeyCode::F20 => input::keycode::KeyCode::F20,
        winit::keyboard::KeyCode::F21 => input::keycode::KeyCode::F21,
        winit::keyboard::KeyCode::F22 => input::keycode::KeyCode::F22,
        winit::keyboard::KeyCode::F23 => input::keycode::KeyCode::F23,
        winit::keyboard::KeyCode::F24 => input::keycode::KeyCode::F24,
        winit::keyboard::KeyCode::F25 => input::keycode::KeyCode::F25,
        winit::keyboard::KeyCode::F26 => input::keycode::KeyCode::F26,
        winit::keyboard::KeyCode::F27 => input::keycode::KeyCode::F27,
        winit::keyboard::KeyCode::F28 => input::keycode::KeyCode::F28,
        winit::keyboard::KeyCode::F29 => input::keycode::KeyCode::F29,
        winit::keyboard::KeyCode::F30 => input::keycode::KeyCode::F30,
        winit::keyboard::KeyCode::F31 => input::keycode::KeyCode::F31,
        winit::keyboard::KeyCode::F32 => input::keycode::KeyCode::F32,
        winit::keyboard::KeyCode::F33 => input::keycode::KeyCode::F33,
        winit::keyboard::KeyCode::F34 => input::keycode::KeyCode::F34,
        winit::keyboard::KeyCode::F35 => input::keycode::KeyCode::F35,
        _ => input::keycode::KeyCode::None,
    }
}
