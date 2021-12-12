use std::time::{Duration, SystemTime};

use diesel::{RunQueryDsl, SqliteConnection};
use reqwest::Client;

use crate::error::{Error, Result};
use crate::feature::objects::get_listeners;
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
	pub async fn from_url(url: &str, req_client: &Client, conn: &SqliteConnection) -> FeedType {
		// RSS
		match rss::get_from_url(url, req_client).await {
			Ok(c) => return FeedType::Rss(Ok(c)),

			Err(e) => {
				log::error!("rss: {:?}", e);
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
		match atom::get_from_url(url, req_client).await {
			Ok(c) => return FeedType::Atom(Ok(c)),

			Err(e) => {
				log::error!("atom: {:?}", e);
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
		match custom::get_from_url(url, req_client, conn).await {
			Ok(i) => FeedType::Custom(Ok(i)),

			Err(e) => {
				log::error!("custom: {:?}", e);
				FeedType::Custom(Err(e))
			}
		}
	}

	pub async fn req_from_feed_type(feed_type: i32, url: &str, req_client: &Client, conn: &SqliteConnection) -> FeedType {
		match feed_type {
			0 => FeedType::Rss(rss::get_from_url(url, req_client).await),
			1 => FeedType::Atom(atom::get_from_url(url, req_client).await),
			2 => FeedType::Custom(custom::get_from_url(url, req_client, conn).await),
			_ => FeedType::__Unknown
		}
	}
}


pub struct RequestManager {
	pub is_idle: bool,
	pub concurrency: i32,
}

impl RequestManager {
	pub fn new() -> Self {
		Self {
			is_idle: true,
			concurrency: 2,
		}
	}

	pub async fn create_new_feed(&self, url: String, custom_item_id: Option<QueryId>, req_client: &Client, conn: &SqliteConnection) -> Result<NewFeedModel> {
		Ok(match FeedType::from_url(&url, req_client, conn).await {
			FeedType::Rss(Ok(feed)) => rss::new_from_feed(url, feed),
			FeedType::Atom(Ok(feed)) => atom::new_from_feed(url, feed),
			FeedType::Custom(Ok(_)) => custom::new_from_url(url, custom_item_id, conn)?,

			FeedType::Custom(Err(e))
			| FeedType::Atom(Err(e))
			| FeedType::Rss(Err(e)) => return Err(e),

			_ => return Err("Unknown Feed.. It didn't match the current supported ones.".into())
		})
	}

	pub async fn request_all_if_idle(&mut self, is_manual: bool, req_client: &Client, connection: &SqliteConnection) -> RequestResults {
		let mut results = InnerRequestResults {
			general_error: None,
			start_time: SystemTime::now(),
			duration: Duration::new(0, 0),
			was_manual: is_manual,
			concurrency: 0,
			items: Vec::new()
		};

		let timestamp = chrono::Utc::now().timestamp();

		let feeds: Vec<_> = get_listeners(connection)
			.unwrap()
			.into_iter()
			.filter(|i| i.enabled && timestamp - i.last_called - i.sec_interval as i64 > 0)
			.collect();


		if !self.is_idle {
			results.general_error = Some("Request Manager is already running!".into());
			return RequestResults::Feed(results);
		}

		if feeds.is_empty() {
			// results.error = Some("No feeds to run...".into());
			return RequestResults::Feed(results);
		}

		self.is_idle = false;

		log::debug!("Starting Requests.. Found: {}", feeds.len());


		for feed in &feeds {
			let cloned_feed = (*feed).clone();

			results.items.push(ItemResults {
				results: request_feed(&cloned_feed, req_client, connection).await,
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

		results.duration = results.start_time.elapsed().unwrap();

		self.is_idle = true;

		RequestResults::Feed(results)
	}
}


pub async fn request_feed(feed: &FeedModel, req_client: &Client, conn: &SqliteConnection) -> CollectedResult {
	log::debug!(" - Requesting: {}", feed.url);

	let mut feed_res = RequestItemResults {
		start_time: SystemTime::now(),
		duration: Duration::new(0, 0),
		new_item_count: 0,
		item_count: 0,
		to_insert: Vec::new()
	};

	match FeedType::req_from_feed_type(feed.feed_type, &feed.url, req_client, conn).await {
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

	feed_res.duration = feed_res.start_time.elapsed().unwrap();

	Ok(feed_res)
}


pub fn update_feed_last_called_db(set_last_called: i64, feeds_arr: Vec<FeedModel>, connection: &SqliteConnection) {
	use diesel::prelude::*;
	use FeedsSchema::dsl::*;

	for feed in feeds_arr {
		let _ = diesel::update(&feed)
			.set(last_called.eq(set_last_called))
			.execute(connection);
	}
}