use std::io::Read;
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use url::Url;
use log::info;

use crate::state::CoreState;
use crate::request::watcher;
use crate::request::RequestResults;

use crate::Result;
use crate::filter::filter_items;

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

				// Requests aren't enabled? Break out of loop.
				if !inner.get_config().request.enabled {
					info!("Requests disabled.");
					break;
				}

				let reqs = inner.run_all_requests();

				for req in reqs {
					match req {
						RequestResults::Feed(req) => {
							if let Some(e) = req.general_error.as_ref() {
								info!("Feed Request Error: {:?}", e);
							} else {
								let mut encountered_error = false;

								for feed in &req.items {
									if let Err(e) = feed.results.as_ref() {
										info!("Feed \"{}\" Error: {:#?}", feed.item.title, e);
										encountered_error = true;
									}
								}

								if !encountered_error && !req.items.is_empty() {
									info!("Feeds ran without error. Took: {}s", req.duration.as_secs());
								}
							}
						}

						RequestResults::Watcher(req) => {
							if let Some(e) = req.general_error.as_ref() {
								info!("Watcher Request Error: {:?}", e);
							} else {
								let mut encountered_error = false;

								for feed in &req.items {
									if let Err(e) = feed.results.as_ref() {
										info!("Watcher \"{}\" Error: {:#?}", feed.item.title, e);
										encountered_error = true;
									}
								}

								if !encountered_error && !req.items.is_empty() {
									info!("Watchers ran without error. Took: {}s", req.duration.as_secs());
								}
							}
						}
					}
				}
			}

			// Sleep otherwise loop will make the process use lots of cpu power.
			std::thread::sleep(std::time::Duration::from_secs(10));
		}

		let inner = self.to_inner();
		let mut frontend = inner.frontend.to_inner();

		if let Some(handle) = frontend.web.thread_handle.take() {
			std::mem::drop(frontend);
			std::mem::drop(inner);
			handle.join().expect("join");
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
use crate::feature::{objects, models, schema::feeds as FeedsSchema};
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
		let inner = upgrade.to_inner();

		let conn = inner.connection.connection();

		match rpc {
			// Dashboard


			// Feed Variants

			Front2CoreNotification::FeedUpdates { since } => {
				let new_feeds = objects::get_item_count_since(since, conn)?;
				let new_watches = objects::get_watch_history_count_since(since, conn)?;

				let updates = Core2FrontNotification::FeedUpdates {
					since,
					new_feeds,
					new_watches
				};

				ctx.respond_with(msg_id_opt, updates);
			}

			Front2CoreNotification::ItemList { category_id, item_count, skip_count } => {
				let total_items = objects::get_item_total(category_id, conn)?;
				let items = objects::get_items_in_range(category_id, item_count, skip_count, conn)?;

				// ID's of items that should be alerted.
				let notification_ids = filter_items(&items, conn)?.into_iter().map(|i| i.id).collect();

				let list = Core2FrontNotification::ItemList {
					items,

					item_count,
					skip_count,

					total_items,
					notification_ids
				};

				ctx.respond_with(msg_id_opt, list);
			}

			Front2CoreNotification::FeedList(..) => {
				let list = Core2FrontNotification::FeedList {
					items: objects::get_listeners(conn)?
				};

				ctx.respond_with(msg_id_opt, list);
			}

			Front2CoreNotification::CategoryList(..) => {
				let list = Core2FrontNotification::CategoryList {
					categories: objects::get_categories(conn)?,
					category_feeds: objects::get_feed_categories(conn)?
				};

				ctx.respond_with(msg_id_opt, list);
			}


			Front2CoreNotification::AddListener { url, custom_item_id } => {
				use diesel::RunQueryDsl;

				let feed = inner.feed_requests.create_new_feed(url, custom_item_id, conn)?;

				let affected = diesel::insert_into(FeedsSchema::table)
					.values(&feed)
					.execute(conn)?;

				let new_feed = Core2FrontNotification::NewListener {
					affected,
					listener: feed,
				};

				ctx.respond_with(msg_id_opt, new_feed);
			}

			Front2CoreNotification::RemoveListener { id, rem_stored } => {
				let affected = objects::remove_listener(id, rem_stored, conn)?;

				ctx.respond_with(msg_id_opt, Core2FrontNotification::RemoveListener { affected });
			}

			Front2CoreNotification::EditListener { id, editing } => {
				// TODO: Check if changed url. If so; call it and return url it gives us. Will prevent duplicates/redirects.

				let affected = objects::update_listener(id, &editing, conn)?;

				ctx.respond_with(msg_id_opt, Core2FrontNotification::EditListener { affected, listener: editing });
			}


			Front2CoreNotification::AddCategory { name, position } => {
				// TODO: count categories, set position based on that.

				let cat = models::NewCategoryModel {
					name_lowercase: name.to_lowercase(),
					date_added: chrono::Utc::now().timestamp(),
					position,
					name,
				};

				let affected = objects::create_category(&cat, conn)?;

				let new_cat = Core2FrontNotification::NewCategory {
					affected,
					category: cat,
				};

				ctx.respond_with(msg_id_opt, new_cat);
			}

			Front2CoreNotification::RemoveCategory { id } => {
				let affected = objects::remove_category(id, conn)?;

				ctx.respond_with(msg_id_opt, Core2FrontNotification::RemoveCategory { affected });
			}

			Front2CoreNotification::EditCategory { id, editing } => {
				let affected = objects::update_category(id, &editing, conn)?;

				ctx.respond_with(msg_id_opt, Core2FrontNotification::EditCategory { affected, category: editing });
			}


			Front2CoreNotification::AddFeedCategory { feed_id, category_id } => {
				let cat = models::NewFeedCategoryModel {
					feed_id,
					category_id
				};

				let affected = objects::create_category_feed(&cat, conn)?;

				let new_cat = Core2FrontNotification::NewFeedCategory {
					affected,
					category: cat,
				};

				ctx.respond_with(msg_id_opt, new_cat);
			}

			Front2CoreNotification::RemoveFeedCategory { id } => {
				let affected = objects::remove_category_feed(id, conn)?;

				ctx.respond_with(msg_id_opt, Core2FrontNotification::RemoveFeedCategory { affected });
			}


			Front2CoreNotification::GetWebpage { url } => {
				let mut content = String::new();

				let mut resp = reqwest::get(&url)?;
				resp.read_to_string(&mut content)?;

				ctx.respond_with(msg_id_opt, Core2FrontNotification::WebpageSource { html: content });
			}


			Front2CoreNotification::CustomItemList(..) => {
				let items = Core2FrontNotification::CustomItemList {
					items: objects::get_custom_items(conn)?
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

				let affected = objects::create_custom_item(&model, conn)?;

				let new_item = Core2FrontNotification::NewCustomItem {
					affected,
					item,
				};

				ctx.respond_with(msg_id_opt, new_item);
			}


			Front2CoreNotification::FilterList(..) => {
				let mut items = Vec::new();

				for filter in objects::get_filters(conn)? {
					let feeds = objects::get_feed_filters_from_filter_id(filter.id, conn)?
						.into_iter()
						.map(|f| f.feed_id)
						.collect();

					items.push(objects::FilterGrouping {
						filter,
						feeds
					});
				}

				let response = Core2FrontNotification::FeedFilterList {
					items
				};

				ctx.respond_with(msg_id_opt, response);
			}

			Front2CoreNotification::NewFilter { title , filter } => {
				let new_filter = objects::NewFilter {
					title,
					filter
				};

				let affected = objects::create_filter(
					new_filter.clone(),
					conn
				)?;

				ctx.respond_with(
					msg_id_opt,
					Core2FrontNotification::NewFilter {
						filter: new_filter,
						affected
					}
				);
			}

			Front2CoreNotification::UpdateFilter { id, title , filter } => {
				let affected = objects::update_filter(
					id,
					objects::EditFilter {
						title: Some(title),
						filter: Some(filter)
					},
					conn
				)?;

				ctx.respond_with(
					msg_id_opt,
					Core2FrontNotification::EditFilter {
						affected
					}
				);
			}

			Front2CoreNotification::RemoveFilter { id } => {
				let (affected_filters, affected_feeds) = objects::remove_filter(id, conn)?;

				ctx.respond_with(
					msg_id_opt,
					Core2FrontNotification::RemoveFilter {
						affected_filters,
						affected_feeds,
					}
				);
			}


			Front2CoreNotification::NewFeedFilter { feed_id, filter_id } => {
				let affected = objects::create_feed_and_filter_link(filter_id, feed_id, conn)?;

				ctx.respond_with(
					msg_id_opt,
					Core2FrontNotification::LinkFeedAndFilter {
						affected
					}
				);
			}

			Front2CoreNotification::RemoveFeedFilter { feed_id, filter_id } => {
				let affected = objects::remove_feed_and_filter_link(filter_id, feed_id, conn)?;

				ctx.respond_with(
					msg_id_opt,
					Core2FrontNotification::LinkFeedAndFilter {
						affected
					}
				);
			}



			// Watching Variants

			Front2CoreNotification::WatcherList(..) => {
				let watchers = objects::get_watchers(conn)?
					.into_iter()
					.map(|w| {
						let id = w.id;
						(w, objects::get_last_watch_history(id, conn).ok().flatten())
					})
					.collect();

				let list = Core2FrontNotification::WatcherList {
					items: watchers
				};

				ctx.respond_with(msg_id_opt, list);
			}

			Front2CoreNotification::AddWatcher { url, custom_item_id } => {
				let watcher = inner.watcher_requests.verify_new_watcher(url, custom_item_id, conn)?;

				let affected = objects::create_watcher(&watcher, conn)?;

				// Cache first History Item.
				{
					let parser = if let Some(parser_id) = custom_item_id {
						objects::get_watch_parser_by_id(parser_id, conn)?
					} else {
						objects::get_watch_parser_from_url(Url::parse(&watcher.url).unwrap(), conn)?
					};

					let new_watcher = objects::get_watcher_by_url(&watcher.url, conn)?;

					// let new_item = watcher::get_from_url(&new_watcher.url, conn)?;
					let new_items = watcher::get_from_url_parser(&new_watcher.url, &parser.match_opts)?;

					objects::create_last_watch_history(&models::NewWatchHistoryModel {
						watch_id: new_watcher.id,
						items: serde_json::to_string(&new_items).unwrap(),

						date_added: chrono::Utc::now().timestamp()
					}, conn)?;
				}


				let new_feed = Core2FrontNotification::NewWatcher {
					affected,
					listener: watcher,
				};

				ctx.respond_with(msg_id_opt, new_feed);
			}

			Front2CoreNotification::RemoveWatcher { id, rem_stored } => {
				let affected = objects::remove_watcher(id, rem_stored, conn)?;

				ctx.respond_with(msg_id_opt, Core2FrontNotification::RemoveWatcher { affected });
			}

			Front2CoreNotification::EditWatcher { id, editing } => {
				// TODO: Check if changed url. If so; call it and return url it gives us. Will prevent duplicates/redirects.

				let affected = objects::update_watcher(id, &editing, conn)?;

				ctx.respond_with(msg_id_opt, Core2FrontNotification::EditWatcher { affected, listener: editing });
			}

			Front2CoreNotification::WatchParserList(..) => {
				ctx.respond_with(msg_id_opt, Core2FrontNotification::WatchParserList {
					items: objects::get_watch_parsers(conn)?
				});
			}

			Front2CoreNotification::NewWatchParser { item } => {
				let model = item.clone().into();

				let affected = objects::create_watch_parser(&model, conn)?;

				let new_item = Core2FrontNotification::NewWatchParser {
					affected,
					item,
				};

				ctx.respond_with(msg_id_opt, new_item);
			}

			Front2CoreNotification::UpdateWatchParser { id, item } => {
				let model = item.clone().into();

				let affected = objects::update_watch_parser(id, &model, conn)?;

				let new_item = Core2FrontNotification::UpdateWatchParser {
					affected,
					item
				};

				ctx.respond_with(msg_id_opt, new_item);
			}

			Front2CoreNotification::RemoveWatchParser { id } => {
				let affected = objects::delete_watch_parser(id, conn)?;

				let new_item = Core2FrontNotification::RemoveWatchParser {
					affected
				};

				ctx.respond_with(msg_id_opt, new_item);
			}


			// History
			Front2CoreNotification::WatchHistoryList { watch_id, item_count, skip_count } => {
				ctx.respond_with(msg_id_opt, Core2FrontNotification::WatchHistoryList {
					items: objects::get_watch_history_list(watch_id, item_count, skip_count, conn)?
				});
			}

			// Test
			Front2CoreNotification::TestWatcher { url, parser } => {
				if let Some(parser) = parser {
					let items = watcher::get_from_url_parser(&url, &parser)?;

					ctx.respond_with(msg_id_opt, Core2FrontNotification::TestWatcher { success: true, items });
				} else {
					// TODO: Get parser based on url.
					println!("No parser...");
					ctx.respond_with(msg_id_opt, Core2FrontNotification::TestWatcher { success: false, items: Vec::new() });
				}
			}
		}

		Ok(())
	}
}