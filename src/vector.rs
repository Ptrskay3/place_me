use crate::point::Point;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
}

impl Vector {
    pub fn zero() -> Self {
        Self { x: 0., y: 0. }
    }

    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn norm(&self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }

    pub fn normalize(&self) -> Self {
        let factor = 1.0 / self.length();

        Self {
            x: self.x * factor,
            y: self.y * factor,
        }
    }

    pub fn dot(&self, other: &Vector) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn as_point(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }
}

impl Add for Vector {
    type Output = Vector;

    fn add(self, other: Vector) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Vector {
    type Output = Vector;

    fn sub(self, other: Vector) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<Vector> for f64 {
    type Output = Vector;

    fn mul(self, other: Vector) -> Vector {
        other * self
    }
}

impl Mul for Vector {
    type Output = Vector;

    fn mul(self, other: Vector) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl Mul<f64> for Vector {
    type Output = Vector;

    fn mul(self, other: f64) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Div<f64> for Vector {
    type Output = Vector;

    fn div(self, other: f64) -> Self {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}
