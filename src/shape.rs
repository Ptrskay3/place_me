use std::cmp::Ordering;

use crate::point::Point;
use crate::rangestack::{Range, RangeStack};
use crate::ray::Ray;
use crate::vector::Vector;

pub const TWO_PI: f64 = 2.0 * std::f64::consts::PI;
pub trait Hittable {
    fn hit(&self, ray: &Ray) -> Option<f64>;
}

#[derive(Debug, Clone)]
pub struct Circle {
    pub center: Point,
    pub radius: f64,
    pub range_stack: RangeStack,
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub p1: Point,
    pub p2: Point,
}

impl Segment {
    pub fn new(p1: Point, p2: Point) -> Self {
        Self { p1, p2 }
    }

    pub fn get_coeffs(&self) -> (f64, f64) {
        let x = self.p2.x - self.p1.x;
        let y = self.p2.y - self.p1.y;
        let slope = y / x;
        let intercept = self.p1.y - slope * self.p1.x;

        (slope, intercept)
    }
}

impl Circle {
    pub fn new(center: Point, radius: f64, range_stack: RangeStack, id: String) -> Self {
        Self {
            center,
            radius,
            range_stack,
            id,
        }
    }

    pub fn get_range_for_ray_pair(&self, r1: &Ray, r2: &Ray) -> Range {
        let alpha1 = self.hit_angle(r1);
        let alpha2 = self.hit_angle(r2);
        Range::new(alpha1, alpha2)
    }

    // Unused currently
    pub fn arclength_spanned_by(&mut self, r1: &Ray, r2: &Ray) -> f64 {
        // https://math.stackexchange.com/questions/1595872/arclength-between-two-points-on-a-circle-not-knowing-theta
        let dist = r1.origin.distance_from(&r2.origin);
        let (theta1, theta2) = self.hit_interval(r1, r2);
        self.range_stack.wrapping_add(&Range::new(theta1, theta2));

        2.0 * self.radius * (dist / (2.0 * self.radius)).asin()
    }

    // Unused currently (old implementation)
    pub fn approx_hitbox_angle(&self, ray: &Ray, resolution: u32) -> Range {
        let center = &self.center;
        let radius = &self.radius;
        let hit_radius = self.hit(ray).unwrap();
        let hit_point = ray.at(hit_radius);
        let alpha_tick = (hit_point.y - center.y).atan2(hit_point.x - center.x);
        let alpha = if alpha_tick < 0.0 {
            alpha_tick + std::f64::consts::PI
        } else {
            alpha_tick
        };
        let hitbox_angle = 2.0 * std::f64::consts::PI * hit_radius / (resolution as f64 * radius);

        let lower = alpha - hitbox_angle / 2.0;
        let upper = alpha + hitbox_angle / 2.0;

        Range::new(lower, upper)
    }

    pub fn hit_angle(&self, ray: &Ray) -> f64 {
        let center = &self.center;
        let hit_radius = self.hit(ray).unwrap();
        let hit_point = ray.at(hit_radius);
        let alpha_tick = (hit_point.y - center.y).atan2(hit_point.x - center.x);
        alpha_tick.rem_euclid(TWO_PI)
    }

    // Unused currently
    pub fn hit_interval(&self, r1: &Ray, r2: &Ray) -> (f64, f64) {
        let angle1 = self.hit_angle(r1);
        let angle2 = self.hit_angle(r2);

        match angle1.partial_cmp(&angle2) {
            Some(Ordering::Less) => (angle1, angle2),
            Some(Ordering::Equal) => (angle1, angle2),
            Some(Ordering::Greater) => (angle2, angle1),
            None => (0.0, 0.0),
        }
    }
}

impl Hittable for Circle {
    fn hit(&self, ray: &Ray) -> Option<f64> {
        // vector between the ray origin and the center of circle
        let l: Vector = self.center - ray.origin;

        // length of the hypotenuse
        let hypo: f64 = l.dot(&ray.direction);

        let dist = l.dot(&l) - (hypo * hypo);

        let radius_squared = self.radius * self.radius;
        if dist > radius_squared {
            return None;
        }

        let thc = (radius_squared - dist).sqrt();
        let t0 = hypo - thc;
        let t1 = hypo + thc;

        if t0 < 0.0 && t1 < 0.0 {
            None
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

impl Hittable for Segment {
    fn hit(&self, ray: &Ray) -> Option<f64> {
        let (r1, r2) = ray.spanned_points();
        let determinant =
            ((r1.x - r2.x) * (self.p1.y - self.p2.y)) - ((r1.y - r2.y) * (self.p1.x - self.p2.x));
        if -1E-6 <= determinant && determinant <= 1E-6 {
            return None;
        }

        let ix = (r1.x * r2.y - r1.y * r2.x) * (self.p1.x - self.p2.x)
            - (r1.x - r2.x) * (self.p1.x * self.p2.y - self.p1.y * self.p2.x);
        let iy = (r1.x * r2.y - r1.y * r2.x) * (self.p1.y - self.p2.y)
            - (r1.y - r2.y) * (self.p1.x * self.p2.y - self.p1.y * self.p2.x);
        let intersection = Point {
            x: ix / determinant,
            y: iy / determinant,
        };
        if !intersection.is_aabb() {
            return None;
        }

        Some(intersection.distance_from(&ray.origin))
    }
}

#[derive(Debug, Clone)]
pub enum Element {
    Circle(Circle),
    Segment(Segment),
}

impl Hittable for Element {
    fn hit(&self, ray: &Ray) -> Option<f64> {
        match self {
            Element::Circle(ref c) => c.hit(ray),
            Element::Segment(ref s) => s.hit(ray),
        }
    }
}

#[derive(Debug)]
pub struct Intersection<'a> {
    pub distance: f64,
    pub element: &'a Element,
}

impl<'a> Intersection<'a> {
    pub fn new<'b>(distance: f64, element: &'b Element) -> Intersection<'b> {
        if !distance.is_finite() {
            panic!("Intersection must have a finite distance.");
        }
        Intersection { distance, element }
    }
}
