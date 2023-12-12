use crate::{
    app::Plugin,
    ecs::world::World,
    math::{
        transform2d::{self, Transform2d},
        vector2::Vector2,
    },
    plugins::core::timer_plugin::Time,
    query_mut, zip,
};

pub struct TweenerPlugin;

pub enum Easing {
    Linear,
    Sin,
}

struct TweenerState<T: Interpolate + Clone> {
    pub start: T,
    pub end: T,
    pub duration: f32,
    pub easing: Easing,

    time_elapsed: f32,
}

impl<T: Interpolate + Clone> TweenerState<T> {
    pub fn tween(&mut self, delta_time: f32) -> T {
        self.time_elapsed += delta_time;

        if self.time_elapsed > self.duration {
            self.time_elapsed = self.duration;
        }

        let uniform_time_elapsed = self.time_elapsed / self.duration;

        let eased_time = get_eased_value(uniform_time_elapsed, &self.easing);

        if uniform_time_elapsed == 1.0 {
            // just loop here so I can smile while looking at things moving
            // (self.start, self.end) = (self.end.clone(), self.start.clone());
            // self.time_elapsed = 0.0;
        }

        T::interpolate(&self.start, &self.end, eased_time)
    }
}

pub struct PositionTweener {
    tweener_state: TweenerState<Vector2<f32>>,
}

impl PositionTweener {
    pub fn new(start: Vector2<f32>, end: Vector2<f32>, duration: f32, easing: Easing) -> Self {
        Self {
            tweener_state: TweenerState {
                start,
                end,
                duration,
                easing,
                time_elapsed: 0.0,
            },
        }
    }
}

pub struct ScaleTweener {
    tweener_state: TweenerState<Vector2<f32>>,
}

impl ScaleTweener {
    pub fn new(start: Vector2<f32>, end: Vector2<f32>, duration: f32, easing: Easing) -> Self {
        Self {
            tweener_state: TweenerState {
                start,
                end,
                duration,
                easing,
                time_elapsed: 0.0,
            },
        }
    }
}

pub struct CustomTweener {
    tweener_state: TweenerState<f32>,
    pub callback: Box<dyn FnMut(f32) -> ()>,
}

fn get_eased_value(x: f32, easing: &Easing) -> f32 {
    match easing {
        Easing::Linear => x,
        Easing::Sin => x.sin(),
    }
}

trait Interpolate {
    fn interpolate(start: &Self, end: &Self, weight: f32) -> Self;
}

impl Interpolate for f32 {
    fn interpolate(start: &Self, end: &Self, weight: f32) -> Self {
        start + (end - start) * weight
    }
}

impl Interpolate for Vector2<f32> {
    fn interpolate(start: &Self, end: &Self, weight: f32) -> Self {
        Vector2 {
            x: f32::interpolate(&start.x, &end.x, weight),
            y: f32::interpolate(&start.y, &end.y, weight),
        }
    }
}

fn tweener_update(world: &mut World) {
    let delta_time = world.singletons.get::<Time>().unwrap().delta_time;

    for (transform2d, position_tweener) in query_mut!(world, Transform2d, PositionTweener) {
        transform2d.position = position_tweener.tweener_state.tween(delta_time);
    }

    for (transform2d, scale_tweener) in query_mut!(world, Transform2d, ScaleTweener) {
        transform2d.scale = scale_tweener.tweener_state.tween(delta_time);
    }

    for custom_tweener in query_mut!(world, CustomTweener) {
        let eased_time = custom_tweener.tweener_state.tween(delta_time);

        (custom_tweener.callback)(eased_time);
    }
}

impl Plugin for TweenerPlugin {
    fn build(app: &mut crate::app::App) {
        
        app.world.register_component::<PositionTweener>();
        app.world.register_component::<ScaleTweener>();
        app.world.register_component::<CustomTweener>();

        app.schedular
            .add_system(crate::app::SystemStage::Update, tweener_update)
    }
}
