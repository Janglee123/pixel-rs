use winit::{
    dpi::PhysicalSize,
    event::{Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};

pub mod render_plugin;

pub struct AppBuilder {}

impl AppBuilder {
    pub fn new() {
        let event_loop = EventLoop::new();

        let monitor = event_loop
            .available_monitors()
            .next()
            .expect("no monitor found!");

        let mode = monitor.video_modes().next().expect("no mode found");

        let window = WindowBuilder::new()
            // .with_fullscreen(Some(Fullscreen::Borderless(Some(monitor))))
            .build(&event_loop)
            .unwrap();

        let mut app = App {
            counter: 0,
            render_plugin: render_plugin::RenderPlugin::new(&window),
        };

        event_loop.run(move |event, _, r_control_flow| {
            *r_control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent { window_id, event } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => *r_control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => app.on_resize(physical_size),
                        WindowEvent::KeyboardInput {
                            device_id: _,
                            input,
                            is_synthetic: _,
                        } => app.on_keyboard_input(input),
                        _ => (),
                    }
                }

                Event::MainEventsCleared => app.update(),

                _ => (),
            }
        });
    }
}

pub struct App {
    counter: i32,
    render_plugin: render_plugin::RenderPlugin,
}

impl App {
    fn update(&mut self) {
        self.counter += 1;
        self.render_plugin.draw();
    }

    fn on_resize(&mut self, physical_size: PhysicalSize<u32>) {
        self.render_plugin
            .resize(physical_size.width, physical_size.height);
    }

    fn on_keyboard_input(&self, input: KeyboardInput) {
        println!("Keyboard input {:?}", input);
    }
}
