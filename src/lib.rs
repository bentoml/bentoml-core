mod bentos;
mod exceptions;
mod tag;
mod utils;

use exceptions::BentoMLExceptionType;
use pyo3::prelude::*;

use utils::py_constrain;

pub const BENTOML_VERSION: &str = env!("CARGO_PKG_VERSION");

#[pymodule]
fn _bentoml_core(py: Python, m: &PyModule) -> PyResult<()> {
	m.add_class::<tag::Tag>()?;
	m.add_function(wrap_pyfunction!(tag::validate_tag_str, m)?)?;

	let exceptions_module = PyModule::new(py, "exceptions")?;
	exceptions_module.add(
		"BentoMLException",
		py.get_type::<exceptions::BentoMLException>(),
	)?;
	exceptions_module.add("BentoMLStatus", py.get_type::<exceptions::BentoMLStatus>())?;
	for exc in inventory::iter::<BentoMLExceptionType> {
		exceptions_module.add(exc.name, (py_constrain(exc.register_fn))(py))?;
	}

	m.add_submodule(exceptions_module)?;

	let bentos_module = PyModule::new(py, "bentos")?;
	bentos_module.add_function(wrap_pyfunction!(bentos::pack_bento, bentos_module)?)?;
	m.add_submodule(bentos_module)?;

	Ok(())
}
