use bytemuck::Pod;
use hashbrown::HashMap;
use std::{
    alloc::Layout,
    any::{Any, TypeId},
    marker::PhantomData,
    mem::MaybeUninit,
    sync::Arc,
};

use super::{
    archetype::{self, Archetype},
    bitset::{self, BitSet},
    entity::{self, EntityId},
};

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub struct ComponentTypeId(pub u8);

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub struct ComponentId(u32);

pub struct Components {
    // get_next_type_id
    // get_component_vec_operator
    //
    type_counter: u8,
    vec_operator_map: HashMap<ComponentTypeId, ComponentVecOperator>,
    type_map: HashMap<TypeId, ComponentTypeId>,
    type_names: HashMap<ComponentTypeId, String>,
}

impl Components {
    pub fn new() -> Self {
        Self {
            type_counter: 0,
            vec_operator_map: HashMap::new(),
            type_map: HashMap::new(),
            type_names: HashMap::new(),
        }
    }

    pub fn get_new_component_type_id(&mut self) -> ComponentTypeId {
        let id = self.type_counter;
        self.type_counter += 1;

        ComponentTypeId(id)
    }

    pub fn get_component_id(&self, type_id: &TypeId) -> Option<ComponentTypeId> {
        self.type_map.get(type_id).map(|f| *f)
    }

    pub fn get_component_vec_operator(
        &self,
        id: &ComponentTypeId,
    ) -> Option<&ComponentVecOperator> {
        self.vec_operator_map.get(id)
    }

    pub fn register_component<T: Component>(&mut self) {
        let type_id = TypeId::of::<T>();

        if let None = self.type_map.get(&type_id) {
            let id = self.get_new_component_type_id();
            self.type_map.insert(type_id.clone(), id.clone());

            let vec_operator = ComponentVecOperator::new::<T>();
            self.vec_operator_map.insert(id, vec_operator);

            self.type_names.insert(id, std::any::type_name::<T>().to_string());
        }
    }

    pub fn get_name(&self, id: &ComponentTypeId) -> &String {
        self.type_names.get(id).unwrap()
    }
}

pub trait Component: 'static {}

type ComponentVec<T: Component> = Vec<T>;

#[derive(Debug)]
pub struct TypeErasedComponentVec {
    vec: Box<dyn Any>,
}

impl TypeErasedComponentVec {
    pub fn new<T: Component>() -> Self {

        println!("Created type erased map with TypeId: {:?}", TypeId::of::<T>());
        Self {
            vec: Box::new(ComponentVec::<T>::new()),
        }
    }

    pub fn get<T: Component>(&self) -> &ComponentVec<T> {
        self.vec.downcast_ref().unwrap()
    }

    pub fn get_mut<T: Component>(&mut self) -> &mut ComponentVec<T> {
        self.vec.downcast_mut().unwrap()
    }

    pub fn push<T: Component>(&mut self, data: Box<dyn Any>) {
        let component = *data.downcast::<T>().unwrap();

        self.vec
            .downcast_mut::<ComponentVec<T>>()
            .unwrap()
            .push(component);
    }

    pub fn insert<T: Component>(&mut self, data: Box<dyn Any>, index: usize) {
        let component = *data.downcast::<T>().unwrap();

        self.vec
            .downcast_mut::<ComponentVec<T>>()
            .unwrap()
            .insert(index, component);
    }

    pub fn remove<T: Component>(&mut self, index: usize) {
        self.vec
            .downcast_mut::<ComponentVec<T>>()
            .unwrap()
            .remove(index);
    }

    pub fn swap_remove<T: Component>(&mut self, index: usize) {

        let a = self.vec
            .downcast_mut::<ComponentVec<T>>()
            .unwrap()
            .swap_remove(index);
    }

    pub fn migrate_push<T: Component>(src: &mut Self, dest: &mut Self, src_index: usize) {
        let src = src.vec.downcast_mut::<ComponentVec<T>>().unwrap();
        let dest = dest.vec.downcast_mut::<ComponentVec<T>>().unwrap();

        // This can be swap remove
        let element = src.swap_remove(src_index);

        // This can be push only
        dest.push(element);
    }

    pub fn migrate_insert<T: Component>(
        src: &mut Self,
        dest: &mut Self,
        src_index: usize,
        dest_index: usize,
    ) {
        let src = src.vec.downcast_mut::<ComponentVec<T>>().unwrap();
        let dest = dest.vec.downcast_mut::<ComponentVec<T>>().unwrap();

        // This can be swap remove
        let element = src.swap_remove(src_index);

        dest.insert(dest_index, element);
    }
}

pub struct ComponentVecOperator {
    pub creator: fn() -> TypeErasedComponentVec,
    pub pusher: fn(&mut TypeErasedComponentVec, Box<dyn Any>) -> (),
    pub remover: fn(&mut TypeErasedComponentVec, usize),
    pub swap_remover: fn(&mut TypeErasedComponentVec, usize),
    pub migrator: fn(&mut TypeErasedComponentVec, &mut TypeErasedComponentVec, usize) -> (),
    pub inserter: fn(&mut TypeErasedComponentVec, Box<dyn Any>, usize) -> (),
}

impl ComponentVecOperator {
    pub fn new<T: Component>() -> Self {
        Self {
            creator: TypeErasedComponentVec::new::<T>,
            pusher: TypeErasedComponentVec::push::<T>,
            remover: TypeErasedComponentVec::remove::<T>,
            swap_remover: TypeErasedComponentVec::swap_remove::<T>,
            migrator: TypeErasedComponentVec::migrate_push::<T>,
            inserter: TypeErasedComponentVec::insert::<T>,
        }
    }
}
