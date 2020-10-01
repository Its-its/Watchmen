use std::io::Read;
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use log::info;

use crate::state::CoreState;
use crate::request::CollectedResult;

use crate::Result;
// use crate::Filter;

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
				let req = inner.run_all_requests();

				if let Some(e) = req.error.as_ref() {
					info!("Request Error: {:?}", e);
				} else {
					let feed_errors = req.feeds.iter()
						.filter(|f| f.is_err())
						.collect::<Vec<&CollectedResult>>();

					if !feed_errors.is_empty() {
						info!("Feed Errors: {:#?}", feed_errors);
					} else {
						info!("Feeds ran without error. Took: {}s :)", req.duration.as_secs());
					}
				}
			}

			// Sleep otherwise loop will make the process use lots of cpu power.
			std::thread::sleep(std::time::Duration::from_secs(10));
		}
	}

	// Util

	pub fn to_inner(&self) -> MutexGuard<'_, CoreState> {
		self.0.lock().unwrap()
	}

	pub fn to_weak(&self) -> WeakFeederCore {
		WeakFeederCore(Arc::downgrade(&self.0))
	}
}


// Weak Core | sent to the plugins / WebSocket
use crate::feature::{models, schema::feeds as FeedsSchema};
use crate::feature::ResponseWrapper;
use crate::feature::{Core2FrontNotification, Front2CoreNotification};
use crate::types::MessageId;

#[derive(Clone)]
pub struct WeakFeederCore(Weak<Mutex<CoreState>>);

impl WeakFeederCore {
	pub fn upgrade(&self) -> Option<FeederCore> {
		self.0.upgrade().map(FeederCore)
	}

	pub fn handle_response(
		&self,
		ctx: &mut dyn ResponseWrapper,
		msg_id_opt: Option<MessageId>,
		rpc: Front2CoreNotification
	) -> Result<()> {
		let upgrade = self.upgrade().unwrap();
		let mut inner = upgrade.to_inner();

		let conn = inner.connection.connection();

		match rpc {
			// Read only

			// Return updates since time
			Front2CoreNotification::Updates { since } => {
				let new_count = models::get_item_count_since(since, &conn)?;

				let updates = Core2FrontNotification::Updates {
					since,
					new_items: new_count,
					notifications: 0
				};

				ctx.respond_with(msg_id_opt, updates);
			}

			Front2CoreNotification::ItemList { category_id, items, skip } => {
				let total_amount = models::get_item_total(category_id, &conn)?;
				let items_found = models::get_items_in_range(category_id, items, skip, &conn)?;

				// let filter = Filter::Regex("[0-9]+\\s?tb".into(), Default::default());

				let list = Core2FrontNotification::ItemList {
					items: items_found,

					item_count: items,
					skip_count: skip,

					total_items: total_amount
				};

				ctx.respond_with(msg_id_opt, list);
			}

			Front2CoreNotification::FeedList(..) => {
				let list = Core2FrontNotification::FeedList { items: inner.requester.feeds.clone() };

				ctx.respond_with(msg_id_opt, list);
			}

			Front2CoreNotification::CategoryList(..) => {
				let list = Core2FrontNotification::CategoryList {
					categories: models::get_categories(&conn)?,
					category_feeds: models::get_feed_categories(&conn)?
				};

				ctx.respond_with(msg_id_opt, list);
			}


			// Write Only
			Front2CoreNotification::AddListener { url } => {
				use diesel::RunQueryDsl;

				let feed = inner.requester.create_new_feed(url, conn)?;

				let affected = diesel::insert_or_ignore_into(FeedsSchema::table)
					.values(&feed)
					.execute(conn)?;

				let new_feed = Core2FrontNotification::NewListener {
					affected,
					listener: feed,
				};

				ctx.respond_with(msg_id_opt, new_feed);
			}

			Front2CoreNotification::RemoveListener { id, rem_stored } => {
				let affected = models::remove_listener(id, rem_stored, &mut inner)?;

				ctx.respond_with(msg_id_opt, Core2FrontNotification::RemoveListener { affected });
			}

			Front2CoreNotification::EditListener { id, editing } => {
				let affected = models::update_listener(id, &editing, &mut inner)?;

				ctx.respond_with(msg_id_opt, Core2FrontNotification::EditListener { affected, listener: editing });
			}


			Front2CoreNotification::AddCategory { name, position } => {
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

			Front2CoreNotification::RemoveCategory { id } => {
				let affected = models::remove_category(id, conn)?;

				ctx.respond_with(msg_id_opt, Core2FrontNotification::RemoveCategory { affected });
			}

			Front2CoreNotification::EditCategory { id, editing } => {
				let affected = models::update_category(id, &editing, conn)?;

				ctx.respond_with(msg_id_opt, Core2FrontNotification::EditCategory { affected, category: editing });
			}


			Front2CoreNotification::AddFeedCategory { feed_id, category_id } => {
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

			Front2CoreNotification::RemoveFeedCategory { id } => {
				let affected = models::remove_category_feed(id, conn)?;

				ctx.respond_with(msg_id_opt, Core2FrontNotification::RemoveFeedCategory { affected });
			}


			// Scaper Editor

			Front2CoreNotification::GetWebpage { url } => {
				let mut content = String::new();

				let mut resp = reqwest::get(&url)?;
				resp.read_to_string(&mut content)?;

				ctx.respond_with(msg_id_opt, Core2FrontNotification::WebpageSource { html: content });
			}


			// Custom Item

			Front2CoreNotification::CustomItemList(..) => {
				let items = Core2FrontNotification::CustomItemList {
					items: models::get_custom_items(conn)?
				};

				ctx.respond_with(msg_id_opt, items);
			}

			Front2CoreNotification::UpdateCustomItem { id, item } => {
				println!("UpdateCustomItem: {:?}", id);
				println!("{:#?}", item);
			}

			Front2CoreNotification::NewCustomItem { item } => {
				println!("NewCustomItem");
				println!("{:#?}", item);

				let model = item.clone().into();

				let affected = models::create_custom_item(&model, conn)?;

				let new_item = Core2FrontNotification::NewCustomItem {
					affected,
					item,
				};

				ctx.respond_with(msg_id_opt, new_item);
			}

			//
		}

		Ok(())
	}
}