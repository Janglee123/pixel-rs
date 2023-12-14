use std::sync::Arc;

use hashbrown::HashMap;

use crate::{
    app::Plugin,
    math::{
        honeycomb::Hextor,
        transform2d::{self, Transform2d},
        vector2::Vector2,
    },
    plugins::{
        core::render_plugin::{Gpu, Renderer},
        other::tweener::{PositionTweener, ScaleTweener},
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
    pub tile_pos: Hextor,
    pub dir: Vec<u8>,
}

pub struct Roads {
    roads: HashMap<Hextor, RoadTileData>,
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

        // let gpu = app.world.singletons.get::<Gpu>().unwrap();

        // let texture = Texture::from_bytes(gpu, include_bytes!("assets/grass.png"), "my texture")
        //     .ok()
        //     .unwrap();

        // let bind_group_layout = app
        //     .world
        //     .singletons
        //     .get::<MultiInstanceMeshBindGroupLayout>()
        //     .unwrap();

        // let mut multi_mesh = MultiInstanceMesh::new(
        //     gpu,
        //     &bind_group_layout,
        //     Arc::new(Mesh::get_quad_mesh()),
        //     texture,
        // );

        for x in 0..10 {
            let x = (x as f32 - 5.0) * 64.0;

            for y in 0..10 {
                let y = (y as f32 - 5.0) * 64.0;

                let gpu = app.world.singletons.get::<Gpu>().unwrap();

                let texture =
                    Texture::from_bytes(gpu, include_bytes!("assets/grass.png"), "my texture")
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

                multi_mesh
                    .instances
                    .push(InstanceData::new(&Transform2d::IDENTITY, [1.0, 1.0, 1.0]));

                let transform2d = Transform2d::new(
                    Vector2::new(x, y),
                    x + y,
                    Vector2::new(64.0, 64.0), // Vector2::new(64.0, 64.0)
                );

                let position_tweener = PositionTweener::new(
                    Vector2::new((x + y).cos() * 400.0, y.cos() * 400.0), 
                    Vector2::new(x.sin() * 400.0, (x-y).sin() * 400.0), 
                    1.5 + 0.5 * (x - y).cos().abs(),
                    crate::plugins::other::tweener::Easing::Linear,
                );

                let scale_tweener = ScaleTweener::new(
                    Vector2::new(0.0, 0.0),
                    Vector2::new(y.sin() * 64.0, x.sin() * 64.0), 
                    1.5 + 0.5 * (x + y).sin().abs(),
                    crate::plugins::other::tweener::Easing::Linear,
                );

                // app.world
                //     .insert_entity((transform2d, position_tweener, scale_tweener, multi_mesh));

                // let color = [x, y, x - y];

                // multi_mesh
                //     .instances
                //     .push(InstanceData::new(&transform2d, color));
            }
        }

        // let transform2d = Transform2d::IDENTITY;

        // app.world.insert_entity((multi_mesh, transform2d));
    }
}
