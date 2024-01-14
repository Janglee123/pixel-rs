use glam::UVec2;
use image::GenericImageView;

use crate::plugins::core::asset_storage::Asset;

#[derive(Debug)]
pub struct Image {
    data: Vec<u8>,
    size: UVec2,
}

impl Asset for Image {
    fn from_binary(binary: Vec<u8>) -> Self {
        let img = image::load_from_memory(&binary).unwrap();
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let data = rgba.into_raw();
        let size = UVec2::new(dimensions.0, dimensions.1);

        Self { data, size }
    }
}

impl Image {
    pub fn new(size: UVec2, data: Vec<u8>) -> Self {
        Self { size, data }
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }
}
