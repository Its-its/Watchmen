use std::time::Duration;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::thread;

use futures::StreamExt;
use tokio::{runtime::Runtime};

use telegram_bot::*;

use crate::feature::database::objects;
use crate::core::WeakFeederCore;
use crate::filter::filter_items;

pub struct TelegramCore(Arc<Mutex<TelegramState>>);

impl TelegramCore {
	pub fn new() -> Self {
		Self(Arc::new(Mutex::new(TelegramState::new())))
	}


	pub fn init(&mut self, weak_core: WeakFeederCore) {
		let weak = self.to_weak();

		let mut inner = self.to_inner();

		inner.init(weak_core, weak);
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
	api: Api,
	chat_ref: Option<ChatRef>,
	last_grabbed: Arc<Mutex<Option<i64>>>
}

impl TelegramState {
	pub fn new() -> Self {
		TelegramState {
			api: Api::new("777709278:AAEaI-XSvZX5RMpGuZOqILsxij5lXPFMjUs"),
			chat_ref: Some(ChatRef::from_chat_id(296604566.into())),
			last_grabbed: Arc::new(Mutex::new(None)),
		}
	}

	pub fn init(&mut self, weak_core: WeakFeederCore, _weak_telegram: WeakTelegramCore) {
		let chat_ref = self.chat_ref.clone();
		let api = self.api.clone();

		thread::spawn(move || {
			let mut rt = Runtime::new().expect("runtime");

			rt.block_on(async {
				let mut stream = api.stream();
				while let Some(update) = stream.next().await {
					// If the received update contains a new message...
					if let UpdateKind::Message(message) = update.unwrap().kind {
						if let MessageKind::Text { ref data, .. } = message.kind {
							println!("[{}]: <{}>: {}", message.chat.id(), &message.from.first_name, data);

							let send = api.send(
								SendMessage::new(
									chat_ref.as_ref().unwrap(),
									format!("Received:\n{}", data)
								)
							).await;

							if let Err(e) = send {
								eprintln!("{:?}", e);
							}
						}
					}
				}
			});
		});


		let last_grabbed = self.last_grabbed.clone();
		let chat_ref = self.chat_ref.clone();
		let api = self.api.clone();

		thread::spawn(move || {
			{
				let core = weak_core.upgrade().unwrap();
				let inner = core.to_inner();

				let conn = inner.connection.connection();

				if let Ok(items) = objects::get_items_in_range(None, 1, 0, &conn) {
					*last_grabbed.lock().unwrap() = Some(items[0].date);
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
						if let Ok(count) = objects::get_item_count_since(*since.as_ref().unwrap(), &conn) {
							if count != 0 {
								if let Ok(items) = objects::get_items_in_range(None, count, 0, &conn) {
									{ // Update last grabbed (ensuring newest first)
										let mut newest_time: Vec<i64> = items.iter().map(|i| i.date).collect();
										newest_time.sort_by(|a, b| b.partial_cmp(a).unwrap());

										*since = Some(newest_time[0]);
									}

									let filtered = match filter_items(&items, conn) {
										Ok(v) => v,
										Err(e) => {
											eprintln!("{}", e);
											continue;
										}
									};

									if !filtered.is_empty() {
										rt.block_on(async {
											for item in filtered {
												let send = api.send(
													SendMessage::new(
														chat_ref.as_ref().unwrap(),
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
				}

				thread::sleep(Duration::from_secs(30));
			}
		});
	}
}