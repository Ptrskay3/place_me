use std::collections::HashMap;

use crate::rangestack::{Range, RangeStack};
use crate::ray::Ray;
use crate::shape::{Element, Hittable, Intersection};

#[derive(Debug, Clone)]
pub struct Field {
    pub elements: Vec<Element>,
    pub res: u32,
    pub width: u32,
    pub height: u32,
}

impl Field {
    pub fn new(elements: Vec<Element>, res: u32, width: u32, height: u32) -> Self {
        Self {
            elements,
            res,
            width,
            height,
        }
    }
    pub fn trace(&self, ray: &Ray) -> Option<Intersection> {
        self.elements
            .iter()
            .filter_map(|s| s.hit(ray).map(|d| Intersection::new(d, s)))
            .min_by(|i1, i2| i1.distance.partial_cmp(&i2.distance).unwrap())
    }

    pub fn update_stack(&mut self, id: String, range: Range) {
        // unwrapping here is ok, because the caller ensures that the id is valid
        let circle: &mut Element = self
            .elements
            .iter_mut()
            .find(|c| match c {
                Element::Circle(c) => c.id == id,
                _ => false,
            })
            .unwrap();
        if let Element::Circle(c) = circle {
            c.range_stack.wrapping_add(&range)
        }
    }

    pub fn get_coverage(&self) -> HashMap<String, RangeStack> {
        let mut coverages = HashMap::new();
        for circle in &self.elements {
            match circle {
                Element::Circle(c) => {
                    let rs = c.range_stack.ranges.iter().collect::<RangeStack>();
                    coverages.insert(c.id.clone(), rs);
                }
                _ => {}
            }
        }
        coverages
    }
}

pub fn cast_ray<'f>(field: &'f Field, ray: &Ray) -> Option<&'f Element> {
    if let Some(intersection) = field.trace(ray) {
        if let Element::Segment(s) = intersection.element {
            println!("the closest is {:?}", s);
        }
        return Some(intersection.element);
    }
    None
}
