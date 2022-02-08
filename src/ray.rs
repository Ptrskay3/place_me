use crate::field::Field;
use crate::point::Point;
use crate::vector::Vector;

#[derive(Clone, Debug)]
pub struct Ray {
    pub origin: Point,
    pub direction: Vector,
}

impl Ray {
    /// Return the ray at point `t`.
    pub fn at(&self, t: f64) -> Point {
        self.origin + t * self.direction
    }

    pub fn create_prime_ray(x: f64, y: f64, field: &Field) -> Self {
        Self {
            origin: field.origin,
            direction: Vector { x, y }.normalize(),
        }
    }
}
