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

const FULL_CIRCLE: f64 = 2.0 * std::f64::consts::PI;

fn main() {
    let circles = vec![
        shape::Circle::new(point::Point::new(2722.0, 472.0), 70.0, RangeStack::new()),
        shape::Circle::new(point::Point::new(2015.0, 445.0), 70.0, RangeStack::new()),
    ];
    let sensor = Sensor::new_at(&point::Point::new(1005.0, 0.0)).with_resolution(2880);

    let rays = sensor.rays.clone();

    let mut field = Field::new(circles, sensor, 3840, 1080);

    for ray in rays {
        cast_ray(&mut field, &ray);
    }

    let full_arclength = FULL_CIRCLE * field.circles.len() as f64;

    let covered: f64 = field
        .circles
        .iter()
        .map(|circle| {
            circle
                .range_stack
                .ranges
                .iter()
                .collect::<RangeStack>()
                .length()
        })
        .sum();

    println!("percentage covered {:?}", 100.0 * covered / full_arclength);
    // println!("{:?}", field.circles);
}
