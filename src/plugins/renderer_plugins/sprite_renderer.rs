use bytemuck::{Pod, Zeroable};
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
        transform2d::{self, Matrix3, Transform2d},
        vector2::Vector2,
    },
    plugins::{
        asset_types::image::Image,
        core::{
            asset_storage::AssetRef,
            camera_plugin::Camera,
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

    camera_bind_group: wgpu::BindGroup,
    camera_buffer: Buffer,

    texture_id_transform_list_cache: HashMap<u64, Vec<SpriteData>>,
    sprite_data_list: Vec<SpriteData>,
    texture_id_range: Vec<TextureDrawData>,
}

struct TextureDrawData {
    range: Range<u32>,
    texture_id: u64,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct SpriteData {
    color: [f32; 4],
    matrix: Matrix3,
    z_index: i32,
    _padding: [u32; 3],
}

impl SpriteData {
    fn new(color: [f32; 4], matrix: Matrix3, z_index: i32) -> Self {
        Self {
            color,
            matrix,
            z_index,
            _padding: [0; 3],
        }
    }

    const EMPTY: SpriteData = SpriteData {
        color: [0.0; 4],
        matrix: Matrix3::IDENTITY,
        z_index: 1,
        _padding: [0; 3],
    };
}

pub struct Quad {
    transform_buffer: Buffer,
    transform_bind_group: wgpu::BindGroup,
    texture_bind_group: wgpu::BindGroup,
    texture: texture::Texture, //Todo: Asset manager
    size: Vector2<u32>,
}

impl Quad {
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

        gpu.queue.write_buffer(
            &data.sprite_data_buffer,
            0,
            bytemuck::cast_slice(&data.sprite_data_list),
        );

        render_pass.set_bind_group(0, &data.sprite_data_bind_group, &[]);
        render_pass.set_bind_group(2, &data.camera_bind_group, &[]);

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
        let gpu = app.world.singletons.get::<Gpu>().unwrap();

        let shader = gpu
            .device
            .create_shader_module(include_wgsl!("sprite_shader.wgsl"));

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

        let sprite_data_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Sprite data buffer"),
                contents: bytemuck::cast_slice(
                    &[SpriteData::EMPTY; 512], // Lets assume there wont be more than 512 instance of
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
                        &camera_bind_group_layout,
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
            camera_bind_group,
            camera_buffer,
            texture_id_transform_list_cache: HashMap::new(),
            sprite_data_bind_group,
            sprite_data_buffer,
            sprite_data_list: Vec::new(),
            texture_id_range: Vec::new(),
        };

        app.renderers.push(Box::new(SpritePlugin {}));

        app.world.register_component::<Quad>();
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

        let sprite_data = SpriteData::new(sprite.color.into(), transform2d.create_matrix(), sprite.z_index);

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
