#[macro_use] extern crate diesel;


pub mod rpc;
pub mod types;
pub mod error;
pub mod filter;
pub mod config;


pub mod core;
pub mod state;
pub mod request;
pub mod feature;


pub use filter::{Filter, RegexOpts};
pub use error::Result;


fn main() {
	let mut core = core::FeederCore::new();

	core.init();

	core.run_loop();
}
