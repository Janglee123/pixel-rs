use crate::app::Plugin;

use self::ground::GroundPlugin;
use self::road::RoadPlugin;

mod ground;
mod road;
mod core;
mod resources;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(app: &mut crate::app::App) {
        app.register_plugin::<GroundPlugin>();
        app.register_plugin::<RoadPlugin>();
    }
}
