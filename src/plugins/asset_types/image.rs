use image::GenericImageView;

use crate::{math::vector2::Vector2, plugins::core::asset_storage::Asset};

pub struct Image {
    data: Vec<u8>,
    size: Vector2<u32>,
}

impl Asset for Image {
    fn from_binary(binary: Vec<u8>) -> Self {
        let img = image::load_from_memory(&binary).unwrap();
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let data = rgba.into_raw();
        let size = Vector2::new(dimensions.0, dimensions.1);

        Self { data, size }
    }
}

impl Image {
    pub fn new(size: Vector2<u32>, data: Vec<u8>) -> Self {
        Self { size, data }
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }
}
