use std::time::Duration;

use pyo3::{Python, PyResult, PyErr};

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
}

pub(crate) fn py_constrain<F, T>(f: F) -> F
where
	F: for<'a> Fn(Python<'a>) -> &'a T + Sync + 'static,
{
	f
}