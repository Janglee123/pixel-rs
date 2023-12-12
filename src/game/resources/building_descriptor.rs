use crate::math::honeycomb::Hexter;

use super::{neighborhood_effect::NeighborhoodEffect, resource_stack::ResourceStack};

pub struct BuildingDescriptor {
    pub id: u8,
    pub shape: Vec<Hexter>,
    pub price: Vec<ResourceStack>,
    pub reward: Vec<ResourceStack>,
    pub neighborhood_effects: Vec<NeighborhoodEffect>,
    pub base_score: i8,
}