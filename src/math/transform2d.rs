use std::ops::Mul;

use bytemuck::{Pod, Zeroable};
use glam::{Mat3, Vec2, Vec3};

#[derive(Debug, Default, Clone)]

pub struct Transform2d {
    pub position: Vec2,
    pub rotation: f32, // Rotation in radiance
    pub scale: Vec2,
}

impl Transform2d {
    pub fn new(position: Vec2, rotation: f32, scale: Vec2) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn from_xy(x: f32, y: f32) -> Self {
        Self {
            position: Vec2 { x, y },
            rotation: 0.0,
            scale: Vec2 { x: 1.0, y: 1.0 },
        }
    }

    pub fn create_matrix(&self) -> Mat3 {
        Mat3 {
            x_axis: Vec3::new(
                self.scale.x * self.rotation.cos(),
                self.scale.x * self.rotation.sin(),
                0.0,
            ),
            y_axis: Vec3::new(
                -self.scale.y * self.rotation.sin(),
                self.scale.y * self.rotation.cos(),
                0.0,
            ),
            z_axis: Vec3::new(self.position.x, self.position.y, 1.0),
        }
    }

    pub const IDENTITY: Transform2d = Transform2d {
        position: Vec2 { x: 0.0, y: 0.0 },
        rotation: 0.0,
        scale: Vec2 { x: 1.0, y: 1.0 },
    };
}
