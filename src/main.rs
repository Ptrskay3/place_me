//! This file is for testing purposes, ignore it entirely.

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
const WIDTH: u32 = 1080;
const HEIGHT: u32 = 1080;
const RESOLUTION: u32 = 2880;

fn main() {
    let report = Arc::new(Mutex::new(Report {
        max_coverage: 0.0,
        sensor_positions: Vec::new(),
        extra: Vec::new(),
    }));

    let (x_range, y_range) = Sensor::coordinates_along_circumference(WIDTH, HEIGHT, 10);
    let circles = vec![
        shape::Circle::new(point::Point::new(450., 712.), 70.0, RangeStack::new()),
        shape::Circle::new(point::Point::new(437., 1693.), 70.0, RangeStack::new()),
        shape::Circle::new(point::Point::new(461., 375.), 70.0, RangeStack::new()),
        shape::Circle::new(point::Point::new(539., 1473.), 70.0, RangeStack::new()),
        shape::Circle::new(point::Point::new(279., 425.), 70.0, RangeStack::new()),
        // shape::Circle::new(point::Point::new(140., 1613.), 70.0, RangeStack::new()),
        // shape::Circle::new(point::Point::new(2712., 449.), 400.0, RangeStack::new()),
        // shape::Circle::new(point::Point::new(2015., 445.), 70.0, RangeStack::new()),
    ];

    let full_arclength = FULL_CIRCLE * circles.len() as f64;

    x_range.par_iter().zip(y_range.clone()).for_each(|(&x, y)| {
        let sensor =
            Sensor::new_at(&point::Point::new(x as f64, y as f64)).with_resolution(RESOLUTION);
        let rays = sensor.rays.clone();
        let field = Field::new(circles.clone(), RESOLUTION, WIDTH, HEIGHT);
        let mut field_res = field.clone();
        rays.windows(2).for_each(|pair| {
            let i1 = cast_ray(&field, &pair[0]);
            let i2 = cast_ray(&field, &pair[1]);
            if let Some(elem1) = i1 {
                if let Some(elem2) = i2 {
                    if elem1.id == elem2.id {
                        let range = elem1.get_range_for_ray_pair(&pair[0], &pair[1]);
                        // println!("adding {:?}", range);
                        field_res.update_stack(elem1.id, range);
                    }
                }
            }
        });

        let restore = field_res.circles.clone();
        // let cov = 100.0
        //     * restore[0]
        //         .range_stack
        //         .ranges
        //         .iter()
        //         .collect::<RangeStack>()
        //         .length()
        //     / full_arclength;
        // if cov > 35.0 {
        //     println!("at {:?}, {:?}", x, y);
        //     println!(
        //         "current rangestack is {:?}",
        //         field_res.circles[0].range_stack.ranges
        //     );
        //     println!("percentage covered outside is {:?}", cov);
        // }

        // println!(
        //     "we're at ({:?}, {:?}), rs range is {:?}\n___________________",
        //     x,
        //     y,
        //     field_res.circles[0]
        //         .range_stack
        //         .ranges
        //         .iter()
        //         .collect::<RangeStack>()
        // );
        x_range.iter().zip(y_range.clone()).for_each(|(&x2, y2)| {
            // println!("at {:?} {:?}", x2, y2);
            // println!(
            //     "original is {:?}",
            //     field_res.circles[0]
            //         .range_stack
            //         .ranges
            //         .iter()
            //         .collect::<RangeStack>()
            // );
            let sensor2 = Sensor::new_at(&point::Point::new(x2 as f64, y2 as f64))
                .with_resolution(RESOLUTION);
            let rays2 = sensor2.rays.clone();

            rays2.windows(2).for_each(|pair| {
                let i1 = cast_ray(&field, &pair[0]);
                let i2 = cast_ray(&field, &pair[1]);
                if let Some(elem1) = i1 {
                    if let Some(elem2) = i2 {
                        if elem1.id == elem2.id {
                            let range = elem1.get_range_for_ray_pair(&pair[0], &pair[1]);
                            // println!("with other at ({:?}, {:?}), adding {:?}", x2, y2, range);
                            field_res.update_stack(elem1.id, range);
                        }
                    }
                }
                // println!(
                //     "{:?}",
                //     field_res.circles[0]
                //         .range_stack
                //         .ranges
                //         .iter()
                //         .collect::<RangeStack>()
                // );
            });

            // let outter = restore[0].range_stack.ranges.iter().collect::<RangeStack>();
            // let covered_2 = field_res.circles[0]
            //     .range_stack
            //     .ranges
            //     .iter()
            //     .collect::<RangeStack>();

            let covered_len = field_res
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
                .sum::<f64>()
                / full_arclength;

            // The number of seen objects.
            let seen = field_res
                .clone()
                .circles
                .iter()
                .take_while(|circle| !circle.range_stack.is_empty())
                .count();

            println!("we see {:?}", seen);
            println!("covered {:?}", covered_len);
            let cov = seen as f64 + covered_len * circles.len() as f64;

            // Set the results if the coverage is equal or higher than the previous one.
            let mut result = report.lock().unwrap();
            if cov > result.max_coverage {
                result.max_coverage = cov;
                result.sensor_positions = vec![
                    point::Point::new(x as f64, y as f64),
                    point::Point::new(x2 as f64, y2 as f64),
                ];
            // TODO: floating-point arithmetic, it's kind of dumb to check for equality
            } else if cov == result.max_coverage {
                result.extra.push(vec![
                    point::Point::new(x as f64, y as f64),
                    point::Point::new(x2 as f64, y2 as f64),
                ]);
            }
            field_res.circles = restore.clone();
        });
    });

    // Print report at the end, if RUST_LOG environment variable is set.
    let rep = report.lock().unwrap();
    rep.pprint(circles.len());
}
