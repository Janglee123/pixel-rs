use bytemuck::{Pod, Zeroable};
use std::time::{SystemTime, UNIX_EPOCH};
use wgpu::{include_wgsl, util::DeviceExt, BindGroupLayout, Buffer, Device, RenderPipeline};
use winit::window::Window;

use crate::{
    app::Plugin,
    ecs::world::{Component, World},
    math::{
        transform2d::{self, Transform2d},
        vector2::Vector2,
    },
    plugins::core::render_plugin::{Gpu, Renderer},
    query, query_mut, zip,
};

use super::vertex::Vertex;

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [1.0, 1.0, 1.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 1.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, 1.0],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 1.0],
        color: [0.0, 0.0, 1.0],
    },
];

const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

pub struct SpriteRendererData {
    pub render_pipeline: RenderPipeline,
    pub vertex_buffer: Buffer,
    index_buffer: Buffer,
    transform_bind_group_layout: BindGroupLayout,
}

pub struct Quad {
    transform_buffer: Buffer,
    transform_bind_group: wgpu::BindGroup,
}

impl Quad {
    pub fn new(device: &Device, transform_bind_group_layout: &BindGroupLayout) -> Self {
        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[Transform2d::IDENTITY.into_matrix()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // I am passing bind group layout
        let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Transform buffer"),
            layout: transform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            }],
        });

        Self {
            transform_buffer,
            transform_bind_group,
        }
    }
}

pub struct SpritePlugin;

impl Renderer for SpritePlugin {
    fn render<'pass, 'encoder: 'pass, 'world: 'encoder>(
        &self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        world: &'world World,
    ) {
        let window = world.singletons.get::<Window>().unwrap();
        let size = window.inner_size();

        let gpu = world.singletons.get::<Gpu>().unwrap();
        let data = world.singletons.get::<SpriteRendererData>().unwrap();

        render_pass.set_pipeline(&data.render_pipeline);
        render_pass.set_vertex_buffer(0, data.vertex_buffer.slice(..));
        render_pass.set_index_buffer(data.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        for (transform2d, quad) in query!(world, Transform2d, Quad) {
            gpu.queue.write_buffer(
                &quad.transform_buffer,
                0,
                bytemuck::cast_slice(&[transform2d.into_matrix()]),
            );

            render_pass.set_bind_group(0, &quad.transform_bind_group, &[]);

            render_pass.draw_indexed(0..6, 0, 0..1);
        }
    }
}

impl Plugin for SpritePlugin {
    fn build(app: &mut crate::app::App) {
        let gpu = app.world.singletons.get::<Gpu>().unwrap();

        let shader = gpu
            .device
            .create_shader_module(include_wgsl!("sprite_shader.wgsl"));

        let transform_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let render_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&transform_bind_group_layout],
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

        let index_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            });

        let sprite_renderer_data = SpriteRendererData {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            transform_bind_group_layout,
        };

        // let transform2d = Transform2d {
        //     position: Vector2 { x: 0.5, y: 0.5 },
        //     rotation: 0.0,
        //     scale: Vector2 { x: 0.2, y: 0.2 },
        // };

        // let transform2d2 = Transform2d {
        //     position: Vector2 { x: -0.5, y: -0.5 },
        //     rotation: 0.0,
        //     scale: Vector2 { x: 0.2, y: 0.3 },
        // };

        // let quad1 = Quad::new(
        //     &gpu.device,
        //     &triangle_renderer_data.transform_bind_group_layout,
        // );
        // let quad2 = Quad::new(
        //     &gpu.device,
        //     &triangle_renderer_data.transform_bind_group_layout,
        // );

        app.renderers.push(Box::new(SpritePlugin {}));

        app.world.singletons.insert(sprite_renderer_data);

        // app.schedular
        //     .add_system(crate::app::SystemStage::Update, draw);
    }
}
