use crate::point::Point;
use crate::vector::Vector;

#[derive(Clone, Debug)]
pub struct Ray {
    pub origin: Point,
    pub direction: Vector,
}

impl Ray {
    /// Return the ray at point `t`.
    #[inline]
    pub fn at(&self, t: f64) -> Point {
        self.origin + t * self.direction
    }

    #[inline]
    pub fn get_coeffs(&self) -> (f64, f64) {
        let x = self.direction.x - self.origin.x;
        let y = self.direction.y - self.origin.y;
        let slope = y / x;
        let intercept = self.origin.y - slope * self.origin.x;

        (slope, intercept)
    }

    pub fn spanned_points(&self) -> (Point, Point) {
        (self.origin, self.at(4000.0))
    }
}
