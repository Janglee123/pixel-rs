use wgpu::{Device, DeviceDescriptor, RequestAdapterOptions, Surface, SurfaceConfiguration, Queue};
use winit::{
    event::{Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .build(&event_loop)
        .expect("Do you expect a game to run without a window?");

    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);

    for adapter in instance.enumerate_adapters(wgpu::Backends::all()) {
        println!("{:?}\n", adapter.get_info())
    }

    let surface = unsafe { instance.create_surface(&window) };

    let adapter_options = RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    };

    let adapter = pollster::block_on(instance.request_adapter(&adapter_options)).unwrap();

    let device_descriptor = DeviceDescriptor {
        label: None,
        features: wgpu::Features::default(),
        limits: wgpu::Limits::default(), // TODO: learn about this
    };

    let (device, queue) =
        pollster::block_on(adapter.request_device(&device_descriptor, None)).unwrap();

    let windw_size = window.inner_size();

    let surface_config = SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_supported_formats(&adapter)[0],
        width: windw_size.width,
        height: windw_size.height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
    };

    surface.configure(&device, &surface_config);

    let mut color = wgpu::Color{
        r: 0.1,
        g: 0.2,
        b: 0.3,
        a: 0.4,
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => on_close_reqested(control_flow),
                WindowEvent::KeyboardInput { input, .. } => on_keyboard_input(input, control_flow),
                WindowEvent::CursorMoved { device_id, position, modifiers } => {
                    color.r = position.x / (windw_size.width as f64);
                    color.b = position.y / (windw_size.height as f64);
                }               

                _ => (),
            },
            Event::RedrawRequested(_) => on_redraw(&surface, &device, &queue, &color),
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => (),
        }
    });
}

fn on_redraw(surface: &Surface, device: &Device, queue: &Queue, color: &wgpu::Color) {
    let output = surface.get_current_texture().unwrap();

    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    {
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
    }

    queue.submit(std::iter::once(encoder.finish()));
    output.present();
}

fn on_close_reqested(control_flow: &mut ControlFlow) {
    *control_flow = ControlFlow::Exit;
}

fn on_keyboard_input(input: KeyboardInput, control_flow: &mut ControlFlow) {
    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::Escape) {
        *control_flow = ControlFlow::Exit;
    }
}
