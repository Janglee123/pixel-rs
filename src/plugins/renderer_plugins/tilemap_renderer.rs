use crate::ecs::world::Component;

pub struct TileData {
    pos: Vector2,
    color: Color,
}

pub struct TileMap {}

impl Component for TileMap {}
