use std::time::{Duration, Instant};

use log::{info};
use diesel::{RunQueryDsl, SqliteConnection};

use crate::error::{Error, Result};
use crate::feature::schema::{items as ItemsSchema, feeds as FeedsSchema};
use crate::feature::models::{QueryId, NewFeedItemModel, FeedModel, NewFeedModel};
use super::{RequestResults, ItemResults, RequestItemResults, InnerRequestResults};

pub mod rss;
pub mod atom;
pub mod custom;

type CollectedResult = Result<RequestItemResults<NewFeedItemModel>>;


pub enum FeedType {
	Rss(rss::FeedResult),
	Atom(atom::FeedResult),
	Custom(custom::CustomResult),

	__Unknown
}

impl FeedType {
	pub fn from_url(url: &str, conn: &SqliteConnection) -> FeedType {
		// RSS
		match rss::get_from_url(url) {
			Ok(c) => return FeedType::Rss(Ok(c)),

			Err(e) => {
				info!("rss: {:?}", e);
				if let Error::Rss(e) = e {
					use ::rss::Error::InvalidStartTag;

					if let InvalidStartTag = e {
						// TODO: Fix this so it's not an if else
					} else {
						return FeedType::Rss(Err(Error::Rss(e)));
					}
				} else {
					return FeedType::Rss(Err(e));
				}
			}
		}

		// ATOM
		match atom::get_from_url(url) {
			Ok(c) => return FeedType::Atom(Ok(c)),

			Err(e) => {
				info!("atom: {:?}", e);
				if let Error::Atom(e) = e {
					use atom_syndication::Error::InvalidStartTag;

					if let InvalidStartTag = e {
						//
					} else {
						return FeedType::Atom(Err(Error::Atom(e)));
					}
				} else {
					return FeedType::Atom(Err(e));
				}
			}
		}

		// CUSTOM
		match custom::get_from_url(url, conn) {
			Ok(i) => FeedType::Custom(Ok(i)),

			Err(e) => {
				info!("custom: {:?}", e);
				FeedType::Custom(Err(e))
			}
		}
	}

	pub fn req_from_feed_type(feed_type: i32, url: &str, conn: &SqliteConnection) -> FeedType {
		match feed_type {
			0 => FeedType::Rss(rss::get_from_url(url)),
			1 => FeedType::Atom(atom::get_from_url(url)),
			2 => FeedType::Custom(custom::get_from_url(url, conn)),
			_ => FeedType::__Unknown
		}
	}
}


pub struct RequestManager {
	pub feeds: Vec<FeedModel>,
	pub is_idle: bool,
	pub concurrency: i32,
}

impl RequestManager {
	pub fn new() -> Self {
		Self {
			feeds: Vec::new(),

			is_idle: true,
			concurrency: 2,
		}
	}

	pub fn init(&mut self, connection: &SqliteConnection) -> Result<usize> {
		use diesel::prelude::*;
		use FeedsSchema::dsl::*;

		let found = feeds.filter(id.ne(0)).load::<FeedModel>(connection)?;

		self.feeds = found;

		Ok(self.feeds.len())
	}

	pub fn create_new_feed(&self, url: String, custom_item_id: Option<QueryId>, conn: &SqliteConnection) -> Result<NewFeedModel> {
		Ok(match FeedType::from_url(&url, conn) {
			FeedType::Rss(Ok(feed)) => rss::new_from_feed(url, feed),
			FeedType::Atom(Ok(feed)) => atom::new_from_feed(url, feed),
			FeedType::Custom(Ok(_)) => custom::new_from_url(url, custom_item_id, conn)?,

			FeedType::Custom(Err(e))
			| FeedType::Atom(Err(e))
			| FeedType::Rss(Err(e)) => return Err(e),

			_ => return Err("Unknown Feed.. It didn't match the current supported ones.".into())
		})
	}

	pub fn request_all_if_idle(&mut self, is_manual: bool, connection: &SqliteConnection) -> RequestResults {
		let mut results = InnerRequestResults {
			general_error: None,
			start_time: Instant::now(),
			duration: Duration::new(0, 0),
			was_manual: is_manual,
			concurrency: 0,
			items: Vec::new()
		};


		let feeds: Vec<&mut FeedModel> = {
			let timestamp = chrono::Utc::now().timestamp();

			self.feeds.iter_mut()
			.filter(|i: &&mut FeedModel| timestamp - i.last_called - i.sec_interval as i64 > 0)
			.collect()
		};


		if !self.is_idle {
			results.general_error = Some("Request Manager is already running!".into());
			return RequestResults::Feed(results);
		}

		if feeds.is_empty() {
			// results.error = Some("No feeds to run...".into());
			return RequestResults::Feed(results);
		}

		self.is_idle = false;

		info!("Starting Requests.. Found: {}", feeds.len());


		for feed in &feeds {
			let cloned_feed = (*feed).clone();

			results.items.push(ItemResults {
				results: request_feed(&cloned_feed, connection),
				item: cloned_feed
			});
		}


		// Database
		{
			// Update Last Called
			let new_timestamp = chrono::Utc::now().timestamp();
			update_feed_last_called_db(new_timestamp, feeds, connection);

			// After finished insert new items to DB.
			for res in results.items.iter_mut() {
				if let Ok(res) = res.results.as_mut() {
					let e = diesel::insert_or_ignore_into(ItemsSchema::table)
						.values(&res.to_insert)
						.execute(connection);

					if let Ok(count) = e {
						res.new_item_count = count;
					}
				}
			}
		}

		results.duration = results.start_time.elapsed();

		self.is_idle = true;

		RequestResults::Feed(results)
	}
}


pub fn request_feed(feed: &FeedModel, conn: &SqliteConnection) -> CollectedResult {
	info!(" - Requesting: {}", feed.url);

	let mut feed_res = RequestItemResults {
		start_time: Instant::now(),
		duration: Duration::new(0, 0),
		new_item_count: 0,
		item_count: 0,
		to_insert: Vec::new()
	};

	match FeedType::req_from_feed_type(feed.feed_type, &feed.url, conn) {
		FeedType::Rss(Ok(channel)) => {
			feed_res.to_insert = channel.items()
			.iter()
			.map(|i| {
				let mut item: NewFeedItemModel = i.into();
				item.feed_id = feed.id;
				item
			})
			.collect();
		}

		FeedType::Atom(Ok(atom_feed)) => {
			feed_res.to_insert = atom_feed.entries()
			.iter()
			.map(|i| {
				let mut item: NewFeedItemModel = i.into();
				item.feed_id = feed.id;
				item
			})
			.collect();
		}

		FeedType::Custom(Ok(custom_feed_items)) => {
			feed_res.to_insert = custom_feed_items
			.into_iter()
			.map(|i| {
				let mut item: NewFeedItemModel = i.into();
				item.feed_id = feed.id;
				item
			})
			.collect();
		}

		FeedType::Atom(Err(e))
		// | FeedType::Custom(Err(e))
		| FeedType::Rss(Err(e)) => return Err(e),

		_ => return Err("Unknown Feed.. It didn't match the current supported ones.".into())
	};

	feed_res.duration = feed_res.start_time.elapsed();

	Ok(feed_res)
}


pub fn update_feed_last_called_db(set_last_called: i64, feeds_arr: Vec<&mut FeedModel>, connection: &SqliteConnection) {
	use diesel::prelude::*;
	use FeedsSchema::dsl::*;

	for feed in feeds_arr {
		let _ = diesel::update(&*feed)
			.set(last_called.eq(set_last_called))
			.execute(connection);

		feed.last_called = set_last_called;
	}
}