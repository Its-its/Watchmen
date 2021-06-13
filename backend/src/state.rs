use std::sync::RwLock;


#[cfg(feature = "website")]
use crate::feature::FrontendCore;
#[cfg(feature = "terminal")]
use crate::feature::TerminalCore;
#[cfg(feature = "telegram")]
use crate::feature::TelegramCore;

use crate::{config::Config, feature::Connection};

use crate::core::WeakFeederCore;
use crate::request::{
	RequestResults,
	feeds::RequestManager as FeedRequestManager,
	watcher::RequestManager as WatcherRequestManager
};
use crate::config::ConfigManager;


pub struct CoreState {
	#[cfg(feature = "website")]
	pub frontend: FrontendCore,
	#[cfg(feature = "terminal")]
	pub terminal: TerminalCore,
	#[cfg(feature = "telegram")]
	pub telegram: TelegramCore,

	pub connection: Connection,
	pub feed_requests: FeedRequestManager,
	pub watcher_requests: WatcherRequestManager,
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
			feed_requests: FeedRequestManager::new(),
			watcher_requests: WatcherRequestManager::new(),
			config: RwLock::new(ConfigManager::new()),
		}
	}

	#[allow(unused_variables)]
	pub fn init(&mut self, weak_core: WeakFeederCore) {
		{
			let mut write = self.config.write().unwrap();

			write.init();
			write.load().unwrap_or_else(|e| panic!("Loading Config Error: {}", e));
		}

		#[cfg(feature = "website")]
		self.frontend.init(weak_core.clone());
		#[cfg(feature = "terminal")]
		self.terminal.init(weak_core.clone());
		#[cfg(feature = "telegram")]
		self.telegram.init(self.config.read().unwrap().config(), weak_core);

		self.connection.init_sql().unwrap_or_else(|e| panic!("Loading Database Error: {}", e));
	}

	//
	pub fn run_all_requests(&mut self) -> Vec<RequestResults> {
		vec![
			self.feed_requests.request_all_if_idle(
				false,
				self.connection.connection()
			),
			self.watcher_requests.request_all_if_idle(
				false,
				self.connection.connection()
			)
		]
	}

	pub fn get_config(&self) -> Config {
		self.config.read().unwrap().config()
	}
}