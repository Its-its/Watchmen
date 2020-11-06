use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use url::Url;
use log::info;
use diesel::SqliteConnection;


use crate::feature::schema::{watching as WatchingSchema};
use crate::feature::models::{QueryId, Watching, NewWatching, NewWatchHistory};
use crate::{Result, Error};
use super::feeds::custom::ParseOpts;
use super::RequestResults;

use crate::feature::objects::{
	get_watch_parser_from_url, get_watch_parser_by_id,
	get_watchers,
	get_last_watch_history, create_last_watch_history
};



pub type WatcherResult = Result<RequestWatcherResults>;


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WatchParserItem {
	pub id: Option<i32>,

	pub title: String,
	pub description: String,
	pub match_url: String, // TODO: Ensure always lowercase. I have unique index.

	pub match_opts: MatchParser
}


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MatchParser {
	pub value: ParseOpts
}



#[derive(Debug, Clone, Default)]
pub struct FoundItem {
	pub value: String
}


#[derive(Debug)]
pub struct WatcherRequestResults {
	pub error: Option<String>,
	pub was_manual: bool,
	pub start_time: Instant,
	pub duration: Duration,
	pub concurrency: i32,
	pub items: Vec<WatcherResult>
}

#[derive(Debug)]
pub struct RequestWatcherResults {
	pub start_time: Instant,
	pub duration: Duration,
	pub new_item_count: usize,
	pub item_count: i32,
	pub insert: Option<NewWatchHistory>
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

	pub fn verify_new_watcher(&self, url: String, custom_item_id: Option<QueryId>, conn: &SqliteConnection) -> Result<NewWatching> {
		let item = if let Some(id) = custom_item_id {
			get_watch_parser_by_id(id, conn)?
		} else {
			get_watch_parser_from_url(Url::parse(&url).unwrap(), conn)?
		};

		let watcher = NewWatching {
			url,

			title: item.title,
			description: item.description,

			sec_interval: 60 * 10,
			remove_after: 0,

			date_added: chrono::Utc::now().naive_utc().timestamp(),
			last_called: chrono::Utc::now().naive_utc().timestamp(),
		};

		Ok(watcher)
	}

	pub fn request_all_if_idle(&mut self, is_manual: bool, connection: &SqliteConnection) -> RequestResults {
		let mut results = WatcherRequestResults {
			error: None,
			start_time: Instant::now(),
			duration: Duration::new(0, 0),
			was_manual: is_manual,
			concurrency: 0,
			items: Vec::new()
		};


		let feeds: Vec<_> = {
			let timestamp = chrono::Utc::now().timestamp();

			let watchers = match get_watchers(connection) {
				Ok(i) => i,
				Err(e) => {
					results.error = Some(format!("{:?}", e));
					return RequestResults::Watcher(results);
				}
			};

			watchers.into_iter()
			.filter(|i| timestamp - i.last_called - i.sec_interval as i64 > 0)
			.collect()
		};


		if !self.is_idle {
			results.error = Some("Request Manager is already running!".into());
			return RequestResults::Watcher(results);
		}

		if feeds.is_empty() {
			// results.error = Some("No feeds to run...".into());
			return RequestResults::Watcher(results);
		}

		self.is_idle = false;

		info!("Starting Requests.. Found: {}", feeds.len());


		for feed in &feeds {
			results.items.push(request_feed((*feed).clone(), connection));
		}


		// Database
		{
			// Update Last Called
			let new_timestamp = chrono::Utc::now().timestamp();
			update_last_called_db(new_timestamp, feeds, connection);

			// After finished insert new items to DB.
			for res in results.items.iter_mut() {
				if let Ok(res) = res {
					if let Some(item) = res.insert.as_ref() {
						let count = create_last_watch_history(item, connection);

						if let Ok(count) = count {
							res.new_item_count = count;
						}
					}
				}
			}
		}

		results.duration = results.start_time.elapsed();

		self.is_idle = true;

		RequestResults::Watcher(results)
	}
}




pub fn get_from_url(url: &str, conn: &diesel::SqliteConnection) -> Result<FoundItem> {
	let found = get_watch_parser_from_url(Url::parse(url).unwrap(), conn)?;

	// turn found into SearchParser

	get_from_url_parser(url, &found.match_opts)
}

pub fn get_from_url_parser(url: &str, parser: &MatchParser) -> Result<FoundItem> {
	let mut resp = reqwest::get(url)?;

	let doc = xpath::parse_doc(&mut resp);

	let value = doc.evaluate(&parser.value.xpath)
		.map(|v| v.vec_string())
		.transpose()?
		.and_then(|v| v.first().cloned())
		.ok_or_else(|| Error::Other("Unable Value.".into()))?;

	Ok(FoundItem {
		value: parser.value.parse(value)?.trim().to_string()
	})
}


pub fn request_feed(feed: Watching, conn: &SqliteConnection) -> WatcherResult {
	info!(" - Requesting: {}", feed.url);

	let mut feed_res = RequestWatcherResults {
		start_time: Instant::now(),
		duration: Duration::new(0, 0),
		new_item_count: 0,
		item_count: 0,
		insert: None
	};

	let new_item = get_from_url(&feed.url, conn)?;

	let last_item = get_last_watch_history(feed.id, conn)?;

	if new_item.value != last_item.value {
		println!(" | New item Value found!");

		feed_res.insert = Some(NewWatchHistory {
			watch_id: feed.id,
			value: new_item.value,

			date_added: chrono::Utc::now().timestamp()
		});
	} else {
		println!(" | Unchanged.");
	}

	feed_res.duration = feed_res.start_time.elapsed();

	Ok(feed_res)
}


pub fn update_last_called_db(set_last_called: i64, feeds_arr: Vec<Watching>, connection: &SqliteConnection) {
	use diesel::prelude::*;
	use WatchingSchema::dsl::*;

	for feed in feeds_arr {
		let _ = diesel::update(&feed)
			.set(last_called.eq(set_last_called))
			.execute(connection);
	}
}