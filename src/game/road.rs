use std::sync::Arc;

use glam::Vec2;
use hashbrown::HashMap;

use crate::{
    app::Plugin,
    ecs::world::World,
    math::{
        color::Color, honeycomb::{Hextor, SpiralLoop}, transform2d::{self, Transform2d}
    },
    plugins::{
        asset_types::image::Image,
        core::{
            asset_storage::{self, AssetStorage},
            render_plugin::{Gpu, Renderer},
        },
        other::tweener::{PositionTweener, ScaleTweener},
        renderer_plugins::{
            mesh::Mesh, multi_instance_mesh_renderer::{
                InstanceData, MultiInstanceMesh, MultiInstanceMeshBindGroupLayout,
            }, sprite_renderer::Sprite, texture::Texture
        },
    },
    query_mut, zip,
};

use super::core::level_manager::{self, LevelManager, RoadAddedEvent};

pub struct RoadPlugin;
pub struct Roads;

impl Plugin for RoadPlugin {
    fn build(app: &mut crate::app::App) {
        let (asset_storage, gpu) = app
            .world
            .singletons
            .get_many_mut::<(AssetStorage, Gpu)>()
            .unwrap();

        let road_texture = asset_storage
            .get::<Image>(
                "/mnt/09cbb5c3-3c84-4ea4-b328-254e96041faf/pixel-rs/src/game/assets/road.png",
            )
            .unwrap();

        let data = asset_storage.get_data(&road_texture);

        gpu.create_texture(
            road_texture.get_id(),
            "Road Texture",
            &data.get_data(),
            64,
            9,
        );

        app.world.register_component::<Roads>();
        app.world.add_listener::<RoadAddedEvent>(on_road_added);
    }
}

fn on_road_added(world: &mut World, data: &RoadAddedEvent) {
    let asset_storage = world.singletons.get_mut::<AssetStorage>().unwrap();
    let road_texture = asset_storage
        .get("/mnt/09cbb5c3-3c84-4ea4-b328-254e96041faf/pixel-rs/src/game/assets/road.png")
        .unwrap();

    // let level_manager = world.singletons.get::<LevelManager>().unwrap();

    let center = data.new_road;

    let center_pos: Vec2 = center.to_vector(32.0).into();

    for neighbor in SpiralLoop::new(center, 1) {
        if world.singletons.get::<LevelManager>().unwrap().is_road(&neighbor) {
            if neighbor == center {
                continue;
            }

            let neighbor_pos: Vec2 = neighbor.to_vector(32.0).into();

            let center_transform = Transform2d::new(
                center_pos,
                (neighbor_pos - center_pos).to_angle() as f32,
                Vec2::new(1.0, 1.0),
            );

            let neighbor_transform = Transform2d::new(
                neighbor_pos,
                (center_pos - neighbor_pos).to_angle() as f32,
                Vec2::new(1.0, 1.0),
            );


            let sprite_a = Sprite::new(road_texture.clone(), Color::WHITE, Vec2::new(64.0,18.0), 100);
            let sprite_b = Sprite::new(road_texture.clone(), Color::WHITE, Vec2::new(64.0, 18.0), 100);
            world.insert_entity((center_transform, sprite_a));
            world.insert_entity((neighbor_transform, sprite_b));
        }
    }
}
