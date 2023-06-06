use std::{
	collections::hash_map::DefaultHasher,
	fmt::{Display, Formatter},
	hash::{Hash, Hasher},
	path::Path,
	str::FromStr,
};

use lazy_static::lazy_static;
use pyo3::{
	exceptions::{PyTypeError, PyValueError},
	prelude::*,
	pyclass::CompareOp,
	types::PyBytes,
};
use rand::Rng;
use regex::Regex;

use uuid;

use bincode::{deserialize, serialize};
use serde;
use serde::{Deserialize, Serialize};

use crate::{create_bentoml_exception, exceptions::BentoMLException};

create_bentoml_exception!(InvalidTagException, InternalServerError, BentoMLException);

#[pyclass(module = "bentoml_core")]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Tag {
	#[pyo3(get, set)]
	name: String,
	#[pyo3(get, set)]
	version: Option<String>,
}

impl PartialOrd for Tag {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		if self.name < other.name {
			return Some(std::cmp::Ordering::Less);
		} else {
			match (&self.version, &other.version) {
				(Some(sv), Some(ov)) => return Some(sv.cmp(ov)),
				(Some(_), None) => return Some(std::cmp::Ordering::Greater),
				(None, Some(_)) => return Some(std::cmp::Ordering::Less),
				(None, None) => return None,
			}
		}
	}
}

impl Display for Tag {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match &self.version {
			Some(v) => write!(f, "{}:{}", self.name, v),
			None => write!(f, "{}", self.name),
		}
	}
}

impl FromStr for Tag {
	type Err = InvalidTagException;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut parts = s.split(':');
		let name = parts.next().unwrap().to_lowercase();
		validate_tag_str(&name)?;
		let version = parts.next().map(|v| v.to_lowercase());
		if let Some(v) = &version {
			validate_tag_str(&v)?;
		}
		if parts.next().is_some() {
			return Err(InvalidTagException::new(
				"Tag must be in the format name:version".to_string(),
			));
		}
		Ok(Self {
			name: name.to_string(),
			version: match version {
				Some(v) => Some(v.to_string()),
				None => None,
			},
		})
	}
}

impl TryFrom<&str> for Tag {
	type Error = InvalidTagException;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		<Tag as FromStr>::from_str(value)
	}
}

lazy_static! {
	static ref TAG_REGEX: Regex =
		Regex::new(r"^[a-z0-9]([-._a-z0-9]*[a-z0-9])?$").expect("Error parsing tag regex!");
}
const TAG_MAX_LENGTH: usize = 63;

#[pyfunction]
pub fn validate_tag_str(value: &str) -> Result<(), InvalidTagException> {
	if value.len() > TAG_MAX_LENGTH {
		return Err(InvalidTagException::new(format!(
			"Tag length must be less than or equal to {} characters",
			TAG_MAX_LENGTH
		)));
	}
	if !TAG_REGEX.is_match(value) {
		return Err(InvalidTagException::new("A tag's name or version must consist of alphanumeric characters, '_', '-', or '.', and must start and end with an alphanumeric character".to_string()));
	}
	Ok(())
}

impl Hash for Tag {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name.hash(state);
		self.version.hash(state);
	}
}

#[pymethods]
impl Tag {
	#[new]
	pub fn new(name: String, version: Option<String>) -> PyResult<Self> {
		let lname = name.to_lowercase();
		validate_tag_str(&lname)?;
		let lversion = match version {
			Some(v) => {
				let lv = v.to_lowercase();
				validate_tag_str(&lv)?;
				Some(lv)
			}
			None => None,
		};
		Ok(Self {
			name: lname,
			version: lversion,
		})
	}

	pub fn __str__(&self) -> PyResult<String> {
		Ok(format!("{}", self))
	}

	pub fn __repr__(&self) -> PyResult<String> {
		match &self.version {
			Some(v) => Ok(format!("Tag(name={}, version={})", self.name, v)),
			None => Ok(format!("Tag(name={})", self.name)),
		}
	}

	pub fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<bool> {
		let other_tag = other.extract::<Tag>()?;
		match op {
			CompareOp::Eq => Ok(self == &other_tag),
			CompareOp::Lt => Ok(self < &other_tag),
			CompareOp::Le => Ok(self <= &other_tag),
			CompareOp::Ne => Ok(self != &other_tag),
			CompareOp::Gt => Ok(self > &other_tag),
			CompareOp::Ge => Ok(self >= &other_tag),
		}
	}

	pub fn __hash__(&self) -> u64 {
		let mut s = DefaultHasher::new();
		self.hash(&mut s);
		s.finish()
	}

	pub fn __setstate__(&mut self, state: &PyBytes) -> PyResult<()> {
		match deserialize(state.as_bytes()) {
			Ok(tag) => *self = tag,
			Err(e) => return Err(PyValueError::new_err(format!("Invalid tag state: {}", e))),
		}
		Ok(())
	}

	pub fn __getstate__<'py>(&self, py: Python<'py>) -> PyResult<&'py PyBytes> {
		Ok(PyBytes::new(py, &serialize(self).unwrap()))
	}

	pub fn __getnewargs__(&self) -> PyResult<(String, Option<String>)> {
		Ok((self.name.clone(), self.version.clone()))
	}

	#[staticmethod]
	pub fn from_taglike(taglike: &PyAny) -> PyResult<Self> {
		if let Ok(tag) = taglike.extract::<Tag>() {
			return Ok(tag);
		}
		if let Ok(s) = taglike.extract::<&str>() {
			return Ok(Self::from_str(s)?);
		}
		return Err(PyTypeError::new_err(format!(
			"Attempted to convert a {} to a tag, input must be a str or a Tag!",
			taglike.get_type().name()?
		)));
	}

	#[staticmethod]
	#[pyo3(name = "from_str")]
	pub fn py_from_str(s: &str) -> PyResult<Tag> {
		return Ok(<Tag as FromStr>::from_str(s)?);
	}

	pub fn make_new_version(&self) -> Tag {
		let mac = mac_address::get_mac_address()
			.unwrap_or(Some(mac_address::MacAddress::new(rand::thread_rng().gen())))
			.unwrap_or(mac_address::MacAddress::new(rand::thread_rng().gen()));
		let uuid = uuid::Uuid::now_v1(&mac.bytes());
		let ver_bytes = uuid.as_bytes();
		// cut out the time_h1 and node bits of the uuid
		let ver_bytes = [&ver_bytes[0..6], &ver_bytes[8..12]].concat();
		let encoded_ver = data_encoding::BASE32_NOPAD.encode(&ver_bytes);
		return Tag::new(self.name.clone(), Some(encoded_ver))
			.expect("Invalid version generated by make_new_version");
	}

	pub fn path(&self) -> String {
		let path_str = match &self.version {
			Some(v) => format!("{}/{}", self.name, v),
			None => self.name.clone(),
		};
		Path::new(&path_str).to_string_lossy().to_string()
	}

	pub fn latest_path(&self) -> String {
		Path::new(&format!("{}/latest", self.name))
			.to_string_lossy()
			.to_string()
	}
}
