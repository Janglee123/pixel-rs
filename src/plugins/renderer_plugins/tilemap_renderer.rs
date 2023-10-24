use wgpu::{include_wgsl, util::DeviceExt};
use crate::BitSet;
use std::any::{Any, TypeId};

use crate::{
    app::Plugin,
    ecs::world::{Component, World},
    math::{color::Color, vector2::Vector2},
    plugins::core::render_plugin::Gpu,
    query, zip
};

use super::vertex::Vertex;

const QUAD: &[Vertex] = &[
    Vertex {
        position: [1.0, 0.0, 0.0],
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

pub struct TileData {
    pos: Vector2<i32>,
    color: Color,
}

pub struct TileMap {
    pub tiles: Vec<TileData>,
    pub tile_size: Vector2<i32>,
}

pub struct TileMapRendererData {}

pub struct TileMapRenderer;

impl Plugin for TileMapRenderer {
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

        // let vertex_buffer = gpu
        //     .device
        //     .create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //         label: Some("Vertex Buffer"),
        //         contents: bytemuck::cast_slice(VERTICES),
        //         usage: wgpu::BufferUsages::VERTEX,
        //     });

        app.world.singletons.insert(TileMapRendererData {});
        app.schedular.add_system(1, draw);
    }
}

pub fn draw(world: &mut World) {
    for tile_map in query!(world, TileMap) {
        // Set Pipeline
        // Set buffers
        // Call draw with instanced
    }
}
