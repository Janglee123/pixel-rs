use bytemuck::{Pod, Zeroable};
use std::time::{SystemTime, UNIX_EPOCH};
use wgpu::{include_wgsl, util::DeviceExt, Buffer, RenderPipeline};

use crate::{
    app::Plugin,
    ecs::world::{Component, World},
    math::{
        transform2d::{self, Transform2d},
        vector2::Vector2,
    },
    query, zip,
};

use super::{core::render_plugin::Gpu, renderer_plugins::vertex::Vertex};

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

pub struct TriangleRendererData {
    pub render_pipeline: RenderPipeline,
    pub vertex_buffer: Buffer,
    index_buffer: Buffer,
    transform_buffer: Buffer,
    transform_bind_group: wgpu::BindGroup,
}

pub struct Quad;

pub struct TrianglePlugin;

impl Plugin for TrianglePlugin {
    fn build(app: &mut crate::app::App) {
        let gpu = app.world.singletons.get::<Gpu>().unwrap();

        let shader = gpu
            .device
            .create_shader_module(include_wgsl!("shader.wgsl"));

        let transform_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[Transform2d::IDENTITY.into_matrix()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

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

        let transform_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Transform buffer"),
            layout: &transform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
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

        let triangle_renderer_data = TriangleRendererData {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            transform_buffer,
            transform_bind_group,
        };

        let mut transform2d = Transform2d {
            position: Vector2 { x: 0.5, y: 0.5 },
            rotation: 0.1,
            scale: Vector2 { x: 0.2, y: 0.2 },
        };

        app.world.insert_entity((transform2d, Quad));

        app.world.singletons.insert(triangle_renderer_data);
        app.schedular
            .add_system(crate::app::SystemStage::Update, draw);
    }
}

pub fn draw(world: &mut World) {
    // let (transform2d, _) = query!(world, Transform2d, Quad).next().unwrap();

    let gpu = world.singletons.get::<Gpu>().unwrap();
    let data = world.singletons.get::<TriangleRendererData>().unwrap();

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
    render_pass.set_bind_group(0, &data.transform_bind_group, &[]);
    render_pass.set_vertex_buffer(0, data.vertex_buffer.slice(..));
    render_pass.set_index_buffer(data.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

    // let start = SystemTime::now();
    // let since_the_epoch = start
    //     .duration_since(UNIX_EPOCH)
    //     .expect("Time went backwards");

    // println!("{:?}", start);
    // println!(
    //     "{:?} {:?}",
    //     since_the_epoch,
    //     since_the_epoch.as_secs_f64().cos()
    // );

    // let mut trans = transform2d.clone();
    // trans.rotation = since_the_epoch.as_secs_f64().cos() as f32;
    // // println!("{:?}", trans.rotation);

    for (transform2d, _) in query!(world, Transform2d, Quad) {
        transform2d.rotation += 0.0001;
        transform2d.position.x += 0.001;
        println!("{:?} {:?}", transform2d, transform2d.into_matrix());

        gpu.queue.write_buffer(
            &data.transform_buffer,
            0,
            bytemuck::cast_slice(&[transform2d.into_matrix()]),
        );

        render_pass.draw_indexed(0..6, 0, 0..1);
    }

    drop(render_pass);
    gpu.queue.submit(std::iter::once(encoder.finish()));

    output.present();
}
