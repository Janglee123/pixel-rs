use std::{num::NonZeroU32, rc::Rc, string, sync::Arc};

use bytemuck::{Pod, Zeroable};
use hashbrown::HashMap;
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, CommandEncoder, Device, DeviceDescriptor, Queue, RenderPass,
    RequestAdapterOptions, Surface, SurfaceConfiguration, SurfaceTexture,
};
use winit::window::Window;

use crate::{
    app::{App, Plugin},
    ecs::world::{self, World},
    plugins::renderer_plugins::texture::Texture,
};

pub trait Renderer {
    fn render<'pass, 'encoder: 'pass, 'world: 'encoder>(
        &self,
        render_pass: &mut RenderPass<'encoder>,
        world: &'world World,
    );
}

pub struct Gpu {
    pub surface: Surface,
    pub queue: Queue,
    pub device: Device,
    pub surface_config: wgpu::SurfaceConfiguration,

    pub texture_bing_group_map: HashMap<u64, wgpu::BindGroup>, // Why bind group and not texture buffer??
    pub texture_map: HashMap<u64, Texture>,
    pub texture_bind_group_layout: wgpu::BindGroupLayout, // Sprite render needs this

    draw_index_buffers: [wgpu::Buffer; 512],
    draw_index_bing_group: [wgpu::BindGroup; 512],
    pub draw_index_bind_group_layout: wgpu::BindGroupLayout,
}

impl Gpu {
    pub fn create_texture(&mut self, id: u64, label: &str, data: &[u8], width: u32, height: u32) {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        let image_copy_texture = wgpu::ImageCopyTexture {
            aspect: wgpu::TextureAspect::All,
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        };

        let image_data_layout = wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: NonZeroU32::new(4 * width),
            rows_per_image: NonZeroU32::new(height),
        };

        self.queue
            .write_texture(image_copy_texture, data, image_data_layout, size);

        // Todo: I think there is no need to make it again and again
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture = Texture {
            texture,
            view,
            sampler,
        };

        let texture_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: &self.texture_bind_group_layout,
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

        self.texture_map.insert(id, texture);
        self.texture_bing_group_map.insert(id, texture_bind_group);

        // Now I need to create a bind group
    }

    pub fn get_draw_index_bind_group(&self, index: usize) -> &wgpu::BindGroup {
        &self.draw_index_bing_group[index]
    }
}

pub fn render_function(world: &mut World, renderers: &Vec<Box<dyn Renderer>>) {
    let gpu = world.singletons.get::<Gpu>().unwrap();

    let output = gpu.surface.get_current_texture().unwrap();

    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    let mut render_pass = (encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
    }));

    // dynamic dispatch lol
    for renderer in renderers.iter() {
        renderer.render(&mut render_pass, world);
    }

    drop(render_pass);

    // I need to get gpu again because borrow checker doesn't allow me to use above gpu again
    // How is the turn table borrow checker
    let gpu = world.singletons.get::<Gpu>().unwrap();
    gpu.queue.submit(std::iter::once(encoder.finish()));

    output.present();
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(app: &mut App) {
        let window = app.world.singletons.get::<Window>().unwrap();

        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);

        let surface = unsafe { instance.create_surface(&window) };

        let adapter_options = RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        };

        for adapter in instance.enumerate_adapters(wgpu::Backends::VULKAN) {
            println!("{:?}", adapter.get_info());
        }

        let adapter = pollster::block_on(instance.request_adapter(&adapter_options)).unwrap();

        println!("Selected adapter {:?}", adapter.get_info());

        let device_descriptor = DeviceDescriptor {
            label: None,
            features: wgpu::Features::default()
                | wgpu::Features::POLYGON_MODE_LINE
                | wgpu::Features::POLYGON_MODE_POINT,
            limits: wgpu::Limits::default(), // TODO: learn about this
        };

        let (device, queue) =
            pollster::block_on(adapter.request_device(&device_descriptor, None)).unwrap();

        let format = surface.get_supported_formats(&adapter)[0];

        let surface_config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };

        surface.configure(&device, &surface_config);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let draw_index_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Draw index bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let mut draw_index_buffers = Vec::with_capacity(512);
        let mut draw_index_bing_group = Vec::with_capacity(512);

        for i in 0..512 {
            draw_index_buffers.push(device.create_buffer_init(&BufferInitDescriptor {
                label: Some(format!("Draw index {i} buffer").as_str()),
                contents: bytemuck::cast_slice(&[i]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }));

            draw_index_bing_group.push(device.create_bind_group(&BindGroupDescriptor {
                label: Some(format!("Draw index {i} bind group").as_str()),
                layout: &draw_index_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: draw_index_buffers[i].as_entire_binding(),
                }],
            }));
        }

        let draw_index_buffers = draw_index_buffers.try_into().unwrap();
        let draw_index_bing_group = draw_index_bing_group.try_into().unwrap();

        let gpu = Gpu {
            surface,
            queue,
            device,
            surface_config,
            texture_bing_group_map: HashMap::new(),
            texture_map: HashMap::new(),
            texture_bind_group_layout,

            draw_index_bind_group_layout,
            draw_index_buffers,
            draw_index_bing_group,
        };

        app.set_renderer(render_function);
        app.world.singletons.insert(gpu);
        app.schedular
            .add_system(crate::app::SystemStage::Resize, on_resize)
        // app.schedular.add_system(1, draw);
    }
}

fn on_resize(world: &mut World) {
    let window = world.singletons.get::<Window>().unwrap();
    let size = window.inner_size();

    let gpu = world.singletons.get_mut::<Gpu>().unwrap();
    gpu.surface_config.width = size.width;
    gpu.surface_config.height = size.height;

    gpu.surface.configure(&gpu.device, &gpu.surface_config);
}
