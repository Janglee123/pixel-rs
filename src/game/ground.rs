use std::{env, path::Path, sync::Arc};

use glam::Vec2;

use crate::{
    app::Plugin,
    ecs::world::World,
    game::core::level_manager::TilesAddedEvent,
    math::{
        color::Color,
        honeycomb::{Hextor, SpiralLoop},
        transform2d::{self, Transform2d},
    },
    plugins::{
        asset_types::image::Image,
        core::{asset_storage::AssetStorage, render_plugin::Gpu, timer_plugin::Time},
        renderer_plugins::{
            mesh::Mesh,
            sprite_renderer::Sprite,
            texture::Texture,
            tilemap_renderer::{TileData, TileMap},
            // tilemap_renderer::{TileData, TileMap, TileMapBindGroupLayout},
        },
    },
    query_mut, zip,
};

use super::core::level_manager::LevelManager;

pub struct GroundPlugin;
pub struct Ground;

impl Plugin for GroundPlugin {
    fn build(app: &mut crate::app::App) {
        let (asset_storage, gpu) = app
            .world
            .singletons
            .get_many_mut::<(AssetStorage, Gpu)>()
            .unwrap();

        // Todo: Find out how to deal with paths
        let grass_texture = asset_storage
            .get::<Image>(
                "/mnt/09cbb5c3-3c84-4ea4-b328-254e96041faf/pixel-rs/src/game/assets/grass.png",
            )
            .unwrap();

        let road_texture = asset_storage
            .get::<Image>(
                "/mnt/09cbb5c3-3c84-4ea4-b328-254e96041faf/pixel-rs/src/game/assets/road.png",
            )
            .unwrap();

        let data = asset_storage.get_data(&road_texture).get_data().clone();
        let grass_texture_data = asset_storage.get_data(&grass_texture).get_data().clone();

        gpu.create_texture(road_texture.get_id(), "road", &data, 64, 9);
        gpu.create_texture(grass_texture.get_id(), "grass", &grass_texture_data, 86, 86);

        let tile_map = TileMap::new(Vec2::new(64.0, 64.0), grass_texture);
        let transform2d = Transform2d::IDENTITY;

        app.world.register_component::<Ground>();

        app.world.insert_entity((tile_map, transform2d, Ground));

        let tile_map = TileMap::new(Vec2::new(64.0, 64.0), road_texture);
        let mut transform2d = Transform2d::IDENTITY;
        transform2d.position += Vec2::splat(128.0);

        app.world.insert_entity((tile_map, transform2d, Ground));

        app.world.add_listener::<TilesAddedEvent>(on_tiles_added);
    }
}

pub fn on_tiles_added(world: &mut World, _: &TilesAddedEvent) {
    println!("TILES ADDED EVENT RECEIVED");

    let level_manager = world.singletons.get::<LevelManager>().unwrap();

    for (tile_map, _) in query_mut!(world, TileMap, Ground) {
        tile_map.tiles.clear();

        for hexter in level_manager.get_tiles() {
            let [x, y] = hexter.to_vector(tile_map.tile_size.x * 0.5);

            let tile_data = TileData::new([x, y], [1.0, 1.0, 1.0, 1.0]);

            tile_map.tiles.push(tile_data);
        }

        println!("tiles count: {}", tile_map.tiles.len());
    }
}
