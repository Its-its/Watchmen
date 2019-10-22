use std::sync::mpsc::{channel, Sender};
use std::time::{Duration, Instant};
use std::thread;

use log::{info, error};
use diesel::RunQueryDsl;

use crate::error::{Error, Result};
use crate::feature::schema::{items as ItemsSchema, feeds as FeedsSchema};
use crate::feature::models::{NewItem, Feed, NewFeed};


pub type CollectedResult = Result<RequestFeedResults>;


pub enum FeedType {
	Rss(Result<rss::Channel>),
	Atom(Result<atom_syndication::Feed>),

	__Unknown
}

impl FeedType {
	pub fn from_url(url: &str) -> FeedType {
		// RSS
		match get_rss_from_url(url) {
			Ok(c) => return FeedType::Rss(Ok(c)),

			Err(e) => {
				info!("rss: {:?}", e);
				if let Error::Rss(e) = e {
					use rss::Error::InvalidStartTag;

					if let InvalidStartTag = e {
						// TODO: Fix this so it's not an if else
					} else {
						return FeedType::Rss(Err(Error::Rss(e)));
					}
				} else {
					return FeedType::Rss(Err(e));
				}
			}
		};

		// ATOM
		match get_atom_from_url(url) {
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
		};

		FeedType::__Unknown
	}

	pub fn from_feed_type(feed_type: i32, url: &str) -> FeedType {
		match feed_type {
			0 => FeedType::Rss(get_rss_from_url(url)),
			1 => FeedType::Atom(get_atom_from_url(url)),
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

	pub fn init(&mut self, connection: &diesel::SqliteConnection) -> Result<usize> {
		use diesel::prelude::*;
		use FeedsSchema::dsl::*;

		let found = feeds.filter(id.ne(0)).load::<Feed>(connection);

		match found {
			Ok(found) => {
				self.feeds = found;
				Ok(self.feeds.len())
			}

			Err(e) => Err(e.into())
		}
	}

	pub fn create_new_feed(&self, url: String) -> Result<NewFeed> {
		Ok(match FeedType::from_url(&url) {
			FeedType::Rss(Ok(feed)) => {
				NewFeed {
					url: url,

					title: feed.title().to_string(),
					description: feed.description().to_string(),
					generator: feed.generator().unwrap_or_default().to_string(),

					feed_type: 0,

					sec_interval: 60 * 5,
					remove_after: 0,

					global_show: true,
					ignore_if_not_new: true,

					date_added: chrono::Utc::now().naive_utc().timestamp(),
					last_called: chrono::Utc::now().naive_utc().timestamp(),
				}
			}

			FeedType::Atom(Ok(feed)) => {
				NewFeed {
					url: url,

					title: feed.title().to_string(),
					description: feed.subtitle().unwrap_or_default().to_string(),
					generator: feed.generator().unwrap_or(&atom_syndication::Generator::default()).value().to_string(),

					feed_type: 1,

					sec_interval: 60 * 5,
					remove_after: 0,

					global_show: true,
					ignore_if_not_new: true,

					date_added: chrono::Utc::now().naive_utc().timestamp(),
					last_called: chrono::Utc::now().naive_utc().timestamp(),
				}
			}

			FeedType::__Unknown => {
				return Err("Unknown Feed.. It didn't match the current supported ones.".into())
			}

			FeedType::Atom(Err(e))
			| FeedType::Rss(Err(e)) => return Err(e)
		})
	}

	pub fn request_all_if_idle(&mut self, is_manual: bool, connection: &diesel::SqliteConnection) -> RequestResults {
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
			results.error = Some("No feeds to run...".into());
			return results;
		}

		self.is_idle = false;

		info!("Starting Requests.. Found: {}", feeds.len());

		let (tx, rx) = channel();


		let spawn_next = |tx: Sender<CollectedResult>, feed: Feed| {
			let thread = thread::Builder::new()
			.name(format!("Feed: {}", feed.url))
			.spawn(move || tx.send(request_feed(feed)).expect("send"));

			if let Err(e) = thread {
				error!("Thread Error: {}", e);
			}
		};


		let mut feed_iter = feeds.iter();

		for _ in 0..self.concurrency {
			if let Some(feed) = feed_iter.next() {
				spawn_next(tx.clone(), (*feed).clone());
			}
		}


		// Loop until finished.
		while results.feeds.len() != feeds.len() {
			results.feeds.push(rx.recv().expect("Error Receving Request:"));

			if let Some(feed) = feed_iter.next() {
				spawn_next(tx.clone(), (*feed).clone());
			}
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


pub fn request_feed(feed: Feed) -> CollectedResult {
	let mut feed_res = RequestFeedResults {
		start_time: Instant::now(),
		duration: Duration::new(0, 0),
		new_item_count: 0,
		item_count: 0,
		to_insert: Vec::new()
	};

	match FeedType::from_feed_type(feed.feed_type, &feed.url) {
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

		FeedType::__Unknown => {
			return Err("Unknown Feed.. It didn't match the current supported ones.".into())
		}

		FeedType::Atom(Err(e))
		| FeedType::Rss(Err(e)) => return Err(e)
	};

	feed_res.duration = feed_res.start_time.elapsed();

	Ok(feed_res)
}


pub fn update_feed_last_called_db(set_last_called: i64, feeds_arr: Vec<&mut Feed>, connection: &diesel::SqliteConnection) {
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


pub fn get_rss_from_url(url: &str) -> Result<rss::Channel> {
	use std::io::Read;

	let mut content = Vec::new();

	let mut resp = reqwest::get(url)?;
	resp.read_to_end(&mut content)?;

	Ok(rss::Channel::read_from(&content[..])?)
}

pub fn get_atom_from_url(url: &str) -> Result<atom_syndication::Feed> {
	use std::io::Read;

	let mut content = Vec::new();

	let mut resp = reqwest::get(url)?;
	resp.read_to_end(&mut content)?;

	Ok(atom_syndication::Feed::read_from(&content[..])?)
}