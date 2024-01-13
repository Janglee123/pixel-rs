use glam::Vec2;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::app::{App, Plugin};

use super::input_plugin::{Input, MouseButtonInput};

pub struct WindowPlugin;

fn runner(mut app: App) {
    let event_loop = app.world.singletons.remove::<EventLoop<()>>().unwrap();
    let w_id = app.world.singletons.get::<Window>().unwrap().id();

    event_loop.run(move |event, window_target| {
        // *r_control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { window_id, event } if window_id == w_id => match event {
                WindowEvent::CloseRequested => {
                    println!("bye bye");
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
                    let input = MouseButtonInput { button, state };
                    on_mouse_input(&mut app, input);
                }
                WindowEvent::CursorMoved {
                    device_id,
                    position,
                    ..
                } => {
                    on_curser_moved(&mut app, position.cast::<f32>());
                }
                _ => (),
            },

            Event::AboutToWait => {
                app.update();
                let window = app.world.singletons.get::<Window>().unwrap();
                window.request_redraw();
            }
            _ => (),
        }
    });
}

fn on_curser_moved(app: &mut App, cursor_pos: PhysicalPosition<f32>) {
    let (input, window) = app
        .world
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

fn on_mouse_input(app: &mut App, mouse_input: MouseButtonInput) {
    // Why accessing input here??

    let input = app.world.singletons.get_mut::<Input>().unwrap();
    input.on_mouse_input(mouse_input);
    app.on_mouse_input();
}

fn on_keyboard_input(app: &mut App, key_input: KeyEvent) {
    let input = app.world.singletons.get_mut::<Input>().unwrap();
    input.on_keyboard_input(key_input);
    app.on_keyboard_input();
}

impl Plugin for WindowPlugin {
    fn build(app: &mut crate::app::App) {
        let event_loop = EventLoop::new().unwrap();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        app.world.singletons.insert(window);
        app.world.singletons.insert(event_loop);

        app.set_runner(runner);
    }
}
