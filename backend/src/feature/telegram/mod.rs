use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::thread;

use tokio::sync::mpsc;
use tokio::{runtime::Runtime};

use teloxide::prelude::*;

use crate::request::RequestResults;
use crate::request::watcher::FoundItem;
use crate::state::RequestResponse;
use crate::{config::Config, feature::database::objects};
use crate::core::WeakFeederCore;
use crate::filter;

pub struct TelegramCore(Arc<Mutex<TelegramState>>, mpsc::Sender<RequestResponse>);

impl TelegramCore {
	pub fn new() -> Self {
		let (tx, rx) = mpsc::channel::<RequestResponse>(100);
		Self(Arc::new(Mutex::new(TelegramState::new(rx))), tx)
	}


	pub fn init(&mut self, config: Config, weak_core: WeakFeederCore) {
		let weak = self.to_weak();

		let mut inner = self.to_inner();

		inner.init(config, weak_core, weak);
	}


	pub async fn send(&self, value: RequestResponse) {
		if let Err(e) = self.1.send(value).await {
			log::error!("{}", e);
		}
	}


	pub fn to_inner(&self) -> MutexGuard<'_, TelegramState> {
		self.0.lock().unwrap()
	}

	pub fn to_weak(&self) -> WeakTelegramCore {
		WeakTelegramCore(Arc::downgrade(&self.0), self.1.clone())
	}
}


#[derive(Clone)]
pub struct WeakTelegramCore(Weak<Mutex<TelegramState>>, mpsc::Sender<RequestResponse>);

impl WeakTelegramCore {
	pub fn upgrade(&self) -> Option<TelegramCore> {
		self.0.upgrade().map(|v| TelegramCore(v, self.1.clone()))
	}
}


pub struct TelegramState {
	receiver: Option<mpsc::Receiver<RequestResponse>>
}

impl TelegramState {
	pub fn new(receiver: mpsc::Receiver<RequestResponse>) -> Self {
		TelegramState {
			receiver: Some(receiver)
		}
	}

	pub fn init(&mut self, config: Config, weak_core: WeakFeederCore, _weak_telegram: WeakTelegramCore) {
		if config.telegram.chat_id.is_none() {
			for _ in 0..5 {
				println!(r#"TELEGRAM NOTIFICATIONS DISABLED!\nPLEASE PUT THE CHAT ID IN THE CONFIG FILE AFTER api_key as "chat_id" = XXXX"#);
			}

			return;
		}

		let mut receiver = self.receiver.take().unwrap();

		thread::spawn(move || {
			let rt = Runtime::new().expect("runtime");

			rt.block_on(async move {
				let chat_id = config.telegram.chat_id.unwrap();

				// Telegram API Listener
				let bot = Bot::new(config.telegram.api_key).auto_send();

				// Backend Listener
				while let Some(resp) = receiver.recv().await {
					let core = weak_core.upgrade().unwrap();
					let inner = core.to_inner();

					let conn = inner.connection.connection();

					for item in resp.results {
						match item {
							RequestResults::Feed(v) => {
								let items = v.items.into_iter()
									.filter_map(|v| v.results.ok())
									.map(|v| v.to_insert)
									.flatten();

								let feed_filters = objects::get_feed_filters(conn).unwrap();
								let filter_models = objects::get_filters(conn).unwrap();

								for item in items {
									// Filter
									if filter::filter_item(&item, &filter_models, &feed_filters, conn) {
										let send = bot.send_message(
											chat_id,
											format!(
												"{}\n{}",
												item.title,
												item.link
											)
										).await;

										if let Err(e) = send {
											eprintln!("{:?}", e);
										}
									}
								}
							}

							RequestResults::Watcher(v) => {
								let items = v.items.into_iter()
									.filter_map(|v| v.results.ok())
									.map(|v| v.to_insert)
									.flatten();

								for item in items {
									let watcher = objects::get_watcher_by_id(item.watch_id, conn).unwrap();

									let watcher_items: Vec<FoundItem> = serde_json::from_str(&item.items).unwrap();

									let send = bot.send_message(
										chat_id,
										format!(
											"{}\n{}\n{}",
											watcher.title,
											if watcher_items.len() == 1 {
												watcher_items.first()
												.map(|i| i.value.clone())
												.unwrap_or_default()
											} else {
												format!("{} items", watcher_items.len())
											},
											watcher.url
										)
									).await;

									if let Err(e) = send {
										eprintln!("{:?}", e);
									}
								}
							}
						}
					}
				}
			});

			println!("Stopped running telegram thread.");
		});
	}
}