use std::sync::RwLock;

use crate::frontend::FrontendCore;
use crate::core::WeakFeederCore;
use crate::database::Connection;
use crate::request::RequestManager;
use crate::config::ConfigManager;


pub struct CoreState {
	pub frontend: FrontendCore,
	pub connection: Connection,
	pub requester: RequestManager,
	pub config: RwLock<ConfigManager>,
}

impl CoreState {
	pub fn new() -> Self {
		Self {
			frontend: FrontendCore::new(),
			connection: Connection::new(),
			requester: RequestManager::new(),
			config: RwLock::new(ConfigManager::new()),
		}
	}

	pub fn init(&mut self, weak_core: WeakFeederCore) {
		{
			let mut write = self.config.write().unwrap();

			write.init();
			write.load().unwrap_or_else(|e| panic!("Loading Config Error: {}", e));
		}

		self.connection.init_sql().unwrap_or_else(|e| panic!("Loading Database Error: {}", e));

		self.frontend.init(weak_core);
	}
}