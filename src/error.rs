use std::fmt;

use std::io::Error as IoError;
use serde_json::Error as JsonError;
use rss::Error as RssError;
use reqwest::Error as HttpError;
use diesel::result::Error as DieselError;

#[derive(Debug)]
pub enum Error {
	Io(IoError),
	Json(JsonError),

	Diesel(DieselError),
	Http(HttpError),
	Rss(RssError),
}


impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use Error::*;

		match *self {
			Io(ref e) => write!(f, "IO Error: {:?}", e),
			Json(ref e) => write!(f, "JSON Error: {:?}", e),

			Rss(ref e) => write!(f, "RSS Error: {:?}", e),
			Http(ref e) => write!(f, "HTTP Error: {:?}", e),
			Diesel(ref e) => write!(f, "Diesel Error: {:?}", e),
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