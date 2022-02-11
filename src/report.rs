use std::num;

// TODO: This is kind of bad, some rework needed..
use crate::point::Point;

#[derive(Debug, Clone)]
pub struct Report {
    pub max_coverage: f64,
    pub sensor_positions: Vec<Point>,
    pub extra: Vec<Vec<Point>>,
}

impl Report {
    pub fn pprint(&self, num_circles: usize) {
        println!("\ncovered {:?}/{:?}", self.max_coverage, num_circles * 2);
        println!("optimal positions {:#?}", self.sensor_positions);
        println!("also at {:#?}", self.extra);
    }
}
