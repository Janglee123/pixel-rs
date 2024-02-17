use crate::app::Plugin;
use crate::math::honeycomb::Hextor;

use self::camera_controller::CameraControllerPlugin;
use self::core::level_manager::{self, LevelManager, RoadAddedEvent, TilesAddedEvent};
use self::ground::GroundPlugin;
use self::resources::level_descriptors::get_dummy_level;
use self::road::RoadPlugin;
use self::road_placer::RoadPlacerPlugin;

mod camera_controller;
mod core;
mod ground;
mod resources;
mod road;
mod road_placer;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(app: &mut crate::app::App) {
        app.register_plugin::<GroundPlugin>();
        app.register_plugin::<RoadPlugin>();
        app.register_plugin::<RoadPlacerPlugin>();
        app.register_plugin::<CameraControllerPlugin>();

        let level_descriptor = get_dummy_level();
        let level_manager = LevelManager::new(&level_descriptor);
        app.storage.singletons.insert(level_manager);

        app.storage.emit(TilesAddedEvent);

        app.storage.emit(RoadAddedEvent {
            new_road: Hextor::new(0, 0),
        });

        let level_manager = app.storage.singletons.get_mut::<LevelManager>().unwrap();
        level_manager.place_road(Hextor::new(1, 0));

        app.storage.emit(RoadAddedEvent {
            new_road: Hextor::new(1, 0),
        })
    }
}
