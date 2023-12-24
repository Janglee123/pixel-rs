use std::sync::Arc;

use crate::{
    app::Plugin,
    ecs::world::World,
    game::core::{level_manager::TilesAddedEvent},
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
    query_mut, zip,
};

use super::core::level_manager::LevelManager;

pub struct GroundPlugin;
pub struct Ground;

impl Plugin for GroundPlugin {
    fn build(app: &mut crate::app::App) {
        let bind_group_layout = app
            .world
            .singletons
            .get::<TileMapBindGroupLayout>()
            .unwrap();

        let gpu = app.world.singletons.get::<Gpu>().unwrap();

        // let texture = Texture::from_bytes(gpu, include_bytes!("assets/grass.png"), "grass texture")
        //     .ok()
        //     .unwrap();

        // let mut tile_map = TileMap::new(
        //     gpu,
        //     &bind_group_layout.bind_group_layout,
        //     Arc::new(Mesh::get_hex_mesh()),
        //     texture,
        // );

        let transform2d = Transform2d::IDENTITY;
        let tile_size = Vector2::new(64.0, 64.0);
        // tile_map.tile_size = tile_size;

        // app.world.insert_entity((tile_map, transform2d, Ground));
        app.world.add_listener::<TilesAddedEvent>(on_tiles_added);

    }
}

pub fn on_tiles_added(world: &mut World, _: &TilesAddedEvent) {
    println!("TILES ADDED EVENT RECEIVED");

    let level_manager = world.singletons.get::<LevelManager>().unwrap();

    let (tile_map, _) = query_mut!(world, TileMap, Ground).next().unwrap();

    tile_map.tiles.clear();

    for hexter in level_manager.get_tiles() {
        let [x, y] = hexter.to_vector(tile_map.tile_size.x * 0.5);

        let tile_data = TileData::new([x, y], [1.0, 1.0, 1.0]);

        tile_map.tiles.push(tile_data);
    }

    println!("tiles count: {}", tile_map.tiles.len());
}
