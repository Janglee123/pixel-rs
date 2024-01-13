use std::default;

use glam::{Mat3, Vec2, Vec3};
use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer};
use winit::window::Window;

use crate::{
    app::{Plugin, SystemStage},
    ecs::world::{self, World},
    math::transform2d::{AlignedMatrix, Transform2d},
    query, query_mut, zip,
};

use super::render_plugin::{Gpu, Renderer};

pub struct Camera {
    pub projection: Mat3,
}

pub struct CameraBindGroup {
    pub layout: BindGroupLayout,
    pub bind_group: BindGroup,
    buffer: Buffer,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(app: &mut crate::app::App) {
        // First I need to know that its -1 to 1 or -0.5 to 0.5 I know that center is zero zero
        app.world.insert_entity((
            Camera {
                projection: Mat3::IDENTITY,
            },
            Transform2d::IDENTITY,
        ));

        let gpu = app.world.singletons.get::<Gpu>().unwrap();

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
                contents: bytemuck::cast_slice(&[AlignedMatrix::IDENTITY]),
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

        let data = CameraBindGroup {
            layout: camera_bind_group_layout,
            bind_group: camera_bind_group,
            buffer: camera_buffer,
        };

        app.world.singletons.insert(Viewport::default());
        app.world.singletons.insert(data);
        app.schedular.add_system(SystemStage::Resize, on_resize);
        app.schedular.add_system(SystemStage::PreRender, on_update);
        app.renderers.push(Box::new(CameraPlugin));
    }
}

impl Renderer for CameraPlugin {
    fn render<'pass, 'encoder: 'pass, 'world: 'encoder>(
        &self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        world: &'world World,
    ) {
        let (gpu, data, viewport) = world
            .singletons
            .get_many::<(Gpu, CameraBindGroup, Viewport)>()
            .unwrap();

        gpu.queue.write_buffer(
            &data.buffer,
            0,
            bytemuck::cast_slice(&[AlignedMatrix::from_mat3(&viewport.projection_view_mat)]),
        );
    }
}

pub fn on_resize(world: &mut World) {
    let size = world.singletons.get::<Window>().unwrap().inner_size();

    let camera = query_mut!(world, Camera).next().unwrap();

    camera.projection.x_axis.x = 2.0 / size.width as f32;
    camera.projection.y_axis.y = 2.0 / size.height as f32;
}

#[derive(Debug, Default)]
pub struct Viewport {
    projection_view_mat: Mat3,
    inv_projection_view_mat: Mat3,
}

impl Viewport {
    /// converts screen pos to world pos based on orientation of camera.
    /// NOTE: `screen_pos` must be in range of `-0.5` to `0.5` for `x` and `y`.
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let a = self.inv_projection_view_mat * Vec3::new(screen_pos.x, screen_pos.y, 0.0);

        Vec2::new(a.x, a.y)
    }

    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let a = self.projection_view_mat * Vec3::new(world_pos.x, world_pos.y, 0.0);

        Vec2::new(a.x, a.y)
    }
}

pub fn on_update(world: &mut World) {
    let size = world.singletons.get::<Window>().unwrap().inner_size();

    let (camera, transform2d) = query!(world, Camera, Transform2d).next().unwrap();
    let projection = camera.projection * transform2d.create_matrix();

    let viewport = world.singletons.get_mut::<Viewport>().unwrap();
    viewport.inv_projection_view_mat = projection.inverse();
    viewport.projection_view_mat = projection; // but this is camera projection I want world projection hmm
}
