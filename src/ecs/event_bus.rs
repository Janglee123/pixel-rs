use crate::storage::Storage;
use hashbrown::HashMap;
use std::any::{Any, TypeId};

pub trait WorldEvent: 'static {}

pub struct WorldEventListenerList<T: WorldEvent> {
    listeners: Vec<fn(&mut Storage, &T)>,
}

impl<T: WorldEvent> WorldEventListenerList<T> {
    fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }
}

pub struct EventBus {
    map: HashMap<TypeId, Box<dyn Any>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn add_listener<T: WorldEvent>(&mut self, fun: fn(&mut Storage, &T)) {
        if !self.map.contains_key(&TypeId::of::<T>()) {
            let listener_list = WorldEventListenerList::<T>::new();
            self.map.insert(TypeId::of::<T>(), Box::new(listener_list));
        }

        let listener_list = self
            .map
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .downcast_mut::<WorldEventListenerList<T>>()
            .unwrap();

        listener_list.listeners.push(fun);
    }

    pub fn remove_listener<T: WorldEvent>(&mut self, fun: fn(&mut Storage, &T)) {
        if let Some(list) = self.map.get_mut(&TypeId::of::<T>()) {
            let list = list.downcast_mut::<WorldEventListenerList<T>>().unwrap();

            if let Some(index) = list.listeners.iter().position(|x| *x == fun) {
                list.listeners.remove(index);
            }
        }
    }

    pub fn get_list<T: WorldEvent + 'static>(&self) -> Option<Vec<fn(&mut Storage, &T)>> {
        if let Some(list) = self.map.get(&TypeId::of::<T>()) {
            let list = list.downcast_ref::<WorldEventListenerList<T>>().unwrap();
            // Clone AAAAAAAA
            return Some(list.listeners.clone());
        }

        None
    }
}
