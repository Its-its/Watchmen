use crypto::digest::Digest;

use url::Url;
use serde::{Serialize, Deserialize};

use chrono::{DateTime, Utc};
use rss::Item as RssItem;
use atom_syndication::Entry as AtomItem;

use diesel::{SqliteConnection, QueryResult};
use diesel::prelude::*;

use crate::Filter;
use super::schema::*;
use super::models::{
	QueryId,
	CustomItem, NewCustomItem, EditCustomItem,
	FilterDB, NewFilterDB, EditFilterDB,
	FeedFilterDB, NewFeedFilter,
	Item, NewItem,
	Feed, EditFeed,
	Category, NewCategory, EditCategory,
	FeedCategory, NewFeedCategory,
	Watching, NewWatching, EditWatching,
	WatchParserItem, NewWatchParserItem,
	WatchHistory, NewWatchHistory
};
use crate::state::CoreState;
use crate::request::feeds::custom::{CustomItem as CustomItemBase, FoundItem as CustomFoundItem};
use crate::request::watcher::WatchParserItem as WatchParserItemBase;



// Custom Item

impl Into<CustomItemBase> for CustomItem {
	fn into(self) -> CustomItemBase {
		CustomItemBase {
			id: Some(self.id),
			title: self.title,
			match_url: self.match_url,
			description: self.description,
			search_opts: serde_json::from_str(&self.search_opts).unwrap()
		}
	}
}

impl From<CustomItemBase> for NewCustomItem {
	fn from(item: CustomItemBase) -> Self {
		Self {
			title: item.title,
			match_url: item.match_url,
			description: item.description,
			search_opts: serde_json::to_string(&item.search_opts).unwrap()
		}
	}
}

impl From<CustomItemBase> for EditCustomItem {
	fn from(item: CustomItemBase) -> Self {
		Self {
			title: Some(item.title),
			match_url: Some(item.match_url),
			description: Some(item.description),
			search_opts: Some(serde_json::to_string(&item.search_opts).unwrap())
		}
	}
}


pub fn create_custom_item(item: &NewCustomItem, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::custom_item::dsl::*;

	diesel::insert_into(custom_item).values(item).execute(conn)
}

pub fn get_custom_item_by_id(f_id: QueryId, conn: &SqliteConnection) -> QueryResult<CustomItemBase> {
	use self::custom_item::dsl::*;

	Ok(
		custom_item.find(f_id)
		.get_result::<CustomItem>(conn)?
		.into()
	)
}

pub fn get_custom_item_from_url(f_url: Url, conn: &SqliteConnection) -> QueryResult<CustomItemBase> {
	use self::custom_item::dsl::*;

	let host_str = f_url.host_str().unwrap();

	let period_count = host_str.bytes().filter(|v| v == &b'.').count();

	let mut values = vec![host_str.to_owned()];

	if period_count > 1 {
		for (i, byte) in host_str.bytes().enumerate() {
			if byte == b'.' {
				values.push(format!("*{}", &host_str[i..]));

				if period_count == values.len() {
					break;
				}
			}
		}
	} else {
		values.push(format!("*.{}", host_str));
	}

	Ok(
		custom_item.filter(match_url.eq_any(values))
		.get_result::<CustomItem>(conn)?
		.into()
	)
}

pub fn get_custom_items(conn: &SqliteConnection) -> QueryResult<Vec<CustomItemBase>> {
	Ok(
		self::custom_item::table.load::<CustomItem>(conn)?
		.into_iter()
		.map(|i| i.into())
		.collect()
	)
}


// Grouped Filter
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FilterGrouping {
	pub filter: FilterModel,
	pub feeds: Vec<QueryId>
}


// Filters

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FilterModel {
	pub id: QueryId,
	pub title: String,
	pub filter: Filter
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewFilterModel {
	pub title: String,
	pub filter: Filter
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditFilterModel {
	pub title: Option<String>,
	pub filter: Option<Filter>
}

impl From<EditFilterModel> for EditFilterDB {
	fn from(filter: EditFilterModel) -> Self {
		EditFilterDB {
			title: filter.title,
			filter: filter.filter.map(|v| serde_json::to_string(&v).unwrap())
		}
	}
}

impl From<NewFilterModel> for NewFilterDB {
	fn from(filter: NewFilterModel) -> Self {
		NewFilterDB {
			title: filter.title,
			filter: serde_json::to_string(&filter.filter).unwrap()
		}
	}
}

impl From<FilterDB> for FilterModel {
	fn from(filter: FilterDB) -> Self {
		FilterModel {
			id: filter.id,
			title: filter.title,
			filter: serde_json::from_str(&filter.filter).unwrap()
		}
	}
}


pub fn create_filter(f_filter: NewFilterModel, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::filters::dsl::*;

	diesel::insert_into(filters)
	.values(NewFilterDB::from(f_filter))
	.execute(conn)
}

pub fn update_filter(f_id: QueryId, f_feed: EditFilterModel, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::filters::dsl::*;

	diesel::update(self::filters::table.filter(id.eq(f_id)))
	.set(EditFilterDB::from(f_feed))
	.execute(conn)
}

pub fn remove_filter(f_filter_id: QueryId, conn: &SqliteConnection) -> QueryResult<(usize, usize)> {
	let ff_amount = { // Remove "Feed Filters" too.
		use self::feed_filters::dsl::*;

		diesel::delete(feed_filters.filter(filter_id.eq(f_filter_id))).execute(conn)?
	};

	use self::filters::dsl::*;

	let amount = diesel::delete(filters.filter(id.eq(f_filter_id))).execute(conn)?;

	Ok((amount, ff_amount))
}

pub fn get_filters_for_feed(f_feed_id: QueryId, conn: &SqliteConnection) -> QueryResult<Vec<FilterModel>> {
	use self::filters::dsl::*;

	let filter_ids: Vec<QueryId> = get_filters_from_feed_id(f_feed_id, conn)?
		.into_iter()
		.map(|f| f.filter_id)
		.collect();

	filters.filter(id.eq_any(filter_ids))
		.get_results::<FilterDB>(conn)
		.map(|f| f.into_iter().map(FilterModel::from).collect())
}

pub fn get_filter(f_filter_id: QueryId, conn: &SqliteConnection) -> QueryResult<FilterModel> {
	use self::filters::dsl::*;

	filters.filter(id.eq(f_filter_id))
	.get_result::<FilterDB>(conn)
	.map(FilterModel::from)
}

pub fn get_filters(conn: &SqliteConnection) -> QueryResult<Vec<FilterModel>> {
	use self::filters::dsl::*;

	filters.filter(id.ne(0))
	.get_results::<FilterDB>(conn)
	.map(|v| v.into_iter().map(FilterModel::from).collect())
}



// Feed Filter Listeners.

pub fn get_feed_filters(conn: &SqliteConnection) -> QueryResult<Vec<FeedFilterDB>> {
	use self::feed_filters::dsl::*;

	feed_filters.filter(id.ne(0)).get_results(conn)
}

pub fn get_filters_from_feed_id(f_feed_id: QueryId, conn: &SqliteConnection) -> QueryResult<Vec<FeedFilterDB>> {
	use self::feed_filters::dsl::*;

	feed_filters.filter(feed_id.eq(f_feed_id)).get_results(conn)
}

pub fn get_filters_from_filter_id(f_filter_id: QueryId, conn: &SqliteConnection) -> QueryResult<Vec<FeedFilterDB>> {
	use self::feed_filters::dsl::*;

	feed_filters.filter(filter_id.eq(f_filter_id)).get_results(conn)
}

pub fn create_feed_and_filter_link(f_filter_id: QueryId, f_feed_id: QueryId, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::feed_filters::dsl::*;

	let found: i64 = feed_filters
		.filter(filter_id.eq(f_filter_id))
		.filter(feed_id.eq(f_feed_id))
		.count()
		.get_result(conn)?;

	if found == 0 {
		diesel::insert_into(feed_filters)
		.values(NewFeedFilter {
			filter_id: f_filter_id,
			feed_id: f_feed_id
		})
		.execute(conn)
	} else {
		Ok(0)
	}
}

pub fn remove_feed_and_filter_link(f_filter_id: QueryId, f_feed_id: QueryId, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::feed_filters::dsl::*;

	diesel::delete(
		feed_filters
					.filter(filter_id.eq(f_filter_id))
					.filter(feed_id.eq(f_feed_id))
	).execute(conn)
}



// Feed Items

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

impl From<CustomFoundItem> for NewItem {
	fn from(item: CustomFoundItem) -> NewItem {
		let mut new_item = NewItem {
			guid: item.guid,

			title: item.title,
			author: item.author.unwrap_or_default(),
			content: item.content.unwrap_or_default(),
			link: item.link,
			date: Some(item.date)
				.and_then(|d| DateTime::parse_from_rfc3339(&d).map(|i| i.naive_utc()).ok())
				.unwrap_or_else(|| Utc::now().naive_utc())
				.timestamp(),

			hash: String::default(),

			date_added: Utc::now().timestamp(),
			is_read: false,
			is_starred: false,
			is_removed: false,
			tags: String::default(),

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




// Feeds / Listeners

pub fn get_listeners(conn: &SqliteConnection) -> QueryResult<Vec<Feed>> {
	self::feeds::table.load(conn)
}

pub fn remove_listener(f_id: QueryId, rem_stored: bool, state: &mut CoreState) -> QueryResult<usize> {
	let conn = state.connection.connection();

	if let Some(index) = state.feed_requests.feeds.iter().position(|f| f.id == f_id) {
		state.feed_requests.feeds.remove(index);
	}

	if rem_stored {
		use self::items::dsl::*;
		diesel::delete(items.filter(feed_id.eq(f_id))).execute(conn)?;
	} else {
		// TODO: If not removing everything. We need to keep the listener otherwise we can't display the items.
	}

	{ // Remove Feed Categories
		use self::feed_categories::dsl::*;
		diesel::delete(feed_categories.filter(feed_id.eq(f_id))).execute(conn)?;
	}

	use self::feeds::dsl::*;
	diesel::delete(feeds.filter(id.eq(f_id))).execute(conn)
}

pub fn update_listener(f_id: QueryId, edit: &EditFeed, state: &mut CoreState) -> QueryResult<usize> {
	{ // Update Stored Feed
		if let Some(feed) = state.feed_requests.feeds.iter_mut().find(|f| f.id == f_id) {
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



// Categories

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



// Category Feeds

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


// =================
// ==== WATCHER ====
// =================


// Watchers

pub fn get_watchers(conn: &SqliteConnection) -> QueryResult<Vec<Watching>> {
	self::watching::table.load(conn)
}

pub fn get_watcher_by_url(f_url: &str, conn: &SqliteConnection) -> QueryResult<Watching> {
	use self::watching::dsl::*;

	watching.filter(url.eq(f_url)).get_result(conn)
}

pub fn remove_watcher(f_id: QueryId, rem_stored: bool, conn: &SqliteConnection) -> QueryResult<usize> {
	if rem_stored {
		use self::watch_history::dsl::*;
		diesel::delete(watch_history.filter(watch_id.eq(f_id))).execute(conn)?;
	} else {
		// TODO: If not removing everything. We need to keep the listener otherwise we can't display the items.
	}

	// { // Remove Watch Categories
	// 	use self::feed_categories::dsl::*;
	// 	diesel::delete(feed_categories.filter(feed_id.eq(f_id))).execute(conn)?;
	// }

	use self::watching::dsl::*;
	diesel::delete(watching.filter(id.eq(f_id))).execute(conn)
}

pub fn update_watcher(f_id: QueryId, edit: &EditWatching, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::watching::dsl::*;

	diesel::update(watching.filter(id.eq(f_id)))
		.set(edit)
		.execute(conn)
}

pub fn create_watcher(watcher: &NewWatching, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::watching::dsl::*;

	diesel::insert_into(watching).values(watcher).execute(conn)
}



// Watch Parser

impl Into<WatchParserItemBase> for WatchParserItem {
	fn into(self) -> WatchParserItemBase {
		WatchParserItemBase {
			id: Some(self.id),
			title: self.title,
			match_url: self.match_url,
			description: self.description,
			match_opts: serde_json::from_str(&self.match_opts).unwrap()
		}
	}
}

pub fn create_watch_parser(item: &NewWatchParserItem, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::watch_parser::dsl::*;

	diesel::insert_into(watch_parser).values(item).execute(conn)
}

pub fn get_watch_parser_by_id(f_id: QueryId, conn: &SqliteConnection) -> QueryResult<WatchParserItemBase> {
	use self::watch_parser::dsl::*;

	Ok(
		watch_parser.find(f_id)
		.get_result::<WatchParserItem>(conn)?
		.into()
	)
}

pub fn get_watch_parser_from_url(f_url: Url, conn: &SqliteConnection) -> QueryResult<WatchParserItemBase> {
	use self::watch_parser::dsl::*;

	let host_str = f_url.host_str().unwrap();

	let period_count = host_str.bytes().filter(|v| v == &b'.').count();

	let mut values = vec![host_str.to_owned()];

	if period_count > 1 {
		for (i, byte) in host_str.bytes().enumerate() {
			if byte == b'.' {
				values.push(format!("*{}", &host_str[i..]));

				if period_count == values.len() {
					break;
				}
			}
		}
	} else {
		values.push(format!("*.{}", host_str));
	}

	Ok(
		watch_parser.filter(match_url.eq_any(values))
		.get_result::<WatchParserItem>(conn)?
		.into()
	)
}

pub fn get_watching_items(conn: &SqliteConnection) -> QueryResult<Vec<WatchParserItemBase>> {
	Ok(
		self::watch_parser::table.load::<WatchParserItem>(conn)?
		.into_iter()
		.map(|i| i.into())
		.collect()
	)
}

// Watch History

pub fn get_last_watch_history(f_watch_id: QueryId, conn: &SqliteConnection) -> QueryResult<WatchHistory> {
	use self::watch_history::dsl::*;

	Ok(
		watch_history
		.filter(watch_id.eq(f_watch_id))
		.order_by(date_added.desc())
		.get_result(conn)?
	)
}



pub fn create_last_watch_history(item: &NewWatchHistory, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::watch_history::dsl::*;

	diesel::insert_into(watch_history).values(item).execute(conn)
}