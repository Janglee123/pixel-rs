use crate::{
    app::Plugin,
    ecs::world::World,
    math::{
        honeycomb::Hextor,
        transform2d::{self, Transform2d},
        vector2::Vector2,
    },
    plugins::{
        core::{input_plugin::Input, render_plugin::Gpu},
        renderer_plugins::{
            sprite_renderer::{self, Quad, SpriteRendererData},
            texture::{self, Texture},
        },
    },
    query_mut, zip, game::core::level_manager::{self, LevelManager, RoadAddedEvent},
};

pub struct RoadPlacer {
    current_pos: Hextor,
}

pub struct RoadPlacerPlugin;

impl Plugin for RoadPlacerPlugin {
    fn build(app: &mut crate::app::App) {
        let gpu = app.world.singletons.get::<Gpu>().unwrap();

        // let texture = Texture::from_bytes(
        //     gpu,
        //     include_bytes!("assets/selector.png"),
        //     "selector texture",
        // )
        // .ok()
        // .unwrap();

        let sprite_renderer_data = app.world.singletons.get::<SpriteRendererData>().unwrap();

        // let sprite = Quad::new(
        //     &gpu.device,
        //     &sprite_renderer_data.transform_bind_group_layout,
        //     &sprite_renderer_data.texture_bind_group_layout,
        //     texture,
        //     Vector2::new(32, 32),
        // );

        let mut transform2d = Transform2d::IDENTITY;
        transform2d.scale = Vector2::new(32.0, 32.0);

        let road_placer = RoadPlacer {
            current_pos: Hextor::new(0, 0),
        };

        // app.world.insert_entity((sprite, transform2d, road_placer));
        app.schedular
            .add_system(crate::app::SystemStage::Input, on_input);
    }
}

fn on_input(world: &mut World) {
    
    let (transform2d, road_placer) = query_mut!(world, Transform2d, RoadPlacer).next().unwrap();

    let input = world.singletons.get::<Input>().unwrap();

    if input.is_key_pressed(winit::event::VirtualKeyCode::W) {
        road_placer.current_pos.r += 1;
    }

    if input.is_key_pressed(winit::event::VirtualKeyCode::S) {
        road_placer.current_pos.r -= 1;
    }

    if input.is_key_pressed(winit::event::VirtualKeyCode::A) {
        road_placer.current_pos.q -= 1;
    }

    if input.is_key_pressed(winit::event::VirtualKeyCode::D) {
        road_placer.current_pos.q += 1;
    }

    transform2d.position = road_placer.current_pos.to_vector(32.0).into();
    println!("new position: {:?}", transform2d.position);


    let tile = road_placer.current_pos;

    if input.is_key_pressed(winit::event::VirtualKeyCode::Space) {
        let level_manager: &mut LevelManager = world.singletons.get_mut().unwrap();
        if level_manager.can_place_road(&tile) {
            level_manager.place_road(tile);
            world.emit(RoadAddedEvent {
                new_road: tile
            });
        }
    }

}
