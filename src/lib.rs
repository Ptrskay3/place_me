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
use point::Point;
use pyo3::prelude::*;
use rangestack::RangeStack;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use report::Report;
use sensor::Sensor;
use shape::Circle;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn place_me(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(run, m)?)?;
    m.add_function(wrap_pyfunction!(run_v2, m)?)?;
    Ok(())
}

// use pyo3::{exceptions::PyRuntimeError, pymodule, types::PyModule, PyErr, PyResult, Python};

#[pyfunction]
fn run<'py>(
    _py: Python<'_>,
    xs: Vec<f64>,
    ys: Vec<f64>,
    radii: Vec<f64>,
    width: u32,
    height: u32,
    resolution: u32,
    pixel_step: usize,
) -> PyResult<Vec<f64>> {
    let (x_range, y_range) = Sensor::coordinates_along_circumference(width, height, pixel_step);

    let circles = initialize_circles(&xs, &ys, &radii);
    let v = inner_calculate(circles, x_range, y_range, resolution, width, height);
    Ok(v)
}

#[pyfunction]
fn run_v2<'py>(
    _py: Python<'_>,
    xs: Vec<f64>,
    ys: Vec<f64>,
    radii: Vec<f64>,
    width: u32,
    height: u32,
    resolution: u32,
    pixel_step: usize,
) -> PyResult<Vec<f64>> {
    let (x_range, y_range) = Sensor::coordinates_along_circumference(width, height, pixel_step);
    let circles = initialize_circles(&xs, &ys, &radii);
    let v = inner_calculate_v2(circles, x_range, y_range, resolution, width, height);
    Ok(v)
}

fn initialize_circles(x: &[f64], y: &[f64], r: &[f64]) -> Vec<Circle> {
    let mut circles = Vec::new();
    for i in 0..x.len() {
        circles.push(Circle::new(Point::new(x[i], y[i]), r[i], RangeStack::new()));
    }
    circles
}

fn inner_calculate(
    circles: Vec<Circle>,
    x_range: Vec<u32>,
    y_range: Vec<u32>,
    resolution: u32,
    width: u32,
    height: u32,
) -> Vec<f64> {
    let report = Arc::new(Mutex::new(Report {
        max_coverage: 0.0,
        sensor_positions: Vec::new(),
        extra: Vec::new(),
    }));

    let full_circle: f64 = 2.0 * std::f64::consts::PI;

    let full_arclength = full_circle * circles.len() as f64;

    x_range.par_iter().zip(y_range.clone()).for_each(|(&x, y)| {
        let sensor =
            Sensor::new_at(&point::Point::new(x as f64, y as f64)).with_resolution(resolution);
        let rays = sensor.rays.clone();
        let mut field = Field::new(circles.clone(), resolution, width, height);

        for ray in rays {
            cast_ray(&mut field, &ray);
        }

        let restore = field.circles.clone();

        x_range.iter().zip(y_range.clone()).for_each(|(&x2, y2)| {
            let sensor2 = Sensor::new_at(&point::Point::new(x2 as f64, y2 as f64))
                .with_resolution(resolution);
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
            } else if covered == result.max_coverage {
                result.extra.push(vec![
                    point::Point::new(x as f64, y as f64),
                    point::Point::new(x2 as f64, y2 as f64),
                ]);
            }
            //     println!(
            //         "percentage covered {:?} at ({:?}, {:?}), other at ({:?}, {:?})",
            //         100.0 * covered / full_arclength,
            //         x2,
            //         y2,
            //         x,
            // y,
            //     );
            field.circles = restore.clone();
        });
    });
    let rep = report.lock().unwrap();
    if rust_print_is_set() {
        rep.pprint(full_arclength);
    }
    // This is safe, because we have two points as result.
    Vec::from([
        rep.sensor_positions[0].x,
        rep.sensor_positions[0].y,
        rep.sensor_positions[1].x,
        rep.sensor_positions[1].y,
    ])
}

fn rust_print_is_set() -> bool {
    match std::env::var("RUST_PRINT") {
        Ok(s) => s == "1",
        _ => false,
    }
}

fn inner_calculate_v2(
    circles: Vec<Circle>,
    x_range: Vec<u32>,
    y_range: Vec<u32>,
    resolution: u32,
    width: u32,
    height: u32,
) -> Vec<f64> {
    let report = Arc::new(Mutex::new(Report {
        max_coverage: 0.0,
        sensor_positions: Vec::new(),
        extra: Vec::new(),
    }));

    let full_circle: f64 = 2.0 * std::f64::consts::PI;

    let full_arclength = full_circle * circles.len() as f64;

    x_range.par_iter().zip(y_range.clone()).for_each(|(&x, y)| {
        let sensor =
            Sensor::new_at(&point::Point::new(x as f64, y as f64)).with_resolution(resolution);
        let rays = sensor.rays.clone();
        let field = Field::new(circles.clone(), resolution, width, height);
        let mut field_res = field.clone();

        rays.windows(2).for_each(|pair| {
            let i1 = cast_ray(&field, &pair[0]);
            let i2 = cast_ray(&field, &pair[1]);
            if let Some(elem1) = i1 {
                if let Some(elem2) = i2 {
                    if elem1.id == elem2.id {
                        let range = elem1.get_range_for_ray_pair(&pair[0], &pair[1]);
                        field_res.update_stack(elem1.id, range);
                    }
                }
            }
        });

        let restore = field_res.circles.clone();

        x_range.iter().zip(y_range.clone()).for_each(|(&x2, y2)| {
            let sensor2 = Sensor::new_at(&point::Point::new(x2 as f64, y2 as f64))
                .with_resolution(resolution);
            let rays2 = sensor2.rays.clone();

            rays2.windows(2).for_each(|pair| {
                let i1 = cast_ray(&field, &pair[0]);
                let i2 = cast_ray(&field, &pair[1]);
                if let Some(elem1) = i1 {
                    if let Some(elem2) = i2 {
                        if elem1.id == elem2.id {
                            let range = elem1.get_range_for_ray_pair(&pair[0], &pair[1]);
                            field_res.update_stack(elem1.id, range);
                        }
                    }
                }
            });

            let covered: f64 = field_res
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
            } else if covered == result.max_coverage {
                result.extra.push(vec![
                    point::Point::new(x as f64, y as f64),
                    point::Point::new(x2 as f64, y2 as f64),
                ]);
            }
            // println!(
            //     "percentage covered {:?} at ({:?}, {:?}), other at ({:?}, {:?})",
            //     100.0 * covered / full_arclength,
            //     x2,
            //     y2,
            //     x,
            //     y,
            // );
            field_res.circles = restore.clone();
        });
    });

    let rep = report.lock().unwrap();
    if rust_print_is_set() {
        rep.pprint(full_arclength);
    }
    // This is safe, because we have two points as result.
    Vec::from([
        rep.sensor_positions[0].x,
        rep.sensor_positions[0].y,
        rep.sensor_positions[1].x,
        rep.sensor_positions[1].y,
    ])
}
