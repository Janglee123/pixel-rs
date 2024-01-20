use glam::{vec2, vec3, Vec2, Vec3};

use crate::{
    app::Plugin,
    math::transform2d::Transform2d,
    plugins::core::{
        camera_plugin::{Camera, Viewport},
        input::input_plugin::{Input, MouseButton},
    },
    query_mut, zip, World,
};

pub struct CameraControllerPlugin;

#[derive(Default)]
pub struct CameraController {
    current_pos: Vec2,

    move_start_offset: Vec2,
    is_right_click_pressed: bool,
    zoom: f32,
}

impl Plugin for CameraControllerPlugin {
    fn build(app: &mut crate::app::App) {
        app.schedular
            .add_system(crate::app::SystemStage::Update, on_update);

        app.world.singletons.insert(CameraController::default());
    }
}

fn on_update(world: &mut World) {
    let (input, viewport, camera_controller) = world
        .singletons
        .get_many_mut::<(Input, Viewport, CameraController)>()
        .unwrap();

    let current_position = input.mouse_position();

    let is_right_pressed = input.is_mouse_button_pressed(MouseButton::Right);

    if camera_controller.is_right_click_pressed != is_right_pressed {
        if is_right_pressed {
            camera_controller.move_start_offset = current_position;
        }

        camera_controller.is_right_click_pressed = is_right_pressed;
    }

    if is_right_pressed {
        let delta =
            (current_position - camera_controller.move_start_offset) * viewport.get_size() * 0.5;
        camera_controller.move_start_offset = current_position;

        println!(
            "c {} s {} d {}",
            current_position, camera_controller.move_start_offset, delta
        );

        let (transform2d, _) = query_mut!(world, Transform2d, Camera).next().unwrap();

        transform2d.position += delta;
    }
}
