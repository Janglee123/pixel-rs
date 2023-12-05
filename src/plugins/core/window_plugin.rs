use winit::{
    event::{Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::app::{App, Plugin};

use super::input_plugin::{Input, MouseButtonInput};

pub struct WindowPlugin;

fn runner(mut app: App) {
    let event_loop = app.world.singletons.remove::<EventLoop<()>>().unwrap();
    let w_id = app.world.singletons.get::<Window>().unwrap().id();

    event_loop.run(move |event, _, r_control_flow| {
        // *r_control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { window_id, event } if window_id == w_id => match event {
                WindowEvent::CloseRequested => *r_control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => app.on_resize(physical_size),
                WindowEvent::KeyboardInput {
                    device_id: _,
                    input,
                    is_synthetic: _,
                } => on_keyboard_input(&mut app, input),
                WindowEvent::MouseInput {
                    device_id,
                    state,
                    button,
                    ..
                } => {
                    let input = MouseButtonInput { button, state };
                    on_mouse_input(&mut app, input);
                }
                _ => (),
            },

            Event::MainEventsCleared => app.update(),
            _ => (),
        }
    });
}

fn on_mouse_input(app: &mut App, mouse_input: MouseButtonInput) {
    let input = app.world.singletons.get_mut::<Input>().unwrap();
    input.on_mouse_input(mouse_input);
    app.on_mouse_input();
}

fn on_keyboard_input(app: &mut App, key_input: KeyboardInput) {
    let input = app.world.singletons.get_mut::<Input>().unwrap();
    input.on_keyboard_input(key_input);
    app.on_keyboard_input();
}

impl Plugin for WindowPlugin {
    fn build(app: &mut crate::app::App) {
        let event_loop = EventLoop::new();
        let monitor = event_loop
            .available_monitors()
            .next()
            .expect("no monitor found!");
        let _mode = monitor.video_modes().next().expect("no mode found");
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        app.world.singletons.insert(window);
        app.world.singletons.insert(event_loop);

        app.set_runner(runner);
    }
}
