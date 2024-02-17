use std::{
    any::Any,
    cell::RefCell,
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
    rc::Rc,
};

use hashbrown::HashMap;

use crate::app::Plugin;

pub trait Asset {
    fn from_binary(binary: Vec<u8>) -> Self;
}

#[derive(Debug)]
pub struct AssetRef<T: Asset> {
    id: u64,
    marker: PhantomData<T>,
    counter: Rc<RefCell<u64>>,
}

impl<T: Asset> Clone for AssetRef<T> {
    fn clone(&self) -> Self {
        Self::new(self.id, Rc::clone(&self.counter))
    }
}

impl<T: Asset> AssetRef<T> {
    pub fn new(id: u64, counter: Rc<RefCell<u64>>) -> Self {
        *counter.borrow_mut() += 1;

        Self {
            id,
            marker: PhantomData,
            counter,
        }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }
}

impl<T: Asset> Drop for AssetRef<T> {
    fn drop(&mut self) {
        *self.counter.borrow_mut() -= 1;
    }
}

#[derive(Debug)]
pub struct AssetStorage {
    data: HashMap<u64, Box<dyn Any>>,
    ref_counters: HashMap<u64, Rc<RefCell<u64>>>,
}

impl AssetStorage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            ref_counters: HashMap::new(),
        }
    }

    fn get_id(value: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        let id = hasher.finish();

        id
    }

    pub fn insert<T: Asset + 'static>(&mut self, asset: T, name: &str) -> bool {
        let id = Self::get_id(name);

        if !self.data.contains_key(&id) {
            self.data.insert(id, Box::new(asset));

            return true;
        }

        return false;
    }

    pub fn get<T: Asset + 'static>(&mut self, path: &str) -> Option<AssetRef<T>> {
        let id = Self::get_id(path);

        if !self.data.contains_key(&id) {
            // Todo: Set the path here
            if let Ok(content) = fs::read(path.clone()) {
                self.data.insert(id, Box::new(T::from_binary(content)));
                self.ref_counters.insert(id, Rc::new(RefCell::new(0)));
            } else {
                panic!("Asset at {} not found", path.clone())
            }
        }

        if self.data.contains_key(&id) {
            let counter_ref = self.ref_counters.get(&id).unwrap();

            return Some(AssetRef::new(id, Rc::clone(counter_ref)));
        } else {
            panic!("Asset at path {} id {} not present", path.clone(), id)
        }

        None
    }

    pub fn get_data<T: Asset + 'static>(&self, asset: &AssetRef<T>) -> &T {
        let whatever = self
            .data
            .get(&asset.id)
            .unwrap()
            .downcast_ref::<T>()
            .unwrap();

        whatever
    }

    pub fn remove_unused(&mut self) {
        let mut unused_assets = Vec::new();

        for (id, count) in self.ref_counters.iter() {
            if *count.borrow().deref() == 0 {
                unused_assets.push(id.clone());
            }
        }

        for unused_asset in &unused_assets {
            self.data.remove(unused_asset);
            self.ref_counters.remove(unused_asset);
        }
    }
}

pub struct AssetStoragePlugin;

impl Plugin for AssetStoragePlugin {
    fn build(app: &mut crate::app::App) {
        app.storage.singletons.insert(AssetStorage::new());
    }
}
