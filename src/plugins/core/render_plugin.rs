use bytemuck::{Pod, Zeroable};
use wgpu::{
    include_wgsl, util::DeviceExt, Device, DeviceDescriptor, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration,
};
use winit::window::Window;

use crate::{app::{App, Plugin}, ecs::world::{self, World}};

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

        // let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        // let render_pipeline_layout =
        //     device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //         label: Some("Render Pipeline Layout"),
        //         bind_group_layouts: &[],
        //         push_constant_ranges: &[],
        //     });

        // let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        //     label: Some("Render Pipeline"),
        //     layout: Some(&render_pipeline_layout),
        //     vertex: wgpu::VertexState {
        //         module: &shader,
        //         entry_point: "vs_main",
        //         buffers: &[Vertex::decs()],
        //     },

        //     fragment: Some(wgpu::FragmentState {
        //         module: &shader,
        //         entry_point: "fs_main",
        //         targets: &[Some(wgpu::ColorTargetState {
        //             format: surface_config.format,
        //             blend: Some(wgpu::BlendState::REPLACE),
        //             write_mask: wgpu::ColorWrites::ALL,
        //         })],
        //     }),

        //     primitive: wgpu::PrimitiveState {
        //         topology: wgpu::PrimitiveTopology::TriangleList,
        //         strip_index_format: None,
        //         front_face: wgpu::FrontFace::Ccw,
        //         cull_mode: Some(wgpu::Face::Back),
        //         unclipped_depth: false,
        //         polygon_mode: wgpu::PolygonMode::Fill,
        //         conservative: false,
        //     },
        //     depth_stencil: None,
        //     multisample: wgpu::MultisampleState {
        //         count: 1,
        //         mask: !0,
        //         alpha_to_coverage_enabled: false,
        //     },
        //     multiview: None,
        // });

        // let render_pipeline2 = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        //     label: Some("Render Pipeline"),
        //     layout: Some(&render_pipeline_layout),
        //     vertex: wgpu::VertexState {
        //         module: &shader,
        //         entry_point: "vs_main",
        //         buffers: &[Vertex::decs()],
        //     },

        //     fragment: Some(wgpu::FragmentState {
        //         module: &shader,
        //         entry_point: "fs_main",
        //         targets: &[Some(wgpu::ColorTargetState {
        //             format: surface_config.format,
        //             blend: Some(wgpu::BlendState::REPLACE),
        //             write_mask: wgpu::ColorWrites::ALL,
        //         })],
        //     }),

        //     primitive: wgpu::PrimitiveState {
        //         topology: wgpu::PrimitiveTopology::PointList,
        //         strip_index_format: None,
        //         front_face: wgpu::FrontFace::Ccw,
        //         cull_mode: Some(wgpu::Face::Back),
        //         unclipped_depth: false,
        //         polygon_mode: wgpu::PolygonMode::Fill,
        //         conservative: false,
        //     },
        //     depth_stencil: None,
        //     multisample: wgpu::MultisampleState {
        //         count: 1,
        //         mask: !0,
        //         alpha_to_coverage_enabled: false,
        //     },
        //     multiview: None,
        // });

        // let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Vertex Buffer"),
        //     contents: bytemuck::cast_slice(VERTICES),
        //     usage: wgpu::BufferUsages::VERTEX,
        // });

        // let vertex_buffer2 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Vertex Buffer"),
        //     contents: bytemuck::cast_slice(VERTICES2),
        //     usage: wgpu::BufferUsages::VERTEX,
        // });


        let gpu = Gpu {
            surface,
            queue,
            device,
            surface_config,
            // render_pipeline,
            // render_pipeline2,
            // vertex_buffer,
            // vertex_buffer2,
            // num_vertices,
        };

        app.world.singletons.insert(gpu);
        // app.schedular.add_system(1, draw);
    }
}

// pub fn draw(world: &mut World) {

//     let mut gpu = world.singletons.get::<Gpu>().unwrap();

//     let output = gpu.surface.get_current_texture().unwrap();

//     let view = output
//         .texture
//         .create_view(&wgpu::TextureViewDescriptor::default());

//     let mut encoder = gpu
//         .device
//         .create_command_encoder(&wgpu::CommandEncoderDescriptor {
//             label: Some("Render Encoder"),
//         });

//     let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//         label: Some("Render Pass"),
//         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//             view: &view,
//             resolve_target: None,
//             ops: wgpu::Operations {
//                 load: wgpu::LoadOp::Clear(wgpu::Color {
//                     r: 1.0,
//                     g: 1.0,
//                     b: 1.0,
//                     a: 1.0,
//                 }),
//                 store: true,
//             },
//         })],
//         depth_stencil_attachment: None,
//     });

//     render_pass.set_pipeline(&gpu.render_pipeline);
//     render_pass.set_vertex_buffer(0, gpu.vertex_buffer.slice(..));
//     render_pass.draw(0..3, 0..1);

//     render_pass.set_pipeline(&gpu.render_pipeline2);
//     render_pass.set_vertex_buffer(0, gpu.vertex_buffer2.slice(..));
//     render_pass.draw(0..3, 0..1);

//     drop(render_pass);
//     gpu.queue.submit(std::iter::once(encoder.finish()));

//     output.present();
// }

// pub fn resize(&mut self, width: u32, height: u32) {
//     if width > 0 && height > 0 {
//         self.surface_config.width = width;
//         self.surface_config.height = height;
//         self.surface.configure(&self.device, &self.surface_config);
//     }
// }

