use std::sync::{Arc, Mutex, MutexGuard, Weak};

use crate::core::WeakFeederCore;

pub mod web;
pub mod socket;

pub use web::Web;
pub use socket::WebsocketWrapper;
use super::rpc::{Front2CoreNotification, Core2FrontNotification};


pub struct FrontendCore(Arc<Mutex<FrontendState>>);

impl FrontendCore {
	pub fn new() -> Self {
		Self(Arc::new(Mutex::new(FrontendState::new())))
	}


	pub fn init(&mut self, weak_core: WeakFeederCore) {
		let weak = self.to_weak();

		let mut inner = self.to_inner();

		inner.init(weak_core, weak);
	}


	pub fn to_inner(&self) -> MutexGuard<FrontendState> {
		self.0.lock().unwrap()
	}

	pub fn to_weak(&self) -> WeakFrontendCore {
		WeakFrontendCore(Arc::downgrade(&self.0))
	}
}


#[derive(Clone)]
pub struct WeakFrontendCore(Weak<Mutex<FrontendState>>);

impl WeakFrontendCore {
	pub fn upgrade(&self) -> Option<FrontendCore> {
		self.0.upgrade().map(FrontendCore)
	}
}


pub struct FrontendState {
	pub web: Web
}

impl FrontendState {
	pub fn new() -> Self {
		FrontendState {
			web: Web::new()
		}
	}

	pub fn init(&mut self, weak_core: WeakFeederCore, weak_frontend: WeakFrontendCore) {
		self.web.listen(weak_core, weak_frontend).expect("Web.listen()");
	}
}