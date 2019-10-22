use std::sync::{Arc, Mutex, MutexGuard, Weak};

use crate::core::WeakFeederCore;

pub mod web;

pub use web::Web;
use super::rpc::{Front2CoreNotification, Core2FrontNotification};


pub struct DaemonCore(Arc<Mutex<DaemonState>>);

impl DaemonCore {
	pub fn new() -> Self {
		Self(Arc::new(Mutex::new(DaemonState::new())))
	}


	pub fn init(&mut self, weak_core: WeakFeederCore) {
		let weak = self.to_weak();

		let mut inner = self.to_inner();

		inner.init(weak_core, weak);
	}


	pub fn to_inner(&self) -> MutexGuard<DaemonState> {
		self.0.lock().unwrap()
	}

	pub fn to_weak(&self) -> WeakDaemonCore {
		WeakDaemonCore(Arc::downgrade(&self.0))
	}
}


#[derive(Clone)]
pub struct WeakDaemonCore(Weak<Mutex<DaemonState>>);

impl WeakDaemonCore {
	pub fn upgrade(&self) -> Option<DaemonCore> {
		self.0.upgrade().map(DaemonCore)
	}
}


pub struct DaemonState {
	pub web: Web
}

impl DaemonState {
	pub fn new() -> Self {
		DaemonState {
			web: Web::new()
		}
	}

	pub fn init(&mut self, weak_core: WeakFeederCore, weak_frontend: WeakDaemonCore) {
		self.web.listen(weak_core, weak_frontend).expect("Web.listen()");
	}
}