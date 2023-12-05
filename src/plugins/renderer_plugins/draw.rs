use wgpu::include_wgsl;

use crate::{
    app::Plugin,
    ecs::world::World,
    math::vector2::{self, Vector2},
    plugins::core::render_plugin::{Gpu, Renderer},
};

pub struct LineShape {
    a: Vector2<f32>,
    b: Vector2<f32>,
}

pub struct RectShape {
    pos: Vector2<f32>,
    size: Vector2<f32>,
}

pub enum Shape {
    Line(LineShape),
    RectShape(RectShape),
}

pub struct Draw {
    pub draw_commands: Vec<Shape>,
}

impl Draw {
    pub fn new() -> Self {
        Self {
            draw_commands: Vec::new(),
        }
    }

    pub fn circle(&mut self, pos: Vector2<f32>, radius: f32) {}

    pub fn line(&mut self, a: Vector2<f32>, b: Vector2<f32>) {}

    pub fn rect(&mut self, pos: Vector2<f32>, size: Vector2<f32>) {
        // store a command then draw it. Okay
    }
}

pub struct DrawPlugin;

impl Plugin for DrawPlugin {
    fn build(app: &mut crate::app::App) {
        let gpu = app.world.singletons.get::<Gpu>().unwrap();
        let shader = gpu
            .device
            .create_shader_module(include_wgsl!("tilemap_shader.wgsl"));

        // Hmm so I just need a one render pipeline that can have many but what about lines?? I need another pipeline for hollow shapes

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
                    buffers: &[],
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

        app.world.singletons.insert(Draw::new());

        app.schedular
            .add_system(crate::app::SystemStage::Render, draw)
    }
}

impl Renderer for DrawPlugin {
    fn render<'pass, 'encoder: 'pass, 'world: 'encoder>(
        &self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        world: &'world World,
    ) {
        let draw = world.singletons.get::<Draw>().unwrap();

        for command in draw.draw_commands.iter() {
            match command {
                Shape::Line(line_shape) => {
                    // I have to set render_pipeline and stuff
                }
                Shape::RectShape(rect_shape) => {}
            }
        }
    }
}

fn draw(world: &mut World) {
    let draw = world.singletons.get_mut::<Draw>().unwrap();

    draw.draw_commands.clear();
}
