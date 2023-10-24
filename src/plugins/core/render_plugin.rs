use bytemuck::{Pod, Zeroable};
use wgpu::{
    include_wgsl, util::DeviceExt, Device, DeviceDescriptor, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration,
};
use winit::window::Window;

use crate::{
    app::{App, Plugin},
    ecs::world::{self, World},
};

pub struct Gpu {
    pub surface: Surface,
    pub queue: Queue,
    pub device: Device,
    pub surface_config: wgpu::SurfaceConfiguration,
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(app: &mut App) {
        let window = app.world.singletons.get::<Window>().unwrap();

        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);

        let surface = unsafe { instance.create_surface(&window) };

        let adapter_options = RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        };

        for adapter in instance.enumerate_adapters(wgpu::Backends::VULKAN) {
            println!("{:?}", adapter.get_info());
        }

        let adapter = pollster::block_on(instance.request_adapter(&adapter_options)).unwrap();

        println!("Selected adapter {:?}", adapter.get_info());

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

        let gpu = Gpu {
            surface,
            queue,
            device,
            surface_config,
        };

        app.world.singletons.insert(gpu);
        app.schedular
            .add_system(crate::app::SystemStage::Resize, on_resize)
        // app.schedular.add_system(1, draw);
    }
}

fn on_resize(world: &mut World) {
    let window = world.singletons.get::<Window>().unwrap();
    let size = window.inner_size();

    let gpu = world.singletons.get_mut::<Gpu>().unwrap();
    gpu.surface_config.width = size.width;
    gpu.surface_config.height = size.height;

    gpu.surface.configure(&gpu.device, &gpu.surface_config);
}
