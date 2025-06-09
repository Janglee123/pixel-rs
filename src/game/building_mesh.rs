use std::ops::Range;

use bytemuck::{Pod, Zeroable};
use wgpu::{include_wgsl, util::DeviceExt, Buffer};

use crate::{
    app::Plugin,
    ecs::{singletons::Singletons, world::World},
    math::{color::Color, transform2d::Transform2d},
    plugins::core::{
        camera_plugin::CameraBindGroup,
        render_plugin::{Gpu, Renderer},
    },
    storage::{self, Storage},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct BuildingMeshVertex {
    pub position: [f32; 3],
}

impl BuildingMeshVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x3];

    pub fn decs<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BuildingMeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

struct BuildingRendererData {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    render_pipeline: wgpu::RenderPipeline,
    buildings_data_buffer: Buffer,
    building_data_bind_group: wgpu::BindGroup,
    building_data_bind_group_layout: wgpu::BindGroupLayout,

    vertices: Vec<BuildingMeshVertex>,
    indices: Vec<u32>,
    instance_data: Vec<BuildingInstanceData>,
    instance_index: Vec<BuildingInstanceIndex>,
}

pub struct BuildingMesh {
    pub vertices: Vec<BuildingMeshVertex>,
    pub indices: Vec<u32>,
    pub color: Color,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable, Default)]
pub struct BuildingInstanceData {
    pub color: [f32; 4],
}

pub struct BuildingInstanceIndex {
    instance_data_index: u32,
    indices_range: Range<u32>,
}

pub struct BuildingRenderPlugin;

impl Plugin for BuildingRenderPlugin {
    fn build(app: &mut crate::app::App) {
        let (gpu, camera_data) = app
            .storage
            .singletons
            .get_many_mut::<(Gpu, CameraBindGroup)>()
            .unwrap();

        let shader = gpu
            .device
            .create_shader_module(include_wgsl!("building_shader.wgsl"));

        let buildings_data_buffer =
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Building data buffer"),
                    contents: bytemuck::cast_slice(
                        &[BuildingInstanceData::default(); 1000], // Lets assume there wont be more than 512 instance of
                    ),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                });

        let building_data_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("building_data_bind_group_layout"),
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

        let building_data_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Sprite data bind group"),
            layout: &building_data_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buildings_data_buffer.as_entire_binding(),
            }],
        });

        let render_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&camera_data.layout, &building_data_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("building Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[BuildingMeshVertex::decs()],
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
                    cull_mode: None,
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
                contents: bytemuck::cast_slice(&[0; 1024]),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let index_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[0; 1024 * 3]),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });

        let building_renderer_data = BuildingRendererData {
            vertex_buffer,
            index_buffer,
            render_pipeline,
            buildings_data_buffer,
            building_data_bind_group,
            building_data_bind_group_layout,
            vertices: Vec::new(),
            indices: Vec::new(),
            instance_data: Vec::new(),
            instance_index: Vec::new(),
        };

        app.storage.singletons.insert(building_renderer_data);
        app.renderers.push(Box::new(BuildingRenderPlugin));

        app.schedular
            .add_system(crate::app::SystemStage::PreRender, prepare_renderer_data);
    }
}

fn prepare_renderer_data(storage: &mut Storage) {
    let mut list = Vec::new();

    list.push(BuildingMesh {
        vertices: vec![
            BuildingMeshVertex {
                position: [0.5, 0.5, 1.0],
            },
            BuildingMeshVertex {
                position: [-0.5, 0.5, 1.0],
            },
            BuildingMeshVertex {
                position: [-0.5, -0.5, 1.0],
            },
            BuildingMeshVertex {
                position: [0.5, -0.5, 1.0],
            },
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
        color: Color::new(0.3, 0.4, 0.6, 1.0),
    });

    let (render_data,) = storage
        .singletons
        .get_many_mut::<(BuildingRendererData,)>()
        .unwrap();

    render_data.vertices.clear();
    render_data.indices.clear();
    render_data.instance_data.clear();
    render_data.instance_index.clear();

    for building in list.iter_mut() {
        let offset = render_data.vertices.len() as u32;
        let indices_len = render_data.indices.len();
        let index_range = indices_len as u32..(indices_len + building.indices.len()) as u32;

        render_data.vertices.append(&mut building.vertices);
        render_data
            .indices
            .append(&mut building.indices.iter().map(|x| *x + offset).collect());

        let building_instance_data = BuildingInstanceData {
            color: building.color.into(),
        };

        render_data.instance_data.push(building_instance_data);

        let color_index = render_data.instance_data.len() - 1;

        let index_data = BuildingInstanceIndex {
            indices_range: index_range,
            instance_data_index: color_index as u32,
        };

        render_data.instance_index.push(index_data);

    }
}

impl Renderer for BuildingRenderPlugin {
    fn render<'pass, 'encoder: 'pass, 'world: 'encoder>(
        &self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        world: &'world World,
        singletons: &'world Singletons,
    ) {
        let (gpu, camera_data) = singletons.get_many::<(Gpu, CameraBindGroup)>().unwrap();

        let data = singletons.get::<BuildingRendererData>().unwrap();

        render_pass.set_pipeline(&data.render_pipeline);
        render_pass.set_vertex_buffer(0, data.vertex_buffer.slice(..));
        render_pass.set_index_buffer(data.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        gpu.queue.write_buffer(
            &data.buildings_data_buffer,
            0,
            bytemuck::cast_slice(&data.instance_data),
        );

        gpu.queue
            .write_buffer(&data.index_buffer, 0, bytemuck::cast_slice(&data.indices));

        println!("{:?}", data.vertices);

        gpu.queue
            .write_buffer(&data.vertex_buffer, 0, bytemuck::cast_slice(&data.vertices));

        render_pass.set_bind_group(0, &camera_data.bind_group, &[]);
        render_pass.set_bind_group(1, &data.building_data_bind_group, &[]);

        println!("index {} vertex {} index data {}", data.indices.len(), data.vertices.len(), data.instance_index.len());

        for index_data in &data.instance_index {
            render_pass.draw_indexed(0..6, 0, 0..1);
        }
    }
}
