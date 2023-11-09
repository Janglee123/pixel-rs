use crate::{
    app::Plugin,
    math::{
        transform2d::{self, Transform2d},
        vector2::Vector2,
    },
    plugins::{
        core::render_plugin::Gpu,
        renderer_plugins::tilemap_renderer::{TileData, TileMap, TileMapBindGroupLayout},
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

        let mut tile_map = TileMap::new(gpu, &bind_group_layout.layout);
        let transform2d = Transform2d::IDENTITY;

        let tile_size = Vector2::new(32.0, 32.0);
        tile_map.tile_size = tile_size;

        let range = 10;

        let gap = 32.0;

        for x in 0..range {
            for y in 0..range {
                let tile_data = TileData::new(
                    [x as f32 * gap, y as f32 * gap],
                    [x as f32 / range as f32, y as f32 / range as f32, (x + y) as f32 / range as f32 * 2.0],
                );
                tile_map.tiles.push(tile_data);
            }
        }

        app.world.insert_entity((tile_map, transform2d));
    }
}
