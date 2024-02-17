#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub struct EntityId(u64);

impl EntityId {
    pub const INVALID: EntityId = EntityId(u64::MAX);

    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

pub struct EntityRef {}

pub struct Entities {
    counter: u64,
}

impl Entities {
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    pub fn get_new_entity_id(&mut self) -> EntityId {
        let id = self.counter;

        self.counter += 1;

        EntityId(id)
    }
}
