use hashbrown::{HashMap, HashSet};
use log::Level;

use crate::{
    game::resources::{
        self,
        building_descriptor::BuildingDescriptor,
        resource_stack::{GameResource, ResourceStack},
    },
    math::honeycomb::{self, Hextor},
};

pub struct InventoryManager {
    inventory: HashMap<GameResource, u16>,
}

impl InventoryManager {
    pub fn new() -> Self {
        let mut inventory = HashMap::new();

        inventory.insert(GameResource::Dood, 0);
        inventory.insert(GameResource::Wood, 0);
        inventory.insert(GameResource::Food, 0);

        Self { inventory }
    }

    pub fn add_items(&mut self, resources: &[ResourceStack]) {
        for resource in resources {
            let count = self.inventory[&resource.resource_type];

            self.inventory
                .insert(resource.resource_type, count + (resource.count as u16));
        }
    }

    pub fn remove_items(&mut self, resources: &[ResourceStack]) {
        for resource in resources {
            let count = self.inventory[&resource.resource_type];

            self.inventory
                .insert(resource.resource_type, count - (resource.count as u16));
        }
    }

    pub fn is_available(&self, resources: &[ResourceStack]) -> bool {
        for resource in resources {
            if (resource.count as u16) < *self.inventory.get(&resource.resource_type).unwrap() {
                return false;
            }
        }

        true
    }
}

pub struct StatsManager {
    pub score: u16,
}

impl StatsManager {
    pub fn new() -> Self {
        Self { score: 0 }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct BuildingInstanceId(u32);

pub struct BuildingInstance {
    pub instance_id: BuildingInstanceId,
    pub center: Hextor,
    pub rotated_tiles: Vec<Hextor>,
    pub descriptor: &'static BuildingDescriptor,
}

pub struct Buildings {
    tile_map: HashMap<Hextor, BuildingInstanceId>,
    id_map: HashMap<BuildingInstanceId, BuildingInstance>,
    instance_id: u32,
}

impl Buildings {
    pub fn new() -> Self {
        Self {
            tile_map: HashMap::new(),
            id_map: HashMap::new(),
            instance_id: 1u32,
        }
    }

    pub fn add_building(&mut self, building: BuildingInstance) {
        for tile in building.rotated_tiles.iter() {
            let absolute_tile = *tile + building.center;
            self.tile_map.insert(absolute_tile, building.instance_id);
        }

        self.id_map.insert(building.instance_id, building);
    }

    pub fn remove_building(&mut self, id: BuildingInstanceId) -> BuildingInstance {
        let building = self.id_map.remove(&id).unwrap();

        for tile in building.rotated_tiles.iter() {
            let absolute_tile = *tile + building.center;
            self.tile_map.remove(&absolute_tile);
        }

        building
    }

    pub fn get_building_instance_id(&mut self) -> BuildingInstanceId {
        let id = BuildingInstanceId(self.instance_id);

        self.instance_id += 1;

        id
    }

    pub fn is_empty(&self, tile: &Hextor) -> bool {
        return !self.tile_map.contains_key(tile);
    }
}

pub struct Roads {
    tile_map: HashSet<Hextor>,
}

impl Roads {
    pub fn new() -> Self {
        Self {
            tile_map: HashSet::new(),
        }
    }

    pub fn add_road(&mut self, tile: Hextor) {
        self.tile_map.insert(tile);
    }

    pub fn remove_road(&mut self, tile: Hextor) {
        self.tile_map.remove(&tile);
    }

    pub fn is_connected_to_road(&self, tile: &Hextor) -> bool {
        for adj in honeycomb::DIRECTION_VECTORS.iter() {
            let tile = *adj + *tile;

            if self.tile_map.contains(&tile) {
                return true;
            }
        }

        false
    }

    pub fn is_empty(&self, tile: &Hextor) -> bool {
        !self.tile_map.contains(tile)
    }
}

pub struct Ground {
    pub tiles: HashSet<Hextor>,
}

impl Ground {
    pub fn new() -> Self {
        Self {
            tiles: HashSet::new(),
        }
    }

    pub fn add_tiles(&mut self, tiles: &Vec<Hextor>) {
        for tile in tiles {
            self.tiles.insert(*tile);
        }
    }

    pub fn remove_tiles(&mut self, tiles: &Vec<Hextor>) {
        for tile in tiles {
            self.tiles.remove(tile);
        }
    }

    pub fn has_tile(&self, tile: &Hextor) -> bool {
        self.tiles.contains(tile)
    }
}

pub struct UndoRedo {}

impl UndoRedo {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct LevelManager {
    inventory: InventoryManager,
    stats: StatsManager,
    buildings: Buildings,
    roads: Roads,
    ground: Ground,
    undo_redo: UndoRedo,
}

impl LevelManager {
    pub fn new() -> Self {
        Self {
            inventory: InventoryManager::new(),
            stats: StatsManager::new(),
            buildings: Buildings::new(),
            roads: Roads::new(),
            ground: Ground::new(),
            undo_redo: UndoRedo::new(),
        }
    }

    pub fn place_building(&mut self, building: BuildingInstance) {
        // Update score
        // Update undo redo
        self.buildings.add_building(building)
    }

    pub fn can_place_building(&self, building: BuildingInstance) -> bool {
        if !self.inventory.is_available(&building.descriptor.price) {
            return false;
        }

        let mut is_connected = false;
        // for all tiles
        for tile in building.rotated_tiles {
            let absolute_tile = tile + building.center;

            if !self.buildings.is_empty(&absolute_tile) {
                return false;
            }

            if !self.roads.is_empty(&absolute_tile) {
                return false;
            }

            is_connected = is_connected || self.roads.is_connected_to_road(&absolute_tile)
        }

        if !is_connected {
            return false;
        }

        return true;
    }

    pub fn can_place_road(&self, tile: &Hextor) -> bool {
        self.buildings.is_empty(tile)
            && self.roads.is_empty(tile)
            && self.roads.is_connected_to_road(tile)
    }

    pub fn place_road(&mut self, tile: Hextor) {
        self.roads.add_road(tile)
    }
}
