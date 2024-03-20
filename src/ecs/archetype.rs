use hashbrown::HashMap;
use std::any::{Any, TypeId};
use std::fmt::Debug;

use super::component::{Component, ComponentTypeId, TypeErasedComponentVec};
use super::entity::EntityId;

#[derive(Debug)]
pub struct Archetype {
    columns: Vec<TypeErasedComponentVec>,
    pub component_id_column_index_map: HashMap<ComponentTypeId, usize>,
    pub len: usize,
    // do I need to store bitset??
}

impl Archetype {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            component_id_column_index_map: HashMap::new(),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn insert_column(&mut self, id: ComponentTypeId, vec: TypeErasedComponentVec) {
        let len = self.columns.len();
        self.component_id_column_index_map.insert(id, len);
        self.columns.push(vec);
    }

    pub fn get_column(&self, id: &ComponentTypeId) -> &TypeErasedComponentVec {
        let index = self.component_id_column_index_map.get(id).unwrap();

        &self.columns[*index]
    }

    pub fn get_column_mut(&mut self, id: &ComponentTypeId) -> &mut TypeErasedComponentVec {
        let index = self.component_id_column_index_map.get(id).unwrap();
        &mut self.columns[*index]
    }

    pub fn get_many_columns_mut(
        &mut self,
        vec: &Vec<ComponentTypeId>,
    ) -> Vec<&mut TypeErasedComponentVec> {
        let mut result = Vec::new();

        for id in vec {
            let index = *self.component_id_column_index_map.get(id).unwrap();

            // Well people say this is very unsafe and UB
            // breaks many assumptions made by the compiler
            // I think UnsafeCell would be what I am looking for. Will learn about it later
            // https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html
            result.push(unsafe {
                let a: *mut TypeErasedComponentVec = &mut self.columns[index];
                &mut *a
            });
        }

        result
    }

    pub fn get_many_columns(&self, vec: &Vec<ComponentTypeId>) -> Vec<&TypeErasedComponentVec> {
        let mut result = Vec::new();

        for id in vec {
            let index = *self.component_id_column_index_map.get(id).unwrap();

            unsafe {
                result.push(&self.columns[index]);
            }
        }

        result
    }
}
