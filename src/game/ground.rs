use std::sync::Arc;

use crate::{
    app::Plugin,
    ecs::world::World,
    math::{
        honeycomb::{Hextor, SpiralLoop},
        transform2d::{self, Transform2d},
        vector2::Vector2,
    },
    plugins::{
        core::{render_plugin::Gpu, timer_plugin::Time},
        renderer_plugins::{
            mesh::Mesh,
            texture::Texture,
            tilemap_renderer::{TileData, TileMap, TileMapBindGroupLayout},
        },
    },
    zip, query_mut,
};

use super::core::level_manager::LevelManager;

pub struct GroundPlugin;

impl Plugin for GroundPlugin {
    fn build(app: &mut crate::app::App) {
        
        let bind_group_layout = app
            .world
            .singletons
            .get::<TileMapBindGroupLayout>()
            .unwrap();

        let gpu = app.world.singletons.get::<Gpu>().unwrap();

        let texture = Texture::from_bytes(gpu, include_bytes!("assets/grass.png"), "my texture")
            .ok()
            .unwrap();

        let mut tile_map = TileMap::new(
            gpu,
            &bind_group_layout.bind_group_layout,
            Arc::new(Mesh::get_hex_mesh()),
            texture,
        );
        let transform2d = Transform2d::IDENTITY;

        let tile_size = Vector2::new(64.0, 64.0) * 2.0;
        tile_map.tile_size = tile_size;


        let level_manager = app.world.singletons.get::<LevelManager>().unwrap();

        for hexter in level_manager.get_tiles() {
            let [x, y] = hexter.to_vector(tile_size.x * 0.5);
            
            let tile_data = TileData::new([x, y], [1.0, 1.0, 1.0]);

            tile_map.tiles.push(tile_data);
        }

        println!("tiles count: {}", tile_map.tiles.len());

        app.world.insert_entity((tile_map, transform2d));
        
    }
}