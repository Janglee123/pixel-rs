
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum GameResource {
    Dood,
    Wood,
    Food,


    MAX,
}

pub struct ResourceStack {
    pub resource_type: GameResource,
    pub count: u8,
}