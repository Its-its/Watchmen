#[macro_use] extern crate diesel;


pub mod rpc;
pub mod types;
pub mod error;
pub mod config;
pub mod database;


pub mod core;
pub mod state;
pub mod request;
pub mod frontend;

fn main() {
	let mut core = core::FeederCore::new();

	core.init();

	core.run_loop();
}
