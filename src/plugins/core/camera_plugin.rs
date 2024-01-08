use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer};
use winit::window::Window;

use crate::{
    app::{Plugin, SystemStage},
    ecs::world::{self, World},
    math::{
        transform2d::{Matrix3, Transform2d},
        vector2::Vector2,
    },
    query, query_mut, zip,
};

use super::render_plugin::{Gpu, Renderer};

pub struct Camera {
    pub projection: Matrix3,
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
                projection: Matrix3::IDENTITY,
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

        let data = CameraBindGroup {
            layout: camera_bind_group_layout,
            bind_group: camera_bind_group,
            buffer: camera_buffer,
        };

        app.world.singletons.insert(data);
        app.schedular.add_system(SystemStage::Resize, on_resize);
        app.renderers.push(Box::new(CameraPlugin));
    }
}

impl Renderer for CameraPlugin {
    fn render<'pass, 'encoder: 'pass, 'world: 'encoder>(
        &self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        world: &'world World,
    ) {
        let (gpu, data) = world
            .singletons
            .get_many::<(Gpu, CameraBindGroup)>()
            .unwrap();

        let (camera, transform2d) = query!(world, Camera, Transform2d).next().unwrap();
        let projection = transform2d.create_matrix() * camera.projection;

        gpu.queue
            .write_buffer(&data.buffer, 0, bytemuck::cast_slice(&[projection]));
    }
}

pub fn on_resize(world: &mut World) {
    let size = world.singletons.get::<Window>().unwrap().inner_size();

    let camera = query_mut!(world, Camera).next().unwrap();

    camera.projection.x[0] = 2.0 / size.width as f32;
    camera.projection.y[1] = 2.0 / size.height as f32;
}
