use std::time::{Duration, Instant};

use log::{info};
use diesel::{RunQueryDsl, SqliteConnection};

use crate::error::{Error, Result};
use crate::feature::schema::{items as ItemsSchema, feeds as FeedsSchema};
use crate::feature::models::{NewItem, Feed, NewFeed};


pub mod rss;
pub mod atom;
pub mod custom;

pub type CollectedResult = Result<RequestFeedResults>;


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
	pub feeds: Vec<Feed>,
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

		let found = feeds.filter(id.ne(0)).load::<Feed>(connection)?;

		self.feeds = found;

		Ok(self.feeds.len())
	}

	pub fn create_new_feed(&self, url: String, conn: &SqliteConnection) -> Result<NewFeed> {
		Ok(match FeedType::from_url(&url, conn) {
			FeedType::Rss(Ok(feed)) => rss::new_from_feed(url, feed),
			FeedType::Atom(Ok(feed)) => atom::new_from_feed(url, feed),
			FeedType::Custom(Ok(_)) => custom::new_from_url(url, conn)?,

			FeedType::Custom(Err(e))
			| FeedType::Atom(Err(e))
			| FeedType::Rss(Err(e)) => return Err(e),

			_ => return Err("Unknown Feed.. It didn't match the current supported ones.".into())
		})
	}

	pub fn request_all_if_idle(&mut self, is_manual: bool, connection: &SqliteConnection) -> RequestResults {
		let mut results = RequestResults {
			error: None,
			start_time: Instant::now(),
			duration: Duration::new(0, 0),
			was_manual: is_manual,
			concurrency: 0,
			feeds: Vec::new()
		};


		let feeds: Vec<&mut Feed> = {
			let timestamp = chrono::Utc::now().timestamp();

			self.feeds.iter_mut()
			.filter(|i: &&mut Feed| timestamp - i.last_called - i.sec_interval as i64 > 0)
			.collect()
		};


		if !self.is_idle {
			results.error = Some("Request Manager is already running!".into());
			return results;
		}

		if feeds.is_empty() {
			// results.error = Some("No feeds to run...".into());
			return results;
		}

		self.is_idle = false;

		info!("Starting Requests.. Found: {}", feeds.len());


		for feed in &feeds {
			results.feeds.push(request_feed((*feed).clone(), connection));
		}


		// Database
		{
			// Update Last Called
			let new_timestamp = chrono::Utc::now().timestamp();
			update_feed_last_called_db(new_timestamp, feeds, connection);

			// After finished insert new items to DB.
			for res in results.feeds.iter_mut() {
				if let Ok(res) = res {
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

		results
	}
}


pub fn request_feed(feed: Feed, conn: &SqliteConnection) -> CollectedResult {
	info!(" - Requesting: {}", feed.url);

	let mut feed_res = RequestFeedResults {
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
				let mut item: NewItem = i.into();
				item.feed_id = feed.id;
				item
			})
			.collect();
		}

		FeedType::Atom(Ok(atom_feed)) => {
			feed_res.to_insert = atom_feed.entries()
			.iter()
			.map(|i| {
				let mut item: NewItem = i.into();
				item.feed_id = feed.id;
				item
			})
			.collect();
		}

		FeedType::Custom(Ok(custom_feed_items)) => {
			feed_res.to_insert = custom_feed_items
			.into_iter()
			.map(|i| {
				let mut item: NewItem = i.into();
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


pub fn update_feed_last_called_db(set_last_called: i64, feeds_arr: Vec<&mut Feed>, connection: &SqliteConnection) {
	use diesel::prelude::*;
	use FeedsSchema::dsl::*;

	for feed in feeds_arr {
		let _ = diesel::update(&*feed)
			.set(last_called.eq(set_last_called))
			.execute(connection);

		feed.last_called = set_last_called;
	}
}


#[derive(Debug)]
pub struct RequestResults {
	pub error: Option<String>,
	pub was_manual: bool,
	pub start_time: Instant,
	pub duration: Duration,
	pub concurrency: i32,
	pub feeds: Vec<CollectedResult>
}


#[derive(Debug)]
pub struct RequestFeedResults {
	pub start_time: Instant,
	pub duration: Duration,
	pub new_item_count: usize,
	pub item_count: i32,
	pub to_insert: Vec<NewItem>
}