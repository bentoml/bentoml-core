use std::fs::File;

use pyo3::prelude::*;
use tar;
use zstd;

use crate::utils::run_cancellable;

// TODO: convert this to take tag once bento is moved to rust.
#[pyfunction]
pub fn pack_bento(py: Python<'_>, path: &str, name: &str) -> PyResult<String> {
	let out_path = format!("{}.bento", name);
	let out_file = File::create(&out_path)?;

	let mut tar = tar::Builder::new(zstd::Encoder::new(out_file, 3)?);

	let path: String = path.to_string();
	run_cancellable(py, move || tar.append_dir_all("", &path))?;

	Ok(out_path)
}
