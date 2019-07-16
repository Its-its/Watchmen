use std::thread;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::time::{Duration, Instant};


pub struct RequestManager {
	pub is_running: bool,
	pub concurrency: i32,
}

impl RequestManager {
	pub fn new() -> Self {
		Self {
			is_running: false,
			concurrency: 2
		}
	}

	pub fn start_if_idle(&self) {
		if self.is_running {
			println!("Request Manager is already running!");
			return;
		}

		thread::spawn(move || {
			//
		});
	}
}


pub struct RequestResults {
	pub was_manual: bool,
	pub start_time: Instant,
	pub duration: Duration,
	pub item_count: i32,
	pub concurrency: i32,
	pub items: Vec<RequestItemResults>
}

pub struct RequestItemResults {
	pub start_time: Instant,
	pub duration: Duration,
	pub item_count: i32,
}