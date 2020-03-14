use std::sync::RwLock;


#[cfg(feature = "website")]
use crate::feature::FrontendCore;
#[cfg(feature = "terminal")]
use crate::feature::TerminalCore;
#[cfg(feature = "telegram")]
use crate::feature::TelegramCore;

use crate::feature::Connection;

use crate::core::WeakFeederCore;
use crate::request::{RequestManager, RequestResults};
use crate::config::ConfigManager;


pub struct CoreState {
	#[cfg(feature = "website")]
	pub frontend: FrontendCore,
	#[cfg(feature = "terminal")]
	pub terminal: TerminalCore,
	#[cfg(feature = "telegram")]
	pub telegram: TelegramCore,

	pub connection: Connection,
	pub requester: RequestManager,
	pub config: RwLock<ConfigManager>,
}

impl CoreState {
	pub fn new() -> Self {
		Self {
			#[cfg(feature = "website")]
			frontend: FrontendCore::new(),
			#[cfg(feature = "terminal")]
			terminal: TerminalCore::new(),
			#[cfg(feature = "telegram")]
			telegram: TelegramCore::new(),

			connection: Connection::new(),
			requester: RequestManager::new(),
			config: RwLock::new(ConfigManager::new()),
		}
	}

	#[allow(unused_variables)]
	pub fn init(&mut self, weak_core: WeakFeederCore) {
		#[cfg(feature = "website")]
		self.frontend.init(weak_core.clone());
		#[cfg(feature = "terminal")]
		self.terminal.init(weak_core.clone());
		#[cfg(feature = "telegram")]
		self.telegram.init(weak_core.clone());

		{
			let mut write = self.config.write().unwrap();

			write.init();
			write.load().unwrap_or_else(|e| panic!("Loading Config Error: {}", e));
		}

		self.connection.init_sql().unwrap_or_else(|e| panic!("Loading Database Error: {}", e));

		self.requester.init(self.connection.connection())
			.unwrap_or_else(|e| panic!("Requester Initiation Error: {}", e));
	}

	//
	pub fn run_all_requests(&mut self) -> RequestResults {
		self.requester.request_all_if_idle(
			false,
			self.connection.connection()
		)
	}
}