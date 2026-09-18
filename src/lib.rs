//! Stateless, offline recommendations. `recommend` is the single recommendation operation.
mod catalog;
mod engine;
mod model;
pub use catalog::Catalog;
pub use engine::recommend;
pub use model::*;

#[cfg(feature = "python")]
mod python {
    use pyo3::{exceptions::PyValueError, prelude::*};

    #[pyfunction]
    fn recommend_json(py: Python<'_>, options_json: &str) -> PyResult<String> {
        let options: crate::Options = serde_json::from_str(options_json)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        py.allow_threads(move || {
            crate::recommend(&options)
                .and_then(|r| serde_json::to_string(&r).map_err(|e| crate::Error(e.to_string())))
        })
        .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[pymodule]
    fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(recommend_json, m)?)?;
        Ok(())
    }
}
