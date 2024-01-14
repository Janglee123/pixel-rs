use crate::{
    math::{
        honeycomb::HEXAGON_INDICES,
        transform2d::{self, AlignedMatrix, Transform2d},
    },
    plugins::{
        asset_types::image::Image,
        core::{
            asset_storage::AssetRef,
            camera_plugin::{Camera, CameraBindGroup},
            render_plugin::Renderer,
        },
    },
};

use glam::Vec2;
use std::{
    any::{Any, TypeId},
    borrow::BorrowMut,
    mem,
    ops::Deref,
    rc::Rc,
    sync::Arc, num::NonZeroU64,
};
use wgpu::{include_wgsl, util::DeviceExt, BindGroup, BindGroupLayout, RenderPass, RenderPipeline, BindingResource, BufferBinding};

use crate::{
    app::Plugin,
    ecs::world::{Component, World},
    math::color::Color,
    plugins::core::render_plugin::Gpu,
    query, query_mut, zip,
};

use super::{mesh::Mesh, texture::Texture, vertex::Vertex};

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.5, 0.5, 1.0],
        color: [0.5, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, 1.0],
        color: [0.0, 0.5, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 1.0],
        color: [0.0, 0.0, 0.5],
    },
    Vertex {
        position: [0.5, -0.5, 1.0],
        color: [0.0, 0.0, 0.0],
    },
];

const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct TileData {
    pos: [f32; 2],
    _padding: [f32; 2],
    color: [f32; 4],
}

impl TileData {
    pub fn new(pos: [f32; 2], color: [f32; 4]) -> Self {
        Self {
            pos,
            _padding: [0.0; 2],
            color,
        }
    }
}

#[derive(Debug)]
pub struct TileMap {
    pub tiles: Vec<TileData>,
    pub tile_size: Vec2,
    pub texture: AssetRef<Image>,
}

impl TileMap {
    pub fn new(tile_size: Vec2, texture: AssetRef<Image>) -> Self {
        Self {
            tiles: Vec::new(),
            tile_size,
            texture,
        }
    }
}

pub struct TileMapRendererData {
    render_pipeline: RenderPipeline,
    tile_map_data_bind_group: BindGroup,
    tile_map_data_buffer: wgpu::Buffer,
    buffer_offset_list: Vec<u32>,
    data_list: Vec<TileMapData>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
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
        let camera_data = world.singletons.get::<CameraBindGroup>().unwrap();

        render_pass.set_pipeline(&data.render_pipeline);

        render_pass.set_vertex_buffer(0, data.vertex_buffer.slice(..));
        render_pass.set_index_buffer(data.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.set_bind_group(2, &camera_data.bind_group, &[]);

        for TileMapData {
            texture_id,
            tile_count,
            offset,
        } in &data.data_list
        {
            render_pass.set_bind_group(0, &data.tile_map_data_bind_group, &[*offset]);

            let texture_group = gpu.texture_bing_group_map.get(texture_id).unwrap();
            render_pass.set_bind_group(1, texture_group, &[]);

            render_pass.draw_indexed(0..6, 0, 0..tile_count.clone() as u32);
        }
    }
}

impl Plugin for TileMapRenderer {
    fn build(app: &mut crate::app::App) {
        let (gpu, camera_bind_group) = app
            .world
            .singletons
            .get_many::<(Gpu, CameraBindGroup)>()
            .unwrap();

        let shader = gpu
            .device
            .create_shader_module(include_wgsl!("tilemap_shader.wgsl"));

        let tile_map_data_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("tilemap_bind_group_layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: true,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let tile_map_data_buffer =
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("tilemap_data_buffer"),
                    contents: bytemuck::cast_slice(&[0u8; 4096]),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                });

        let buffer_binding = BufferBinding {
            buffer: &tile_map_data_buffer,
            offset: 0,
            size: Some(NonZeroU64::new(1024).unwrap()),
        };

        let tile_map_data_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &tile_map_data_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(buffer_binding),
            }],
        });

        let render_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &tile_map_data_bind_group_layout,
                        &gpu.texture_bind_group_layout,
                        &camera_bind_group.layout,
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

        let tile_map_data = TileMapRendererData {
            render_pipeline,
            tile_map_data_bind_group,
            tile_map_data_buffer,
            buffer_offset_list: Vec::new(),
            data_list: Vec::new(),

            vertex_buffer,
            index_buffer,
        };

        app.world.register_component::<TileMap>();

        app.renderers.push(Box::new(TileMapRenderer {}));

        app.world.singletons.insert(tile_map_data);
        app.schedular
            .add_system(crate::app::SystemStage::PreRender, prepare_tilemap_data);
    }
}

struct TileMapData {
    texture_id: u64,
    tile_count: u64,
    offset: u32,
}

fn prepare_tilemap_data(world: &mut World) {
    let mut offset = 0u32;

    let mut offset_list = vec![];
    let mut data_list = vec![];

    // Note: This is coping data of Vec<TileData> into another vec and that might be expensive thing to do
    // for every tilemap every frame.
    for (tile_map, transform2d) in query!(world, TileMap, Transform2d) {
        offset_list.push(offset);

        data_list.push(TileMapData {
            texture_id: tile_map.texture.get_id(),
            tile_count: tile_map.tiles.len() as u64,
            offset,
        });

        let s = &tile_map.tiles[..];

        let matrix = &[AlignedMatrix::from_transform(transform2d)];

        let [x, y] = tile_map.tile_size.to_array();
        let tile_size = [x, y, 0.0, 0.0];

        let matrix_data: &[u8] = bytemuck::cast_slice(matrix);
        let tile_size_data: &[u8] = bytemuck::cast_slice(&tile_size);
        let tile_data: &[u8] = bytemuck::cast_slice(&tile_map.tiles);

        let data = [matrix_data, tile_size_data, tile_data].concat();

        // I should write this buffer into
        let (gpu, render_data) = world
            .singletons
            .get_many::<(Gpu, TileMapRendererData)>()
            .unwrap();

        // So buffer is written Hurray!
        gpu.queue
            .write_buffer(&render_data.tile_map_data_buffer, offset.into(), &data);

        let size = data.len() as u32;
        offset += gpu.get_aligned_storage_buffer_size(size);
    }

    let render_data = world.singletons.get_mut::<TileMapRendererData>().unwrap();
    render_data.buffer_offset_list = offset_list;
    render_data.data_list = data_list;
}
