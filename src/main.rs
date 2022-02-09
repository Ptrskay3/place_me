pub mod field;
pub mod point;
pub mod rangestack;
pub mod ray;
pub mod sensor;
pub mod shape;
pub mod vector;
use field::{cast_ray, Field};
use rangestack::RangeStack;
use sensor::Sensor;

use rayon::prelude::*;

const FULL_CIRCLE: f64 = 2.0 * std::f64::consts::PI;
const WIDTH: u32 = 3840;
const HEIGHT: u32 = 1080;
const RESOLUTION: u32 = 2880;

fn main() {
    let (x_range, y_range) = Sensor::coordinates_along_circumference(WIDTH, HEIGHT, 20);
    let circles = vec![
        shape::Circle::new(point::Point::new(2722.0, 472.0), 70.0, RangeStack::new()),
        shape::Circle::new(point::Point::new(2015.0, 445.0), 70.0, RangeStack::new()),
    ];

    let full_arclength = FULL_CIRCLE * circles.len() as f64;

    x_range.par_iter().zip(y_range).for_each(|(&x, y)| {
        let sensor =
            Sensor::new_at(&point::Point::new(x as f64, y as f64)).with_resolution(RESOLUTION);
        let rays = sensor.rays.clone();
        let mut field = Field::new(circles.clone(), sensor, WIDTH, HEIGHT);

        for ray in rays {
            cast_ray(&mut field, &ray);
        }

        let covered: f64 = field
            .circles
            .par_iter() // using par iter here might be an overkill, as we don't really have that many circles
            .map(|circle| {
                circle
                    .range_stack
                    .ranges
                    .par_iter()
                    .collect::<RangeStack>()
                    .length()
            })
            .sum();

        println!(
            "percentage covered {:?} at ({:?}, {:?})",
            100.0 * covered / full_arclength,
            x,
            y
        );
    });

    // println!("{:?}", field.circles);
}
