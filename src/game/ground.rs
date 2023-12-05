use std::sync::Arc;

use crate::{
    app::Plugin,
    ecs::world::World,
    math::{
        honeycomb::{Hexter, SpiralLoop},
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

        let range = 3;

        for hexter in SpiralLoop::new(Hexter::new(0, 0), range) {
            let [x, y] = hexter.to_vector(tile_size.x * 0.5);

            let mut r = (hexter.q as f32) * 1.0 / range as f32;
            let mut g = (hexter.r as f32) * 1.0 / range as f32;
            let mut b = ((-hexter.q - hexter.r) as f32) * 1.0 / range as f32;

            r = 0.5 * r + 0.5;
            g = 0.5 * g + 0.5;
            b = 0.5 * b + 0.5;
            // let r = (x / tile_size.x / range as f32) * 0.5 + 0.5;
            // let g = (y / tile_size.x / range as f32) * 0.5 + 0.5;
            // let b = ((x + y) / tile_size.x / range as f32) * 0.25 + 0.5;

            let tile_data = TileData::new([x, y], [r, g, b]);

            tile_map.tiles.push(tile_data);
        }

        println!("tiles count: {}", tile_map.tiles.len());

        // app.world.insert_entity((tile_map, transform2d));
        
        app.schedular
            .add_system(crate::app::SystemStage::Update, rotate)
    }
}

pub fn rotate(world: &mut World) {
    let time = world.singletons.get::<Time>().unwrap().delta_time;

    for (_, transform2d) in query_mut!(world, TileMap, Transform2d) { 
        transform2d.rotation += time * 0.5;
    }
}
