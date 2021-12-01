use crypto::digest::Digest;

use url::Url;
use serde::{Serialize, Deserialize};

use chrono::{DateTime, Utc};
use rss::Item as RssItem;
use atom_syndication::Entry as AtomItem;

use diesel::{SqliteConnection, QueryResult};
use diesel::prelude::*;

use crate::FilterType;
use crate::feature::models::{NewRequestHistoryGroupModel, RequestHistoryGroupModel};
use crate::request::RequestResults;
use crate::state::RequestResponse;
use super::schema::*;
use super::models::{CategoryModel, CustomItemModel, EditCategoryModel, EditCustomItemModel, EditFeedModel, EditFilterModel, EditWatchParserItemModel, EditWatchingModel, FeedCategoryModel, FeedFilterModel, FeedItemModel, FeedModel, FilterModel, NewCategoryModel, NewCustomItemModel, NewFeedCategoryModel, NewFeedFilterModel, NewFeedItemModel, NewFilterModel, NewRequestHistoryItemModel, NewWatchHistoryModel, NewWatchParserItemModel, NewWatchingModel, QueryId, RequestHistoryItemModel, WatchHistoryModel, WatchParserItemModel, WatchingModel};

use crate::request::feeds::custom::{CustomItem as CustomItemBase, FoundItem as CustomFoundItem};
use crate::request::watcher::{self, WatchParserItem as WatchParserItemBase};



// Custom Item

impl From<CustomItemModel> for CustomItemBase {
	fn from(val: CustomItemModel) -> Self {
		CustomItemBase {
			id: Some(val.id),
			title: val.title,
			match_url: val.match_url,
			description: val.description,
			search_opts: serde_json::from_str(&val.search_opts).unwrap()
		}
	}
}

impl From<CustomItemBase> for NewCustomItemModel {
	fn from(item: CustomItemBase) -> Self {
		Self {
			title: item.title,
			match_url: item.match_url,
			description: item.description,
			search_opts: serde_json::to_string(&item.search_opts).unwrap()
		}
	}
}

impl From<CustomItemBase> for EditCustomItemModel {
	fn from(item: CustomItemBase) -> Self {
		Self {
			title: Some(item.title),
			match_url: Some(item.match_url),
			description: Some(item.description),
			search_opts: Some(serde_json::to_string(&item.search_opts).unwrap())
		}
	}
}


pub fn create_custom_item(item: &NewCustomItemModel, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::custom_item::dsl::*;

	diesel::insert_into(custom_item).values(item).execute(conn)
}

pub fn get_custom_item_by_id(f_id: QueryId, conn: &SqliteConnection) -> QueryResult<CustomItemBase> {
	use self::custom_item::dsl::*;

	Ok(
		custom_item.find(f_id)
		.get_result::<CustomItemModel>(conn)?
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
		.get_result::<CustomItemModel>(conn)?
		.into()
	)
}

pub fn get_custom_items(conn: &SqliteConnection) -> QueryResult<Vec<CustomItemBase>> {
	Ok(
		self::custom_item::table.load::<CustomItemModel>(conn)?
		.into_iter()
		.map(|i| i.into())
		.collect()
	)
}


// Grouped Filter
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FilterGrouping {
	pub filter: Filter,
	pub feeds: Vec<QueryId>
}


// Filters

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Filter {
	pub id: QueryId,
	pub title: String,
	pub filter: FilterType
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewFilter {
	pub title: String,
	pub filter: FilterType
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditFilter {
	pub title: Option<String>,
	pub filter: Option<FilterType>
}

impl From<EditFilter> for EditFilterModel {
	fn from(filter: EditFilter) -> Self {
		EditFilterModel {
			title: filter.title,
			filter: filter.filter.map(|v| serde_json::to_string(&v).unwrap())
		}
	}
}

impl From<NewFilter> for NewFilterModel {
	fn from(filter: NewFilter) -> Self {
		NewFilterModel {
			title: filter.title,
			filter: serde_json::to_string(&filter.filter).unwrap()
		}
	}
}

impl From<FilterModel> for Filter {
	fn from(filter: FilterModel) -> Self {
		Filter {
			id: filter.id,
			title: filter.title,
			filter: serde_json::from_str(&filter.filter).unwrap()
		}
	}
}


pub fn create_filter(f_filter: NewFilter, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::filters::dsl::*;

	diesel::insert_into(filters)
	.values(NewFilterModel::from(f_filter))
	.execute(conn)
}

pub fn update_filter(f_id: QueryId, f_feed: EditFilter, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::filters::dsl::*;

	diesel::update(self::filters::table.filter(id.eq(f_id)))
	.set(EditFilterModel::from(f_feed))
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

pub fn get_filters_for_feed(f_feed_id: QueryId, conn: &SqliteConnection) -> QueryResult<Vec<Filter>> {
	use self::filters::dsl::*;

	let filter_ids: Vec<QueryId> = get_filters_from_feed_id(f_feed_id, conn)?
		.into_iter()
		.map(|f| f.filter_id)
		.collect();

	filters.filter(id.eq_any(filter_ids))
		.get_results::<FilterModel>(conn)
		.map(|f| f.into_iter().map(Filter::from).collect())
}

pub fn get_filter(f_filter_id: QueryId, conn: &SqliteConnection) -> QueryResult<Filter> {
	use self::filters::dsl::*;

	filters.filter(id.eq(f_filter_id))
		.get_result::<FilterModel>(conn)
		.map(Filter::from)
}

pub fn get_filters(conn: &SqliteConnection) -> QueryResult<Vec<Filter>> {
	use self::filters::dsl::*;

	filters.filter(id.ne(0))
		.get_results::<FilterModel>(conn)
		.map(|v| v.into_iter().map(Filter::from).collect())
}



// Feed Filter Listeners.

pub fn get_feed_filters(conn: &SqliteConnection) -> QueryResult<Vec<FeedFilterModel>> {
	use self::feed_filters::dsl::*;

	feed_filters.filter(id.ne(0)).get_results(conn)
}

pub fn get_filters_from_feed_id(f_feed_id: QueryId, conn: &SqliteConnection) -> QueryResult<Vec<FeedFilterModel>> {
	use self::feed_filters::dsl::*;

	feed_filters.filter(feed_id.eq(f_feed_id)).get_results(conn)
}

pub fn get_feed_filters_from_filter_id(f_filter_id: QueryId, conn: &SqliteConnection) -> QueryResult<Vec<FeedFilterModel>> {
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
		.values(NewFeedFilterModel {
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

pub fn get_item_total(search_query: Option<&str>, category_id: Option<QueryId>, conn: &SqliteConnection) -> QueryResult<i64> {
	use self::items::dsl::*;

	match category_id {
		Some(cat_id) => {
			let feeds = get_category_feeds(cat_id, conn)?;
			let feed_ids: Vec<QueryId> = feeds.iter().map(|f| f.id).collect();

			match search_query {
				Some(val) => {
					self::items::table
						.filter(feed_id.eq_any(feed_ids))
						.filter(title.like(format!("%{}%", val)))
						.count()
						.get_result(conn)
				}

				None => {
					self::items::table
						.filter(feed_id.eq_any(feed_ids))
						.count()
						.get_result(conn)
				}
			}
		}

		None => {
			match search_query {
				Some(val) => self::items::table.filter(title.like(format!("%{}%", val))).count().get_result(conn),
				None => self::items::table.count().get_result(conn),
			}
		}
	}
}

pub fn get_items_in_range(search_query: Option<&str>, category_id: Option<QueryId>, item_count: i64, skip_count: i64, conn: &SqliteConnection) -> QueryResult<Vec<FeedItemModel>> {
	use self::items::dsl::*;

	match category_id {
		Some(cat_id) => {
			let feeds = get_category_feeds(cat_id, conn)?;
			let feed_ids: Vec<QueryId> = feeds.iter().map(|f| f.id).collect();

			match search_query {
				Some(val) => {
					self::items::table
						.filter(feed_id.eq_any(feed_ids))
						.filter(title.like(format!("%{}%", val)))
						.limit(item_count)
						.offset(skip_count)
						.order(self::items::dsl::date.desc())
						.load(conn)
				}

				None => {
					self::items::table
						.filter(feed_id.eq_any(feed_ids))
						.limit(item_count)
						.offset(skip_count)
						.order(self::items::dsl::date.desc())
						.load(conn)
				}
			}
		}

		None => {
			match search_query {
				Some(val) => {
					self::items::table
						.filter(title.like(format!("%{}%", val)))
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


impl From<&RssItem> for NewFeedItemModel {
	fn from(item: &RssItem) -> NewFeedItemModel {
		let mut new_item = NewFeedItemModel {
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

impl From<&AtomItem> for NewFeedItemModel {
	fn from(item: &AtomItem) -> NewFeedItemModel {
		let mut new_item = NewFeedItemModel {
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

impl From<CustomFoundItem> for NewFeedItemModel {
	fn from(item: CustomFoundItem) -> NewFeedItemModel {
		let mut new_item = NewFeedItemModel {
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

pub fn get_listeners(conn: &SqliteConnection) -> QueryResult<Vec<FeedModel>> {
	self::feeds::table.load(conn)
}


pub fn get_listener_by_id(f_id: QueryId, conn: &SqliteConnection) -> QueryResult<Option<FeedModel>> {
	use self::feeds::dsl::*;
	feeds.filter(id.eq(f_id)).get_result(conn).optional()
}

pub fn remove_listener(f_id: QueryId, rem_stored: bool, conn: &SqliteConnection) -> QueryResult<usize> {
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

pub fn update_listener(f_id: QueryId, edit: &EditFeedModel, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::feeds::dsl::*;

	diesel::update(feeds.filter(id.eq(f_id)))
		.set(edit)
		.execute(conn)
}



// Categories

pub fn get_categories(conn: &SqliteConnection) -> QueryResult<Vec<CategoryModel>> {
	self::categories::table.load(conn)
}

pub fn get_category(cat_id: QueryId, conn: &SqliteConnection) -> QueryResult<CategoryModel> {
	use self::categories::dsl::*;

	categories.filter(id.eq(cat_id)).get_result(conn)
}

pub fn create_category(category: &NewCategoryModel, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::categories::dsl::*;
	diesel::insert_into(categories).values(category).execute(conn)
}

pub fn remove_category(cat_id: QueryId, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::categories::dsl::*;

	diesel::delete(categories.filter(id.eq(cat_id))).execute(conn)
}

pub fn update_category(c_id: QueryId, edit: &EditCategoryModel, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::categories::dsl::*;

	diesel::update(categories.filter(id.eq(c_id)))
		.set(edit)
		.execute(conn)
}



// Category Feeds

pub fn create_category_feed(category: &NewFeedCategoryModel, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::feed_categories::dsl::*;
	diesel::insert_into(feed_categories).values(category).execute(conn)
}

pub fn remove_category_feed(f_id: QueryId, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::feed_categories::dsl::*;
	diesel::delete(feed_categories.filter(id.eq(f_id))).execute(conn)
}


pub fn get_feed_categories(conn: &SqliteConnection) -> QueryResult<Vec<FeedCategoryModel>> {
	self::feed_categories::table.load(conn)
}

pub fn get_category_feeds(cat_id: QueryId, conn: &SqliteConnection) -> QueryResult<Vec<FeedCategoryModel>> {
	use self::feed_categories::dsl::*;

	feed_categories.filter(category_id.eq(cat_id)).get_results(conn)
}


// =================
// ==== WATCHER ====
// =================


// Watchers

pub fn get_watchers(conn: &SqliteConnection) -> QueryResult<Vec<WatchingModel>> {
	self::watching::table.load(conn)
}

pub fn get_watcher_by_url(f_url: &str, conn: &SqliteConnection) -> QueryResult<WatchingModel> {
	use self::watching::dsl::*;

	watching.filter(url.eq(f_url)).get_result(conn)
}

pub fn get_watcher_by_id(f_id: QueryId, conn: &SqliteConnection) -> QueryResult<WatchingModel> {
	use self::watching::dsl::*;

	watching.filter(id.eq(f_id)).get_result(conn)
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

pub fn update_watcher(f_id: QueryId, edit: &EditWatchingModel, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::watching::dsl::*;

	diesel::update(watching.filter(id.eq(f_id)))
		.set(edit)
		.execute(conn)
}

pub fn create_watcher(watcher: &NewWatchingModel, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::watching::dsl::*;

	diesel::insert_into(watching).values(watcher).execute(conn)
}



// Watch Parser

impl From<WatchParserItemModel> for WatchParserItemBase {
	fn from(val: WatchParserItemModel) -> Self {
		WatchParserItemBase {
			id: Some(val.id),
			title: val.title,
			match_url: val.match_url,
			description: val.description,
			match_opts: serde_json::from_str(&val.match_opts).unwrap()
		}
	}
}

pub fn create_watch_parser(item: &NewWatchParserItemModel, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::watch_parser::dsl::*;

	diesel::insert_into(watch_parser).values(item).execute(conn)
}

pub fn update_watch_parser(f_id: QueryId, item: &EditWatchParserItemModel, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::watch_parser::dsl::*;

	diesel::update(watch_parser.filter(id.eq(f_id))).set(item).execute(conn)
}

pub fn delete_watch_parser(f_id: QueryId, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::watch_parser::dsl::*;

	diesel::delete(watch_parser.filter(id.eq(f_id))).execute(conn)
}


pub fn get_watch_parser_by_id(f_id: QueryId, conn: &SqliteConnection) -> QueryResult<WatchParserItemBase> {
	use self::watch_parser::dsl::*;

	Ok(
		watch_parser.find(f_id)
		.get_result::<WatchParserItemModel>(conn)?
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
		.get_result::<WatchParserItemModel>(conn)?
		.into()
	)
}

pub fn get_watch_parsers(conn: &SqliteConnection) -> QueryResult<Vec<WatchParserItemBase>> {
	Ok(
		self::watch_parser::table.load::<WatchParserItemModel>(conn)?
		.into_iter()
		.map(|i| i.into())
		.collect()
	)
}



// Watch Item History
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WatchHistoryBase {
	pub id: QueryId,

	pub watch_id: QueryId,
	pub items: Vec<watcher::FoundItem>,

	pub date_added: i64
}

impl From<WatchHistoryModel> for WatchHistoryBase {
	fn from(history: WatchHistoryModel) -> Self {
		WatchHistoryBase {
			id: history.id,

			watch_id: history.watch_id,
			items: serde_json::from_str(&history.items).unwrap(),

			date_added: history.date_added
		}
	}
}

pub fn get_watch_history_count_since(since: i64, conn: &SqliteConnection) -> QueryResult<i64> {
	self::watch_history::table
		.filter(self::watch_history::dsl::date_added.gt(since))
		.count()
		.get_result(conn)
}

pub fn get_last_watch_history(f_watch_id: QueryId, conn: &SqliteConnection) -> QueryResult<Option<WatchHistoryBase>> {
	use self::watch_history::dsl::*;

	watch_history
	.filter(watch_id.eq(f_watch_id))
	.order_by(date_added.desc())
	.get_result::<WatchHistoryModel>(conn)
	.map(WatchHistoryBase::from)
	.optional()
}

pub fn get_last_watch_history_list(f_watch_id: QueryId, conn: &SqliteConnection) -> QueryResult<Vec<WatchHistoryBase>> {
	use self::watch_history::dsl::*;

	watch_history
	.filter(watch_id.eq(f_watch_id))
	.order_by(date_added.desc())
	.get_results::<WatchHistoryModel>(conn)
	.map(|i| i.into_iter().map(WatchHistoryBase::from).collect())
}

pub fn get_watch_history_list(f_watch_id: Option<QueryId>, item_count: i64, skip_count: i64, conn: &SqliteConnection) -> QueryResult<Vec<WatchHistoryBase>> {
	use self::watch_history::dsl::*;

	match f_watch_id {
		Some(f_watch_id) => {
			self::watch_history::table
				.filter(watch_id.eq(f_watch_id))
				.limit(item_count)
				.offset(skip_count)
				.order(date_added.desc())
				.load::<WatchHistoryModel>(conn)
				.map(|i| i.into_iter().map(WatchHistoryBase::from).collect())
		}

		None => {
			self::watch_history::table
				.limit(item_count)
				.offset(skip_count)
				.order(date_added.desc())
				.load::<WatchHistoryModel>(conn)
				.map(|i| i.into_iter().map(WatchHistoryBase::from).collect())
		}
	}
}


pub fn create_last_watch_history(item: &NewWatchHistoryModel, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::watch_history::dsl::*;

	diesel::insert_into(watch_history).values(item).execute(conn)
}



// Request History

pub fn insert_request_history(resp: &RequestResponse, conn: &SqliteConnection) -> QueryResult<()> {
	// Join request batches together (ex. RSS Feed Watcher & Changes)
	// TODO: Remove tuple.
	let (req_history_group, items_found) = resp.results.iter()
		.fold(
			(NewRequestHistoryGroupModel {
				is_manual: false,
				concurrency: 0,
				start_time: i64::MAX,
				duration: 0
			}, 0),
			|(mut model, mut items_found), res| {
				match res {
					RequestResults::Feed(item) => {
						model.is_manual = item.was_manual;
						model.concurrency = item.concurrency;

						let start_time = item.start_time.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
						model.duration += item.duration.as_millis() as i32;

						if start_time < model.start_time {
							model.start_time = start_time;
						}

						items_found += item.items.len();
					}

					RequestResults::Watcher(item) => {
						model.is_manual = item.was_manual;
						model.concurrency = item.concurrency;

						let start_time = item.start_time.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
						model.duration += item.duration.as_millis() as i32;

						if start_time < model.start_time {
							model.start_time = start_time;
						}

						items_found += item.items.len();
					}
				}

				(model, items_found)
			}
		);

	if items_found == 0 {
		return Ok(());
	}

	create_request_history_group(&req_history_group, conn)?;
	let group_id = get_request_history_group_by_start_time(req_history_group.start_time, conn)?.id;

	for res in &resp.results {
		match res {
			RequestResults::Feed(item) => {
				let values = item.items.iter()
					.map(|v| {
						match  &v.results {
							Ok(res) => NewRequestHistoryItemModel {
								group_id,
								new_items: Some(res.new_item_count as i32),
								start_time: Some(res.start_time.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64),
								duration: Some(res.duration.as_millis() as i32),
								error: None,

								feed_id: Some(v.item.id),
								watch_id: None
							},
							Err(e) => NewRequestHistoryItemModel {
								group_id,
								new_items: None,
								start_time: None,
								duration: None,
								error: Some(e.to_string()),

								feed_id: Some(v.item.id),
								watch_id: None
							}
						}
					})
					.collect::<Vec<_>>();

				insert_request_history_items(&values, conn)?;
			}

			RequestResults::Watcher(item) => {
				let values = item.items.iter()
					.map(|v| {
						match &v.results {
							Ok(res) => NewRequestHistoryItemModel {
								group_id,
								new_items: Some(res.new_item_count as i32),
								start_time: Some(res.start_time.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64),
								duration: Some(res.duration.as_millis() as i32),
								error: None,

								feed_id: None,
								watch_id: Some(v.item.id)
							},
							Err(e) => NewRequestHistoryItemModel {
								group_id,
								new_items: None,
								start_time: None,
								duration: None,
								error: Some(e.to_string()),

								feed_id: None,
								watch_id: Some(v.item.id)
							}
						}
					})
					.collect::<Vec<_>>();

				insert_request_history_items(&values, conn)?;
			}
		}
	}

	Ok(())
}


pub fn create_request_history_group(value: &NewRequestHistoryGroupModel, conn: &SqliteConnection) -> QueryResult<usize> {
	use self::request_history_group::dsl::*;

	diesel::insert_into(request_history_group).values(value).execute(conn)
}

pub fn get_request_history_group_by_start_time(value: i64, conn: &SqliteConnection) -> QueryResult<RequestHistoryGroupModel> {
	use self::request_history_group::dsl::*;

	self::request_history_group::table.filter(start_time.eq(value)).get_result(conn)
}

pub fn get_request_history_groups(item_count: i64, skip_count: i64, conn: &SqliteConnection) -> QueryResult<Vec<RequestHistoryGroupModel>> {
	use self::request_history_group::dsl::*;

	self::request_history_group::table.filter(id.gt(0)).order(id.desc()).limit(item_count).offset(skip_count).get_results(conn)
}

pub fn count_request_history_groups(conn: &SqliteConnection) -> QueryResult<i64> {
	self::request_history_group::table.count().get_result(conn)
}




pub fn insert_request_history_items(records: &[NewRequestHistoryItemModel], conn: &SqliteConnection) -> QueryResult<usize> {
	use self::request_history_item::dsl::*;

	diesel::insert_into(request_history_item).values(records).execute(conn)
}

pub fn get_request_history_multiple_group_items(f_group_ids: &[QueryId], conn: &SqliteConnection) -> QueryResult<Vec<RequestHistoryItemModel>> {
	use self::request_history_item::dsl::*;

	self::request_history_item::table.filter(group_id.eq_any(f_group_ids)).get_results(conn)
}

pub fn get_request_history_group_items(f_group_id: QueryId, conn: &SqliteConnection) -> QueryResult<Vec<RequestHistoryItemModel>> {
	use self::request_history_item::dsl::*;

	self::request_history_item::table.filter(group_id.eq(f_group_id)).get_results(conn)
}