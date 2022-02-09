pub mod field;
pub mod point;
pub mod rangestack;
pub mod ray;
pub mod report;
pub mod sensor;
pub mod shape;
pub mod vector;

// use numpy::{IntoPyArray, PyArray2, PyReadonlyArray2};
use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn place_me(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    // m.add_function(wrap_pyfunction!(run, m)?)?;
    Ok(())
}

// use pyo3::{exceptions::PyRuntimeError, pymodule, types::PyModule, PyErr, PyResult, Python};

// #[pyfunction]
// fn run<'py>(py: Python<'py>, x: PyReadonlyArray2<'py, f64>) -> PyResult<&'py PyArray2<f64>> {
//     let x = x.as_array();

//     Ok(x)
// }
