use std::{fmt::Debug, num::NonZeroU32};

use image::GenericImageView;

use crate::plugins::core::render_plugin::Gpu;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texture")
            .field("texture", &self.texture)
            .field("view", &self.view)
            .field("sampler", &self.sampler)
            .finish()
    }
}

impl Texture {
    pub fn from_bytes(gpu: &Gpu, bytes: &[u8], label: &str) -> Result<Self, ()> {
        let img = image::load_from_memory(bytes).unwrap();
        Self::from_image(gpu, &img, label)
    }

    pub fn from_image(gpu: &Gpu, img: &image::DynamicImage, label: &str) -> Result<Self, ()> {
        let rgba = img.to_rgba8();
        let dimension = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimension.0,
            height: dimension.1,
            depth_or_array_layers: 1,
        };

        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
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
            bytes_per_row: NonZeroU32::new(4 * dimension.0),
            rows_per_image: NonZeroU32::new(dimension.1),
        };

        gpu.queue
            .write_texture(image_copy_texture, &rgba, image_data_layout, size);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }
}
