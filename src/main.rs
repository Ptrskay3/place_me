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

fn main() {
    let circles = vec![
        shape::Circle::new(point::Point::new(2722.0, 472.0), 70.0, RangeStack::new()),
        shape::Circle::new(point::Point::new(2015.0, 445.0), 70.0, RangeStack::new()),
    ];
    let sensor = Sensor::new_at(&point::Point::new(2015.0, 0.0)).with_resolution(100);

    let rays = sensor.rays.clone();

    let mut field = Field::new(circles, sensor, 3840, 1080);
    // println!("{:#?}", field);
    //
    for ray in rays {
        cast_ray(&mut field, &ray);
    }

    println!("{:?}", field.circles[0]);
}
