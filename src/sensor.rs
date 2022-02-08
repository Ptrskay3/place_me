use crate::{point::Point, ray::Ray, vector::Vector};

#[derive(Clone, Debug)]
pub struct Sensor {
    pub origin: Point,
    pub rays: Vec<Ray>,
}

impl Sensor {
    pub fn new_at(point: &Point) -> Self {
        Self {
            origin: *point,
            rays: Vec::new(),
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
        }
    }
}
