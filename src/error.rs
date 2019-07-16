use std::fmt;
use std::io::Error as IoError;

use serde_json::Error as JsonError;

pub enum Error {
	Io(IoError),
	Json(JsonError)
}


impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use Error::*;

		match *self {
			Io(ref e) => write!(f, "IO Error: {:?}", e),
			Json(ref e) => write!(f, "Json Parsing Error: {:?}", e)
		}
	}
}


impl From<JsonError> for Error {
	fn from(error: JsonError) -> Self {
		Error::Json(error)
	}
}

impl From<IoError> for Error {
	fn from(error: IoError) -> Self {
		Error::Io(error)
	}
}