use super::is_zero::IsZero;
use core::num;
use std::{
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
    process::Output, iter::Once,
};

#[derive(PartialEq, Clone, Copy, Debug, Hash, Default)]
pub struct Vector2<T: Mul + Add + Sub + Div + IsZero + Into<f64> + Copy> {
    pub x: T,
    pub y: T,
}

impl<
        T: Mul<Output = T>
            + Add<Output = T>
            + Sub<Output = T>
            + Div<Output = T>
            + IsZero
            + PartialEq
            + Into<f64>
            + Copy,
    > Vector2<T>
{
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn dot(&self, other: &Vector2<T>) -> T {
        ((self.x * other.x) as T) + ((self.y * other.y) as T)
    }

    pub fn cross(&self, other: &Vector2<T>) -> T {
        self.x * other.y - self.y * other.x
    }

    pub fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y.is_zero()
    }

    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y
    }

    pub fn length(&self) -> f64 {
        let x: f64 = self.x.into();
        let y: f64 = self.y.into();
        (x * x + y * y).sqrt()
    }

    pub fn normalize(&self) -> Vector2<f64> {
        let length = self.length();

        assert!(
            length.is_zero(),
            "Length of the vector is zero while trying to normalize"
        );

        Vector2 {
            x: self.x.into() / length,
            y: self.y.into() / length,
        }
    }

    pub fn angle(&self) -> f64 {
        let x: f64 = self.x.into();
        let y: f64 = self.y.into();

        y.atan2(x)
    }

    pub fn angle_to(&self, other: &Self) -> f64 {
        let y: f64 = (self.y - other.y).into();
        let x: f64 = (self.x - other.x).into();

        y.atan2(x)
    }

    pub fn aspect(&self) -> f64 {
        self.x.into() / self.y.into()
    }

    pub fn distance_to(&self, other: Self) -> f64 {
        let y: f64 = (self.y - other.y).into();
        let x: f64 = (self.x - other.x).into();

        (x * x - y * y).sqrt()
    }

    pub fn distance_squared_to(&self, other: Self) -> f64 {
        let y: f64 = (self.y - other.y).into();
        let x: f64 = (self.x - other.x).into();

        x * x - y * y
    }

    pub fn is_normalized(&self) -> bool {
        self.length_squared().into() == 1.0
    }
}

impl<
        T: Mul<Output = T>
            + Add<Output = T>
            + Sub<Output = T>
            + Div<Output = T>
            + IsZero
            + Into<f64>
            + Copy,
    > Add for Vector2<T>
{
    type Output = Vector2<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<
        T: Mul<Output = T>
            + Add<Output = T>
            + Sub<Output = T>
            + Div<Output = T>
            + IsZero
            + Into<f64>
            + AddAssign
            + Copy,
    > AddAssign for Vector2<T>
{
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<
        T: Mul<Output = T>
            + Add<Output = T>
            + Sub<Output = T>
            + Div<Output = T>
            + IsZero
            + Into<f64>
            + Copy,
    > Add<T> for Vector2<T>
{
    type Output = Vector2<T>;

    fn add(self, rhs: T) -> Self::Output {
        Self::Output {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}

impl<
        T: Mul<Output = T>
            + Add<Output = T>
            + Sub<Output = T>
            + Div<Output = T>
            + IsZero
            + Into<f64>
            + AddAssign
            + Copy,
    > AddAssign<T> for Vector2<T>
{
    fn add_assign(&mut self, rhs: T) {
        self.x += rhs;
        self.y += rhs;
    }
}

impl<
        T: Mul<Output = T>
            + Add<Output = T>
            + Sub<Output = T>
            + Div<Output = T>
            + IsZero
            + Into<f64>
            + Copy,
    > Sub for Vector2<T>
{
    type Output = Vector2<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<
        T: Mul<Output = T>
            + Add<Output = T>
            + Sub<Output = T>
            + Div<Output = T>
            + IsZero
            + Into<f64>
            + SubAssign
            + Copy,
    > SubAssign for Vector2<T>
{
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<
        T: Mul<Output = T>
            + Add<Output = T>
            + Sub<Output = T>
            + Div<Output = T>
            + IsZero
            + Into<f64>
            + Copy,
    > Sub<T> for Vector2<T>
{
    type Output = Vector2<T>;

    fn sub(self, rhs: T) -> Self::Output {
        Self::Output {
            x: self.x - rhs,
            y: self.y - rhs,
        }
    }
}

impl<
        T: Mul<Output = T>
            + Add<Output = T>
            + Sub<Output = T>
            + Div<Output = T>
            + IsZero
            + Into<f64>
            + SubAssign
            + Copy,
    > SubAssign<T> for Vector2<T>
{
    fn sub_assign(&mut self, rhs: T) {
        self.x -= rhs;
        self.y -= rhs;
    }
}

impl<
        T: Mul<Output = T>
            + Add<Output = T>
            + Sub<Output = T>
            + Div<Output = T>
            + IsZero
            + Into<f64>
            + Copy,
    > Mul for Vector2<T>
{
    type Output = Vector2<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl<
        T: Mul<Output = T>
            + Add<Output = T>
            + Sub<Output = T>
            + Div<Output = T>
            + IsZero
            + Into<f64>
            + MulAssign
            + Copy,
    > MulAssign for Vector2<T>
{
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl<
        T: Mul<Output = T>
            + Add<Output = T>
            + Sub<Output = T>
            + Div<Output = T>
            + IsZero
            + Into<f64>
            + Copy,
    > Mul<T> for Vector2<T>
{
    type Output = Vector2<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<
        T: Mul<Output = T>
            + Add<Output = T>
            + Sub<Output = T>
            + Div<Output = T>
            + IsZero
            + Into<f64>
            + MulAssign
            + Copy,
    > MulAssign<T> for Vector2<T>
{
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl<
        T: Mul<Output = T>
            + Add<Output = T>
            + Sub<Output = T>
            + Div<Output = T>
            + IsZero
            + Into<f64>
            + Copy,
    > Div for Vector2<T>
{
    type Output = Vector2<T>;

    fn div(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl<
        T: Mul<Output = T>
            + Add<Output = T>
            + Sub<Output = T>
            + Div<Output = T>
            + IsZero
            + Into<f64>
            + DivAssign
            + Copy,
    > DivAssign for Vector2<T>
{
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl<
        T: Mul<Output = T>
            + Add<Output = T>
            + Sub<Output = T>
            + Div<Output = T>
            + IsZero
            + Into<f64>
            + Copy,
    > Div<T> for Vector2<T>
{
    type Output = Vector2<T>;

    fn div(self, rhs: T) -> Self::Output {
        Self::Output {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<
        T: Mul<Output = T>
            + Add<Output = T>
            + Sub<Output = T>
            + Div<Output = T>
            + IsZero
            + Into<f64>
            + DivAssign
            + Copy,
    > DivAssign<T> for Vector2<T>
{
    fn div_assign(&mut self, rhs: T) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl Vector2<f64> {
    pub fn from_angle(angle: f64) -> Self {
        Self {
            x: angle.cos(),
            y: angle.sin(),
        }
    }
}