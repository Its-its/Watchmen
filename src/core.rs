use std::sync::{Arc, Mutex, MutexGuard, Weak};


use crate::types::MessageId;
use crate::frontend::socket::WebsocketWrapper;
use crate::state::CoreState;
use crate::frontend::rpc::{Front2CoreNotification};

pub struct FeederCore(Arc<Mutex<CoreState>>);

impl FeederCore {
	pub fn new() -> Self {
		Self(Arc::new(Mutex::new(CoreState::new())))
	}

	pub fn init(&mut self) {
		let mut inner = self.to_inner();

		inner.init(self.to_weak());
	}

	pub fn run_loop(&self) {
		loop {
			// Sleep otherwise loop will make the process use lots of cpu power.
			std::thread::sleep(std::time::Duration::from_millis(100));
		}
	}

	// Util

	pub fn to_inner(&self) -> MutexGuard<CoreState> {
		self.0.lock().unwrap()
	}

	pub fn to_weak(&self) -> WeakFeederCore {
		WeakFeederCore(Arc::downgrade(&self.0))
	}
}


// Weak Core | sent to the plugins / WebSocket

#[derive(Clone)]
pub struct WeakFeederCore(Weak<Mutex<CoreState>>);

impl WeakFeederCore {
	pub fn upgrade(&self) -> Option<FeederCore> {
		self.0.upgrade().map(FeederCore)
	}
}

impl WeakFeederCore {
	pub fn handle_frontend(
		&mut self,
		ctx: &mut WebsocketWrapper,
		message_id: Option<MessageId>,
		rpc: Front2CoreNotification
	) {
		use Front2CoreNotification::*;

		let upgrade = self.upgrade().unwrap();
		let inner = upgrade.to_inner();

		match rpc {
			Update {  } => {
				match message_id {
					Some(message_id) => {
						ctx.respond_request(message_id, Ok(serde_json::json!({ "testing": "asdf" })));
					}

					None => {
						// let update = plugin.send_update_notification(command);
						// ctx.respond_notification(update);
					}
				}
			}

			_ => ()
		}
	}
}