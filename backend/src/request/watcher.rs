use std::time::{Duration, SystemTime};
use reqwest::Client;
use serde::{Serialize, Deserialize};
use url::Url;
use log;
use diesel::SqliteConnection;


use crate::feature::schema::{watching as WatchingSchema};
use crate::feature::models::{EditWatchParserItemModel, NewWatchHistoryModel, NewWatchParserItemModel, NewWatchingModel, QueryId, WatchingModel};
use crate::{Result, Error, xpath};
use super::feeds::custom::ParseOpts;
use super::{RequestResults, ItemResults, RequestItemResults, InnerRequestResults};

use crate::feature::objects::{
	get_watch_parser_from_url, get_watch_parser_by_id,
	get_watchers,
	get_last_watch_history, create_last_watch_history
};



type WatcherResult = Result<RequestItemResults<NewWatchHistoryModel>>;


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WatchParserItem {
	pub id: Option<i32>,

	pub title: String,
	pub description: String,
	pub match_url: String,

	pub match_opts: MatchParser
}

impl From<WatchParserItem> for NewWatchParserItemModel {
	fn from(val: WatchParserItem) -> Self {
		NewWatchParserItemModel {
			title: val.title,
			description: val.description,
			match_url: val.match_url,
			match_opts: serde_json::to_string(&val.match_opts).unwrap()
		}
	}
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateableWatchParser {
	pub title: Option<String>,
	pub description: Option<String>,
	pub match_url: Option<String>,

	pub match_opts: Option<MatchParser>
}


impl From<UpdateableWatchParser> for EditWatchParserItemModel {
	fn from(val: UpdateableWatchParser) -> Self {
		EditWatchParserItemModel {
			title: val.title,
			description: val.description,
			match_url: val.match_url,
			match_opts: val.match_opts.as_ref().map(serde_json::to_string).and_then(|v| v.ok())
		}
	}
}



#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MatchParser {
	pub items: String,

	/// Value is used to check for changes.
	pub value: ParseOpts,

	// Used for watching list of items.
	pub unique_id: Option<ParseOpts>,

	pub title: Option<ParseOpts>,
	pub link: Option<ParseOpts>,
}



#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FoundItem {
	pub value: String,
	pub unique_id: Option<String>,
	pub title: Option<String>,
	pub link: Option<String>,
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

	pub fn verify_new_watcher(&self, url: String, parser_id: Option<QueryId>, conn: &SqliteConnection) -> Result<NewWatchingModel> {
		let item = if let Some(id) = parser_id {
			get_watch_parser_by_id(id, conn)?
		} else {
			get_watch_parser_from_url(Url::parse(&url).unwrap(), conn)?
		};

		let watcher = NewWatchingModel {
			parser_id: item.id,

			enabled: true,

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

	pub async fn request_all_if_idle(&mut self, is_manual: bool, req_client: &Client, connection: &SqliteConnection) -> RequestResults {
		let mut results = InnerRequestResults {
			general_error: None,
			start_time: SystemTime::now(),
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
					results.general_error = Some(format!("{:?}", e));
					return RequestResults::Watcher(results);
				}
			};

			watchers.into_iter()
			.filter(|i| i.enabled && timestamp - i.last_called - i.sec_interval as i64 > 0)
			.collect()
		};


		if !self.is_idle {
			results.general_error = Some("Request Manager is already running!".into());
			return RequestResults::Watcher(results);
		}

		if feeds.is_empty() {
			// results.error = Some("No feeds to run...".into());
			return RequestResults::Watcher(results);
		}

		self.is_idle = false;

		log::debug!("Starting Requests.. Found: {}", feeds.len());

		for feed in &feeds {
			let feed_cloned = (*feed).clone();

			results.items.push(ItemResults {
				results: request_feed(&feed_cloned, req_client, connection).await,
				item: feed_cloned
			});
		}


		// Database
		{
			// Update Last Called
			let new_timestamp = chrono::Utc::now().timestamp();
			update_last_called_db(new_timestamp, feeds, connection);

			// After finished insert new items to DB.
			for res in results.items.iter_mut() {
				if let Ok(res) = res.results.as_mut() {
					// Only have to get(0) since we only ever return 1 watch history.
					if let Some(item) = res.to_insert.get(0) {
						let count = create_last_watch_history(item, connection);

						if let Ok(count) = count {
							res.new_item_count = count;
						}
					}
				}
			}
		}

		results.duration = results.start_time.elapsed().unwrap();

		self.is_idle = true;

		RequestResults::Watcher(results)
	}
}


pub async fn get_from_url_parser(req_client: &Client, url: &str, parser: &MatchParser) -> Result<Vec<FoundItem>> {
	let resp = req_client.get(url).send().await?.text().await?;
	let mut reader = std::io::Cursor::new(resp);

	let doc = xpather::parse_document(&mut reader)?;

	Ok(doc.evaluate(&parser.items)?
		.collect_nodes()?
		.into_iter()
		.map::<Result<FoundItem>, _>(|node| {
			// Find value.
			let value = parser.value.evaluate(&doc, &node)?
				.next()
				.transpose()?
				.map(|v| Result::Ok(parser.value.parse(&v.convert_to_string()?)?))
				.transpose()?
				.map(|v| v.trim().escape_default().to_string())
				.ok_or_else(|| Error::Other("Missing Required Title.".into()))?;

			// Find title.
			let title = xpath::get_optional_string(parser.title.as_ref(), &doc, &node)?
				.map(|v| v.trim().escape_default().to_string());

			// Find link.
			let link = xpath::get_optional_string(parser.link.as_ref(), &doc, &node)?
				.map(|v| v.trim().escape_default().to_string());

			// Unique ID
			let unique_id = xpath::get_optional_string(parser.unique_id.as_ref(), &doc, &node)?
				.map(|v| v.trim().escape_default().to_string());

			Ok(FoundItem {
				value,
				unique_id,
				title,
				link
			})
		})
		.filter_map(|i| {
			if i.is_err() {
				log::error!("EVALUATION ERROR: {:?}", i);
			}

			i.ok()
		})
		.collect()
	)
}


pub async fn request_feed(feed: &WatchingModel, req_client: &Client, conn: &SqliteConnection) -> WatcherResult {
	log::debug!(" - Requesting: {}", feed.url);

	let mut feed_res = RequestItemResults {
		start_time: SystemTime::now(),
		duration: Duration::new(0, 0),
		new_item_count: 0,
		item_count: 0,
		to_insert: Vec::new()
	};

	let parser = if let Some(parser_id) = feed.parser_id {
		get_watch_parser_by_id(parser_id, conn)?
	} else {
		get_watch_parser_from_url(Url::parse(&feed.url).unwrap(), conn)?
	};

	let new_items = get_from_url_parser(req_client, &feed.url, &parser.match_opts).await?;

	if let Some(last_item) = get_last_watch_history(feed.id, conn)? {
		// Anything in the new_items is not in the last_items?
		if new_items.iter().any(|v| !last_item.items.contains(v)) {
			feed_res.to_insert.push(NewWatchHistoryModel {
				watch_id: feed.id,
				items: serde_json::to_string(&new_items).unwrap(),

				date_added: chrono::Utc::now().timestamp()
			});
		}
	} else {
		// No last watch history? Create it.
		create_last_watch_history(&NewWatchHistoryModel {
			watch_id: feed.id,
			items: serde_json::to_string(&new_items).unwrap(),

			date_added: chrono::Utc::now().timestamp()
		}, conn)?;
	}

	feed_res.duration = feed_res.start_time.elapsed()?;

	Ok(feed_res)
}


pub fn update_last_called_db(set_last_called: i64, feeds_arr: Vec<WatchingModel>, connection: &SqliteConnection) {
	use diesel::prelude::*;
	use WatchingSchema::dsl::*;

	for feed in feeds_arr {
		let _ = diesel::update(&feed)
			.set(last_called.eq(set_last_called))
			.execute(connection);
	}
}