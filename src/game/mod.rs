use crate::app::Plugin;

use self::ground::GroundPlugin;

mod ground;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(app: &mut crate::app::App) {
        app.register_plugin::<GroundPlugin>();
        
    }
}
