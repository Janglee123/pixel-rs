use crate::math::honeycomb::{Hextor, SpiralLoop};

use super::resource_stack::ResourceStack;

pub struct AreaDescriptor {
    pub tiles: Vec<Hextor>,
    pub reward: [ResourceStack; 3],
    pub unlock_score: u32,
}

pub struct LevelDescriptor {
    // pub starting_resources: [ResourceStack; 3], // Is it needed? because area will always give resources
    pub areas: Vec<AreaDescriptor>, // Areas array must be sorted based on unlock_score
    pub starting_road: Hextor,
}

pub fn get_dummy_level() -> LevelDescriptor {
    let mut areas = Vec::new();

    let centers = [
        Hextor::new(0, 0),
        Hextor::new(4, -2),
        Hextor::new(2, -4),
        Hextor::new(-2, -4),
        Hextor::new(-4, 2),
        Hextor::new(-2, 4),
        Hextor::new(2, 4),
    ];

    let mut i = 0;
    for center in centers {
        let tiles: Vec<Hextor> = SpiralLoop::new(center, 3).collect();

        let area = AreaDescriptor {
            reward: ResourceStack::resource_array(3, 0, 0),
            unlock_score: i * 3,
            tiles: tiles,
        };

        areas.push(area);

        i += 1;
    }

    LevelDescriptor {
        // starting_resources: ResourceStack::resource_array(0, 0, 0),
        areas: areas,
        starting_road: Hextor::new(0,0),
    }
}
