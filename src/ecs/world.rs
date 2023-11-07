extern crate proc_macro;
use anymap::AnyMap;
use hashbrown::HashMap;
use std::any::{Any, TypeId};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::hash::Hash;

// This is dumb idea, I should use hibitset
// But anyway I think total 256 components are enough for me
// pub struct ComponentId(u8);
pub type ComponentId = u8;

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct BitSet {
    bitmask: [u64; 4],
    pub id: u8,
}

impl BitSet {
    pub fn new() -> Self {
        Self {
            bitmask: [0u64; 4],
            id: 0,
        }
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

        Self { id, bitmask }
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
    list: Box<dyn Any>,
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
pub struct Archetype {
    pub set: HashMap<TypeId, TypelessComponentVec>,
    pub entity_row_map: HashMap<u64, usize>,
}

impl Archetype {
    pub fn new() -> Self {
        Self {
            set: HashMap::new(),
            entity_row_map: HashMap::new(),
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
}

pub trait ComponentSet {
    fn get_bit_id_set(id_map: &HashMap<TypeId, ComponentId>) -> BitSet;
    fn insert(self, archetype: &mut Archetype, entity_id: u64);
    fn create_archetype(&self) -> Archetype;
    fn get_type_id() -> Vec<TypeId>;
}

// impl<T: Component> ComponentSet for T {
//     fn get_bit_id_set(id_map: &HashMap<TypeId, ComponentId>) -> BitSet {
//         let mut bitset = BitSet::new();

//         let id = id_map.get(&TypeId::of::<T>()).unwrap().clone();
//         bitset.insert_id(id as u8);

//         bitset
//     }

//     fn insert(self, archetype: &mut Archetype, entity_id: u64) {
//         let b = archetype
//             .set
//             .get_mut(&TypeId::of::<T>())
//             .unwrap()
//             .get_mut::<T>();
//         b.list.push(self);
//         archetype.entity_row_map.insert(entity_id, b.list.len() - 1);
//     }

//     fn create_archetype(&self) -> Archetype {
//         let mut archetype = Archetype::new();
//         archetype
//             .set
//             .insert(TypeId::of::<T>(), TypelessComponentVec::new::<T>());

//         archetype
//     }

//     fn get_type_id() -> Vec<TypeId> {
//         vec![TypeId::of::<T>()]
//     }
// }

impl<T: Component> ComponentSet for (T,) {
    fn get_bit_id_set(id_map: &HashMap<TypeId, ComponentId>) -> BitSet {
        let mut bitset = BitSet::new();

        let id = id_map.get(&TypeId::of::<T>()).unwrap().clone();
        bitset.insert_id(id as u8);

        bitset
    }

    fn insert(self, archetype: &mut Archetype, entity_id: u64) {
        archetype
            .set
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .get_mut::<T>()
            .push(self.0);

        let len = archetype
            .set
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .get_mut::<T>()
            .len()
            - 1;
        archetype.entity_row_map.insert(entity_id, len);
    }

    fn create_archetype(&self) -> Archetype {
        let mut archetype = Archetype::new();
        archetype
            .set
            .insert(TypeId::of::<T>(), TypelessComponentVec::new::<T>());

        archetype
    }

    fn get_type_id() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }
}

impl<T: Component, V: Component> ComponentSet for (T, V) {
    fn get_bit_id_set(id_map: &HashMap<TypeId, ComponentId>) -> BitSet {
        let mut bitset = BitSet::new();

        let id = id_map.get(&TypeId::of::<T>()).unwrap().clone();
        bitset.insert_id(id as u8);

        let id = id_map.get(&TypeId::of::<V>()).unwrap().clone();
        bitset.insert_id(id as u8);

        bitset
    }

    fn insert(self, archetype: &mut Archetype, entity_id: u64) {
        archetype
            .set
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .get_mut::<T>()
            .push(self.0);
        archetype
            .set
            .get_mut(&TypeId::of::<V>())
            .unwrap()
            .get_mut::<V>()
            .push(self.1);

        let len = archetype
            .set
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .get_mut::<T>()
            .len()
            - 1;
        archetype.entity_row_map.insert(entity_id, len);
    }

    fn create_archetype(&self) -> Archetype {
        let mut archetype = Archetype::new();
        archetype
            .set
            .insert(TypeId::of::<T>(), TypelessComponentVec::new::<T>());
        archetype
            .set
            .insert(TypeId::of::<V>(), TypelessComponentVec::new::<V>());

        archetype
    }

    fn get_type_id() -> Vec<TypeId> {
        vec![TypeId::of::<T>(), TypeId::of::<V>()]
    }
}

pub struct World {
    entity_id_counter: u64,
    component_id_counter: u8,
    pub component_id_map: HashMap<TypeId, ComponentId>,
    pub archetype_id_map: HashMap<BitSet, Archetype>,
    pub singletons: AnyMap,
}

impl World {
    pub fn new() -> Self {
        let mut map: HashMap<BitSet, Archetype> = HashMap::new();

        let vec = map.get_many_mut([&BitSet::new(); 0]);

        Self {
            entity_id_counter: 0,
            component_id_counter: 0,
            component_id_map: HashMap::new(),
            archetype_id_map: HashMap::new(),
            singletons: AnyMap::new(),
        }
    }

    fn get_new_entity_id(&mut self) -> u64 {
        self.entity_id_counter += 1;
        self.entity_id_counter
    }

    fn get_new_component_id(&mut self) -> u8 {
        self.component_id_counter += 1;
        self.component_id_counter
    }

    pub fn insert_entity<T: ComponentSet>(&mut self, component_set: T) -> u64 {
        let entity_id = self.get_new_entity_id();

        for type_id in T::get_type_id() {
            if !self.component_id_map.contains_key(&type_id) {
                let new_id = self.get_new_component_id();
                self.component_id_map.insert(type_id, new_id);
            }
        }

        // let set_id = component_set.get_type_id();
        let bitset = T::get_bit_id_set(&self.component_id_map);

        if !self.archetype_id_map.contains_key(&bitset) {
            let new_archetype = component_set.create_archetype();
            self.archetype_id_map.insert(bitset, new_archetype);
        }

        let archetye = self.archetype_id_map.get_mut(&bitset).unwrap();
        component_set.insert(archetye, entity_id);

        entity_id
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
