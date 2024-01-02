extern crate proc_macro;
use anymap::AnyMap;
use hashbrown::HashMap;
use std::any::{Any, TypeId};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::hash::Hash;

use super::singletons::{self, Singletons};

// This is dumb idea, I should use hibitset
// But anyway I think total 256 components are enough for me
// pub struct ComponentId(u8);
pub type ComponentId = u8;

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct BitSet {
    bitmask: [u64; 4],
}

impl BitSet {
    const INVALID: BitSet = BitSet {
        bitmask: [u64::MAX, u64::MAX, u64::MAX, u64::MAX],
    };

    const EMPTY: BitSet = BitSet {
        bitmask: [u64::MIN, u64::MIN, u64::MIN, u64::MIN],
    };

    pub fn new() -> Self {
        Self { bitmask: [0u64; 4] }
    }

    pub fn insert_id(&mut self, id: u8) -> &mut Self {
        let index = id / 64;
        let position = id - index * 64;
        self.bitmask[index as usize] = self.bitmask[index as usize] | 1 << position;

        self
    }

    pub fn remove_id(&mut self, id: u8) -> &mut Self {
        let index = id / 64;
        let position = id - index * 64;
        let bit_to_remove = 1 << position;
        self.bitmask[index as usize] = !(self.bitmask[index as usize] ^ bit_to_remove);

        self
    }

    pub fn from_id(id: u8) -> Self {
        let mut bitmask = [0u64; 4];
        let index = id / 64;
        let position = id - index * 64;
        bitmask[index as usize] = 1 << position;

        Self { bitmask }
    }

    #[inline(always)]
    pub fn contains(&self, other: &BitSet) -> bool {
        let mut result = true;

        for (a, b) in self.bitmask.iter().zip(other.bitmask.iter()) {
            result &= a & b == *b
        }

        result
    }

    pub fn contains_id(&self, id: u8) -> bool {
        let index = id / 64;
        let position = id - index * 64;

        self.bitmask[index as usize] & 1 << position > 0
    }
}

pub trait Component: 'static {
    fn get_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl<T: 'static> Component for T {}

#[derive(Debug)]
pub struct ComponentVec<T: Component> {
    pub list: Vec<T>,
}

impl<T: Component> ComponentVec<T> {
    pub fn new() -> Self {
        Self { list: Vec::new() }
    }
}

#[derive(Debug)]
pub struct TypelessComponentVec {
    list: Box<dyn Any>, // So inside archetype lists are not stored together how nice ecs
}

impl TypelessComponentVec {
    pub fn new<T: Component>() -> Self {
        Self {
            list: Box::new(ComponentVec::<T>::new()),
        }
    }

    pub fn get_mut<T: Component>(&mut self) -> &mut Vec<T> {
        let a = self.list.downcast_mut::<ComponentVec<T>>().unwrap();
        &mut a.list
    }

    pub fn get<T: Component>(&self) -> &Vec<T> {
        &self.list.downcast_ref::<ComponentVec<T>>().unwrap().list
    }
}

#[derive(Debug)]
pub struct EntityId(u64);

#[derive(Debug)]
pub struct Archetype {
    pub set: HashMap<TypeId, TypelessComponentVec>,
    pub entity_row_map: HashMap<u64, usize>,
    pub length: usize,
}

impl Archetype {
    pub fn new() -> Self {
        Self {
            set: HashMap::new(),
            entity_row_map: HashMap::new(),
            length: 0,
        }
    }

    pub fn get_many<const N: usize>(&self, ks: [TypeId; N]) -> Vec<&TypelessComponentVec> {
        let a: Vec<&TypelessComponentVec> = ks
            .iter()
            .filter(|x| self.set.contains_key(*x))
            .map(|k| self.set.get(k).unwrap())
            .collect();
        a
    }

    pub fn get<T: Component>(&self) -> &Vec<T> {
        let list = self.set.get(&TypeId::of::<T>()).unwrap().get::<T>();

        list
    }

    pub fn get_mut<T: Component>(&mut self) -> &mut Vec<T> {
        let list = self.set.get_mut(&TypeId::of::<T>()).unwrap().get_mut::<T>();

        list
    }

    pub fn add_new_entry(&mut self) -> usize {
        let current_length = self.length;
        self.length += 1;
        current_length
    }
}

pub trait ComponentSet {
    fn get_bit_id_set(id_map: &HashMap<TypeId, ComponentId>) -> BitSet;
    fn insert(self, archetype: &mut Archetype, entity_id: u64);
    fn create_archetype(&self) -> Archetype;
    fn get_type_id() -> Vec<TypeId>;
}

macro_rules! impl_component_set {
    ( $(($t: ident, $index: tt)),+) => {
        impl<$($t: Component,)+> ComponentSet for ($($t,)+) {

            fn get_bit_id_set(id_map: &HashMap<TypeId, ComponentId>) -> BitSet {
                let mut bitset = BitSet::new();

                let id = id_map.get(&TypeId::of::<EntityId>()).unwrap().clone();
                bitset.insert_id(id as u8);

                $(
                    let id = id_map.get(&TypeId::of::<$t>()).unwrap().clone();
                    bitset.insert_id(id as u8);
                )+
                bitset
            }

            fn create_archetype(&self) -> Archetype {
                let mut archetype = Archetype::new();

                archetype
                .set
                .insert(TypeId::of::<EntityId>(), TypelessComponentVec::new::<EntityId>());

                $(
                    archetype
                    .set
                    .insert(TypeId::of::<$t>(), TypelessComponentVec::new::<$t>());
                )+

                archetype
            }

            fn get_type_id() -> Vec<TypeId> {
                vec![
                    $(
                        TypeId::of::<$t>(),
                    )+
                ]
            }

            fn insert(self, archetype: &mut Archetype, entity_id: u64) {

                archetype
                .set
                .get_mut(&TypeId::of::<EntityId>())
                .unwrap()
                .get_mut::<EntityId>()
                .push(EntityId(entity_id));


                $(
                    archetype
                    .set
                    .get_mut(&TypeId::of::<$t>())
                    .unwrap()
                    .get_mut::<$t>()
                    .push(self.$index);
                )+

                let index = archetype.add_new_entry();
                archetype.entity_row_map.insert(entity_id, index);
            }
        }
    };
}

impl_component_set!((A, 0));
impl_component_set!((A, 0), (B, 1));
impl_component_set!((A, 0), (B, 1), (C, 2));
impl_component_set!((A, 0), (B, 1), (C, 2), (D, 3));
impl_component_set!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4));
impl_component_set!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5));
impl_component_set!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6));
impl_component_set!(
    (A, 0),
    (B, 1),
    (C, 2),
    (D, 3),
    (E, 4),
    (F, 5),
    (G, 6),
    (H, 7)
);

pub trait WorldEventData {}

pub struct WorldEvent<T> {
    listeners: Vec<fn(&mut World, &T)>,
}

impl<T> Default for WorldEvent<T> {
    fn default() -> Self {
        Self {
            listeners: Default::default(),
        }
    }
}

pub struct World {
    entity_id_counter: u64,
    component_id_counter: u8,
    pub component_id_map: HashMap<TypeId, ComponentId>,
    pub archetype_id_map: HashMap<BitSet, Archetype>,
    pub singletons: Singletons,
    events: AnyMap,
}

impl World {
    pub fn new() -> Self {
        let mut result = Self {
            entity_id_counter: 0,
            component_id_counter: 0,
            component_id_map: HashMap::new(),
            archetype_id_map: HashMap::new(),
            singletons: Singletons::new(),
            events: AnyMap::new(),
        };

        result.register_component::<EntityId>();

        result
    }

    fn get_new_entity_id(&mut self) -> u64 {
        self.entity_id_counter += 1;
        self.entity_id_counter
    }

    fn get_new_component_id(&mut self) -> u8 {
        self.component_id_counter += 1;
        self.component_id_counter
    }

    pub fn register_component<T: Component>(&mut self) {
        let type_id = TypeId::of::<T>();

        if !self.component_id_map.contains_key(&type_id) {
            let new_id = self.get_new_component_id();
            self.component_id_map.insert(type_id, new_id);
        }
    }

    pub fn insert_entity<T: ComponentSet>(&mut self, component_set: T) -> u64 {
        let entity_id = self.get_new_entity_id();

        for type_id in T::get_type_id() {
            if !self.component_id_map.contains_key(&type_id) {
                let new_id = self.get_new_component_id();
                self.component_id_map.insert(type_id, new_id);
            }
        }

        let bitset = T::get_bit_id_set(&self.component_id_map);

        if !self.archetype_id_map.contains_key(&bitset) {
            let new_archetype = component_set.create_archetype();
            self.archetype_id_map.insert(bitset, new_archetype);
        }

        let archetye = self.archetype_id_map.get_mut(&bitset).unwrap();
        component_set.insert(archetye, entity_id);

        entity_id
    }

    pub fn remove_entity<T: ComponentSet>(&mut self, entity_id: u64) -> Option<T> {
        for archetype in self.archetype_id_map.iter() {
            if let Some(index) = archetype.1.entity_row_map.get(&entity_id) {}
        }

        return None;
    }

    pub fn add_listener<T: WorldEventData + 'static>(&mut self, fun: fn(&mut World, &T)) {
        // Hmm so I have to check if there is any world event already exists or not

        if !self.events.contains::<WorldEvent<T>>() {
            let world_event = WorldEvent::<T>::default();

            self.events.insert(world_event);
        }

        let event = self.events.get_mut::<WorldEvent<T>>().unwrap();
        event.listeners.push(fun);
    }

    pub fn remove_listener<T: WorldEventData + 'static>(&mut self, fun: fn(&mut World, &T)) {
        if let Some(event) = self.events.get_mut::<WorldEvent<T>>() {
            if let Some(index) = event.listeners.iter().position(|x| *x == fun) {
                event.listeners.remove(index);
            }
        }
    }

    pub fn emit<T: WorldEventData + 'static>(&mut self, event_data: T) {
        let vec = Vec::<fn(&mut World, &T)>::new();

        if let Some(event) = self.events.get_mut::<WorldEvent<T>>() {
            // TODO HACK: It will allocate memory every time emit called
            let listener_list = event.listeners.clone();

            listener_list
                .iter()
                .for_each(|fun| (fun)(self, &event_data));
        }
    }
}

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

#[macro_export]
macro_rules! zip {
    ($x: expr) => ($x);
    ($x: expr, $(,)?) => ($x);
    ($x: expr, $($y: expr),+ $(,)?) => (
        $x.zip(
            zip!($($y), +))
    )
}

#[macro_export]
macro_rules! query_mut {
    ($world:expr, $($ty:ty),+ ) => {{
        use std::any::{TypeId};
        use crate::ecs::world::BitSet;

        $world as &World;

        let mut target_bitset = BitSet::new();

        $(
            let id = $world
            .component_id_map
            .get(&TypeId::of::<$ty>())
            .unwrap()
            .clone();

            target_bitset.insert_id(id);
        )*

        let archetype_id_map = &mut $world.archetype_id_map;
        let iter_mut = archetype_id_map.iter_mut();

        iter_mut.filter(move |(bitset, _)| bitset.contains(&target_bitset))
        .map(|(_, archetype)| {

                // So here I have to get the list of all and then zip them all how noice
                let vec_of_vecs = archetype.set.get_many_mut([
                    $(&TypeId::of::<$ty>(),)*
                ]).unwrap();

                let mut iter = vec_of_vecs.into_iter();

                zip!($({
                    iter.next().unwrap().get_mut::<$ty>().iter_mut()
                },)*)
        }).flatten()
    }};
}

#[macro_export]
macro_rules! query {
    ($world:expr, $($ty:ty),+ ) => {{
        use std::any::{TypeId};
        use crate::ecs::world::BitSet;

        $world as &World;

        let mut target_bitset = BitSet::new();

        $(
            let id = $world
            .component_id_map
            .get(&TypeId::of::<$ty>())
            .unwrap()
            .clone();

            target_bitset.insert_id(id);
        )*

        let archetype_id_map = &$world.archetype_id_map;
        let iter_mut = archetype_id_map.iter();

        iter_mut.filter(move |(bitset, _)| bitset.contains(&target_bitset))
        .map(|(_, archetype)| {

                // So here I have to get the list of all and then zip them all how noice
                let vec_of_vecs = archetype.get_many([
                    $(TypeId::of::<$ty>(),)*
                ]);

                let mut iter = vec_of_vecs.into_iter();

                zip!($({
                    let a = iter.next();
                    a.unwrap().get::<$ty>().iter()
                },)*)
        }).flatten()
    }};
}
