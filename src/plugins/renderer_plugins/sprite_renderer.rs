use bytemuck::{Pod, Zeroable};
use std::time::{SystemTime, UNIX_EPOCH};
use wgpu::{include_wgsl, util::DeviceExt, BindGroupLayout, Buffer, Device, RenderPipeline};
use winit::window::Window;

use crate::{
    app::Plugin,
    ecs::world::{Component, World},
    math::{
        transform2d::{self, Matrix3, Transform2d},
        vector2::Vector2,
    },
    plugins::core::{
        camera_plugin::Camera,
        render_plugin::{Gpu, Renderer},
    },
    query, query_mut, zip,
};

use super::{
    texture::{self, Texture},
    vertex::Vertex,
};

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
    pub transform_bind_group_layout: BindGroupLayout, // Hmm I really need to think about how to write a render stuff
    pub texture_bind_group_layout: BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,
    camera_buffer: Buffer,
}

pub struct Quad {
    transform_buffer: Buffer,
    transform_bind_group: wgpu::BindGroup,
    texture_bind_group: wgpu::BindGroup,
    texture: texture::Texture, //Todo: Asset manager
    size: Vector2<u32>,
}

impl Quad {
    // What I want is that quad does not store buffers? No
    // What I want is that creating a quad should be easier? Then another creator can do it
    // What I want is that I do not have to duplicate textures? Do I need it now? Nope
    pub fn new(
        device: &Device,
        transform_bind_group_layout: &BindGroupLayout,
        texture_bind_group_layout: &BindGroupLayout,
        texture: Texture,
        size: Vector2<u32>,
    ) -> Self {
        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[Transform2d::IDENTITY.create_matrix()]),
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

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        });

        Self {
            transform_buffer,
            transform_bind_group,
            texture_bind_group,
            texture,
            size,
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
        let (camera, transform2d) = query!(world, Camera, Transform2d).next().unwrap();
        let projection = transform2d.create_matrix() * camera.projection;

        let data = world.singletons.get::<SpriteRendererData>().unwrap();

        render_pass.set_pipeline(&data.render_pipeline);
        render_pass.set_vertex_buffer(0, data.vertex_buffer.slice(..));
        render_pass.set_index_buffer(data.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        // Todo: Common camera_buffer for all
        gpu.queue
            .write_buffer(&data.camera_buffer, 0, bytemuck::cast_slice(&[projection]));

        render_pass.set_bind_group(2, &data.camera_bind_group, &[]);

        for (transform2d, quad) in query!(world, Transform2d, Quad) {
            gpu.queue.write_buffer(
                &quad.transform_buffer,
                0,
                bytemuck::cast_slice(&[transform2d.create_matrix()]),
            );

            render_pass.set_bind_group(0, &quad.transform_bind_group, &[]);
            render_pass.set_bind_group(1, &quad.texture_bind_group, &[]);

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

        let camera_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Camera bind group layout"),
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

        let camera_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera buffer"),
                contents: bytemuck::cast_slice(&[Matrix3::IDENTITY]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera bind group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let texture_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            // This should match the filterable field of the
                            // corresponding Texture entry above.
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                });

        let render_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &transform_bind_group_layout,
                        &texture_bind_group_layout,
                        &camera_bind_group_layout,
                    ],
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
            transform_bind_group_layout, // Why we have transform bind group layout here
            texture_bind_group_layout,
            camera_bind_group,
            camera_buffer,
        };

        app.renderers.push(Box::new(SpritePlugin {}));

        app.world.register_component::<Quad>();
        app.world.singletons.insert(sprite_renderer_data);
    }
}
