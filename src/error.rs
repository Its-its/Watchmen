use std::fmt;

use std::io::Error as IoError;
use serde_json::Error as JsonError;
use rss::Error as RssError;
use reqwest::Error as HttpError;
use diesel::result::Error as DieselError;
use atom_syndication::Error as AtomError;


pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	Io(IoError),
	Json(JsonError),

	Diesel(DieselError),
	Http(HttpError),

	Rss(RssError),
	Atom(AtomError),

	Other(String)
}


impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use Error::*;

		match *self {
			Io(ref e) => write!(f, "IO Error: {:?}", e),
			Json(ref e) => write!(f, "JSON Error: {:?}", e),

			Rss(ref e) => write!(f, "RSS Error: {:?}", e),
			Atom(ref e) => write!(f, "Atom Error: {:?}", e),
			Http(ref e) => write!(f, "HTTP Error: {:?}", e),
			Diesel(ref e) => write!(f, "Diesel Error: {:?}", e),

			Other(ref e) => write!(f, "Other Error: {:?}", e)
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

impl From<RssError> for Error {
	fn from(error: RssError) -> Self {
		Error::Rss(error)
	}
}

impl From<AtomError> for Error {
	fn from(error: AtomError) -> Self {
		Error::Atom(error)
	}
}

impl From<HttpError> for Error {
	fn from(error: HttpError) -> Self {
		Error::Http(error)
	}
}

impl From<DieselError> for Error {
	fn from(error: DieselError) -> Self {
		Error::Diesel(error)
	}
}

impl From<String> for Error {
	fn from(error: String) -> Self {
		Error::Other(error)
	}
}

impl From<&str> for Error {
	fn from(error: &str) -> Self {
		Error::Other(error.to_owned())
	}
}