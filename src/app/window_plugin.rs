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
