use std::time::Duration;

use pyo3::exceptions::PyOSError;
use pyo3::{Python, PyResult, PyErr};

use pyo3::prelude::*;
use tar;
use zstd;

/// Run a long-running function in a separate thread, checking for python signals every 500ms.
/// This function should be used for any long-running function, so that python signals, such as
/// KeyboardInterrupt, can be handled.
/// Otherwise, the function will not be interrupted until it returns.
pub(crate) fn run_cancellable<
	T: Send + 'static,
	E: Send + 'static,
	F: FnMut() -> Result<T, E> + Send + 'static,
>(
	py: Python,
	f: F,
) -> PyResult<T>
where
	PyErr: From<E>,
{
	let thread = std::thread::spawn(f);
	loop {
		py.check_signals()?;
		if thread.is_finished() {
			return Ok(thread.join().unwrap()?);
		}
		std::thread::sleep(Duration::from_millis(500));
	}
}use std::fs::{File, self};

#[pyfunction]
pub fn pack(py: Python, path: &str, archive: &str) -> PyResult<()> {
	let out_file = File::create(&archive)?;

	let mut tar = tar::Builder::new(zstd::Encoder::new(out_file, 3)?);

	let path: String = path.to_string();
	run_cancellable(py, move || tar.append_dir_all("", &path))?;

	Ok(())
}

#[pyfunction]
pub fn unpack(py: Python, archive: &str, out_path: &str) -> PyResult<()> {
	let in_file = File::open(&archive)?;
	let mut tar = tar::Archive::new(zstd::Decoder::new(in_file)?);

	fs::create_dir_all(&out_path)?;

	let path: String = out_path.to_string();
	run_cancellable(py, move || tar.unpack(&path))?;

	Ok(())
}