use std::io::{Read, Write};
use std::fs::{OpenOptions};
use std::path::PathBuf;

use serde_json::{from_str, to_string_pretty};

use crate::error::Error;

pub use opts::Config;

#[derive(Default)]
pub struct ConfigManager {
	config: Config,
	file_path: PathBuf
}

impl ConfigManager {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn init(&mut self/*, file_path: PathBuf*/) {
		self.file_path = PathBuf::from("app/config.json");
	}

	pub fn load(&mut self) -> Result<(), Error>  {
		let mut is_new_file = false;

		{ // Load File if it exists.
			let mut options = OpenOptions::new();

			match options.read(true).open(&self.file_path) {
				Ok(mut file) => {
					let mut contents = String::new();

					file.read_to_string(&mut contents)?;

					self.set_config(from_str(&contents)?);
				}

				Err(_) => is_new_file = true
			}
		}

		if is_new_file {
			self.save()?;
		}

		Ok(())
	}

	pub fn save(&self) -> Result<(), Error> {
		let mut options = OpenOptions::new();

		let mut file = options
			.write(true)
			.create(true)
			.truncate(true)
			.open(&self.file_path)?;

		let contents = to_string_pretty(&self.config)?;

		file.write_all(contents.as_bytes())?;

		Ok(())
	}

	pub fn set_config(&mut self, config: Config) {
		self.config = config;
	}

	pub fn config(&self) -> Config {
		self.config.clone()
	}
}


mod opts {
	use serde::{Serialize, Deserialize};

	#[derive(Default, Clone, Serialize, Deserialize)]
	pub struct Config {
		pub telegram: ConfigTelegram,
		pub request: ConfigRequest
	}

	#[derive(Clone, Serialize, Deserialize)]
	pub struct ConfigRequest {
		#[serde(default = "default_true")]
		pub enabled: bool,
		pub concurrency: i32 // Currently not used
	}

	impl Default for ConfigRequest {
		fn default() -> Self {
			ConfigRequest {
				enabled: true,
				concurrency: 2
			}
		}
	}


	#[derive(Default, Clone, Serialize, Deserialize)]
	pub struct ConfigTelegram {
		pub api_key: String,
		pub chat_id: Option<i64>
	}


	fn default_true() -> bool {
		true
	}
}