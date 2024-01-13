use glam::Vec2;

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
            input_plugin::Input,
            render_plugin::Gpu,
        },
        renderer_plugins::{
            sprite_renderer::{self, Sprite, SpriteRendererData},
            texture::{self, Texture},
        },
    },
    query_mut, zip,
};

pub struct RoadPlacer {
    current_pos: Hextor,
}

pub struct RoadPlacerPlugin;

impl Plugin for RoadPlacerPlugin {
    fn build(app: &mut crate::app::App) {
        let (asset_storage, gpu) = app
            .world
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

        let sprite = Sprite::new(select_sprite, Color::new(1.0, 1.0, 1.0, 0.5), 0);

        let mut transform2d = Transform2d::IDENTITY;
        transform2d.scale = Vec2::new(32.0, 32.0);

        let road_placer = RoadPlacer {
            current_pos: Hextor::new(0, 0),
        };

        app.world.insert_entity((sprite, transform2d, road_placer));

        // app.schedular
        //     .add_system(crate::app::SystemStage::Input, on_input);

        app.schedular.add_system(crate::app::SystemStage::Update, on_update);
    }
}

fn on_update(world: &mut World) {
    let (input, viewport) = world.singletons.get_many::<(Input, Viewport)>().unwrap();
    let mouse_pos = input.mouse_position();
    let world_mouse_pos = viewport.screen_to_world(mouse_pos);

    let (transform2d, road_placer) = query_mut!(world, Transform2d, RoadPlacer).next().unwrap();
    transform2d.position = world_mouse_pos;
}

fn on_input(world: &mut World) {
    let (transform2d, road_placer) = query_mut!(world, Transform2d, RoadPlacer).next().unwrap();

    let input = world.singletons.get::<Input>().unwrap();

    if input.is_key_pressed(winit::keyboard::KeyCode::KeyW) {
        road_placer.current_pos.r += 1;
    }

    if input.is_key_pressed(winit::keyboard::KeyCode::KeyS) {
        road_placer.current_pos.r -= 1;
    }

    if input.is_key_pressed(winit::keyboard::KeyCode::KeyA) {
        road_placer.current_pos.q -= 1;
    }

    if input.is_key_pressed(winit::keyboard::KeyCode::KeyD) {
        road_placer.current_pos.q += 1;
    }

    // transform2d.position = road_placer.current_pos.to_vector(32.0).into();
    // println!("new position: {:?}", transform2d.position);

    let tile = road_placer.current_pos;

    if input.is_key_pressed(winit::keyboard::KeyCode::Space) {
        let level_manager: &mut LevelManager = world.singletons.get_mut().unwrap();
        if level_manager.can_place_road(&tile) {
            level_manager.place_road(tile);
            world.emit(RoadAddedEvent { new_road: tile });
        }
    }
}
