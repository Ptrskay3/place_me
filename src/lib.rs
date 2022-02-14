pub mod field;
pub mod point;
pub mod rangestack;
pub mod ray;
pub mod report;
pub mod sensor;
pub mod shape;
pub mod vector;

use std::sync::{Arc, Mutex};

use pyo3::prelude::*;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use field::{cast_ray, Field};
use point::Point;
use rangestack::RangeStack;
use report::Report;
use sensor::Sensor;
use shape::Circle;

#[pymodule]
fn place_me(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(optimize_v2, m)?)?;
    m.add_function(wrap_pyfunction!(optimize_v3, m)?)?;
    Ok(())
}

#[pyfunction]
fn optimize_v2<'py>(
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

#[pyfunction]
fn optimize_v3<'py>(
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
    let v = inner_calculate_v3(circles, x_range, y_range, resolution, width, height);
    Ok(v)
}

fn initialize_circles(x: &[f64], y: &[f64], r: &[f64]) -> Vec<Circle> {
    let mut circles = Vec::new();
    for i in 0..x.len() {
        circles.push(Circle::new(Point::new(x[i], y[i]), r[i], RangeStack::new()));
    }
    circles
}

fn inner_calculate_v2(
    circles: Vec<Circle>,
    x_range: Vec<u32>,
    y_range: Vec<u32>,
    resolution: u32,
    width: u32,
    height: u32,
) -> Vec<f64> {
    // The final result. Unfortunately this is behind an Arc (think of it as C++ shared_ptr) and a Mutex, because
    // we're running in parallel. This of course adds some overhead.
    let report = Arc::new(Mutex::new(Report {
        max_coverage: 0.0,
        sensor_positions: Vec::new(),
    }));

    let full_circle: f64 = 2.0 * std::f64::consts::PI;

    // `n` circles have `2 * PI * n` angles in total.
    let full_arclength = full_circle * circles.len() as f64;

    x_range.par_iter().zip(y_range.clone()).for_each(|(&x, y)| {
        // Place a sensor at the current coordinate pair.
        let sensor =
            Sensor::new_at(&point::Point::new(x as f64, y as f64)).with_resolution(resolution);
        let rays = sensor.rays.clone();
        // An immutable Field used for acquiring the subject circle's uuid.
        let field = Field::new(circles.clone(), resolution, width, height);
        // A mutable `Field` that's circle field is updated with every iteration.
        let mut field_res = field.clone();

        // Iterating over the rays of the sensor in pairs.
        // We check whether the pair intersects the same object, and we accumulate the coverage.
        // TODO: This can be easily optimized later with `std::iter::take_while`.
        // The idea is to check whether we hit something with the first ray, then iterating until
        // the same object is hit again. The coverage between last one that's hitting the same object
        // and the first should give us the result and save us from calculating every consecutive hit.
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

        // Copy the state we have in the current iteration for the first sensor. We'll restore this state
        // inside the seconds sensor's loop on every iteration.
        let restore = field_res.circles.clone();

        x_range.iter().zip(y_range.clone()).for_each(|(&x2, y2)| {
            let sensor2 = Sensor::new_at(&point::Point::new(x2 as f64, y2 as f64))
                .with_resolution(resolution);
            let rays2 = sensor2.rays.clone();

            // Same logic as above, just for the second sensor.
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

            // Get the full coverage out of the current state.
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

            let cov = seen as f64 + covered_len * circles.len() as f64;

            // Set the results if the coverage is equal or higher than the previous one.
            let report = report.clone();
            let mut result = report.lock().unwrap();
            if cov > result.max_coverage {
                result.max_coverage = cov;
                result.sensor_positions = vec![
                    point::Point::new(x as f64, y as f64),
                    point::Point::new(x2 as f64, y2 as f64),
                ];
                // TODO: floating-point arithmetic, it's kind of dumb to check for equality
            }
            field_res.circles = restore.clone();
        });
    });

    // Print report at the end, if RUST_LOG environment variable is set.
    let rep = report.lock().unwrap();
    if rust_log_is_set() {
        rep.pprint(circles.len());
    }
    // Return the final two positions as 1D array to Python.
    // This is safe, because we have exactly two points as result.
    Vec::from([
        rep.sensor_positions[0].x,
        rep.sensor_positions[0].y,
        rep.sensor_positions[1].x,
        rep.sensor_positions[1].y,
    ])
}

fn inner_calculate_v3(
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

            let restore2 = field_res.circles.clone();

            x_range.iter().zip(y_range.clone()).for_each(|(&x3, y3)| {
                let sensor3 = Sensor::new_at(&point::Point::new(x3 as f64, y3 as f64))
                    .with_resolution(resolution);
                let rays3 = sensor3.rays.clone();

                rays3.windows(2).for_each(|pair| {
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

                let seen = field_res
                    .clone()
                    .circles
                    .iter()
                    .take_while(|circle| !circle.range_stack.is_empty())
                    .count();

                let cov = seen as f64 + covered_len * circles.len() as f64;

                let report = report.clone();
                let mut result = report.lock().unwrap();
                if cov > result.max_coverage {
                    result.max_coverage = cov;
                    result.sensor_positions = vec![
                        point::Point::new(x as f64, y as f64),
                        point::Point::new(x2 as f64, y2 as f64),
                        point::Point::new(x3 as f64, y3 as f64),
                    ];
                }
                field_res.circles = restore2.clone();
            });
            field_res.circles = restore.clone();
        });
    });

    let rep = report.lock().unwrap();
    if rust_log_is_set() {
        rep.pprint(circles.len());
    }
    Vec::from([
        rep.sensor_positions[0].x,
        rep.sensor_positions[0].y,
        rep.sensor_positions[1].x,
        rep.sensor_positions[1].y,
        rep.sensor_positions[2].x,
        rep.sensor_positions[2].y,
    ])
}

fn rust_log_is_set() -> bool {
    match std::env::var("RUST_LOG") {
        Ok(s) => s == "1",
        _ => false,
    }
}
