mod utils;

use pyo3::prelude::*;

pub const BENTOML_VERSION: &str = env!("CARGO_PKG_VERSION");

fn register_submodule(py: Python, module: &PyModule) -> PyResult<()> {
    py.import("sys")?
        .getattr("modules")?
        .set_item(format!("bentoml_core.{}", module.name()?), module)
}

#[pymodule]
fn _bentoml_core(py: Python, m: &PyModule) -> PyResult<()> {
	let utils_module = PyModule::new(py, "utils")?;
	utils_module.add_function(wrap_pyfunction!(utils::pack, utils_module)?)?;
	utils_module.add_function(wrap_pyfunction!(utils::unpack, utils_module)?)?;
	m.add_submodule(utils_module)?;
	register_submodule(py, utils_module)?;

	Ok(())
}
