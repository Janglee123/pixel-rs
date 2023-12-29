use std::sync::Arc;

use hashbrown::HashMap;

use crate::{
    app::Plugin,
    ecs::world::World,
    math::{
        honeycomb::{Hextor, SpiralLoop},
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
    query_mut, zip,
};

use super::core::level_manager::{self, LevelManager, RoadAddedEvent};

pub struct RoadPlugin;
pub struct Roads;

impl Plugin for RoadPlugin {
    fn build(app: &mut crate::app::App) {
        let gpu = app.world.singletons.get::<Gpu>().unwrap();

        // let texture = Texture::from_bytes(gpu, include_bytes!("assets/road.png"), "road texture")
        //     .ok()
        //     .unwrap();

        let bind_group_layout = app
            .world
            .singletons
            .get::<MultiInstanceMeshBindGroupLayout>()
            .unwrap();

        // let multi_mesh = MultiInstanceMesh::new(
        //     gpu,
        //     &bind_group_layout,
        //     Arc::new(Mesh::get_quad_mesh()),
        //     texture,
        // );

        // app.world
        //     .insert_entity((multi_mesh, Roads, Transform2d::IDENTITY));
        app.world.register_component::<Roads>();
        app.world.add_listener::<RoadAddedEvent>(on_road_added);
    }
}

fn on_road_added(world: &mut World, data: &RoadAddedEvent) {
    
    let entity = query_mut!(world, MultiInstanceMesh, Roads).next();

    if let None = entity {
        return;
    }

    let (multi_mesh, _) = entity.unwrap();

    let level_manager = world.singletons.get::<LevelManager>().unwrap();

    let center = data.new_road;

    let center_pos: Vector2<f32> = center.to_vector(32.0).into();

    for neighbor in SpiralLoop::new(center, 1) {
        if level_manager.is_road(&neighbor) {
            if neighbor == center {
                continue;
            }

            let neighbor_pos: Vector2<f32> = neighbor.to_vector(32.0).into();

            // add tile for center tile
            // get position
            // get rotation
            let center_transform = Transform2d::new(
                center_pos,
                (neighbor_pos - center_pos).angle() as f32,
                Vector2::new(64.0, 16.0),
            );

            let center_instance = InstanceData::new(&center_transform, [1.0, 1.0, 1.0]);

            let neighbor_transform = Transform2d::new(
                neighbor_pos,
                (center_pos - neighbor_pos).angle() as f32,
                Vector2::new(64.0, 16.0),
            );

            let neighbor_instance = InstanceData::new(&neighbor_transform, [1.0, 1.0, 1.0]);

            multi_mesh.instances.push(center_instance);
            multi_mesh.instances.push(neighbor_instance);
        }
    }
}
