use std::sync::Arc;

use crate::{
    app::Plugin,
    math::{
        honeycomb::{Hexter, SpiralLoop},
        transform2d::{self, Transform2d},
        vector2::Vector2,
    },
    plugins::{
        core::render_plugin::Gpu,
        renderer_plugins::{
            mesh::Mesh,
            tilemap_renderer::{TileData, TileMap, TileMapBindGroupLayout},
        },
    },
};

pub struct GroundPlugin;

impl Plugin for GroundPlugin {
    fn build(app: &mut crate::app::App) {
        let bind_group_layout = app
            .world
            .singletons
            .get::<TileMapBindGroupLayout>()
            .unwrap();
        let gpu = app.world.singletons.get::<Gpu>().unwrap();

        let mut tile_map = TileMap::new(
            gpu,
            &bind_group_layout.layout,
            Arc::new(Mesh::get_hex_mesh()),
        );
        let transform2d = Transform2d::IDENTITY;

        let tile_size = Vector2::new(64.0, 64.0);
        tile_map.tile_size = tile_size;

        let range = 5;

        for hexter in SpiralLoop::new(Hexter::new(0, 0), range) {
            let [x, y] = hexter.to_vector(tile_size.x);

            let mut r = (hexter.q as f32) * 1.0 / range as f32;
            let mut g = (hexter.r as f32) * 1.0 / range as f32;
            let mut b = ((-hexter.q - hexter.r) as f32) * 1.0 / range as f32;

            r = 0.5 * r + 0.5;
            g = 0.5 * g + 0.5;
            b = 0.5 * b + 0.5;
            // let r = (x / tile_size.x / range as f32) * 0.5 + 0.5;
            // let g = (y / tile_size.x / range as f32) * 0.5 + 0.5;
            // let b = ((x + y) / tile_size.x / range as f32) * 0.25 + 0.5;

            //Todo: I dont know why I have to multiply 0.5 with coordinates here
            let tile_data = TileData::new([x * 0.5, y * 0.5], [r, g, b]);

            tile_map.tiles.push(tile_data);
        }

        // for x in 0..range {
        //     for y in 0..range {
        //         let tile_data = TileData::new(
        //             [x as f32 * gap, y as f32 * gap],
        //             [x as f32 / range as f32, y as f32 / range as f32, (x + y) as f32 / range as f32 * 2.0],
        //         );
        //         tile_map.tiles.push(tile_data);
        //     }
        // }

        app.world.insert_entity((tile_map, transform2d));
    }
}
