use crate::{
    math::{
        honeycomb::HEXAGON_INDICES,
        transform2d::{Matrix3, Transform2d},
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

use super::{mesh::Mesh, vertex::Vertex};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct TileData {
    pos: [f32; 2],
    _padding: [f32; 2],
    color: [f32; 3],
    _padding2: [f32; 1],
}

impl TileData {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3];

    pub fn new(pos: [f32; 2], color: [f32; 3]) -> Self {
        Self {
            pos,
            _padding: [0.0; 2],
            color,
            _padding2: [0.0; 1],
        }
    }

    pub fn decs<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TileData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[derive(Debug)]
pub struct TileMap {
    pub tiles: Vec<TileData>,
    pub tile_size: Vector2<f32>,
    pub transform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub tile_data_buffer: wgpu::Buffer,
    pub tile_size_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub mesh: Arc<Mesh>,
}

impl TileMap {
    pub fn new(gpu: &Gpu, bind_group_layout: &BindGroupLayout, mesh: Arc<Mesh>) -> Self {
        let device = &gpu.device;

        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Transform Buffer"),
            contents: bytemuck::cast_slice(&[Transform2d::IDENTITY.into_matrix()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let tile_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tile size buffer"),
            contents: bytemuck::cast_slice(&[1.0, 1.0]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let tile_data = TileData::new([0.0; 2], [0.0; 3]);

        let tile_data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&[tile_data; 4096]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: transform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: tile_size_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: tile_data_buffer.as_entire_binding(),
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
            tiles: Vec::new(),
            tile_size: Vector2::default(),
            bind_group,
            transform_buffer,
            tile_data_buffer,
            tile_size_buffer,
            vertex_buffer,
            index_buffer,
            mesh,
        }
    }
}

pub struct TileMapRendererData {
    render_pipeline: RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
}

pub struct TileMapRenderer;

impl Renderer for TileMapRenderer {
    fn render<'pass, 'encoder: 'pass, 'world: 'encoder>(
        &self,
        render_pass: &mut RenderPass<'encoder>,
        world: &'world World,
    ) {
        let data = world.singletons.get::<TileMapRendererData>().unwrap();
        let gpu = world.singletons.get::<Gpu>().unwrap();

        let (camera, transform2d) = query!(world, Camera, Transform2d).next().unwrap();
        let projection = transform2d.into_matrix() * camera.projection;

        render_pass.set_pipeline(&data.render_pipeline);

        gpu.queue
            .write_buffer(&data.camera_buffer, 0, bytemuck::cast_slice(&[projection]));
        render_pass.set_bind_group(1, &data.camera_bind_group, &[]);

        for (tile_map, transform2d) in query!(world, TileMap, Transform2d) {
            render_pass.set_vertex_buffer(0, tile_map.vertex_buffer.slice(..));
            render_pass
                .set_index_buffer(tile_map.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            gpu.queue.write_buffer(
                &tile_map.transform_buffer,
                0,
                bytemuck::cast_slice(&[transform2d.into_matrix()]),
            );

            gpu.queue.write_buffer(
                &tile_map.tile_size_buffer,
                0,
                bytemuck::cast_slice(&[tile_map.tile_size.x, tile_map.tile_size.y]),
            );

            gpu.queue.write_buffer(
                &tile_map.tile_data_buffer,
                0,
                bytemuck::cast_slice(&tile_map.tiles),
            );

            render_pass.set_bind_group(0, &tile_map.bind_group, &[]);

            render_pass.draw_indexed(
                0..tile_map.mesh.indices.len() as u32,
                0,
                0..tile_map.tiles.len() as u32,
            );
        }
    }
}

pub struct TileMapBindGroupLayout {
    pub layout: BindGroupLayout,
}

impl Plugin for TileMapRenderer {
    fn build(app: &mut crate::app::App) {
        let gpu = app.world.singletons.get::<Gpu>().unwrap();
        let shader = gpu
            .device
            .create_shader_module(include_wgsl!("tilemap_shader.wgsl"));

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
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
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

        let tile_map_data = TileMapRendererData {
            render_pipeline,
            camera_buffer,
            camera_bind_group,
        };

        // let mut tileMap = TileMap::new(gpu, &bind_group_layout);
        // tileMap.tile_size = Vector2::new(0.1, 0.1);

        // tileMap.tiles = vec![
        //     TileData::new([0.0, 0.0], [1.0, 0.0, 0.0]),
        //     TileData::new([-3.0, 0.0], [1.0, 0.0, 0.0]),
        //     TileData::new([4.0, 4.0], [1.0, 0.0, 0.0]),
        // ];

        // let mut transform2d = Transform2d::IDENTITY;
        // transform2d.scale = Vector2::new(64.0, 64.0);
        // transform2d.position = Vector2::new(0.5, 0.5);

        // app.world.insert_entity((tileMap, transform2d));

        let tile_map_bind_group_layout = TileMapBindGroupLayout {
            layout: bind_group_layout,
        };

        app.world.singletons.insert(tile_map_bind_group_layout);

        app.renderers.push(Box::new(TileMapRenderer {}));

        app.world.singletons.insert(tile_map_data);
    }
}
