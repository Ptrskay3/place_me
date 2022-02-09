pub mod field;
pub mod point;
pub mod rangestack;
pub mod ray;
pub mod report;
pub mod sensor;
pub mod shape;
pub mod vector;
use std::sync::{Arc, Mutex};

use field::{cast_ray, Field};
use rangestack::RangeStack;
use report::Report;
use sensor::Sensor;

use rayon::prelude::*;

const FULL_CIRCLE: f64 = 2.0 * std::f64::consts::PI;
const WIDTH: u32 = 3840;
const HEIGHT: u32 = 1080;
const RESOLUTION: u32 = 2880;

fn main() {
    let report = Arc::new(Mutex::new(Report {
        max_coverage: 0.0,
        sensor_positions: Vec::new(),
    }));

    let (x_range, y_range) = Sensor::coordinates_along_circumference(WIDTH, HEIGHT, 10);
    let circles = vec![
        shape::Circle::new(point::Point::new(1920.0, 540.0), 250.0, RangeStack::new()),
        shape::Circle::new(point::Point::new(2570.0, 540.0), 250.0, RangeStack::new()),
    ];

    let full_arclength = FULL_CIRCLE * circles.len() as f64;

    x_range.par_iter().zip(y_range.clone()).for_each(|(&x, y)| {
        let sensor =
            Sensor::new_at(&point::Point::new(x as f64, y as f64)).with_resolution(RESOLUTION);
        let rays = sensor.rays.clone();
        let mut field = Field::new(circles.clone(), RESOLUTION, WIDTH, HEIGHT);

        for ray in rays {
            cast_ray(&mut field, &ray);
        }

        let restore = field.circles.clone();

        x_range.iter().zip(y_range.clone()).for_each(|(&x2, y2)| {
            let sensor2 = Sensor::new_at(&point::Point::new(x2 as f64, y2 as f64))
                .with_resolution(RESOLUTION);
            let rays2 = sensor2.rays.clone();

            for ray in rays2 {
                cast_ray(&mut field, &ray);
            }

            let covered: f64 = field
                .circles
                .iter()
                .map(|circle| {
                    circle
                        .range_stack
                        .ranges
                        .par_iter()
                        .collect::<RangeStack>()
                        .length()
                })
                .sum();
            let mut result = report.lock().unwrap();
            if covered > result.max_coverage {
                result.max_coverage = covered;
                result.sensor_positions = vec![
                    point::Point::new(x as f64, y as f64),
                    point::Point::new(x2 as f64, y2 as f64),
                ];
            }
            // println!(
            //     "percentage covered {:?} at ({:?}, {:?}), original at ({:?}, {:?})",
            //     100.0 * covered / full_arclength,
            //     x2,
            //     y2,
            //     x,
            //     y,
            // );
            field.circles = restore.clone();
        });
    });

    report.lock().unwrap().pprint(full_arclength);
}
