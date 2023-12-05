use std::sync::Arc;

use hashbrown::HashMap;

use crate::{
    app::Plugin,
    math::{
        honeycomb::Hexter,
        transform2d::{self, Transform2d},
        vector2::Vector2,
    },
    plugins::{
        core::render_plugin::{Gpu, Renderer},
        renderer_plugins::{
            mesh::Mesh,
            multi_instance_mesh_renderer::{
                InstanceData, MultiInstanceMesh, MultiInstanceMeshBindGroupLayout,
            },
            texture::Texture,
        },
    },
};

pub struct RoadPlugin;

pub struct RoadTileData {
    pub tile_pos: Hexter,
    pub dir: Vec<u8>,
}

pub struct Roads {
    roads: HashMap<Hexter, RoadTileData>,
}

impl Roads {
    pub fn new() -> Self {
        Self {
            roads: HashMap::new(),
        }
    }
}

impl Plugin for RoadPlugin {
    fn build(app: &mut crate::app::App) {
        app.world.singletons.insert(Roads::new());

        let gpu = app.world.singletons.get::<Gpu>().unwrap();

        let texture = Texture::from_bytes(gpu, include_bytes!("assets/grass.png"), "my texture")
            .ok()
            .unwrap();

        let bind_group_layout = app
            .world
            .singletons
            .get::<MultiInstanceMeshBindGroupLayout>()
            .unwrap();

        let mut multi_mesh = MultiInstanceMesh::new(
            gpu,
            &bind_group_layout,
            Arc::new(Mesh::get_quad_mesh()),
            texture,
        );

        for x in 0..10 {
            let x = (x as f32 - 5.0) * 64.0;

            for y in 0..10 {
                let y = (y as f32 - 5.0) * 64.0;

                let transform2d = Transform2d::new(
                    Vector2::new(x, y),
                    x + y,
                    Vector2::new((32.0 + x * 0.5).abs(), (32.0 + y * 0.5).abs()), // Vector2::new(64.0, 64.0)
                );

                let color = [x, y, x - y];

                multi_mesh
                    .instances
                    .push(InstanceData::new(&transform2d, color));
            }
        }

        let transform2d = Transform2d::IDENTITY;

        app.world.insert_entity((multi_mesh, transform2d));
    }
}
