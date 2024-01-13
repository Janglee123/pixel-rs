use bytemuck::{Pod, Zeroable};
use glam::{Mat3, Vec4};
use hashbrown::HashMap;
use std::{
    ops::{Range, RangeBounds},
    time::{SystemTime, UNIX_EPOCH},
};
use wgpu::{include_wgsl, util::DeviceExt, BindGroupLayout, Buffer, Device, RenderPipeline};
use winit::window::Window;

use crate::{
    app::Plugin,
    ecs::world::{Component, World},
    math::{
        color::Color,
        transform2d::{self, AlignedMatrix, Transform2d},
    },
    plugins::{
        asset_types::image::Image,
        core::{
            asset_storage::AssetRef,
            camera_plugin::{Camera, CameraBindGroup},
            render_plugin::{Gpu, Renderer},
        },
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

    sprite_data_bind_group: wgpu::BindGroup, // Hmm I really need to think about how to write a render stuff
    sprite_data_buffer: wgpu::Buffer,

    texture_id_transform_list_cache: HashMap<u64, Vec<SpriteInstanceData>>,
    sprite_data_list: Vec<SpriteInstanceData>,
    texture_id_range: Vec<TextureDrawData>,
}

struct TextureDrawData {
    range: Range<u32>,
    texture_id: u64,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct SpriteInstanceData {
    color: [f32; 4],
    matrix: AlignedMatrix,  // Todo: instead wasting 8 bytes per instance, I can pack them in array and convert into transform inside shader
    z_index: i32,
    _padding: [u32; 3],
}

impl SpriteInstanceData {
    fn new(color: [f32; 4], matrix: AlignedMatrix, z_index: i32) -> Self {
        Self {
            color,
            matrix,
            z_index,
            _padding: [0; 3],
        }
    }

    const EMPTY: SpriteInstanceData = SpriteInstanceData {
        color: [0.0; 4], // 4
        matrix: AlignedMatrix::IDENTITY, //12
        z_index: 1, //1
        _padding: [0; 3], //3
    };
}

pub struct SpritePlugin;

impl Renderer for SpritePlugin {
    fn render<'pass, 'encoder: 'pass, 'world: 'encoder>(
        &self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        world: &'world World,
    ) {
        let (gpu, camera_data) = world
            .singletons
            .get_many::<(Gpu, CameraBindGroup)>()
            .unwrap();

        let data = world.singletons.get::<SpriteRendererData>().unwrap();

        render_pass.set_pipeline(&data.render_pipeline);
        render_pass.set_vertex_buffer(0, data.vertex_buffer.slice(..));
        render_pass.set_index_buffer(data.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        gpu.queue.write_buffer(
            &data.sprite_data_buffer,
            0,
            bytemuck::cast_slice(&data.sprite_data_list),
        );

        render_pass.set_bind_group(0, &data.sprite_data_bind_group, &[]);
        render_pass.set_bind_group(2, &camera_data.bind_group, &[]);

        for TextureDrawData { range, texture_id } in &data.texture_id_range {
            // println!(
            //     "[Draw call] range: {:?} texture_id: {:?} ",
            //     range, texture_id
            // );

            let texture_group = gpu.texture_bing_group_map.get(texture_id).unwrap();
            render_pass.set_bind_group(1, texture_group, &[]);

            render_pass.draw_indexed(0..6, 0, range.clone());
        }
    }
}

impl Plugin for SpritePlugin {
    fn build(app: &mut crate::app::App) {
        let (gpu, camera_data) = app
            .world
            .singletons
            .get_many::<(Gpu, CameraBindGroup)>()
            .unwrap();

        let shader = gpu
            .device
            .create_shader_module(include_wgsl!("sprite_shader.wgsl"));

        let sprite_data_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Sprite data buffer"),
                contents: bytemuck::cast_slice(
                    &[SpriteInstanceData::EMPTY; 512], // Lets assume there wont be more than 512 instance of
                ),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

        let sprite_data_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("sprite_data_bind_group_layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let sprite_data_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Sprite data bind group"),
            layout: &sprite_data_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: sprite_data_buffer.as_entire_binding(),
            }],
        });

        let render_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &sprite_data_bind_group_layout,
                        &gpu.texture_bind_group_layout,
                        &camera_data.layout,
                    ],
                    push_constant_ranges: &[],
                });

        let render_pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Sprite Render Pipeline"),
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

        let sprite_renderer_data = SpriteRendererData {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            texture_id_transform_list_cache: HashMap::new(),
            sprite_data_bind_group,
            sprite_data_buffer,
            sprite_data_list: Vec::new(),
            texture_id_range: Vec::new(),
        };

        app.renderers.push(Box::new(SpritePlugin {}));

        app.world.singletons.insert(sprite_renderer_data);

        app.world.register_component::<Sprite>();

        app.schedular
            .add_system(crate::app::SystemStage::PreRender, update_cache)
    }
}

pub fn update_cache(world: &mut World) {
    let data = world.singletons.get_mut::<SpriteRendererData>().unwrap();

    // Danger used std::mem::take
    let mut sprite_data_list = std::mem::take(&mut data.sprite_data_list);
    let mut texture_id_range = std::mem::take(&mut data.texture_id_range);
    let mut map = std::mem::take(&mut data.texture_id_transform_list_cache);

    for (_, lists) in map.iter_mut() {
        lists.clear();
    }

    // Todo: Culling
    for (transform2d, sprite) in query!(world, Transform2d, Sprite) {
        // Hmm So sprite has reference to texture
        let texture_id = sprite.image.get_id();

        if let None = map.get_mut(&texture_id) {
            map.insert(texture_id, Vec::new());
        }

        let sprite_data = SpriteInstanceData::new(
            sprite.color.into(),
            AlignedMatrix::from_transform(transform2d),
            sprite.z_index,
        );

        map.get_mut(&texture_id)
            .expect("No texture id in map")
            .push(sprite_data);
    }

    sprite_data_list.clear();
    texture_id_range.clear();

    for (texture_id, list) in &mut map {
        let start_length = sprite_data_list.len();
        sprite_data_list.append(list);
        let end_length = sprite_data_list.len();

        let draw_data = TextureDrawData {
            range: start_length as u32..end_length as u32,
            texture_id: *texture_id,
        };
        texture_id_range.push(draw_data);
    }

    // God damn borrow checker
    let data = world.singletons.get_mut::<SpriteRendererData>().unwrap();
    data.texture_id_transform_list_cache = map;
    data.sprite_data_list = sprite_data_list;
    data.texture_id_range = texture_id_range;
}

pub struct Sprite {
    image: AssetRef<Image>,
    color: Color,
    z_index: i32,
}

impl Sprite {
    pub fn new(image: AssetRef<Image>, color: Color, z_index: i32) -> Self {
        Self {
            image,
            color,
            z_index,
        }
    }
}
