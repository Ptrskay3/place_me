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
}
