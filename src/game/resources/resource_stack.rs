#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum GameResource {
    Dood,
    Wood,
    Food,

    MAX,
}

#[derive(Clone, Copy)]
pub struct ResourceStack {
    pub resource_type: GameResource,
    pub count: i16,
}

impl ResourceStack {
    pub fn new(resource_type: GameResource, count: i16) -> Self {
        Self {
            resource_type,
            count,
        }
    }

    pub fn resource_array(dood: i16, wood: i16, food: i16) -> [Self; 3] {
        [
            Self::new(GameResource::Dood, dood),
            Self::new(GameResource::Wood, wood),
            Self::new(GameResource::Food, food),
        ]
    }
}
