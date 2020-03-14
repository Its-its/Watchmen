use crypto::digest::Digest;

use serde::{Serialize, Deserialize};

use chrono::{DateTime, Utc};
use rss::Item as RssItem;
use atom_syndication::Entry as AtomItem;

use diesel::{Queryable, Insertable, SqliteConnection, QueryResult};
use diesel::prelude::*;

use crate::Filter;
use super::schema::*;
use crate::state::CoreState;
use crate::request::custom::CustomItem;


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

#[derive(Serialize, Deserialize, Debug, Default, Clone, Queryable, Identifiable)]
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

#[derive(Serialize, Deserialize, Debug, Clone, AsChangeset)]
#[table_name = "categories"]
pub struct EditCategory {
	pub position: Option<i32>,

	pub name: Option<String>,
	pub name_lowercase: Option<String>,

	pub date_added: Option<i64>,
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


//  Filters

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Identifiable)]
#[table_name = "feed_filter"]
pub struct FeedFilter {
	pub id: QueryId,

	pub feed_id: QueryId, // TODO: Make available for categories too?

	pub title: String,

	pub filter: Option<String>
}


#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[table_name = "feed_filter"]
pub struct NewFeedFilter {
	pub feed_id: QueryId,

	pub title: String,

	pub filter: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone, AsChangeset)]
#[table_name = "feed_filter"]
pub struct EditFeedFilter {
	pub feed_id: Option<QueryId>,
	pub title: Option<String>,
	pub filter: Option<Option<String>>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditFrontFeedFilter { // Used to receive the feed from the front end. And change into DB one.
	pub feed_id: Option<QueryId>,
	pub title: Option<String>,
	pub filter: Option<Option<Filter>>
}


// Custom Items

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[table_name = "custom_item"]
pub struct NewCustomItem {
	match_url: String,
	search_opts: String
}

#[derive(Serialize, Deserialize, Debug, Clone, AsChangeset)]
#[table_name = "custom_item"]
pub struct EditCustomItem {
	match_url: Option<String>,
	search_opts: Option<String>
}

impl From<CustomItem> for NewCustomItem {
	fn from(item: CustomItem) -> Self {
		Self {
			match_url: item.match_url,
			search_opts: serde_json::to_string(&item.search_opts).unwrap()
		}
	}
}

impl From<CustomItem> for EditCustomItem {
	fn from(item: CustomItem) -> Self {
		Self {
			match_url: Some(item.match_url),
			search_opts: Some(serde_json::to_string(&item.search_opts).unwrap())
		}
	}
}

// Listener Calls

pub fn get_item_total(category_id: Option<QueryId>, conn: &SqliteConnection) -> QueryResult<i64> {
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
	self::items::table
		.filter(self::items::dsl::date.gt(since))
		.count()
		.get_result(conn)
}

pub fn remove_item(l_id: QueryId, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::items::dsl::*;

	diesel::delete(items.filter(id.eq(l_id))).execute(conn)
}


// Feed Calls

pub fn get_listeners(conn: &SqliteConnection) -> QueryResult<Vec<Feed>> {
	self::feeds::table.load(conn)
}

pub fn remove_listener(f_id: QueryId, rem_stored: bool, state: &mut CoreState) -> QueryResult<usize> {
	let conn = state.connection.connection();

	if let Some(index) = state.requester.feeds.iter().position(|f| f.id == f_id) {
		state.requester.feeds.remove(index);
	}

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
	f_id: QueryId,
	edit: &EditFeed,
	state: &mut CoreState
) -> QueryResult<usize> {
	{ // Update Stored Feed
		if let Some(feed) = state.requester.feeds.iter_mut().find(|f| f.id == f_id) {
			if let Some(i) = edit.description.as_ref() { feed.description = i.to_owned(); }
			if let Some(i) = edit.generator.as_ref() { feed.generator = i.to_owned(); }
			if let Some(i) = edit.title.as_ref() { feed.title = i.to_owned(); }
			if let Some(i) = edit.global_show { feed.global_show = i; }
			if let Some(i) = edit.ignore_if_not_new { feed.ignore_if_not_new = i; }
			if let Some(i) = edit.remove_after { feed.remove_after = i; }
			if let Some(i) = edit.sec_interval { feed.sec_interval = i; }
		}
	}

	use self::feeds::dsl::*;

	diesel::update(feeds.filter(id.eq(f_id)))
		.set(edit)
		.execute(state.connection.connection())
}


// Category Calls

pub fn get_categories(conn: &SqliteConnection) -> QueryResult<Vec<Category>> {
	self::categories::table.load(conn)
}

pub fn get_category(cat_id: QueryId, conn: &SqliteConnection) -> QueryResult<Category> {
	use self::categories::dsl::*;

	categories.filter(id.eq(cat_id)).get_result(conn)
}

pub fn create_category(category: &NewCategory, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::categories::dsl::*;
	diesel::insert_into(categories).values(category).execute(conn)
}

pub fn remove_category(cat_id: QueryId, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::categories::dsl::*;

	diesel::delete(categories.filter(id.eq(cat_id))).execute(conn)
}

pub fn update_category(c_id: QueryId, edit: &EditCategory, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::categories::dsl::*;

	diesel::update(categories.filter(id.eq(c_id)))
		.set(edit)
		.execute(conn)
}


// Category Feeds Calls

pub fn create_category_feed(category: &NewFeedCategory, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::feed_categories::dsl::*;
	diesel::insert_into(feed_categories).values(category).execute(conn)
}

pub fn remove_category_feed(f_id: QueryId, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::feed_categories::dsl::*;
	diesel::delete(feed_categories.filter(id.eq(f_id))).execute(conn)
}


pub fn get_feed_categories(conn: &SqliteConnection) -> QueryResult<Vec<FeedCategory>> {
	self::feed_categories::table.load(conn)
}

pub fn get_category_feeds(cat_id: QueryId, conn: &SqliteConnection) -> QueryResult<Vec<FeedCategory>> {
	use self::feed_categories::dsl::*;

	feed_categories.filter(category_id.eq(cat_id)).get_results(conn)
}


// Feed Filter

pub fn create_feed_filter(feed: &NewFeedFilter, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::feed_filter::dsl::*;
	diesel::insert_into(feed_filter).values(feed).execute(conn)
}

pub fn remove_feed_filter(f_id: QueryId, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::feed_filter::dsl::*;
	diesel::delete(feed_filter.filter(id.eq(f_id))).execute(conn)
}

pub fn get_filters(f_feed_id: Option<QueryId>, conn: &SqliteConnection) -> QueryResult<Vec<FeedFilter>> {
	use self::feed_filter::dsl::*;
	if let Some(f_feed_id) = f_feed_id {
		feed_filter.filter(feed_id.eq(f_feed_id)).get_results(conn)
	} else {
		feed_filter.load(conn)
	}
}

pub fn get_filter(f_id: QueryId, conn: &SqliteConnection) -> QueryResult<Vec<FeedFilter>> {
	use self::feed_filter::dsl::*;
	feed_filter.filter(id.eq(f_id)).get_results(conn)
}