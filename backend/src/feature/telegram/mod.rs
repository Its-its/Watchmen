use std::time::Duration;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::thread;

use futures::StreamExt;
use tokio::{runtime::Runtime};

use telegram_bot::*;

use crate::{config::Config, feature::database::objects};
use crate::core::WeakFeederCore;
use crate::filter::filter_items;

pub struct TelegramCore(Arc<Mutex<TelegramState>>);

impl TelegramCore {
	pub fn new() -> Self {
		Self(Arc::new(Mutex::new(TelegramState::new())))
	}


	pub fn init(&mut self, config: Config, weak_core: WeakFeederCore) {
		let weak = self.to_weak();

		let mut inner = self.to_inner();

		inner.init(config, weak_core, weak);
	}


	pub fn to_inner(&self) -> MutexGuard<'_, TelegramState> {
		self.0.lock().unwrap()
	}

	pub fn to_weak(&self) -> WeakTelegramCore {
		WeakTelegramCore(Arc::downgrade(&self.0))
	}
}


#[derive(Clone)]
pub struct WeakTelegramCore(Weak<Mutex<TelegramState>>);

impl WeakTelegramCore {
	pub fn upgrade(&self) -> Option<TelegramCore> {
		self.0.upgrade().map(TelegramCore)
	}
}


pub struct TelegramState {
	api: Option<Api>,
	chat_ref: Option<ChatRef>,
	last_grabbed: Arc<Mutex<Option<i64>>>
}

impl TelegramState {
	pub fn new() -> Self { // TODO: Messed up.
		TelegramState {
			api: None,
			chat_ref: None,
			last_grabbed: Arc::new(Mutex::new(None)),
		}
	}

	pub fn init(&mut self, config: Config, weak_core: WeakFeederCore, _weak_telegram: WeakTelegramCore) {
		self.api = Some(Api::new(config.telegram.api_key));
		self.chat_ref = config.telegram.chat_id.map(|v| ChatRef::from_chat_id(v.into()));

		let chat_ref = self.chat_ref.clone();
		let api = self.api.clone().unwrap();


		thread::spawn(move || {
			let mut rt = Runtime::new().expect("runtime");

			rt.block_on(async {
				let mut stream = api.stream();
				while let Some(update) = stream.next().await {
					// If the received update contains a new message...
					if let UpdateKind::Message(message) = update.unwrap().kind {
						if let MessageKind::Text { ref data, .. } = message.kind {
							println!("[{}]: <{}>: {}", message.chat.id(), &message.from.first_name, data);

							if let Some(chat) = chat_ref.as_ref() {
								let send = api.send(
									SendMessage::new(
										chat,
										format!("Received:\n{}", data)
									)
								).await;

								if let Err(e) = send {
									eprintln!("{:?}", e);
								}
							} else {
								println!(r#"TELEGRAM NOTIFICATIONS DISABLED!\nPLEASE PUT THE CHAT ID (inside brackets) IN THE CONFIG FILE AFTER api_key as "chat_id" = XXXX"#);
							}
						}
					}
				}
			});
		});


		let last_grabbed = self.last_grabbed.clone();
		let chat_ref = self.chat_ref.clone();
		let api = self.api.clone().unwrap();

		thread::spawn(move || {
			{
				let core = weak_core.upgrade().unwrap();
				let inner = core.to_inner();

				let conn = inner.connection.connection();

				if let Ok(items) = objects::get_items_in_range(None, None, 1, 0, conn) {
					if !items.is_empty() {
						*last_grabbed.lock().unwrap() = Some(items[0].date);
					}
				}

				if let Ok(items) = objects::get_watch_history_list(None, 1, 0, conn) {
					if !items.is_empty() {
						let mut last = last_grabbed.lock().unwrap();

						if items[0].date_added > *last.as_ref().unwrap() {
							*last = Some(items[0].date_added);
						}
					}
				}
			}

			let mut rt = Runtime::new().expect("runtime");

			loop {
				{
					let core = weak_core.upgrade().unwrap();
					let inner = core.to_inner();

					let conn = inner.connection.connection();

					let mut since = last_grabbed.lock().unwrap();

					if since.is_some() {
						let mut last_ran = *since.as_ref().unwrap();

						// Feed Items
						if let Ok(count) = objects::get_item_count_since(last_ran, conn) {
							if count != 0 {
								if let Ok(items) = objects::get_items_in_range(None, None, count, 0, conn) {
									{ // Update last grabbed (ensuring newest first)
										last_ran = newest_time(items.iter().map(|i| i.date));
									}

									let filtered = match filter_items(&items, conn) {
										Ok(v) => v,
										Err(e) => {
											eprintln!("{}", e);
											continue;
										}
									};

									if !filtered.is_empty() {
										if let Some(chat) = chat_ref.as_ref() {
											rt.block_on(async {
												for item in filtered {
													let send = api.send(
														SendMessage::new(
															chat,
															format!(
																"{}\n{}",
																item.title,
																item.link
															)
														)
													).await;

													if let Err(e) = send {
														eprintln!("{:?}", e);
													}
												}
											});
										}
									}
								}
							}
						}

						// Watching Items
						if let Ok(count) = objects::get_watch_history_count_since(last_ran, conn) {
							if count != 0 {
								if let Ok(items) = objects::get_watch_history_list(None, count, 0, conn) {
									{ // Update last grabbed (ensuring newest first)
										let time = newest_time(items.iter().map(|i| i.date_added));

										if last_ran < time {
											last_ran = time;
										}
									}

									if !items.is_empty() {
										if let Some(chat) = chat_ref.as_ref() {
											rt.block_on(async {
												for item in items {
													let watcher = objects::get_watcher_by_id(item.watch_id, conn).unwrap();
													let send = api.send(
														SendMessage::new(
															chat,
															format!(
																"{}\n{}\n{}",
																watcher.title,
																if item.items.len() == 1 {
																	item.items.first()
																	.map(|i| i.value.clone())
																	.unwrap_or_default()
																} else {
																	format!("{} items", item.items.len())
																},
																watcher.url
															)
														)
													).await;

													if let Err(e) = send {
														eprintln!("{:?}", e);
													}
												}
											});
										}
									}
								}
							}
						}

						*since = Some(last_ran);
					}
				}

				thread::sleep(Duration::from_secs(10));
			}
		});
	}
}

fn newest_time<I: Iterator<Item = i64>>(iter: I) -> i64 {
	let mut items: Vec<_> = iter.collect();

	items.sort_by(|a, b| b.partial_cmp(a).unwrap());

	items[0]
}