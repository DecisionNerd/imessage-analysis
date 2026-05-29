// Python bindings — Phase 5.
use pyo3::prelude::*;

#[pymodule]
fn imessage_analysis(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
