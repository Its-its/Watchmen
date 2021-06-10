use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::thread;

use lazy_static::lazy_static;
use log::info;

use crate::core::WeakFeederCore;
use super::rpc::{Front2CoreNotification, Core2FrontNotification};

pub mod core;
pub mod util;



use log::Level;


lazy_static! {
	pub static ref PRINT_LINES: Mutex<Vec<(String, Level)>> = Mutex::new(Vec::new());
}


pub struct TerminalCore(Arc<Mutex<TerminalState>>);

impl TerminalCore {
	pub fn new() -> Self {
		Self(Arc::new(Mutex::new(TerminalState::new())))
	}


	pub fn init(&mut self, weak_core: WeakFeederCore) {
		let weak = self.to_weak();

		let mut inner = self.to_inner();

		inner.init(weak_core, weak);
	}


	pub fn to_inner(&self) -> MutexGuard<TerminalState> {
		self.0.lock().unwrap()
	}

	pub fn to_weak(&self) -> WeakTerminalCore {
		WeakTerminalCore(Arc::downgrade(&self.0))
	}
}


#[derive(Clone)]
pub struct WeakTerminalCore(Weak<Mutex<TerminalState>>);

impl WeakTerminalCore {
	pub fn upgrade(&self) -> Option<TerminalCore> {
		self.0.upgrade().map(TerminalCore)
	}
}


pub struct TerminalState {
	//
}

impl TerminalState {
	pub fn new() -> Self {
		TerminalState {
		}
	}

	pub fn init(&mut self, weak_feeder: WeakFeederCore, weak_terminal: WeakTerminalCore) {
		info!("Initiaiting Terminal");

		thread::spawn(move || {
			core::display().expect("Terminal");
		});
	}
}