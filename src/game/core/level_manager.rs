use hashbrown::{HashMap, HashSet};

use crate::{
    game::{
        resources::{
            building_descriptor::BuildingDescriptor,
            level_descriptors::{self, LevelDescriptor},
            resource_stack::{self, GameResource, ResourceStack},
        },
        road,
    },
    math::honeycomb::{self, Hextor}, ecs::world::WorldEventData,
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

    pub fn is_available(&self, resource: &ResourceStack) -> bool {
        if (resource.count as u16) < *self.inventory.get(&resource.resource_type).unwrap() {
            return false;
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

#[derive(Clone)]
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

pub struct TilesAddedEvent;
impl WorldEventData for TilesAddedEvent {}

pub struct RoadAddedEvent {
    pub new_road: Hextor
}

impl WorldEventData for RoadAddedEvent {}

#[derive(Clone)]
pub struct Action {
    pub building: Option<BuildingInstance>,
    pub road: Option<Hextor>,
    pub tiles_added: Option<Vec<Hextor>>,
    pub score_change: u16,
    pub resource_change: [ResourceStack; 3],
}

pub struct UndoRedo {
    pub undo_stack: Vec<Action>,
    pub redo_stack: Vec<Action>,
}

impl UndoRedo {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn add_action(&mut self, action: Action) {
        self.undo_stack.push(action);
        self.redo_stack.clear();
    }

    pub fn get_undo_action(&mut self) -> Action {
        let action = self.undo_stack.pop().unwrap();
        self.redo_stack.push(action.clone());

        return action;
    }

    pub fn get_redo_action(&mut self) -> Action {
        let action = self.redo_stack.pop().unwrap();

        self.undo_stack.push(action.clone());

        return action;
    }

    pub fn is_undo_empty(&self) -> bool {
        self.undo_stack.len() == 0
    }

    pub fn is_redo_empty(&self) -> bool {
        self.redo_stack.len() == 0
    }
}

pub struct CanBuyBuildingResult {
    pub can_buy: bool,
    pub inefficient_resources: Vec<ResourceStack>,
}

pub struct BuildingPlaceQueryResult {
    pub can_place: bool,
    pub not_empty_tiles: Vec<Hextor>, // tiles which are not empty
    pub not_connected_to_road: bool,

    pub score_change: i8,
    // pub effects: HashMap<>
}

pub struct BuildingPlaceResult {
    pub tiles_added: bool,
    pub new_score: u16,
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
    pub fn new(level_descriptor: &LevelDescriptor) -> Self {
        let mut manager = Self {
            inventory: InventoryManager::new(),
            stats: StatsManager::new(),
            buildings: Buildings::new(),
            roads: Roads::new(),
            ground: Ground::new(),
            undo_redo: UndoRedo::new(),
        };

        // unlock zero'th area noooo

        manager.ground.add_tiles(&level_descriptor.areas[0].tiles);
        manager
            .inventory
            .add_items(&level_descriptor.areas[0].reward);
        manager.roads.add_road(level_descriptor.starting_road);

        manager
    }

    pub fn undo(&mut self) {
        let action = self.undo_redo.get_undo_action();

        if let Some(building_instance) = action.building {
            // Remove that building
        }

        if let Some(Hextor) = action.road {
            // Remove that road
        }

        if let Some(tiles) = action.tiles_added {
            // Remove those tiles
        }

        self.stats.score -= action.score_change;
        self.inventory.add_items(&action.resource_change);
    }

    pub fn redo(&mut self) {
        let action = self.undo_redo.get_redo_action();

        if let Some(building_instance) = action.building {
            // add that building
        }

        if let Some(Hextor) = action.road {
            // add that road
        }

        if let Some(tiles) = action.tiles_added {
            // add those tiles
        }

        self.stats.score += action.score_change;
        self.inventory.remove_items(&action.resource_change);
    }

    pub fn place_building(&mut self, building: BuildingInstance) {
        // Update score

        // check if new area is unlocked? How to check? I need progress manager or ground can do it?

        let mut resource_change = ResourceStack::resource_array(0, 0, 0);

        for price in building.descriptor.price.iter() {
            resource_change[price.resource_type as usize].count -= price.count;
        }

        for reward in building.descriptor.reward.iter() {
            resource_change[reward.resource_type as usize].count += reward.count;
        }

        let action = Action {
            building: Some(building.clone()),
            road: None,
            tiles_added: None,
            score_change: 0, // Todo
            resource_change,
        };

        self.undo_redo.add_action(action);

        self.buildings.add_building(building)
    }

    pub fn can_buy_building(&self, building: &BuildingInstance) -> CanBuyBuildingResult {
        let mut can_buy = true;
        let mut inefficient_resources = Vec::new();

        for resource in building.descriptor.price.iter() {
            if !self.inventory.is_available(resource) {
                can_buy = false;
                inefficient_resources.push(*resource);
            }
        }

        CanBuyBuildingResult {
            can_buy,
            inefficient_resources,
        }
    }

    pub fn can_place_building(&self, building: BuildingInstance) -> BuildingPlaceQueryResult {
        let mut can_place = true;
        let mut not_empty_tiles = Vec::<Hextor>::new(); // tiles which are not empty
        let mut score_change = building.descriptor.base_score;
        let mut is_connected = false;

        for tile in building.rotated_tiles {
            let absolute_tile = tile + building.center;

            if !self.buildings.is_empty(&absolute_tile) {
                not_empty_tiles.push(absolute_tile);
                can_place = false;
            }

            if !self.roads.is_empty(&absolute_tile) {
                not_empty_tiles.push(absolute_tile);
                can_place = false;
            }

            is_connected = is_connected || self.roads.is_connected_to_road(&absolute_tile)
        }

        BuildingPlaceQueryResult {
            can_place,
            not_empty_tiles,
            not_connected_to_road: !is_connected,
            score_change,
        }
    }

    pub fn can_place_road(&self, tile: &Hextor) -> bool {
        self.buildings.is_empty(tile)
            && self.roads.is_empty(tile)
            && self.roads.is_connected_to_road(tile)
    }

    pub fn place_road(&mut self, tile: Hextor) -> BuildingPlaceResult {
        let action = Action {
            building: None,
            road: Some(tile),
            tiles_added: None,
            score_change: 0,
            resource_change: ResourceStack::resource_array(0, 0, 0),
        };

        self.undo_redo.add_action(action);
        self.roads.add_road(tile);

        BuildingPlaceResult {
            tiles_added: false,
            new_score: 0,
        }
    }

    pub fn get_tiles(&self) -> &HashSet<Hextor> {
        &self.ground.tiles
    }

    pub fn is_road(&self, tile: &Hextor) -> bool {
        !self.roads.is_empty(tile)
    }
}
