// https://codereview.stackexchange.com/questions/103864/merging-an-overlapping-collection-of-intervals

use std::iter::FromIterator;
use std::{cmp, fmt};

use crate::shape::TWOPI;

use rayon::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct Range {
    pub start: f64,
    pub end: f64,
}

impl Range {
    pub fn new(start: f64, end: f64) -> Range {
        Range { start, end }
    }

    fn overlaps(&self, other: &Range) -> bool {
        (other.start >= self.start && other.start <= self.end)
            || (other.end >= self.start && other.end <= self.end)
    }

    fn merge(&mut self, other: &Range) {
        self.start = cmp::min_by(self.start, other.start, |x: &f64, y: &f64| {
            x.partial_cmp(y).unwrap()
        });
        self.end = cmp::max_by(self.end, other.end, |x: &f64, y: &f64| {
            x.partial_cmp(y).unwrap()
        });
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{},{}]", self.start, self.end)
    }
}

#[derive(Debug, Clone)]
pub struct RangeStack {
    pub ranges: Vec<Range>,
}

impl Default for RangeStack {
    fn default() -> Self {
        Self::new()
    }
}

impl RangeStack {
    pub fn new() -> RangeStack {
        RangeStack { ranges: Vec::new() }
    }

    fn merging_add(&mut self, range: &Range) {
        if let Some(last) = self.ranges.last_mut() {
            if last.overlaps(range) {
                last.merge(range);
                return;
            }
        }

        self.ranges.push(*range);
    }

    pub fn add(&mut self, range: &Range) {
        if range.end < range.start {
            self.add(&Range::new(range.end, range.start));
            return;
        }
        self.ranges.push(*range);
    }

    /// Add a range to the stack, wrapping around 2 * PI.
    pub fn wrapping_add(&mut self, range: &Range) {
        let end = range.end;
        let start = range.start;
        match (start < 0.0, end > TWOPI) {
            (true, true) => {
                let end_overlap = end - TWOPI;
                let start_overlap = TWOPI + start;
                self.add(&Range::new(TWOPI - start, end - TWOPI));
                self.add(&Range::new(0.0, end_overlap));
                self.add(&Range::new(start_overlap, TWOPI));
            }
            (true, false) => {
                let start_overlap = TWOPI + start;
                self.add(&Range::new(0.0, end));
                self.add(&Range::new(start_overlap, TWOPI));
            }
            (false, true) => {
                let end_overlap = end - TWOPI;
                self.add(&Range::new(start, end - TWOPI));
                self.add(&Range::new(0.0, end_overlap));
            }
            (false, false) => {
                self.add(range);
            }
        }
    }

    pub fn length(&self) -> f64 {
        self.ranges.par_iter().map(|r| r.end - r.start).sum()
    }
}

impl fmt::Display for RangeStack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for range in &self.ranges {
            write!(f, " {}", range)?;
        }
        Ok(())
    }
}

impl FromIterator<Range> for RangeStack {
    fn from_iter<I>(iterator: I) -> Self
    where
        I: IntoIterator<Item = Range>,
    {
        let mut raw_ranges: Vec<_> = iterator.into_iter().collect();
        raw_ranges.sort_by(|a, b| {
            a.start
                .partial_cmp(&b.start)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut range_stack = RangeStack { ranges: Vec::new() };

        for range in &raw_ranges {
            range_stack.merging_add(range);
        }

        range_stack
    }
}

impl<'p> FromParallelIterator<&'p Range> for RangeStack {
    fn from_par_iter<I>(iterator: I) -> Self
    where
        I: IntoParallelIterator<Item = &'p Range>,
    {
        let mut raw_ranges: Vec<_> = iterator.into_par_iter().collect();
        raw_ranges.sort_by(|a, b| {
            a.start
                .partial_cmp(&b.start)
                .unwrap_or(std::cmp::Ordering::Greater)
        });

        let mut range_stack = RangeStack { ranges: Vec::new() };

        for range in &raw_ranges {
            range_stack.merging_add(range);
        }

        range_stack
    }
}

impl<'a> FromIterator<&'a Range> for RangeStack {
    fn from_iter<I>(iterator: I) -> Self
    where
        I: IntoIterator<Item = &'a Range>,
    {
        iterator.into_iter().cloned().collect()
    }
}
