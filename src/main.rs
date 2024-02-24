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
use shape::Element;

use rayon::prelude::*;

const FULL_CIRCLE: f64 = 2.0 * std::f64::consts::PI;
const WIDTH: u32 = 1080;
const HEIGHT: u32 = 1080;
const RESOLUTION: u32 = 2880;

fn main() {
    let report = Arc::new(Mutex::new(Report {
        max_coverage: 0.0,
        sensor_positions: Vec::new(),
        sensor_coverages: Vec::new(),
    }));

    let (x_range, y_range) = Sensor::coordinates_along_circumference(WIDTH, HEIGHT, 100);
    let circles = vec![
        Element::Circle(shape::Circle::new(
            point::Point::new(500., 500.),
            60.0,
            RangeStack::new(),
            "1".to_string(),
        )),
        // Element::Circle(shape::Circle::new(
        //     point::Point::new(1840., 543.),
        //     60.0,
        //     RangeStack::new(),
        //     "2".to_string(),
        // )),
        Element::Segment(shape::Segment::new(
            point::Point::new(750., 0.),
            point::Point::new(750., 1080.),
        )),
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
            if let Some(Element::Circle(elem1)) = i1 {
                if let Some(Element::Circle(elem2)) = i2 {
                    if elem1.id == elem2.id {
                        let range = elem1.get_range_for_ray_pair(&pair[0], &pair[1]);
                        field_res.update_stack(&elem1.id, range);
                    }
                }
            }
        });

        let restore = field_res.elements.clone();

        x_range.iter().zip(y_range.clone()).for_each(|(&x2, y2)| {
            let sensor2 = Sensor::new_at(&point::Point::new(x2 as f64, y2 as f64))
                .with_resolution(RESOLUTION);
            let rays2 = sensor2.rays.clone();

            rays2.windows(2).for_each(|pair| {
                let i1 = cast_ray(&field, &pair[0]);
                let i2 = cast_ray(&field, &pair[1]);
                if let Some(Element::Circle(elem1)) = i1 {
                    if let Some(Element::Circle(elem2)) = i2 {
                        if elem1.id == elem2.id {
                            let range = elem1.get_range_for_ray_pair(&pair[0], &pair[1]);
                            field_res.update_stack(&elem1.id, range);
                        }
                    }
                }
            });

            let covered_len = field_res
                .elements
                .iter()
                .map(|circle| {
                    if let Element::Circle(c) = circle {
                        c.range_stack
                            .ranges
                            .par_iter()
                            .collect::<RangeStack>()
                            .length()
                    } else {
                        0.0
                    }
                })
                .sum::<f64>()
                / full_arclength;

            // The number of seen objects.
            let seen = field_res
                .clone()
                .elements
                .iter()
                .take_while(|circle| {
                    if let Element::Circle(c) = circle {
                        !c.range_stack.is_empty()
                    } else {
                        false
                    }
                })
                .count();
            let cov = seen as f64 + covered_len * circles.len() as f64;

            let mut result = report.lock().unwrap();
            if cov > result.max_coverage {
                result.max_coverage = cov;
                result.sensor_positions = vec![
                    point::Point::new(x as f64, y as f64),
                    point::Point::new(x2 as f64, y2 as f64),
                ];
            }
            field_res.elements = restore.clone();
        });
    });

    let rep = report.lock().unwrap();
    rep.pprint(circles.len());
}
