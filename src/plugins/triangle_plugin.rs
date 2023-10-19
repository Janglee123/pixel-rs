use bytemuck::{Pod, Zeroable};
use wgpu::{include_wgsl, util::DeviceExt, RenderPipeline, Buffer};

use crate::{app::Plugin, ecs::world::World};

use super::core::render_plugin::Gpu;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    fn decs<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.25, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.25, -0.25, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.25, -0.25, 0.0],
        color: [0.0, 0.0, 1.0],
    },
];

pub struct TriangleRendererData {
    pub render_pipeline: RenderPipeline,
    pub vertex_buffer: Buffer,
}

pub struct TrianglePlugin;

impl Plugin for TrianglePlugin {
    fn build(app: &mut crate::app::App) {
        let gpu = app.world.singletons.get::<Gpu>().unwrap();

        let shader = gpu
            .device
            .create_shader_module(include_wgsl!("shader.wgsl"));

        let render_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let render_pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::decs()],
                },

                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: gpu.surface_config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),

                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        let vertex_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let triangle_renderer_data = TriangleRendererData {
            render_pipeline,
            vertex_buffer,
        };

        app.world.singletons.insert(triangle_renderer_data);
        app.schedular.add_system(1, draw);
    }
}

pub fn draw(world: &mut World) {
    let mut gpu = world.singletons.get::<Gpu>().unwrap();
    let mut data = world.singletons.get::<TriangleRendererData>().unwrap();

    let output = gpu.surface.get_current_texture().unwrap();

    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                }),
                store: true,
            },
        })],
        depth_stencil_attachment: None,
    });

    render_pass.set_pipeline(&data.render_pipeline);
    render_pass.set_vertex_buffer(0, data.vertex_buffer.slice(..));
    render_pass.draw(0..3, 0..1);

    drop(render_pass);
    gpu.queue.submit(std::iter::once(encoder.finish()));

    output.present();
}
