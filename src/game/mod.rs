use crate::app::Plugin;

use self::core::level_manager::{self, LevelManager};
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

        let level_descriptor = get_dummy_level();
        let level_manager = LevelManager::new(&level_descriptor);

        app.world.singletons.insert(level_manager);

        app.register_plugin::<GroundPlugin>();
        app.register_plugin::<RoadPlugin>();
    }
}
