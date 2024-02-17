use crate::ecs::{
    event_bus::{EventBus, WorldEvent},
    singletons::Singletons,
    world::World,
};

pub struct Storage {
    pub world: World,
    pub singletons: Singletons,
    event_bus: EventBus,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            singletons: Singletons::new(),
            event_bus: EventBus::new(), // I can use commands for this? No I want to call them immediately
        }
    }

    #[inline(always)]
    pub fn add_listener<T: WorldEvent + 'static>(&mut self, fun: fn(&mut Storage, &T)) {
        self.event_bus.add_listener(fun);
    }

    #[inline(always)]
    pub fn remove_listener<T: WorldEvent + 'static>(&mut self, fun: fn(&mut Storage, &T)) {
        self.event_bus.remove_listener(fun);
    }

    #[inline(always)]
    pub fn emit<T: WorldEvent + 'static>(&mut self, event_data: T) {
        for something in self.event_bus.get_list().unwrap_or_default() {
            (something)(self, &event_data);
        }
    }
}
