use std::default;

use rand::seq::index;

use crate::ecs::world::World;

#[derive(Default)]
pub struct GameEvent<T> {
    listeners: Vec<fn(&mut World, &T)>,
}

impl<T> GameEvent<T> {
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    pub fn add_listener(&mut self, fun: fn(&mut World, &T)) {
        self.listeners.push(fun);
    }

    pub fn remove_listener(&mut self, fun: fn(&mut World, &T)) {
        let index = self.listeners.iter().position(|x| *x == fun).unwrap();
        self.listeners.remove(index);
    }

    pub fn emit(&self, world: &mut World, data: &mut T) {
        self.listeners.iter().for_each(|fun| (fun)(world, data));
    }
}

#[derive(Default)]
pub struct GameEventBus {
    pub tiles_added: GameEvent<()>,
}
