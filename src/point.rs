use crate::vector::Vector;
use std::ops::{Add, Div, Sub};

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn zero() -> Self {
        Self { x: 0., y: 0. }
    }

    pub fn distance_from(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;

        (dx * dx + dy * dy).sqrt()
    }
}

impl Sub<Point> for Point {
    type Output = Vector;
    fn sub(self, other: Point) -> Vector {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Div<f64> for Point {
    type Output = Point;
    fn div(self, other: f64) -> Point {
        Point {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl Add<Vector> for Point {
    type Output = Point;
    fn add(self, other: Vector) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Add<Point> for Point {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
