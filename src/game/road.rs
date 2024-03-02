use std::sync::Arc;

use glam::Vec2;
use hashbrown::{HashMap, HashSet};

use crate::{
    app::Plugin,
    ecs::{
        entity::{self, EntityId},
        world::World, component::Component,
    },
    math::{
        color::Color,
        honeycomb::{Hextor, SpiralLoop},
        transform2d::{self, Transform2d},
    },
    plugins::{
        asset_types::image::Image,
        core::{
            asset_storage::{self, AssetStorage},
            render_plugin::{Gpu, Renderer},
        },
        other::tweener::{PositionTweener, ScaleTweener},
        renderer_plugins::{
            mesh::Mesh,
            multi_instance_mesh_renderer::{
                InstanceData, MultiInstanceMesh, MultiInstanceMeshBindGroupLayout,
            },
            sprite_renderer::Sprite,
            texture::Texture,
        },
    },
    storage::Storage,
};

use super::core::level_manager::{self, LevelManager, RoadAddedEvent, RoadRemovedEvent};

pub struct RoadPlugin;
pub struct Roads;

#[derive(Debug)]
pub struct Road {
    center: Hextor,
    neighbor: Hextor,
}

impl Component for Road{}
impl Component for Roads{}


impl Plugin for RoadPlugin {
    fn build(app: &mut crate::app::App) {
        app.storage.world.register_component::<Road>();
        app.storage.world.register_component::<Roads>();

        let (asset_storage, gpu) = app
            .storage
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

        app.storage.add_listener::<RoadAddedEvent>(on_road_added);
        app.storage
            .add_listener::<RoadRemovedEvent>(on_road_removed);
    }
}

fn on_road_added(storage: &mut Storage, data: &RoadAddedEvent) {
    let asset_storage = storage.singletons.get_mut::<AssetStorage>().unwrap();
    let road_texture = asset_storage
        .get("/mnt/09cbb5c3-3c84-4ea4-b328-254e96041faf/pixel-rs/src/game/assets/road.png")
        .unwrap();

    let level_manager = storage.singletons.get::<LevelManager>().unwrap();

    let center = data.new_road;

    let center_pos: Vec2 = center.to_vector(32.0).into();

    for neighbor in SpiralLoop::new(center, 1) {
        if level_manager.is_road(&neighbor) {
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

            let sprite_a = Sprite::new(
                road_texture.clone(),
                Color::WHITE,
                Vec2::new(64.0, 18.0),
                1,
            );
            let sprite_b = Sprite::new(
                road_texture.clone(),
                Color::WHITE,
                Vec2::new(64.0, 18.0),
                100,
            );

            storage
                .world
                .insert_entity((center_transform, sprite_a, Road { center, neighbor }));
            storage
                .world
                .insert_entity((neighbor_transform, sprite_b, Road { center, neighbor }));
        }
    }
}

fn on_road_removed(storage: &mut Storage, data: &RoadRemovedEvent) {
    // find roads and remove them lol

    let mut entities_to_delete = Vec::new();

    // am I not adding Entities while making archetype and entities? maybe
    for (entity_id, road) in storage.world.query::<(EntityId, Road)>() {
        if (road.center == data.road || road.neighbor == data.road) {
            entities_to_delete.push(*entity_id);
        }
    }

    for delete_entity in entities_to_delete {
        storage.world.remove_entity(delete_entity);
    }
}
