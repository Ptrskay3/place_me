use crate::ray::Ray;
use crate::sensor::Sensor;
use crate::shape::{Circle, Hittable, Intersection};

#[derive(Debug, Clone)]
pub struct Field {
    pub circles: Vec<Circle>,
    pub sensor: Sensor,
    pub width: u32,
    pub height: u32,
}

impl Field {
    pub fn new(circles: Vec<Circle>, sensor: Sensor, width: u32, height: u32) -> Self {
        Self {
            circles,
            sensor,
            width,
            height,
        }
    }
    pub fn trace(&mut self, ray: &Ray) -> Option<Intersection> {
        self.circles
            .iter_mut()
            .filter_map(|s| s.hit(ray).map(move |d| Intersection::new(d, s)))
            .min_by(|i1, i2| i1.distance.partial_cmp(&i2.distance).unwrap())
    }
}

pub fn cast_ray(field: &mut Field, ray: &Ray) {
    let res = field.sensor.res;
    if let Some(intersection) = field.trace(ray) {
        // println!("got intersection {:?}", intersection);
        let element = intersection.element;
        let range = element.approx_hitbox_angle(ray, res);
        element.range_stack.add_unchecked(&range);
    };
}
