// use std::rc::Rc;

// use winit::{
//     event::{Event, WindowEvent},
//     event_loop::{self, ControlFlow, EventLoop, EventLoopBuilder},
//     window::{Window, WindowBuilder},
// };

// use super::App;

// pub struct WindowPlugin {
//     pub event_loop: Option<EventLoop<()>>,
//     pub window: Option<Window>,
// }

// impl WindowPlugin {
//     pub fn new() -> Self {
//         Self {
//             event_loop: Some(EventLoop::new()),
//             window: None,
//         }
//     }

//     pub fn run(&mut self, app: &'static App) {

//         // I cant beat the borrow checker, Gave up!
//         let mut event_loop = self.event_loop.take().unwrap();
//         let mut window = WindowBuilder::new().build(&event_loop).unwrap();

//         self.window = Some(window);

//         event_loop.run(move |event, _, r_control_flow| {
//             *r_control_flow = ControlFlow::Poll;

//             match event {
//                 Event::WindowEvent { window_id, event } if window_id == self.window.as_ref().unwrap().id() => {
//                     match event {
//                         WindowEvent::CloseRequested => *r_control_flow = ControlFlow::Exit,

//                         _ => (),
//                     }
//                 }

//                 Event::MainEventsCleared => app.update(),

//                 _ => (),
//             }
//         });

//     }
// }

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::app::{App, Plugin};

pub struct WindowPlugin;

fn runner(mut app: App) {
    let event_loop = app.world.singletons.remove::<EventLoop<()>>().unwrap();
    let w_id = app.world.singletons.get::<Window>().unwrap().id();

    event_loop.run(move |event, _, r_control_flow| {
        *r_control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { window_id, event } if window_id == w_id => match event {
                WindowEvent::CloseRequested => *r_control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => app.on_resize(physical_size),
                WindowEvent::KeyboardInput {
                    device_id: _,
                    input,
                    is_synthetic: _,
                } => app.on_keyboard_input(input),
                _ => (),
            },

            Event::MainEventsCleared => app.update(),
            _ => (),
        }
    });
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
