use crate::storage::Storage;

use glam::{vec2, Vec2};

use crate::{
    app::Plugin,
    ecs::world::{self, World},
    game::core::level_manager::{self, LevelManager, RoadAddedEvent},
    math::{
        color::Color,
        honeycomb::Hextor,
        transform2d::{self, Transform2d},
    },
    plugins::{
        asset_types::image::Image,
        core::{
            asset_storage::{self, AssetStorage},
            camera_plugin::Viewport,
            input::{
                input_plugin::{Input, MouseButton},
                keycode::KeyCode,
            },
            render_plugin::Gpu,
            timer_plugin::Time,
        },
        other::tweener::{self, PositionTweener},
        renderer_plugins::{
            sprite_renderer::{self, Sprite, SpriteRendererData},
            texture::{self, Texture},
        },
    },
};

use super::core::level_manager::RoadRemovedEvent;

pub struct RoadPlacer {
    current_pos: Hextor,
}

pub struct RoadPlacerPlugin;

impl Plugin for RoadPlacerPlugin {
    fn build(app: &mut crate::app::App) {
        app.storage.world.register_component::<RoadPlacer>();

        let (asset_storage, gpu) = app
            .storage
            .singletons
            .get_many_mut::<(AssetStorage, Gpu)>()
            .unwrap();

        let select_sprite = asset_storage
            .get::<Image>(
                "/mnt/09cbb5c3-3c84-4ea4-b328-254e96041faf/pixel-rs/src/game/assets/selector.png",
            )
            .unwrap();

        let data = asset_storage.get_data(&select_sprite);
        gpu.create_texture(
            select_sprite.get_id(),
            "Selector sprite",
            data.get_data(),
            64,
            74,
        );

        let sprite = Sprite::new(
            select_sprite.clone(),
            Color::new(1.0, 1.0, 1.0, 1.0),
            Vec2::new(64.0, 74.0),
            2,
        );

        let mut transform2d = Transform2d::IDENTITY;

        let road_placer = RoadPlacer {
            current_pos: Hextor::new(0, 0),
        };

        let tweener = PositionTweener::default();

        app.storage
            .world
            .insert_entity((sprite, transform2d, road_placer, tweener));

        app.schedular
            .add_system(crate::app::SystemStage::Input, on_input);

        app.schedular
            .add_system(crate::app::SystemStage::Update, on_update);
    }
}

fn on_update(storage: &mut Storage) {
    let (input, viewport) = storage.singletons.get_many::<(Input, Viewport)>().unwrap();
    let mouse_pos = input.mouse_position();
    let world_mouse_pos = viewport.screen_to_world(mouse_pos);

    let (transform2d, road_placer, tweener) =
        storage
            .world
            .query_mut_single::<(Transform2d, RoadPlacer, PositionTweener)>();

    let hex_pos = Hextor::from_vector(world_mouse_pos.x, world_mouse_pos.y, 32.0);

    if (road_placer.current_pos != hex_pos) {
        road_placer.current_pos = hex_pos;

        let end = hex_pos.to_vector(32.0).into();

        tweener.tween(transform2d.position, end, 0.05, tweener::Easing::Linear);
    }

    let (input, viewport) = storage.singletons.get_many::<(Input, Viewport)>().unwrap();
    if input.is_mouse_button_pressed(MouseButton::Left) {
        let level_manager: &mut LevelManager = storage.singletons.get_mut().unwrap();
        if level_manager.can_place_road(&hex_pos) {
            level_manager.place_road(hex_pos);
            storage.emit(RoadAddedEvent { new_road: hex_pos });
        }
    } else if input.is_mouse_button_pressed(MouseButton::Right) {
        let level_manager: &mut LevelManager = storage.singletons.get_mut().unwrap();
        if level_manager.is_road(&hex_pos) {
            level_manager.remove_road(hex_pos);
            storage.emit(RoadRemovedEvent { road: hex_pos });
        }
    }
}

fn on_input(storage: &mut Storage) {
    let (transform2d, road_placer) = storage
        .world
        .query_mut_single::<(Transform2d, RoadPlacer)>();

    let input = storage.singletons.get::<Input>().unwrap();

    if input.is_key_pressed(KeyCode::KeyW) {
        road_placer.current_pos.r += 1;
    }

    if input.is_key_pressed(KeyCode::KeyS) {
        road_placer.current_pos.r -= 1;
    }

    if input.is_key_pressed(KeyCode::KeyA) {
        road_placer.current_pos.q -= 1;
    }

    if input.is_key_pressed(KeyCode::KeyD) {
        road_placer.current_pos.q += 1;
    }

    let tile = road_placer.current_pos;

    if input.is_key_pressed(KeyCode::Space) {
        let level_manager: &mut LevelManager = storage.singletons.get_mut().unwrap();
        if level_manager.can_place_road(&tile) {
            level_manager.place_road(tile);
            storage.emit(RoadAddedEvent { new_road: tile });
        }
    }
}
