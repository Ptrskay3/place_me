use uuid::Uuid;

use crate::rangestack::Range;
use crate::ray::Ray;
use crate::shape::{Circle, Hittable, Intersection};

#[derive(Debug, Clone)]
pub struct Field {
    pub circles: Vec<Circle>,
    pub res: u32,
    pub width: u32,
    pub height: u32,
}

impl Field {
    pub fn new(circles: Vec<Circle>, res: u32, width: u32, height: u32) -> Self {
        Self {
            circles,
            res,
            width,
            height,
        }
    }
    pub fn trace(&self, ray: &Ray) -> Option<Intersection> {
        self.circles
            .iter()
            .filter_map(|s| s.hit(ray).map(|d| Intersection::new(d, s)))
            .min_by(|i1, i2| i1.distance.partial_cmp(&i2.distance).unwrap())
    }

    pub fn update_stack(&mut self, id: Uuid, range: Range) {
        // unwrapping here is ok, because the caller ensures that the id is valid
        let circle: &mut Circle = self.circles.iter_mut().find(|c| c.id == id).unwrap();
        circle.range_stack.wrapping_add(&range);
    }
}

pub fn cast_ray<'f>(field: &'f Field, ray: &Ray) -> Option<&'f Circle> {
    if let Some(intersection) = field.trace(ray) {
        return Some(intersection.element);
    }
    None
}
