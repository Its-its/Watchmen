#![warn(warnings, rust_2018_idioms, unsafe_code, dead_code)]
#![allow(
	clippy::new_without_default,
	clippy::large_enum_variant
)]

#[macro_use] extern crate diesel;

pub mod feature;

pub mod rpc;
pub mod types;
pub mod error;
pub mod filter;
pub mod config;


pub mod core;
pub mod state;
pub mod request;


pub use filter::{FilterType, RegexOpts};
pub use error::{Result, Error};

#[tokio::main]
async fn main() -> std::io::Result<()> {
	feature::logging::configure();

	let mut core = core::FeederCore::new();

	core.init();

	core.run_loop().await;

	Ok(())
}
