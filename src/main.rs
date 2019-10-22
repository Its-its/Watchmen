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


pub use filter::{Filter, RegexOpts};
pub use error::Result;

fn main() {
	feature::logging::configure();

	let mut core = core::FeederCore::new();

	core.init();

	core.run_loop();
}
