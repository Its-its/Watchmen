use std::str::FromStr;

use crypto::digest::Digest;

use chrono::{DateTime, Utc, NaiveDateTime};
use rss::Item as RssItem;

use diesel::{Queryable, Insertable};



use super::schema::{items, feeds};


pub type QueryId = i32;


// Item

#[derive(Clone, Queryable)]
pub struct Item {
	pub id: QueryId,

	// Pre-defined
	pub guid: String, // pre-defined guild OR pre-defined link OR self.hash
	pub title: String,
	pub author: String,
	pub content: String,
	pub link: String,
	pub date: i64,
	pub hash: String, // md5(link + title + authors + content + tags) | Iffy on tags. If tags change then hash needs to change.

	// User defined
	pub date_added: i64,
	pub is_read: bool,
	pub is_starred: bool,
	pub is_removed: bool,
	pub tags: String,
	pub feed_id: QueryId
}

#[derive(Debug, Insertable)]
#[table_name = "items"]
pub struct NewItem {
	pub guid: String,
	pub title: String,
	pub author: String,
	pub content: String,
	pub link: String,
	pub date: i64,
	pub hash: String,

	// User defined
	pub date_added: i64,
	pub is_read: bool,
	pub is_starred: bool,
	pub is_removed: bool,
	pub tags: String,
	pub feed_id: QueryId
}


impl From<&RssItem> for NewItem {
	fn from(item: &RssItem) -> NewItem {
		let mut new_item = NewItem {
			guid: Default::default(),
			title: item.title().unwrap_or_default().to_string(),
			author: item.author().unwrap_or_default().to_string(),
			content: item.content().unwrap_or_default().to_string(),
			link: item.link().unwrap_or_default().to_string(),
			date: item.pub_date()
					.and_then(|d| DateTime::parse_from_rfc2822(d).map(|i| i.naive_utc()).ok())
					.unwrap_or_else(|| Utc::now().naive_utc())
					.timestamp(),

			hash: Default::default(),

			date_added: Utc::now().timestamp(),
			is_read: false,
			is_starred: false,
			is_removed: false,
			tags: Default::default(),

			feed_id: 0
		};

		// md5(link + title + authors + content + tags) | Iffy on tags. If tags change then hash needs to change.
		new_item.hash = {
			let mut md5 = crypto::md5::Md5::new();

			md5.input_str(&format!(
				"{}-{}-{}",
				new_item.link,
				new_item.title,
				new_item.author
				// new_item.content Removed b/c some content updates with random ids.
			));

			md5.result_str()
		};

		// pre-defined guild OR pre-defined link OR self.hash
		new_item.guid = {
			item.guid()
			.map(|g| g.value().to_string())
			.or_else(|| item.link().map(|l| l.to_string()))
			.or_else(|| Some(new_item.hash.clone()))
			.unwrap()
		};


		new_item
	}
}


// Feeds

#[derive(Debug, Clone, Queryable, Identifiable)]
pub struct Feed {
	pub id: QueryId,

	pub url: String,

	pub sec_interval: i32,
	pub remove_after: i32,

	pub ignore_if_not_new: bool,

	pub date_added: i64,
	pub last_called: i64,
}


#[derive(Clone, Insertable)]
#[table_name = "feeds"]
pub struct NewFeed {
	pub url: String,

	pub sec_interval: i32,
	pub remove_after: i32,

	// Ignore Feed Item if it's not "new" (older than last_called basically)
	pub ignore_if_not_new: bool,

	pub date_added: i64,
	pub last_called: i64,
}