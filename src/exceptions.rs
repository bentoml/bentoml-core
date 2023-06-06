use pyo3::{exceptions::PyException, prelude::*, types::PyType};
use tonic;

#[pyclass]
#[derive(Debug,Copy,Clone)]
pub enum BentoMLStatus {
	Ok = 200,
	BadRequest = 400,
	Unauthorized = 401,
	Forbidden = 403,
	NotFound = 404,
	TooManyRequests = 429,
	InternalServerError = 500,
	ServiceUnavailable = 503,
}

#[pymethods]
impl BentoMLStatus {
	fn http_code(&self) -> u32 {
		(match self {
			_ => *self as u32
		}) as u32
	}

	fn grpc_status(&self) -> u32 {
		(match self {
			BentoMLStatus::Ok => tonic::Code::Ok,
			BentoMLStatus::BadRequest => tonic::Code::InvalidArgument,
			BentoMLStatus::Unauthorized => tonic::Code::Unauthenticated,
			BentoMLStatus::Forbidden => tonic::Code::PermissionDenied,
			BentoMLStatus::NotFound => tonic::Code::NotFound,
			BentoMLStatus::TooManyRequests => tonic::Code::ResourceExhausted,
			BentoMLStatus::InternalServerError => tonic::Code::Internal,
			BentoMLStatus::ServiceUnavailable => tonic::Code::Unavailable,
		}) as u32
	}
}

#[pyclass(extends=PyException,subclass)]
#[derive(Debug)]
pub struct BentoMLException {
	#[pyo3(get)]
	message: String,
}

impl From<BentoMLException> for PyErr {
	fn from(err: BentoMLException) -> PyErr {
		PyErr::new::<BentoMLException, _>((err.message,))
	}
}

#[pymethods]
impl BentoMLException {
	#[new]
	pub fn py_new(message: String) -> PyClassInitializer<Self> {
		PyClassInitializer::from( BentoMLException { message } )
	}

	#[classattr]
	fn error_code() -> BentoMLStatus {
		BentoMLStatus::InternalServerError
	}
}

impl BentoMLException {
	pub fn new(message: String) -> Self {
		BentoMLException { message }
	}

	pub fn get_message(&self) -> &str {
		&self.message
	}
}

pub struct BentoMLExceptionType {
	pub name: &'static str,
	pub register_fn: &'static (dyn Fn(Python)->&'static PyType + Sync + 'static),
}

inventory::collect!(BentoMLExceptionType);

#[macro_export]
macro_rules! create_bentoml_exception {
	($name:ident, $error_code:ident, $superclass:ident) => {
		#[pyclass(extends=$superclass,subclass)]
		#[derive(Debug)]
		pub struct $name {
			message: String,
		}

		impl From<$name> for PyErr {
			fn from(err: $name) -> PyErr {
				PyErr::new::<$name, _>((err.message,))
			}
		}

		#[pymethods]
		impl $name {
			#[new]
			pub fn py_new(message: String) -> PyClassInitializer<$name> {
				$superclass::py_new(message.clone()).add_subclass($name { message })
			}

			#[classattr]
			fn error_code() -> crate::exceptions::BentoMLStatus {
				crate::exceptions::BentoMLStatus::$error_code
			}
		}

		impl $name {
			fn new(message: String) -> Self {
				$name { message }
			}
		}

		inventory::submit!(crate::exceptions::BentoMLExceptionType{name: stringify!($name), register_fn: &|py: Python| {py.get_type::<$name>()}});
	};
}
