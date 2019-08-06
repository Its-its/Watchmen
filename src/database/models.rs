use crypto::digest::Digest;

use serde::{Serialize, Deserialize};

use chrono::{DateTime, Utc};
use rss::Item as RssItem;
use atom_syndication::Entry as AtomItem;

use diesel::{Queryable, Insertable, SqliteConnection, QueryResult};
use diesel::prelude::*;

use super::schema::*;

pub type QueryId = i32;

// Item

#[derive(Serialize, Deserialize, Debug, Clone, Queryable)]
pub struct Item {
	pub id: QueryId,

	// Pre-defined
	pub guid: String, // pre-defined guild OR pre-defined link OR self.hash
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

#[derive(Serialize, Deserialize, Debug, Insertable)]
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

impl From<&AtomItem> for NewItem {
	fn from(item: &AtomItem) -> NewItem {
		let mut new_item = NewItem {
			guid: item.id().to_string(),

			title: item.title().to_string(),
			author: item.authors().iter().map(|p| p.name().to_string()).collect::<Vec<String>>().join(" "),
			content: item.content().unwrap_or(&atom_syndication::Content::default()).value().unwrap_or_default().to_string(),
			link: item.links().first().map(|l| l.href()).unwrap_or_default().to_string(),
			date: item.published()
					.or_else(|| Some(item.updated()))
					.and_then(|d| DateTime::parse_from_rfc3339(d).map(|i| i.naive_utc()).ok())
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

		// OR pre-defined link OR self.hash
		if new_item.guid.is_empty() {
			if new_item.link.is_empty() {
				new_item.guid = new_item.link.clone();
			} else {
				new_item.guid = new_item.hash.clone();
			}
		}

		new_item
	}
}


// Feeds

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Identifiable)]
pub struct Feed {
	pub id: QueryId,

	pub url: String,
	pub title: String,
	pub description: String,
	pub generator: String,

	pub feed_type: i32,

	pub sec_interval: i32,
	pub remove_after: i32,

	pub global_show: bool,

	pub ignore_if_not_new: bool,

	pub date_added: i64,
	pub last_called: i64,
}


#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[table_name = "feeds"]
pub struct NewFeed {
	pub url: String,
	pub title: String,
	pub description: String,
	pub generator: String,

	pub feed_type: i32,

	pub sec_interval: i32,
	pub remove_after: i32,

	pub global_show: bool,

	// Ignore Feed Item if it's not "new" (older than last_called basically)
	pub ignore_if_not_new: bool,

	pub date_added: i64,
	pub last_called: i64,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, AsChangeset)]
#[table_name = "feeds"]
pub struct EditFeed {
	pub title: Option<String>,
	pub description: Option<String>,
	pub generator: Option<String>,

	pub ignore_if_not_new: Option<bool>,
	pub global_show: Option<bool>,

	pub sec_interval: Option<i32>,
	pub remove_after: Option<i32>,
}


// Categories

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Identifiable)]
#[table_name = "categories"]
pub struct Category {
	pub id: QueryId,

	pub position: i32,

	pub name: String,
	pub name_lowercase: String,

	pub date_added: i64,
}


#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[table_name = "categories"]
pub struct NewCategory {
	pub position: i32,

	pub name: String,
	pub name_lowercase: String,

	pub date_added: i64,
}


// Feeds Categories

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Identifiable)]
#[table_name = "feed_categories"]
pub struct FeedCategory {
	pub id: QueryId,

	pub feed_id: QueryId,
	pub category_id: QueryId
}


#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[table_name = "feed_categories"]
pub struct NewFeedCategory {
	pub feed_id: QueryId,
	pub category_id: QueryId
}

// Listener Calls

pub fn get_item_total(category_id: Option<QueryId>, conn: &SqliteConnection) -> QueryResult<i64> {
	use diesel::prelude::*;
	use self::items::dsl::*;

	match category_id {
		Some(cat_id) => {
			let feeds = get_category_feeds(cat_id, conn)?;
			let feed_ids: Vec<QueryId> = feeds.iter().map(|f| f.id).collect();

			self::items::table
				.filter(feed_id.eq_any(feed_ids))
				.count()
				.get_result(conn)
		}

		None => {
			self::items::table.count().get_result(conn)
		}
	}
}

pub fn get_items_in_range(category_id: Option<QueryId>, item_count: i64, skip_count: i64, conn: &SqliteConnection) -> QueryResult<Vec<Item>> {
	use diesel::prelude::*;
	use self::items::dsl::*;

	match category_id {
		Some(cat_id) => {
			let feeds = get_category_feeds(cat_id, conn)?;
			let feed_ids: Vec<QueryId> = feeds.iter().map(|f| f.id).collect();

			self::items::table
				.filter(feed_id.eq_any(feed_ids))
				.limit(item_count)
				.offset(skip_count)
				.order(self::items::dsl::date.desc())
				.load(conn)
		}

		None => {
			self::items::table
				.limit(item_count)
				.offset(skip_count)
				.order(self::items::dsl::date.desc())
				.load(conn)
		}
	}
}

pub fn get_item_count_since(since: i64, conn: &SqliteConnection) -> QueryResult<i64> {
	use diesel::prelude::*;

	self::items::table
		.filter(self::items::dsl::date.gt(since))
		.count()
		.get_result(conn)
}

pub fn remove_item(l_id: QueryId, conn: &SqliteConnection) -> QueryResult<usize> {
	use diesel::prelude::*;
	use self::items::dsl::*;

	diesel::delete(items.filter(id.eq(l_id))).execute(conn)
}


// Feed Calls

pub fn get_listeners(conn: &SqliteConnection) -> QueryResult<Vec<Feed>> {
	use diesel::prelude::*;

	self::feeds::table.load(conn)
}

pub fn remove_listener(f_id: QueryId, rem_stored: bool, conn: &SqliteConnection) -> QueryResult<usize> {
	use diesel::prelude::*;

	if rem_stored {
		use self::items::dsl::*;
		diesel::delete(items.filter(feed_id.eq(f_id))).execute(conn)?;
	}

	{ // Remove Feed Categories
		use self::feed_categories::dsl::*;
		diesel::delete(feed_categories.filter(feed_id.eq(f_id))).execute(conn)?;
	}

	use self::feeds::dsl::*;
	diesel::delete(feeds.filter(id.eq(f_id))).execute(conn)
}

pub fn update_listener(
	l_id: QueryId,
	edit: &EditFeed,
	conn: &SqliteConnection
) -> QueryResult<usize> {
	use diesel::prelude::*;
	use self::feeds::dsl::*;

	diesel::update(feeds.filter(id.eq(l_id)))
		.set(edit)
		.execute(conn)
}


// Category Calls

pub fn get_categories(conn: &SqliteConnection) -> QueryResult<Vec<Category>> {
	use diesel::prelude::*;

	self::categories::table.load(conn)
}

pub fn get_category(cat_id: QueryId, conn: &SqliteConnection) -> QueryResult<Category> {
	use diesel::prelude::*;
	use self::categories::dsl::*;

	categories.filter(id.eq(cat_id)).get_result(conn)
}

pub fn create_category(category: &NewCategory, conn: &SqliteConnection) -> QueryResult<usize> {
	use diesel::prelude::*;
	use self::categories::dsl::*;

	diesel::insert_into(categories).values(category).execute(conn)
}


// Category Feeds Calls

pub fn create_category_feed(category: &NewFeedCategory, conn: &SqliteConnection) -> QueryResult<usize> {
	use diesel::prelude::*;
	use self::feed_categories::dsl::*;

	diesel::insert_into(feed_categories).values(category).execute(conn)
}

pub fn remove_category_feed(f_id: QueryId, conn: &SqliteConnection) -> QueryResult<usize> {
	use diesel::prelude::*;
	use self::feed_categories::dsl::*;

	diesel::delete(feed_categories.filter(id.eq(f_id)))
		.execute(conn)
}


pub fn get_feed_categories(conn: &SqliteConnection) -> QueryResult<Vec<FeedCategory>> {
	use diesel::prelude::*;

	self::feed_categories::table.load(conn)
}

pub fn get_category_feeds(cat_id: QueryId, conn: &SqliteConnection) -> QueryResult<Vec<FeedCategory>> {
	use diesel::prelude::*;
	use self::feed_categories::dsl::*;

	feed_categories.filter(category_id.eq(cat_id)).get_results(conn)
}
