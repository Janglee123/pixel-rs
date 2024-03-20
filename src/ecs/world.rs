extern crate proc_macro;
use core::arch;
use hashbrown::{HashMap, HashSet};
use itertools::izip;
use std::any::{Any, TypeId};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::iter;
use std::marker::PhantomData;
use std::path::Iter;
use std::sync::Arc;

use super::archetype::{self, Archetype};
use super::bitset::{self, BitSet};
use super::component::{
    self, Component, ComponentTypeId, ComponentVecOperator, Components, TypeErasedComponentVec,
};
use super::component_set::ComponentSet;
use super::entity::{self, Entities, EntityId};
use super::event_bus::{EventBus, WorldEvent};
use super::singletons::{self, Singletons};

pub struct World {
    pub components: Components,
    pub entities: Entities,
    pub archetype_id_map: HashMap<BitSet, Archetype>,
    entity_archetype_map: HashMap<EntityId, BitSet>,

    entity_id_id: ComponentTypeId, // LOL WHAT A NAME
}

impl World {
    pub fn new() -> Self {
        let mut components = Components::new();
        let entity_id_id = components.register_component::<EntityId>();

        let mut result = Self {
            entities: Entities::new(),
            archetype_id_map: HashMap::new(),
            entity_archetype_map: HashMap::new(),
            components,
            entity_id_id,
        };

        result
    }

    fn get_new_entity_id(&mut self) -> EntityId {
        self.entities.get_new_entity_id()
    }

    pub fn register_component<T: Component>(&mut self) {
        self.components.register_component::<T>();
    }

    pub fn get_column_names(&self, id: &BitSet) -> Vec<&String> {
        let mut type_names = Vec::new();

        for i in 0..255 {
            if id.contains_id(i) {
                type_names.push(self.components.get_name(&ComponentTypeId(i)));
            }
        }

        type_names
    }

    pub fn insert_entity<T: ComponentSet + 'static>(&mut self, component_set: T) -> EntityId {
        let entity_id = self.get_new_entity_id();

        let type_ids = T::get_type_id_vec();
        let bitset = self.get_bit_set_id(&type_ids); // I am converting type ids to component ids 3 times in this method

        if !self.archetype_id_map.contains_key(&bitset) {
            let component_type_ids = type_ids
                .iter()
                .map(|id| self.components.get_component_id(id).unwrap())
                .collect();

            let new_archetype = self.create_archetype_from_type_ids(&component_type_ids);

            self.archetype_id_map.insert(bitset, new_archetype);
        }

        let archetype = self.archetype_id_map.get_mut(&bitset).unwrap();

        // Todo: Using Box<dyn Any> to pass data
        // Not so good idea
        // but it will work for now
        // one way is to add get_as_any inside component trait
        for (type_id, component) in component_set.get_map() {
            let id = self.components.get_component_id(&type_id).unwrap();
            let type_erased_vec = archetype.get_column_mut(&id);
            let operator = self.components.get_component_vec_operator(&id).unwrap();
            (operator.pusher)(type_erased_vec, component);
        }

        let id = self
            .components
            .get_component_id(&TypeId::of::<EntityId>())
            .unwrap();
        let type_erased_vec = archetype.get_column_mut(&id);
        let operator = self.components.get_component_vec_operator(&id).unwrap();
        (operator.pusher)(type_erased_vec, Box::new(entity_id.clone()));

        archetype.len += 1;

        self.entity_archetype_map.insert(entity_id, bitset);
        entity_id
    }

    pub fn remove_entity(&mut self, entity_id: EntityId) {
        if let Some(id) = self.entity_archetype_map.get(&entity_id) {
            let archetype = self.archetype_id_map.get_mut(id).unwrap();

            let entity_type_id = self.entity_id_id;

            let entity_column = archetype.get_column(&entity_type_id).get::<EntityId>();

            let row = entity_column.iter().position(|x| *x == entity_id).unwrap();

            let mut change = Vec::new();

            for (component_id, index) in &archetype.component_id_column_index_map {
                let operator = self
                    .components
                    .get_component_vec_operator(&component_id)
                    .unwrap();
                
                change.push((operator, *component_id));
                
            }

            for (operator, component_id) in change {
                // Meh doing index 2 times
                let type_erased_vec = archetype.get_column_mut(&component_id);
                (operator.swap_remover)(type_erased_vec, row);
            }

            archetype.len -= 1;
        }
    }

    pub fn remove_component(&mut self, entity_id: EntityId, target_id: ComponentTypeId) {
        // Todo: use if let Some(id) =
        let src_id = *self.entity_archetype_map.get(&entity_id).unwrap();

        let src_archetype = self.archetype_id_map.get_mut(&src_id).unwrap();

        let entity_column = src_archetype
            .get_column(&self.entity_id_id)
            .get::<EntityId>();

        let row = entity_column.iter().position(|x| *x == entity_id).unwrap();

        let mut type_ids = Vec::new();

        for i in 0..255 {
            if(src_id.contains_id(i))
            {
                type_ids.push(ComponentTypeId(i));
            }
        }

        let index_of_target = type_ids.iter().position(|id| *id == target_id).unwrap();

        type_ids.remove(index_of_target);

        let mut dest_id = BitSet::new();

        for type_id in &type_ids {
            dest_id.insert_id(target_id.0);
        }

        drop(src_archetype);

        if !self.archetype_id_map.contains_key(&dest_id) {
            let mut archetype = self.create_archetype_from_type_ids(&type_ids);

            self.archetype_id_map.insert(dest_id, archetype);
        };

        // 4. move all components other than target
        self.migrate_components(&type_ids, src_id, dest_id, row);

        let [src_archetype, dest_archetype] = self
            .archetype_id_map
            .get_many_mut([&src_id, &dest_id])
            .unwrap();

        // update stuff
        self.entity_archetype_map.insert(entity_id, dest_id);
        src_archetype.len -= 1;
        dest_archetype.len += 1;
    }

    fn migrate_components(
        &mut self,
        type_ids: &Vec<ComponentTypeId>,
        src_id: BitSet,
        dest_id: BitSet,
        src_index: usize,
    ) {
        let [src_archetype, dest_archetype] = self
            .archetype_id_map
            .get_many_mut([&src_id, &dest_id])
            .unwrap();

        for component_type_id in type_ids {
            let operator = self
                .components
                .get_component_vec_operator(component_type_id)
                .unwrap();

            let src_vec = src_archetype.get_column_mut(component_type_id);
            let dest_vec = dest_archetype.get_column_mut(component_type_id);

            (operator.migrator)(src_vec, dest_vec, src_index);
        }
    }

    fn get_bit_set_id(&self, type_ids: &Vec<TypeId>) -> BitSet {
        let mut bitset = BitSet::new();

        for type_id in type_ids {
            bitset.insert_id(self.components.get_component_id(&type_id).unwrap().0);
        }

        bitset
    }

    fn create_archetype_from_type_ids(&mut self, type_ids: &Vec<ComponentTypeId>) -> Archetype {
        let mut archetype = Archetype::new();

        for component_type_id in type_ids {
            let operator = self
                .components
                .get_component_vec_operator(&component_type_id)
                .unwrap();

            archetype.insert_column(component_type_id.clone(), (operator.creator)());
        }

        archetype
    }

    #[inline(always)]
    pub fn query_single<'a, Q: Query<'a>>(&'a self) -> Q::IterType {
        Q::query(self).next().unwrap()
    }

    #[inline(always)]
    pub fn query<'a, Q: Query<'a>>(&'a self) -> impl Iterator<Item = Q::IterType> {
        Q::query(self)
    }

    #[inline(always)]
    pub fn query_mut_single<'a, Q: Query<'a>>(&'a mut self) -> Q::IterMutType {
        Q::query_mut(self).next().unwrap() // Todo: Custom single query might have better performance than this
    }

    #[inline(always)]
    pub fn query_mut<'a, Q: Query<'a>>(&'a mut self) -> impl Iterator<Item = Q::IterMutType> {
        Q::query_mut(self)
    }
}

#[derive(Default)]
pub struct Schedular<T: Hash + Eq + PartialEq + Copy + Clone, V> {
    systems: HashMap<T, Vec<fn(&mut V)>>,
}

impl<T: Hash + Eq + PartialEq + Copy + Clone, V> Schedular<T, V> {
    pub fn new() -> Self {
        Self {
            systems: HashMap::new(),
        }
    }

    pub fn add_system(&mut self, stage: T, fun: fn(&mut V)) {
        if !self.systems.contains_key(&stage) {
            self.systems.insert(stage, Vec::new());
        }

        self.systems.get_mut(&stage).unwrap().push(fun);
    }

    pub fn run(&mut self, stage: T, data: &mut V) {
        if let Some(systems) = self.systems.get(&stage) {
            for system in systems {
                system(data);
            }
        }
    }
}

pub trait Query<'a> {
    type IterType;
    type IterMutType;

    fn get_bit_set_id(world: &'a World) -> BitSet;
    fn get_component_id(world: &'a World) -> Vec<ComponentTypeId>;

    fn query(world: &'a World) -> impl Iterator<Item = Self::IterType>;
    fn query_mut(world: &'a mut World) -> impl Iterator<Item = Self::IterMutType>;
}

impl<'a, T: Component> Query<'a> for (T,) {
    type IterType = (&'a T,);
    type IterMutType = (&'a mut T,);

    fn get_bit_set_id(world: &'a World) -> BitSet {
        let mut bitset = BitSet::new();

        let id = world
            .components
            .get_component_id(&TypeId::of::<T>())
            .unwrap();
        bitset.insert_id(id.0);

        bitset
    }

    fn get_component_id(world: &'a World) -> Vec<ComponentTypeId> {
        let id = world
            .components
            .get_component_id(&TypeId::of::<T>())
            .unwrap();

        vec![id]
    }

    fn query(world: &'a World) -> impl Iterator<Item = Self::IterType> {
        let mut bitset = BitSet::new();

        bitset.insert_id(
            world
                .components
                .get_component_id(&TypeId::of::<T>())
                .unwrap()
                .0,
        );

        let target_bitset = bitset;

        let a = world
            .archetype_id_map
            .iter()
            .filter(move |&(bitset, _)| bitset.contains(&target_bitset))
            .map(|(bitset, archetype)| {

                let vec_of_vecs = archetype.get_many_columns(&vec![
                    world
                    .components
                    .get_component_id(&TypeId::of::<T>())
                    .unwrap()
                ]);

                let mut iter = vec_of_vecs.into_iter();

                iter.next().unwrap().get::<T>().iter().map(|x| (x,))
            });

        a.flatten()
    }

    fn query_mut(world: &'a mut World) -> impl Iterator<Item = Self::IterMutType> {
        let mut bitset = BitSet::new();

        bitset.insert_id(
            world
                .components
                .get_component_id(&TypeId::of::<T>())
                .unwrap()
                .0,
        );

        let target_bitset = bitset;

        let a = world
            .archetype_id_map
            .iter_mut()
            .filter(move |&(bitset, _)| bitset.contains(&target_bitset))
            .map(|(_, archetype)| {
                
                let vec_of_vecs = archetype.get_many_columns_mut(&vec![
                    world
                    .components
                    .get_component_id(&TypeId::of::<T>())
                    .unwrap(),
                ]);

                let mut iter = vec_of_vecs.into_iter();

                let b = iter.next().unwrap();
                b.get_mut::<T>().iter_mut().map(|x| (x,))
            });

        a.flatten()
    }
}

macro_rules! impl_query {
    ($n:literal, $($t: ident,)+) => {
        impl<'a, $($t: Component,)+> Query<'a> for ($($t,)+) {

            type IterType = ( $(&'a $t,)+ );
            type IterMutType = ( $(&'a mut $t,)+ );

            fn get_bit_set_id(world: &'a World) -> BitSet {
                let mut bitset = BitSet::new();
                $(
                    let id = world.components.get_component_id(&TypeId::of::<$t>()).unwrap();
                    bitset.insert_id(id.0);
                )+

                bitset
            }

            fn get_component_id(world: &'a World) -> Vec<ComponentTypeId> {

                vec![
                    $(world
                    .components
                    .get_component_id(&TypeId::of::<$t>())
                    .unwrap(),)+
                ]
            }

            fn query(world: &'a World) -> impl Iterator<Item = Self::IterType> {

                let mut bitset = BitSet::new();

                $(bitset.insert_id(
                    world
                        .components
                        .get_component_id(&TypeId::of::<$t>())
                        .unwrap()
                        .0,
                );)+

                let target_bitset = bitset;

                let a = world
                    .archetype_id_map
                    .iter()
                    .filter(move |&(bitset, _)| bitset.contains(&target_bitset))
                    .map(|(bitset, archetype)| {

                        let vec_of_vecs = archetype.get_many_columns(&vec![
                            $(world.components.get_component_id(&TypeId::of::<$t>()).unwrap(),)+
                        ]);

                        let mut iter = vec_of_vecs.into_iter();

                        izip!(
                            $(iter.next().unwrap().get::<$t>().iter(),)+
                        )
                    });

                a.flatten()
            }

            fn query_mut(world: &'a mut World) -> impl Iterator<Item = Self::IterMutType> {

                let mut bitset = BitSet::new();

                $(bitset.insert_id(
                    world
                        .components
                        .get_component_id(&TypeId::of::<$t>())
                        .unwrap()
                        .0,
                );)+

                let target_bitset = bitset;

                let a = world
                    .archetype_id_map
                    .iter_mut()
                    .filter(move |&(bitset, _)| bitset.contains(&target_bitset))
                    .map(|(_, archetype)| {

                        let vec_of_vecs = archetype.get_many_columns_mut(&vec![
                            $(world.components.get_component_id(&TypeId::of::<$t>()).unwrap(),)+
                        ]);


                        let mut iter = vec_of_vecs.into_iter();

                        izip!(
                            $({
                                let a: &'a mut TypeErasedComponentVec = iter.next().unwrap();
                                a.get_mut::<$t>().iter_mut()
                            },)+
                        )
                    });

                a.flatten()
            }
        }
    };
}

impl_query!(2, A, B,);
impl_query!(3, A, B, C,);
impl_query!(4, A, B, C, D,);
impl_query!(5, A, B, C, D, E,);
impl_query!(6, A, B, C, D, E, F,);
impl_query!(7, A, B, C, D, E, F, G,);
impl_query!(8, A, B, C, D, E, F, G, H,);
impl_query!(9, A, B, C, D, E, F, G, H, I,);
