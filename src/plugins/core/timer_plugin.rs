use std::time::Instant;

use crate::{app::Plugin, ecs::world::World};

pub struct Time {
    pub frame_count: u64,
    pub delta_time: f32,

    last_frame_instant: Instant,
}

pub struct TimerPlugin;

impl Plugin for TimerPlugin {
    fn build(app: &mut crate::app::App) {
        let time = Time {
            frame_count: 0,
            delta_time: 0.0,
            last_frame_instant: Instant::now(),
        };

        app.world.singletons.insert(time);
        app.schedular
            .add_system(crate::app::SystemStage::PreUpdate, update_timer);
    }
}

fn update_timer(world: &mut World) {
    let time = world.singletons.get_mut::<Time>().unwrap();

    time.delta_time = time.last_frame_instant.elapsed().as_secs_f32();
    time.last_frame_instant = Instant::now();
}
