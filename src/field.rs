use crate::point::Point;
use crate::ray::Ray;
use crate::sensor::Sensor;
use crate::shape::{Circle, Hittable, Intersection};

pub struct Field {
    pub data: Vec<Vec<f64>>,
    pub circles: Vec<Circle>,
    pub sensor: Sensor,
    pub width: u32,
    pub height: u32,
    pub origin: Point,
}

impl Field {
    pub fn trace(&self, ray: &Ray) -> Option<Intersection> {
        self.circles
            .iter()
            .filter_map(|s| s.hit(ray).map(|d| Intersection::new(d, s)))
            .min_by(|i1, i2| i1.distance.partial_cmp(&i2.distance).unwrap())
    }
}

pub fn cast_ray(field: &Field, ray: &Ray) {
    let _intersection = field.trace(&ray);
}
