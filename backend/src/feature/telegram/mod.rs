use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::thread;

use tokio::sync::mpsc::{self, Receiver};
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
		if !self.1.is_closed() {
			if let Err(e) = self.1.send(value).await {
				log::error!("{}", e);
			}
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
		let receiver = self.receiver.take().unwrap();

		if config.telegram.enabled {
			let bot = Bot::new(config.telegram.api_key.clone());

			if config.telegram.chat_id.is_none() || config.telegram.chat_id == Some(0) {
				start_listener(bot, config, weak_core);
			} else {
				start_output(bot, config, receiver, weak_core);
			}
		}
	}
}

fn start_listener(bot: Bot, config: Config, weak_core: WeakFeederCore) {
	for _ in 0..5 {
		log::info!(r#"TELEGRAM NOTIFICATIONS DISABLED!\nPLEASE SEND A MESSAGE TO YOUR BOT!"#);
	}

	thread::spawn(move || {
		let rt = Runtime::new().expect("runtime");

		rt.block_on(async move {
			teloxide::repl(bot, move |cx| {
				{
					let mut config = config.clone();

					let core = weak_core.upgrade().unwrap();
					let inner = core.to_inner();

					config.telegram.chat_id = Some(cx.chat_id());

					let mut cfg = inner.config.write().unwrap();
					cfg.set_config(config);
					let _ = cfg.save();
				}

				for _ in 0..5 {
					log::info!(r#"Bot Received Your Message! Please restart your client!"#);
				}

				log::info!(r#"(If you did not do this please set telegram chat_id to 0 in the config and try messaging again. A bad actor messages your bot.)"#);

				async move {
					let _ = cx.requester.close().send().await;
					respond(())
				}
			}).await;
		});
	});
}

fn start_output(bot: Bot, config: Config, mut receiver: Receiver<RequestResponse>, weak_core: WeakFeederCore) {
	thread::spawn(move || {
		let rt = Runtime::new().expect("runtime");

		rt.block_on(async move {
			let chat_id = config.telegram.chat_id.unwrap();

			// Backend Listener
			while let Some(resp) = receiver.recv().await {
				let core = weak_core.upgrade().unwrap();
				let inner = core.to_inner();

				let conn = inner.connection.connection();

				let started_at = resp.start_time.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;

				for item in resp.results {
					match item {
						RequestResults::Feed(v) => {
							let mut new_item_count = 0;

							// Get New items count.
							v.items.into_iter()
								.filter_map(|v| v.results.ok())
								.for_each(|v| new_item_count += v.new_item_count);

							let feed_filters = objects::get_feed_filters(conn).unwrap();
							let filter_models = objects::get_filters(conn).unwrap();

							for item in objects::get_items_in_range(None, None, new_item_count as i64, 0, conn).unwrap() {
								if item.date_added > started_at && filter::filter_item(&item, &filter_models, &feed_filters) {
									let send = bot.send_message(
										chat_id,
										format!(
											"{}\n{}",
											item.title,
											item.link
										)
									).send().await;

									if let Err(e) = send {
										log::error!("{:?}", e);
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
								if item.date_added > started_at {
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
									).send().await;

									if let Err(e) = send {
										log::error!("{:?}", e);
									}
								}
							}
						}
					}
				}
			}
		});

		log::info!("Stopped running telegram thread.");
	});
}