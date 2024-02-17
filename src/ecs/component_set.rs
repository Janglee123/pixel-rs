use std::any::{Any, TypeId};

use hashbrown::HashMap;

use crate::ecs::entity::EntityId;

use super::{
    archetype::Archetype,
    bitset::BitSet,
    component::{Component, ComponentTypeId, Components, TypeErasedComponentVec},
    world::World,
};

pub trait ComponentSet {
    fn get_type_id_vec() -> Vec<TypeId>;

    fn get_map(self) -> HashMap<TypeId, Box<dyn Any>>;
}

impl<T: Component> ComponentSet for (T,) {
    // I am not adding Entity Id in this!!
    fn get_type_id_vec() -> Vec<TypeId> {
        // Todo: I should not push entity id here
        vec![TypeId::of::<EntityId>(), TypeId::of::<T>()]
    }

    fn get_map(self) -> HashMap<TypeId, Box<dyn Any>> {
        let mut a: HashMap<TypeId, Box<dyn Any>> = HashMap::new();

        a.insert(TypeId::of::<T>(), Box::new(self.0));

        a
    }
}

macro_rules! impl_component_set {
    ($(($t: ident, $index: tt),)+) => {
        impl<$($t: Component,)+> ComponentSet for ($($t,)+) {

            fn get_type_id_vec() -> Vec<TypeId> {
                vec![TypeId::of::<EntityId>(),
                $(TypeId::of::<$t>(),)+
                ]
            }

            fn get_map(self) -> HashMap<TypeId, Box<dyn Any>> {
                let mut a: HashMap<TypeId, Box<dyn Any>> = HashMap::new();

                $( a.insert(TypeId::of::<$t>(), Box::new(self.$index)); )+

                a
            }

        }
    };
}

impl_component_set!((A, 0), (B, 1),);
impl_component_set!((A, 0), (B, 1), (C, 2),);
impl_component_set!((A, 0), (B, 1), (C, 2), (D, 3),);
impl_component_set!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4),);
impl_component_set!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5),);
impl_component_set!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6),);
