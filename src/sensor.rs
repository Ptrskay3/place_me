use crate::{point::Point, ray::Ray, vector::Vector};

#[derive(Clone, Debug)]
pub struct Sensor {
    pub origin: Point,
    pub rays: Vec<Ray>,
    pub res: i32,
}

impl Sensor {
    pub fn new_at(point: &Point) -> Self {
        Self {
            origin: *point,
            rays: Vec::new(),
            res: 0,
        }
    }

    pub fn with_resolution(&self, res: i32) -> Self {
        let steps = 2.0 * std::f64::consts::PI / res as f64;
        let rays = (0..res)
            .map(|i| {
                let rad = i as f64 * steps;
                Ray {
                    origin: self.origin,
                    direction: Vector {
                        x: rad.cos(),
                        y: rad.sin(),
                    },
                }
            })
            .collect::<Vec<_>>();

        Self {
            origin: self.origin,
            rays,
            res,
        }
    }

    pub fn coordinateg_along_circumference(&mut self, x: i32, pixel_step: usize) -> Vec<i32> {
        let bottom = (0..x).step_by(pixel_step);
        let length = bottom.len();
        let right = vec![x; length];
        bottom
            .clone()
            .chain(right.clone())
            .chain(bottom)
            .chain(right)
            .collect::<Vec<_>>()
    }
}
