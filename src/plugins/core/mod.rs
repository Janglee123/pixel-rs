use crate::app::Plugin;

use self::{
    asset_storage::AssetStoragePlugin, input::input_plugin::InputPlugin,
    render_plugin::RenderPlugin, timer_plugin::TimerPlugin, window::window_plugin::WindowPlugin,
};

pub mod asset_storage;
pub mod camera_plugin;
pub mod input;
pub mod render_plugin;
pub mod timer_plugin;
pub mod window;

pub struct CorePlugins;

impl Plugin for CorePlugins {
    fn build(app: &mut crate::app::App) {
        app.register_plugin::<InputPlugin>();
        app.register_plugin::<WindowPlugin>();
        app.register_plugin::<TimerPlugin>();
        app.register_plugin::<AssetStoragePlugin>();
        app.register_plugin::<RenderPlugin>();
    }
}
