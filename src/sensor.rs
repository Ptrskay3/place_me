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

    pub fn coordinates_along_circumference(
        &self,
        x: i32,
        y: i32,
        pixel_step: usize,
    ) -> (Vec<i32>, Vec<i32>) {
        let bottom_x = (0..x).step_by(pixel_step);
        let x_dim = bottom_x.len();
        let right_y = (0..y).step_by(pixel_step);
        let y_dim = right_y.len();
        let right_x = vec![x; y_dim];
        let left_x = vec![0; y_dim];
        let bottom_y = vec![0; x_dim];
        let top_y = vec![y; x_dim];
        (
            bottom_x
                .clone()
                .chain(right_x.clone())
                .chain(bottom_x.rev())
                .chain(left_x)
                .collect::<Vec<_>>(),
            bottom_y
                .clone()
                .into_iter()
                .chain(right_y.clone())
                .chain(top_y)
                .chain(right_y.rev())
                .collect::<Vec<_>>(),
        )
    }
}
