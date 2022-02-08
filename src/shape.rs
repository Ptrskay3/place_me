use crate::point::Point;
use crate::ray::Ray;
use crate::vector::Vector;

pub trait Hittable {
    fn hit(&self, ray: &Ray) -> Option<f64>;
}

pub struct Circle {
    pub center: Point,
    pub radius: f64,
}

impl Circle {
    pub fn arclength_spanned_by(&self, _r1: &Ray, _r2: &Ray) -> f64 {
        // https://math.stackexchange.com/questions/1595872/arclength-between-two-points-on-a-circle-not-knowing-theta
        let Self { radius, center } = self;
        let dist: f64 = todo!();
        2.0 * radius * (dist / (2.0 * radius)).asin()
    }
}

impl Hittable for Circle {
    fn hit(&self, ray: &Ray) -> Option<f64> {
        // line between the ray origin and the center of circle
        let l: Vector = self.center - ray.origin;

        // length of the hypotenuse
        let hypo: f64 = l.dot(&ray.direction);

        // distance from the circle (squared)

        let dist = l.dot(&l) - (hypo * hypo);

        let radius_squared = self.radius * self.radius;
        if dist > radius_squared {
            return None;
        }

        let thc = (radius_squared - dist).sqrt();
        let t0 = hypo - thc;
        let t1 = hypo + thc;

        if t0 < 0.0 && t1 < 0.0 {
            return None;
        } else if t0 < 0.0 {
            Some(t1)
        } else if t1 < 0.0 {
            Some(t0)
        } else {
            // in case there's two solutions, return the closer one
            let distance = if t0 < t1 { t0 } else { t1 };
            Some(distance)
        }
    }
}

pub struct Intersection<'a> {
    pub distance: f64,
    pub element: &'a Circle,
}

impl<'a> Intersection<'a> {
    pub fn new<'b>(distance: f64, element: &'b Circle) -> Intersection<'b> {
        if !distance.is_finite() {
            panic!("Intersection must have a finite distance.");
        }
        Intersection { distance, element }
    }
}
