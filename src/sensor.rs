use std::collections::HashMap;

use uuid::Uuid;

use crate::{point::Point, rangestack::RangeStack, ray::Ray, vector::Vector};

#[derive(Clone, Debug)]
pub struct Sensor {
    pub origin: Point,
    pub rays: Vec<Ray>,
    pub res: u32,
    pub coverages: HashMap<String, RangeStack>,
}

impl Sensor {
    pub fn new_at(point: &Point) -> Self {
        Self {
            origin: *point,
            rays: Vec::new(),
            res: 0,
            coverages: HashMap::new(),
        }
    }

    pub fn with_resolution(&self, res: u32) -> Self {
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
            coverages: HashMap::new(),
        }
    }

    pub fn coordinates_along_circumference(
        width: u32,
        height: u32,
        pixel_step: usize,
    ) -> (Vec<u32>, Vec<u32>) {
        let bottom_x = (0..width).step_by(pixel_step);
        let x_dim = bottom_x.len();
        let right_y = (0..height).step_by(pixel_step);
        let y_dim = right_y.len();
        let right_x = vec![width; y_dim];
        let left_x = vec![0; y_dim];
        let bottom_y = vec![0; x_dim];
        let top_y = vec![height; x_dim];
        (
            bottom_x
                .clone()
                .chain(right_x)
                .chain(bottom_x.rev())
                .chain(left_x)
                .collect::<Vec<_>>(),
            bottom_y
                .into_iter()
                .chain(right_y.clone())
                .chain(top_y)
                .chain(right_y.rev())
                .collect::<Vec<_>>(),
        )
    }

    pub fn move_to(&mut self, x: u32, y: u32) {
        self.origin = Point::new(f64::from(x), f64::from(y));
    }
}
