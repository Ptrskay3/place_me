pub mod field;
pub mod point;
pub mod rangestack;
pub mod ray;
pub mod report;
pub mod sensor;
pub mod shape;
pub mod vector;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use pyo3::prelude::*;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use field::{cast_ray, Field};
use point::Point;
use rangestack::RangeStack;
use report::Report;
use sensor::Sensor;
use shape::{Circle, Element, Segment};

#[pymodule]
fn place_me(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(optimize_v1, m)?)?;
    m.add_function(wrap_pyfunction!(optimize_v2, m)?)?;
    m.add_function(wrap_pyfunction!(optimize_v3, m)?)?;
    Ok(())
}

#[pyfunction]
fn optimize_v1<'py>(
    _py: Python<'_>,
    ys: Vec<f64>,
    xs: Vec<f64>,
    radii: Vec<f64>,
    ids: Vec<String>,
    obstacles: Vec<f64>,
    width: u32,
    height: u32,
    resolution: u32,
    pixel_step: usize,
) -> PyResult<(Vec<f64>, Vec<HashMap<String, Vec<f64>>>)> {
    let mut covs = Vec::new();
    let (x_range, y_range) = Sensor::coordinates_along_circumference(width, height, pixel_step);
    let circles = initialize_elements(&xs, &ys, &radii, &ids, &obstacles);
    let (v, h) = inner_calculate_v1(circles, x_range, y_range, resolution, width, height);
    for i in h {
        let mut hmap = HashMap::new();
        for (k, v) in i {
            let mut inner = Vec::new();
            v.ranges.iter().for_each(|r| {
                inner.push(r.start);
                inner.push(r.end);
            });
            hmap.insert(k, inner);
        }
        covs.push(hmap);
    }
    Ok((v, covs))
}

#[pyfunction]
fn optimize_v2<'py>(
    _py: Python<'_>,
    ys: Vec<f64>,
    xs: Vec<f64>,
    radii: Vec<f64>,
    ids: Vec<String>,
    obstacles: Vec<f64>,
    width: u32,
    height: u32,
    resolution: u32,
    pixel_step: usize,
) -> PyResult<(Vec<f64>, Vec<HashMap<String, Vec<f64>>>)> {
    let mut covs = Vec::new();
    let (x_range, y_range) = Sensor::coordinates_along_circumference(width, height, pixel_step);
    let circles = initialize_elements(&xs, &ys, &radii, &ids, &obstacles);
    let (v, h) = inner_calculate_v2(circles, x_range, y_range, resolution, width, height);
    for i in h {
        let mut hmap = HashMap::new();
        for (k, v) in i {
            let mut inner = Vec::new();
            v.ranges.iter().for_each(|r| {
                inner.push(r.start);
                inner.push(r.end);
            });
            hmap.insert(k, inner);
        }
        covs.push(hmap);
    }
    Ok((v, covs))
}

#[pyfunction]
fn optimize_v3<'py>(
    _py: Python<'_>,
    ys: Vec<f64>,
    xs: Vec<f64>,
    radii: Vec<f64>,
    ids: Vec<String>,
    obstacles: Vec<f64>,
    width: u32,
    height: u32,
    resolution: u32,
    pixel_step: usize,
) -> PyResult<(Vec<f64>, Vec<HashMap<String, Vec<f64>>>)> {
    let mut covs = Vec::new();

    let (x_range, y_range) = Sensor::coordinates_along_circumference(width, height, pixel_step);
    let circles = initialize_elements(&xs, &ys, &radii, &ids, &obstacles);
    let (v, h) = inner_calculate_v3(circles, x_range, y_range, resolution, width, height);
    for i in h {
        let mut hmap = HashMap::new();
        for (k, v) in i {
            let mut inner = Vec::new();
            v.ranges.iter().for_each(|r| {
                inner.push(r.start);
                inner.push(r.end);
            });
            hmap.insert(k, inner);
        }
        covs.push(hmap);
    }
    Ok((v, covs))
}

fn initialize_elements(
    x: &[f64],
    y: &[f64],
    r: &[f64],
    ids: &[String],
    obstacles: &[f64],
) -> Vec<Element> {
    let mut elements = Vec::new();
    for i in 0..x.len() {
        elements.push(Element::Circle(Circle::new(
            Point::new(x[i], y[i]),
            r[i],
            RangeStack::new(),
            ids[i].clone(),
        )));
    }
    for i in (0..obstacles.len()).step_by(4) {
        elements.push(Element::Segment(Segment::new(
            Point::new(obstacles[i], obstacles[i + 1]),
            Point::new(obstacles[i + 2], obstacles[i + 3]),
        )));
    }
    elements
}

fn inner_calculate_v1(
    circles: Vec<Element>,
    x_range: Vec<u32>,
    y_range: Vec<u32>,
    resolution: u32,
    width: u32,
    height: u32,
) -> (Vec<f64>, Vec<HashMap<String, RangeStack>>) {
    // The final result. Unfortunately this is behind an Arc (think of it as C++ shared_ptr) and a Mutex, because
    // we're running in parallel. This of course adds some overhead.
    let report = Arc::new(Mutex::new(Report {
        max_coverage: 0.0,
        sensor_positions: Vec::new(),
        sensor_coverages: Vec::new(),
    }));

    let full_circle: f64 = 2.0 * std::f64::consts::PI;

    // `n` circles have `2 * PI * n` angles in total.
    let full_arclength = full_circle * circles.len() as f64;

    x_range.par_iter().zip(y_range.clone()).for_each(|(&x, y)| {
        // Place a sensor at the current coordinate pair.
        let mut sensor =
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
            if let Some(Element::Circle(elem1)) = i1 {
                if let Some(Element::Circle(elem2)) = i2 {
                    if elem1.id == elem2.id {
                        let range = elem1.get_range_for_ray_pair(&pair[0], &pair[1]);
                        field_res.update_stack(elem1.id.clone(), range);
                    }
                }
            }
        });

        sensor.coverages = field_res.get_coverage();

        // Get the full coverage out of the current state.
        let covered_len = field_res
            .elements
            .iter()
            .map(|element| {
                if let Element::Circle(circle) = element {
                    return circle
                        .range_stack
                        .ranges
                        .par_iter()
                        .collect::<RangeStack>()
                        .length();
                }
                return 0.0;
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

        // Set the results if the coverage is equal or higher than the previous one.
        let report = report.clone();
        let mut result = report.lock().unwrap();
        if cov > result.max_coverage {
            result.sensor_coverages = vec![sensor.coverages.clone()];
            result.max_coverage = cov;
            result.sensor_positions = vec![point::Point::new(x as f64, y as f64)];
            // TODO: floating-point arithmetic, it's kind of dumb to check for equality
        }
    });

    // Print report at the end, if RUST_LOG environment variable is set.
    let rep = report.lock().unwrap();
    if rust_log_is_set() {
        rep.pprint(circles.len());
        // println!("covs {:#?}", rep.sensor_coverages);
    }
    // Return the final two positions as 1D array to Python.
    // This is safe, because we have exactly two points as result.
    (
        Vec::from([rep.sensor_positions[0].x, rep.sensor_positions[0].y]),
        rep.sensor_coverages.clone(),
    )
}

fn inner_calculate_v2(
    circles: Vec<Element>,
    x_range: Vec<u32>,
    y_range: Vec<u32>,
    resolution: u32,
    width: u32,
    height: u32,
) -> (Vec<f64>, Vec<HashMap<String, RangeStack>>) {
    // The final result. Unfortunately this is behind an Arc (think of it as C++ shared_ptr) and a Mutex, because
    // we're running in parallel. This of course adds some overhead.
    let report = Arc::new(Mutex::new(Report {
        max_coverage: 0.0,
        sensor_positions: Vec::new(),
        sensor_coverages: Vec::new(),
    }));

    let full_circle: f64 = 2.0 * std::f64::consts::PI;

    // `n` circles have `2 * PI * n` angles in total.
    let full_arclength = full_circle * circles.len() as f64;

    x_range.par_iter().zip(y_range.clone()).for_each(|(&x, y)| {
        // Place a sensor at the current coordinate pair.
        let mut sensor =
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
            if let Some(Element::Circle(elem1)) = i1 {
                if let Some(Element::Circle(elem2)) = i2 {
                    if elem1.id == elem2.id {
                        let range = elem1.get_range_for_ray_pair(&pair[0], &pair[1]);
                        field_res.update_stack(elem1.id.clone(), range);
                    }
                }
            }
        });

        sensor.coverages = field_res.get_coverage();

        // Copy the state we have in the current iteration for the first sensor. We'll restore this state
        // inside the seconds sensor's loop on every iteration.
        let restore = field_res.elements.clone();

        x_range.iter().zip(y_range.clone()).for_each(|(&x2, y2)| {
            let mut sensor2 = Sensor::new_at(&point::Point::new(x2 as f64, y2 as f64))
                .with_resolution(resolution);
            let rays2 = sensor2.rays.clone();
            let mut _hmap = HashMap::new();
            let mut hmap = HashMap::new();

            // Same logic as above, just for the second sensor.
            rays2.windows(2).for_each(|pair| {
                let i1 = cast_ray(&field, &pair[0]);
                let i2 = cast_ray(&field, &pair[1]);
                if let Some(Element::Circle(elem1)) = i1 {
                    if let Some(Element::Circle(elem2)) = i2 {
                        if elem1.id == elem2.id {
                            let range = elem1.get_range_for_ray_pair(&pair[0], &pair[1]);
                            field_res.update_stack(elem1.id.clone(), range);
                            _hmap.entry(elem1.id.clone()).or_insert(vec![]).push(range);
                        }
                    }
                }
            });

            for (k, v) in _hmap.iter() {
                let mut rs = RangeStack::new();
                for range in v {
                    rs.wrapping_add(range);
                }

                hmap.insert(k.clone(), rs.ranges.iter().collect::<RangeStack>());
            }
            sensor2.coverages = hmap;

            // Get the full coverage out of the current state.
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

            // Set the results if the coverage is equal or higher than the previous one.
            let report = report.clone();
            let mut result = report.lock().unwrap();
            if cov > result.max_coverage {
                result.sensor_coverages = vec![sensor.coverages.clone(), sensor2.coverages.clone()];
                result.max_coverage = cov;
                result.sensor_positions = vec![
                    point::Point::new(x as f64, y as f64),
                    point::Point::new(x2 as f64, y2 as f64),
                ];
            }
            field_res.elements = restore.clone();
        });
    });

    // Print report at the end, if RUST_LOG environment variable is set.
    let rep = report.lock().unwrap();
    if rust_log_is_set() {
        rep.pprint(circles.len());
    }
    // Return the final two positions as 1D array to Python.
    // This is safe, because we have exactly two points as result.
    (
        Vec::from([
            rep.sensor_positions[0].x,
            rep.sensor_positions[0].y,
            rep.sensor_positions[1].x,
            rep.sensor_positions[1].y,
        ]),
        rep.sensor_coverages.clone(),
    )
}

fn inner_calculate_v3(
    circles: Vec<Element>,
    x_range: Vec<u32>,
    y_range: Vec<u32>,
    resolution: u32,
    width: u32,
    height: u32,
) -> (Vec<f64>, Vec<HashMap<String, RangeStack>>) {
    let report = Arc::new(Mutex::new(Report {
        max_coverage: 0.0,
        sensor_positions: Vec::new(),
        sensor_coverages: Vec::new(),
    }));

    let full_circle: f64 = 2.0 * std::f64::consts::PI;

    let full_arclength = full_circle * circles.len() as f64;

    x_range.par_iter().zip(y_range.clone()).for_each(|(&x, y)| {
        let mut sensor =
            Sensor::new_at(&point::Point::new(x as f64, y as f64)).with_resolution(resolution);
        let rays = sensor.rays.clone();
        let field = Field::new(circles.clone(), resolution, width, height);
        let mut field_res = field.clone();

        rays.windows(2).for_each(|pair| {
            let i1 = cast_ray(&field, &pair[0]);
            let i2 = cast_ray(&field, &pair[1]);
            if let Some(Element::Circle(elem1)) = i1 {
                if let Some(Element::Circle(elem2)) = i2 {
                    if elem1.id == elem2.id {
                        let range = elem1.get_range_for_ray_pair(&pair[0], &pair[1]);
                        field_res.update_stack(elem1.id.clone(), range);
                    }
                }
            }
        });

        sensor.coverages = field_res.get_coverage();

        let restore = field_res.elements.clone();

        x_range.iter().zip(y_range.clone()).for_each(|(&x2, y2)| {
            let mut sensor2 = Sensor::new_at(&point::Point::new(x2 as f64, y2 as f64))
                .with_resolution(resolution);
            let rays2 = sensor2.rays.clone();
            let mut _hmap = HashMap::new();
            let mut hmap = HashMap::new();

            rays2.windows(2).for_each(|pair| {
                let i1 = cast_ray(&field, &pair[0]);
                let i2 = cast_ray(&field, &pair[1]);
                if let Some(Element::Circle(elem1)) = i1 {
                    if let Some(Element::Circle(elem2)) = i2 {
                        if elem1.id == elem2.id {
                            let range = elem1.get_range_for_ray_pair(&pair[0], &pair[1]);
                            field_res.update_stack(elem1.id.clone(), range);
                            _hmap.entry(elem1.id.clone()).or_insert(vec![]).push(range);
                        }
                    }
                }
            });

            for (k, v) in _hmap.iter() {
                let mut rs = RangeStack::new();
                for range in v {
                    rs.wrapping_add(range);
                }

                hmap.insert(k.clone(), rs.ranges.iter().collect::<RangeStack>());
            }
            sensor2.coverages = hmap;

            let restore2 = field_res.elements.clone();

            x_range.iter().zip(y_range.clone()).for_each(|(&x3, y3)| {
                let mut sensor3 = Sensor::new_at(&point::Point::new(x3 as f64, y3 as f64))
                    .with_resolution(resolution);
                let rays3 = sensor3.rays.clone();
                let mut _hmap2 = HashMap::new();
                let mut hmap2 = HashMap::new();

                rays3.windows(2).for_each(|pair| {
                    let i1 = cast_ray(&field, &pair[0]);
                    let i2 = cast_ray(&field, &pair[1]);
                    if let Some(Element::Circle(elem1)) = i1 {
                        if let Some(Element::Circle(elem2)) = i2 {
                            if elem1.id == elem2.id {
                                let range = elem1.get_range_for_ray_pair(&pair[0], &pair[1]);
                                field_res.update_stack(elem1.id.clone(), range);
                                _hmap2.entry(elem1.id.clone()).or_insert(vec![]).push(range);
                            }
                        }
                    }
                });

                for (k2, v2) in _hmap2.iter() {
                    let mut rs = RangeStack::new();
                    for range in v2 {
                        rs.wrapping_add(range);
                    }

                    hmap2.insert(k2.clone(), rs.ranges.iter().collect::<RangeStack>());
                }
                sensor3.coverages = hmap2;

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

                let report = report.clone();
                let mut result = report.lock().unwrap();
                if cov > result.max_coverage {
                    result.max_coverage = cov;
                    result.sensor_coverages = vec![
                        sensor.coverages.clone(),
                        sensor2.coverages.clone(),
                        sensor3.coverages.clone(),
                    ];

                    result.sensor_positions = vec![
                        point::Point::new(x as f64, y as f64),
                        point::Point::new(x2 as f64, y2 as f64),
                        point::Point::new(x3 as f64, y3 as f64),
                    ];
                }
                field_res.elements = restore2.clone();
            });
            field_res.elements = restore.clone();
        });
    });

    let rep = report.lock().unwrap();
    if rust_log_is_set() {
        rep.pprint(circles.len());
    }
    (
        Vec::from([
            rep.sensor_positions[0].x,
            rep.sensor_positions[0].y,
            rep.sensor_positions[1].x,
            rep.sensor_positions[1].y,
            rep.sensor_positions[2].x,
            rep.sensor_positions[2].y,
        ]),
        rep.sensor_coverages.clone(),
    )
}

fn rust_log_is_set() -> bool {
    match std::env::var("RUST_LOG") {
        Ok(s) => s == "1",
        _ => false,
    }
}
