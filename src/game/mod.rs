use crate::app::Plugin;

use self::core::event_bus::{GameEventBus, self};
use self::core::level_manager::{self, LevelManager, TilesAddedEvent};
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
        let event_bus = GameEventBus::default();
        app.world.singletons.insert(event_bus);

        app.register_plugin::<GroundPlugin>();
        app.register_plugin::<RoadPlugin>();

        let level_descriptor = get_dummy_level();
        let level_manager = LevelManager::new(&level_descriptor);
        app.world.singletons.insert(level_manager);

        
        let event_data = TilesAddedEvent;
        app.world.emit(event_data)

        
        // event_bus.tiles_added.emit(world, &mut ());
    }
}
