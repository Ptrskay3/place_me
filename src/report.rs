use std::collections::HashMap;


use crate::{point::Point, rangestack::RangeStack};

#[derive(Debug, Clone)]
pub struct Report {
    pub max_coverage: f64,
    pub sensor_positions: Vec<Point>,
    pub sensor_coverages: Vec<HashMap<String, RangeStack>>,
}

impl Report {
    pub fn pprint(&self, num_circles: usize) {
        println!(
            "\n🎯 covered {:?}% ({:?}/{:?})",
            self.max_coverage / (num_circles * 2) as f64 * 100.0,
            self.max_coverage,
            num_circles * 2
        );
        println!("✅ optimal positions {:#?}", self.sensor_positions);
    }
}
