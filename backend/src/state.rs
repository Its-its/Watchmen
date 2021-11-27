use std::sync::RwLock;
use std::time::{Duration, Instant};



use reqwest::Client;

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
	pub async fn run_all_requests(&mut self, is_manual: bool) -> RequestResponse {
		let now = Instant::now();

		// Reqwest Client
		let req_client = Client::builder()
			.timeout(Duration::from_secs(10))
			.connection_verbose(true)
			.build()
			.unwrap();

		let results = vec![
			self.feed_requests.request_all_if_idle(
				is_manual,
				&req_client,
				self.connection.connection()
			).await,
			self.watcher_requests.request_all_if_idle(
				is_manual,
				&req_client,
				self.connection.connection()
			).await
		];

		RequestResponse {
			start_time: now,
			duration: now.elapsed(),
			concurrency: 1,
			is_manual,
			results,
		}
	}

	pub fn get_config(&self) -> Config {
		self.config.read().unwrap().config()
	}
}


pub struct RequestResponse {
	pub start_time: Instant,
	pub duration: Duration,
	pub concurrency: usize,
	pub is_manual: bool,
	pub results: Vec<RequestResults>
}