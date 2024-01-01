use hashbrown::HashMap;
use std::any::Any;
use std::any::TypeId;

pub struct Singletons {
    map: HashMap<TypeId, Box<dyn Any>>,
}

pub trait TypeSet<'a> {
    type Output;
    type OutputMut;
    fn get(singletons: &'a Singletons) -> Option<Self::Output>;
    fn get_mut(singletons: &'a mut Singletons) -> Option<Self::OutputMut>;
}

impl Singletons {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert<T: Any>(&mut self, data: T) -> Option<T> {
        if let Some(old_data) = self.map.insert(TypeId::of::<T>(), Box::new(data)) {
            let a = *old_data.downcast::<T>().unwrap();

            return Some(a);
        }

        None
    }

    pub fn remove<T: Any>(&mut self) -> Option<T> {
        if let Some(old_data) = self.map.remove(&TypeId::of::<T>()) {
            let a: T = *old_data.downcast().unwrap();

            return Some(a);
        }

        None
    }

    pub fn get<T: Any>(&self) -> Option<&T> {
        if let Some(data) = self.map.get(&TypeId::of::<T>()) {
            let a: &T = data.downcast_ref().unwrap();

            return Some(a);
        }

        None
    }

    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        if let Some(data) = self.map.get_mut(&TypeId::of::<T>()) {
            let a: &mut T = data.downcast_mut().unwrap();

            return Some(a);
        }

        None
    }

    pub fn get_many<'a, T: TypeSet<'a>>(&'a self) -> Option<T::Output> {
        T::get(self)
    }

    pub fn get_many_mut<'a, T: TypeSet<'a>>(&'a mut self) -> Option<T::OutputMut> {
        T::get_mut(self)
    }
}

// impl<'a, T: Any, V: Any> TypeSet<'a> for (T, V) {
//     type Output = (&'a T, &'a V);
//     type OutputMut = (&'a mut T, &'a mut V);

//     fn get(singletons: &'a Singletons) -> Option<Self::Output> {
//         if !singletons.map.contains_key(&TypeId::of::<T>())
//             || !singletons.map.contains_key(&TypeId::of::<V>())
//         {
//             return None;
//         }

//         let a = (
//             singletons
//                 .map
//                 .get(&TypeId::of::<T>())
//                 .unwrap()
//                 .downcast_ref::<T>()
//                 .unwrap(),
//             singletons
//                 .map
//                 .get(&TypeId::of::<T>())
//                 .unwrap()
//                 .downcast_ref::<V>()
//                 .unwrap(),
//         );

//         return Some(a);
//     }

//     fn get_mut(singletons: &'a mut Singletons) -> Option<Self::OutputMut> {
//         let a = singletons
//             .map
//             .get_many_mut([&TypeId::of::<T>(), &TypeId::of::<V>()]);

//         if a.is_none() {
//             return None;
//         }

//         let [t, v] = a.unwrap();

//         let c = (
//             t.downcast_mut::<T>().unwrap(),
//             v.downcast_mut::<V>().unwrap(),
//         );

//         return Some(c);
//     }
// }

macro_rules! impl_type_set {
    ( $(($ty: ident, $name: tt)),+) => {
        impl<'a, $($ty: Any,) +> TypeSet<'a> for ($($ty,)+){

            type Output = ($(&'a $ty,)+);
            type OutputMut = ($(&'a mut $ty,)+);

            fn get(singletons: &'a Singletons) -> Option<Self::Output> {

                if $(!singletons.map.contains_key(&TypeId::of::<$ty>()) ||)+ false {
                    return None;
                }

                let a = (
                    $(
                        singletons
                        .map
                        .get(&TypeId::of::<$ty>())
                        .unwrap()
                        .downcast_ref::<$ty>()
                        .unwrap(),
                    )+
                );

                return Some(a);
            }

            fn get_mut(singletons: &'a mut Singletons) -> Option<Self::OutputMut> {

                let a = singletons.map.get_many_mut([$(&TypeId::of::<$ty>(),)+]);

                if a.is_none() {
                    return None;
                }

                let [
                    $($name,)+
                ] = a.unwrap();

                let b = (
                    $(
                        $name.downcast_mut::<$ty>().unwrap(),
                    )+
                );

                return Some(b);
            }
        }
    };
}

impl_type_set!((A, a));
impl_type_set!((A, a), (B, b));
impl_type_set!((A, a), (B, b), (C, c));
impl_type_set!((A, a), (B, b), (C, c), (D, d));
impl_type_set!((A, a), (B, b), (C, c), (D, d), (E, e));
