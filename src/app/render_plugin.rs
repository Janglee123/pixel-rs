use wgpu::{Device, DeviceDescriptor, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration};

pub struct RenderPlugin {
    surface: Surface,
    queue: Queue,
    device: Device,
    surface_config: wgpu::SurfaceConfiguration
}

impl RenderPlugin {
    pub fn new(window: &winit::window::Window) -> Self {
        env_logger::init();

        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);

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

        let format = surface.get_supported_formats(&adapter)[0];
        
        let surface_config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };

        surface.configure(&device, &surface_config);

        Self {
            surface,
            queue,
            device,
            surface_config,
        }
    }

    pub fn draw(&self) {
        let output = self.surface.get_current_texture().unwrap();

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();
    }

    pub fn resize(&mut self, width: u32, height: u32) {

        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }
}
