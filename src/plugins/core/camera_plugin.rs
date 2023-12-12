use winit::window::Window;

use crate::{
    app::{Plugin, SystemStage},
    ecs::world::{self, World},
    math::{
        transform2d::{Matrix3, Transform2d},
        vector2::Vector2,
    },
    query_mut, zip,
};

pub struct Camera {
    pub projection: Matrix3,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(app: &mut crate::app::App) {
        // First I need to know that its -1 to 1 or -0.5 to 0.5 I know that center is zero zero
        app.world.insert_entity((
            Camera {
                projection: Matrix3::IDENTITY,
            },
            Transform2d::IDENTITY,
        ));

        // I need to update camera buffer how??
        app.schedular.add_system(SystemStage::Resize, on_resize);
    }
}

pub fn on_resize(world: &mut World) {
    let size = world.singletons.get::<Window>().unwrap().inner_size();

    let camera = query_mut!(world, Camera).next().unwrap();

    camera.projection.x[0] = 2.0 / size.width as f32;
    camera.projection.y[1] = 2.0 / size.height as f32;
}
