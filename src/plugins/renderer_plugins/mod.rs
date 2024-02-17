use crate::app::Plugin;

use self::{
    mesh::Mesh, 
    multi_instance_mesh_renderer::MultiInstanceMeshRenderer,
    sprite_renderer::SpritePlugin, 
    tilemap_renderer::TileMapRenderer,
};

use super::core::{
    asset_storage::{self, AssetRef, AssetStorage},
    camera_plugin::CameraPlugin,
    render_plugin::RenderPlugin,
};

pub mod mesh;
pub mod multi_instance_mesh_renderer;
pub mod sprite_renderer;
pub mod texture;
pub mod tilemap_renderer;
pub mod vertex;

pub struct Renderer2dPlugin;

impl Plugin for Renderer2dPlugin {
    fn build(app: &mut crate::app::App) {
        app.register_plugin::<TileMapRenderer>();
        app.register_plugin::<SpritePlugin>();
        app.register_plugin::<MultiInstanceMeshRenderer>();

        let asset_storage = app.storage.singletons.get_mut::<AssetStorage>().unwrap();
        asset_storage.insert(Mesh::get_hex_mesh(), "hex_mesh");
        asset_storage.insert(Mesh::get_quad_mesh(), "quad_mesh");
    }
}
