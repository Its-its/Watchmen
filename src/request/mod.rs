use std::sync::mpsc::{channel, Sender};
use std::time::{Duration, Instant};
use std::thread;

use diesel::RunQueryDsl;

use crate::error::Error;
use crate::database::schema::{items as ItemsSchema, feeds as FeedsSchema};
use crate::database::models::{NewItem, Feed, NewFeed};

pub struct RequestManager {
	pub feeds: Vec<Feed>,
	pub is_idle: bool,
	pub concurrency: i32,
}

pub type CollectedResult = Result<RequestFeedResults, Error>;


impl RequestManager {
	pub fn new() -> Self {
		Self {
			feeds: Vec::new(),

			is_idle: true,
			concurrency: 2,
		}
	}

	pub fn init(&mut self, connection: &diesel::SqliteConnection) -> Result<usize, Error> {
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

	pub fn add_feed_url(&self, url: String, connection: &diesel::SqliteConnection) {
		let feed = NewFeed {
			url: url,

			sec_interval: 60 * 5,
			remove_after: 0,

			ignore_if_not_new: true,

			date_added: chrono::Utc::now().naive_utc().timestamp(),
			last_called: chrono::Utc::now().naive_utc().timestamp()
		};


		let e = diesel::insert_or_ignore_into(FeedsSchema::table)
			.values(&feed)
			.execute(connection);

		if let Ok(count) = e {
			println!("{}", count);
		}
	}

	pub fn run_if_idle(&mut self, is_manual: bool, connection: &diesel::SqliteConnection) -> RequestResults {
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
			.filter(|i| timestamp - i.last_called - i.sec_interval as i64 > 0)
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

		println!("Starting Requests..");


		let (tx, rx) = channel();

		// The RSS feed grabber.
		let safe_request = |feed: Feed| -> CollectedResult {
			let mut feed_res = RequestFeedResults {
				start_time: Instant::now(),
				duration: Duration::new(0, 0),
				new_item_count: 0,
				item_count: 0,
				to_insert: Vec::new()
			};

			let channel = match get_rss_from_url(&feed.url) {
				Ok(c) => c,
				Err(e) => return Err(e)
			};

			feed_res.to_insert = channel.items()
				.iter()
				.map(|i| {
					let mut item: NewItem = i.into();
					item.feed_id = feed.id;
					item
				})
				.collect();


			feed_res.duration = feed_res.start_time.elapsed();

			Ok(feed_res)
		};

		let spawn_next = |tx: Sender<CollectedResult>, feed: Feed| {
			let thread = thread::Builder::new()
			.name(format!("Feed: {}", feed.url))
			.spawn(move || tx.send(safe_request(feed)).expect("send"));

			if let Err(e) = thread {
				eprintln!("Thread Error: {}", e);
			}
		};

		let mut feed_iter = feeds.iter();

		println!("Concurrency: {}", self.concurrency);

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

		results.duration = results.start_time.elapsed();

		self.is_idle = true;

		results
	}
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


pub fn get_rss_from_url(url: &str) -> Result<rss::Channel, Error> {
	use std::io::Read;

	let mut content = Vec::new();
	reqwest::get(url)?.read_to_end(&mut content)?;
	Ok(rss::Channel::read_from(&content[..])?)
}