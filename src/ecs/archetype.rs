use hashbrown::HashMap;
use std::any::{Any, TypeId};
use std::fmt::Debug;

use super::component::{Component, ComponentTypeId, TypeErasedComponentVec};
use super::entity::EntityId;

#[derive(Debug)]
pub struct Archetype {
    pub columns: HashMap<ComponentTypeId, TypeErasedComponentVec>,
    pub len: usize,
    // do I need to store bitset??
}

impl Archetype {
    pub fn new() -> Self {
        Self {
            columns: HashMap::new(),
            len: 0,
        }
    }

    // Should be cached
    pub fn get_type_ids(&self) -> Vec<ComponentTypeId> {
        let mut vec = Vec::new();
        for key in self.columns.keys() {
            vec.push(*key);
        }

        vec
    }

    pub fn len(&self) -> usize {
        self.len
    }
}
