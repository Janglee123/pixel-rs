use wgpu::{
    Device, DeviceDescriptor, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration,
};
use winit::{
    event::{Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    env_logger::init();

    // Event_loop to capture event's from OS. i.e. user interaction events and OS calls like close window or render.
    let event_loop = EventLoop::new();

    // Actual window! The Window we need to draw stuff. [I dont know why they need builder here. Maybe to provide comparability across versions?]
    let window = WindowBuilder::new()
        .build(&event_loop)
        .expect("Do you expect a game to run without a window?");

    let window_size = window.inner_size();

    // Create instance of WGPU. Instance's only job is to create adapter and surface.
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);

    // Surface is something that we want to draw on.
    // We will pass window to attach the surface to the window.
    // There might be a way to work with gpu without creating window and surface.
    let surface = unsafe { instance.create_surface(&window) };

    let adapter_options = RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    };

    // Adapter describes features and limitation of GPU. You can list all available adapters with `instance.enumerate_adapters`.
    // Adapter is needed to create a device. Once we create device we don't need adapter anymore.
    let adapter = pollster::block_on(instance.request_adapter(&adapter_options)).unwrap();

    let device_descriptor = DeviceDescriptor {
        label: None,
        features: wgpu::Features::default(),
        limits: wgpu::Limits::default(), // TODO: learn about this
    };

    // Device is logical device that represent GPU. It will allow you to create the data structures you'll need.
    // Queue is command queue. It contains encoded commands for gpu to execute.
    // Encoding is highly specific to the gpu and taken care by the driver.
    // Thankfully wgpu provides `CommandEncoder` to encode commands.
    let (device, queue) =
        pollster::block_on(adapter.request_device(&device_descriptor, None)).unwrap();

    let surface_config = SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_supported_formats(&adapter)[0],
        width: window_size.width,
        height: window_size.height,
        present_mode: wgpu::PresentMode::AutoNoVsync,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
    };

    // Initialize the surface and attach to device.
    // Why it needs a device? TODO
    // Docs only says "Initializes Surface for presentation."
    // Deep under method call. It says "Configured device is needed to know which backend
    // code to execute when acquiring a new frame."
    // It keeps device id and uses it. Its way beyond my current understanding on wgpu
    surface.configure(&device, &surface_config);

    let mut color = wgpu::Color {
        r: 0.1,
        g: 0.2,
        b: 0.3,
        a: 0.4,
    };

    // Event loop emits an event for many things.
    // For now we only need keyboard event, window close event and RedrawRequested.
    // We pass an closure to handle events like _notification in godot.
    // We are also listing to curser moved event to update color variable.
    event_loop.run(move |event, _, r_control_flow| {
        // control_flow is return value to tell what to do next.
        *r_control_flow = ControlFlow::Poll;

        match event {
            // Window event is container for many window related events.
            // We need to check if event's window is current window
            // What a rusty way to write match statement, Utterly un-readable.
            Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => on_close_requested(r_control_flow),
                WindowEvent::KeyboardInput { input, .. } => {
                    on_keyboard_input(input, r_control_flow)
                }

                WindowEvent::CursorMoved {
                    device_id: _,
                    position: pos,
                    modifiers: _,
                } => {
                    color.r = pos.x / (window_size.width as f64);
                    color.b = pos.y / (window_size.height as f64);
                }

                _ => (), // This is default: break; Meh syntax. It can spook any gdscipt programmer!
            },

            // Docs of `Event::MainEventsCleared` says use it instead of RedrawRequested for games.
            // Event::RedrawRequested(_) => on_redraw(&surface, &device, &queue, &color),
            Event::MainEventsCleared => on_redraw(&surface, &device, &queue, &color),

            _ => (), // Boo!!
        }
    });
}

// All rendering magic happens here.
// For now we are just filling whole window with a clear color.
fn on_redraw(surface: &Surface, device: &Device, queue: &Queue, color: &wgpu::Color) {
    // I dont understand output and view.
    // Get Surface texture. Its contains actual texture. This texture will be drawn to the window in next frame.
    let output = surface.get_current_texture().unwrap();

    // TextureView that does stuff which I dont know right now. We are getting right texture from output here.
    // A `TextureView` object describes a texture and associated metadata needed by a [`RenderPipeline`] or [`BindGroup`].
    // So we need to create TextureView to use it with render pass
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    // Initializes a command encoder for encoding operations to the GPU.
    // Sending operations to the GPU in WGPU involves encoding and then queuing up operations for the GPU to perform
    // Look device is used to create command encoder.
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    // This code block is to bypass rust borrow checking of encoder. I don't know how it works.
    // Do I again need to create RenderPassDescriptor struct every time?
    // `encoder.begin_render_pass` returns render_pass but we don't need it right now.
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(*color),
                store: true,
            },
        })],
        depth_stencil_attachment: None,
    });

    // Submit series of commands for execution. But we have only one command for now.
    // std::iter::once creates iterable with only one element. Cant we just use normal array?
    // Encoder.finish() takes ownership of encoder so we cant reuse it again.
    queue.submit(std::iter::once(encoder.finish()));

    // Schedules the texture to be presented on the surface one queue is executed.
    output.present();
}

// Tell winit to close when asked.
// Not closing window directly by winit allow to create confirmation dialog to save work.
fn on_close_requested(control_flow: &mut ControlFlow) {
    *control_flow = ControlFlow::Exit;
}

// Close window on Escape. Dont know why but tutorial does it so do I.
fn on_keyboard_input(input: KeyboardInput, control_flow: &mut ControlFlow) {
    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::Escape) {
        *control_flow = ControlFlow::Exit;
    }
}
