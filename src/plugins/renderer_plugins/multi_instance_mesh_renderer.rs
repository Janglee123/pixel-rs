use crate::{
    math::{
        honeycomb::HEXAGON_INDICES,
        transform2d::{self, Matrix3, Transform2d},
    },
    plugins::core::{camera_plugin::Camera, render_plugin::Renderer},
};

use std::{
    any::{Any, TypeId},
    borrow::BorrowMut,
    mem,
    ops::Deref,
    rc::Rc,
    sync::Arc,
};
use wgpu::{include_wgsl, util::DeviceExt, BindGroupLayout, RenderPass, RenderPipeline};

use crate::{
    app::Plugin,
    ecs::world::{Component, World},
    math::{color::Color, vector2::Vector2},
    plugins::core::render_plugin::Gpu,
    query, query_mut, zip,
};

use super::{mesh::Mesh, texture::Texture, vertex::Vertex};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct InstanceData {
    matrix: Matrix3,
    color: [f32; 3],
    _padding: u32,
}

impl InstanceData {
    const ATTRIBS: [wgpu::VertexAttribute; 4] =
        // lets hope matrix will work with this//
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2=>Float32x3, 3 => Float32x3];

    pub fn new(transform2d: &Transform2d, color: [f32; 3]) -> Self {
        Self {
            matrix: transform2d.create_matrix(),
            color,
            _padding: 0,
        }
    }

    pub fn decs<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[derive(Debug)]
pub struct MultiInstanceMesh {
    pub instances: Vec<InstanceData>,
    pub transform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub instance_data_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub mesh: Arc<Mesh>,
    texture: Texture,
}

impl MultiInstanceMesh {
    pub fn new(
        gpu: &Gpu,
        bind_group_layout: &MultiInstanceMeshBindGroupLayout,
        mesh: Arc<Mesh>,
        texture: Texture,
    ) -> Self {
        let device = &gpu.device;

        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Transform Buffer"),
            contents: bytemuck::cast_slice(&[Transform2d::IDENTITY.create_matrix()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let instance_data = InstanceData::new(&Transform2d::IDENTITY, [0.0; 3]);

        let instance_data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&[instance_data; 4096]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: transform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: instance_data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        });

        let vertex_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let index_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        Self {
            instances: Vec::new(),
            bind_group,
            transform_buffer,
            instance_data_buffer,
            vertex_buffer,
            index_buffer,
            mesh,
            texture,
        }
    }
}

pub struct MultiInstanceMeshRendererData {
    render_pipeline: RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
}

pub struct MultiInstanceMeshRenderer;

impl Renderer for MultiInstanceMeshRenderer {
    fn render<'pass, 'encoder: 'pass, 'world: 'encoder>(
        &self,
        render_pass: &mut RenderPass<'encoder>,
        world: &'world World,
    ) {
        let data = world
            .singletons
            .get::<MultiInstanceMeshRendererData>()
            .unwrap();
        let gpu = world.singletons.get::<Gpu>().unwrap();

        let (camera, transform2d) = query!(world, Camera, Transform2d).next().unwrap();
        let projection = transform2d.create_matrix() * camera.projection;

        render_pass.set_pipeline(&data.render_pipeline);

        gpu.queue
            .write_buffer(&data.camera_buffer, 0, bytemuck::cast_slice(&[projection]));
        render_pass.set_bind_group(1, &data.camera_bind_group, &[]);

        for (multi_instance_mesh, transform2d) in query!(world, MultiInstanceMesh, Transform2d) {
            render_pass.set_vertex_buffer(0, multi_instance_mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                multi_instance_mesh.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );

            gpu.queue.write_buffer(
                &multi_instance_mesh.transform_buffer,
                0,
                bytemuck::cast_slice(&[transform2d.create_matrix()]),
            );

            gpu.queue.write_buffer(
                &multi_instance_mesh.instance_data_buffer,
                0,
                bytemuck::cast_slice(&multi_instance_mesh.instances), // Should I cache it??
            );

            render_pass.set_bind_group(0, &multi_instance_mesh.bind_group, &[]);

            render_pass.draw_indexed(
                0..multi_instance_mesh.mesh.indices.len() as u32,
                0,
                0..multi_instance_mesh.instances.len() as u32,
            );
        }
    }
}

pub struct MultiInstanceMeshBindGroupLayout {
    pub bind_group_layout: BindGroupLayout,
}

impl Plugin for MultiInstanceMeshRenderer {
    fn build(app: &mut crate::app::App) {
        let gpu = app.world.singletons.get::<Gpu>().unwrap();
        let shader = gpu
            .device
            .create_shader_module(include_wgsl!("multi_instance_mesh.wgsl"));

        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("tilemap_bind_group_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            // This should match the filterable field of the
                            // corresponding Texture entry above.
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
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

        let render_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout, &camera_bind_group_layout],
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
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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

        let tile_map_data = MultiInstanceMeshRendererData {
            render_pipeline,
            camera_buffer,
            camera_bind_group,
        };

        let tile_map_bind_group_layout = MultiInstanceMeshBindGroupLayout { bind_group_layout };

        app.world.register_component::<MultiInstanceMesh>();
        app.world.singletons.insert(tile_map_bind_group_layout);

        app.renderers.push(Box::new(MultiInstanceMeshRenderer {}));

        app.world.singletons.insert(tile_map_data);
    }
}
