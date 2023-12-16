use crate::app::Plugin;
use crate::math::honeycomb::Hextor;

use self::core::level_manager::{self, LevelManager, RoadAddedEvent, TilesAddedEvent};
use self::ground::GroundPlugin;
use self::resources::level_descriptors::get_dummy_level;
use self::road::RoadPlugin;

mod core;
mod ground;
mod resources;
mod road;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(app: &mut crate::app::App) {
        app.register_plugin::<GroundPlugin>();
        app.register_plugin::<RoadPlugin>();

        let level_descriptor = get_dummy_level();
        let level_manager = LevelManager::new(&level_descriptor);
        app.world.singletons.insert(level_manager);

        app.world.emit(TilesAddedEvent);

        app.world.emit(RoadAddedEvent {
            new_road: Hextor::new(0, 0),
        });

        let level_manager = app.world.singletons.get_mut::<LevelManager>().unwrap();
        level_manager.place_road(Hextor::new(1, 0));

        app.world.emit(RoadAddedEvent {
            new_road: Hextor::new(1, 0),
        })
    }
}
