use crate::point::Point;

#[derive(Debug, Clone)]
pub struct Report {
    pub max_coverage: f64,
    pub sensor_positions: Vec<Point>,
    pub extra: Vec<Vec<Point>>,
}

impl Report {
    pub fn pprint(&self, full_arclength: f64) {
        let coverage_pretty = 100.0 * self.max_coverage / full_arclength;
        println!("\ncovered {:?}%", coverage_pretty);
        println!("optimal positions {:#?}", self.sensor_positions);
        println!("also at {:#?}", self.extra);
    }
}
