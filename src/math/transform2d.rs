use std::ops::Mul;

use bytemuck::{Pod, Zeroable};

use super::vector2::Vector2;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug, Default)]
pub struct Matrix3 {
    pub x: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    pub y: [f32; 3],
    _padding2: u32,
    pub origin: [f32; 3],
    _padding1: u32,
}

impl Matrix3 {
    pub const IDENTITY: Matrix3 = Matrix3 {
        x: [1.0, 0.0, 0.0],
        y: [0.0, 1.0, 0.0],
        origin: [0.0, 0.0, 1.0],
        _padding: 0,
        _padding1: 0,
        _padding2: 0,
    };
}

impl Mul for Matrix3 {
    type Output = Matrix3;

    fn mul(self, rhs: Self) -> Self::Output {
        // ahh noo
        let x_x = self.x[0] * rhs.x[0] + self.y[0] * rhs.x[1] + self.origin[0] * rhs.x[2];
        let x_y = self.x[1] * rhs.x[0] + self.y[1] * rhs.x[1] + self.origin[1] * rhs.x[2];
        let x_z = self.x[2] * rhs.x[0] + self.y[2] * rhs.x[1] + self.origin[2] * rhs.x[2];

        let y_x = self.x[0] * rhs.y[0] + self.y[0] * rhs.y[1] + self.origin[0] * rhs.y[2];
        let y_y = self.x[1] * rhs.y[0] + self.y[1] * rhs.y[1] + self.origin[1] * rhs.y[2];
        let y_z = self.x[2] * rhs.y[0] + self.y[2] * rhs.y[1] + self.origin[2] * rhs.y[2];

        let z_x =
            self.x[0] * rhs.origin[0] + self.y[0] * rhs.origin[1] + self.origin[0] * rhs.origin[2];
        let z_y =
            self.x[1] * rhs.origin[0] + self.y[1] * rhs.origin[1] + self.origin[1] * rhs.origin[2];
        let z_z =
            self.x[2] * rhs.origin[0] + self.y[2] * rhs.origin[1] + self.origin[2] * rhs.origin[2];

        Self::Output {
            x: [x_x, x_y, x_z],
            y: [y_x, y_y, y_z],
            origin: [z_x, z_y, z_z],
            _padding: 0,
            _padding1: 0,
            _padding2: 0,
        }
    }
}

#[derive(Debug, Default, Clone)]

pub struct Transform2d {
    pub position: Vector2<f32>,
    pub rotation: f32, // Rotation in radiance
    pub scale: Vector2<f32>,
}

impl Transform2d {
    pub fn new(position: Vector2<f32>, rotation: f32, scale: Vector2<f32>) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn from_xy(x: f32, y: f32) -> Self {
        Self {
            position: Vector2 { x, y },
            rotation: 0.0,
            scale: Vector2 { x: 1.0, y: 1.0 },
        }
    }

    pub fn into_matrix(&self) -> Matrix3 {
        Matrix3 {
            x: [
                self.scale.x * self.rotation.cos(),
                self.scale.x * self.rotation.sin(),
                0.0,
            ],
            y: [
                -self.scale.y * self.rotation.sin(),
                self.scale.y * self.rotation.cos(),
                0.0,
            ],
            origin: [self.position.x, self.position.y, 1.0],
            ..Default::default()
        }
    }

    pub const IDENTITY: Transform2d = Transform2d {
        position: Vector2 { x: 0.0, y: 0.0 },
        rotation: 0.0,
        scale: Vector2 { x: 1.0, y: 1.0 },
    };
}
