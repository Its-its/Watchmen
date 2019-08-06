use std::sync::{Arc, Mutex, MutexGuard, Weak};


use crate::error::Result;
use crate::database::models;
use crate::types::MessageId;
use crate::state::CoreState;
use crate::frontend::socket::WebsocketWrapper;
use crate::frontend::rpc::{Front2CoreNotification, Core2FrontNotification};

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
			{
				let mut inner = self.to_inner();
				let req = inner.run_request();

				if let Some(e) = req.error.as_ref() {
					println!("Request Error: {:?}", e);
				} else {
					let feed_errors = req.feeds.iter().filter(|f| f.is_err()).collect::<Vec<_>>();

					if !feed_errors.is_empty() {
						println!("Feed Errors: {:#?}", feed_errors);
					} else {
						println!("Feeds ran without error. Took: {}s :)", req.duration.as_secs());
					}
				}

				// TODO: Check to see if there are new items.
				// Go through filter, then notify.
			}

			// Sleep otherwise loop will make the process use lots of cpu power.
			std::thread::sleep(std::time::Duration::from_secs(30));
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
		msg_id_opt: Option<MessageId>,
		rpc: Front2CoreNotification
	) -> Result<()> {
		use Front2CoreNotification::*;

		let upgrade = self.upgrade().unwrap();
		let inner = upgrade.to_inner();

		let conn = inner.connection.connection();

		match rpc {
			// Read only

			Updates { since } => {
				let new_count = models::get_item_count_since(since, &conn)?;

				let updates = Core2FrontNotification::Updates {
					since: since,
					new_items: new_count
				};

				ctx.respond_with(msg_id_opt, updates);
			}

			ItemList { category_id, items, skip } => {
				let total_amount = models::get_item_total(category_id, &conn)?;
				let items_found = models::get_items_in_range(category_id, items, skip, &conn)?;

				let list = Core2FrontNotification::ItemList {
					items: items_found,

					item_count: items,
					skip_count: skip,

					total_items: total_amount
				};

				ctx.respond_with(msg_id_opt, list);
			}

			FeedList(..) => {
				let list = Core2FrontNotification::FeedList { items: inner.requester.feeds.clone() };

				ctx.respond_with(msg_id_opt, list);
			}

			CategoryList(..) => {
				let list = Core2FrontNotification::CategoryList {
					categories: models::get_categories(&conn)?,
					category_feeds: models::get_feed_categories(&conn)?
				};

				ctx.respond_with(msg_id_opt, list);
			}

			// Write Only
			AddListener { url } => {
				let feed = inner.requester.add_feed_url(url, conn)?;

				let new_feed = Core2FrontNotification::NewListener {
					affected: feed.1,
					listener: feed.0,
				};

				ctx.respond_with(msg_id_opt, new_feed);
			}

			RemoveListener { id, rem_stored } => {
				let affected = models::remove_listener(id, rem_stored, conn)?;

				ctx.respond_with(msg_id_opt, Core2FrontNotification::RemoveListener { affected });
			}

			AddCategory { name, position } => {
				// TODO: count categories, set position based on that.

				let cat = models::NewCategory {
					name_lowercase: name.to_lowercase(),
					date_added: chrono::Utc::now().timestamp(),
					position,
					name,
				};

				let affected = models::create_category(&cat, conn)?;

				let new_cat = Core2FrontNotification::NewCategory {
					affected,
					category: cat,
				};

				ctx.respond_with(msg_id_opt, new_cat);
			}

			AddFeedCategory { feed_id, category_id } => {
				let cat = models::NewFeedCategory {
					feed_id,
					category_id
				};

				let affected = models::create_category_feed(&cat, conn)?;

				let new_cat = Core2FrontNotification::NewFeedCategory {
					affected,
					category: cat,
				};

				ctx.respond_with(msg_id_opt, new_cat);
			}

			RemoveFeedCategory { id } => {
				let affected = models::remove_category_feed(id, conn)?;

				ctx.respond_with(msg_id_opt, Core2FrontNotification::RemoveFeedCategory { affected });
			}

			EditListener { id, editing } => {
				let affected = models::update_listener(id, &editing, &conn)?;

				ctx.respond_with(msg_id_opt, Core2FrontNotification::EditListener { affected, listener: editing });
			}

			_ => ()
		}

		Ok(())
	}
}