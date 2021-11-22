use serde::{Serialize, Deserialize};


use diesel::{Queryable, Insertable};

use super::schema::*;

pub type QueryId = i32;


// Item

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Identifiable)]
#[table_name = "items"]
pub struct FeedItemModel {
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
pub struct NewFeedItemModel {
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


// Feeds

#[derive(Serialize, Deserialize, Debug, Default, Clone, Queryable, Identifiable)]
#[table_name = "feeds"]
pub struct FeedModel {
	pub id: QueryId,

	pub enabled: bool,

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
pub struct NewFeedModel {
	pub enabled: bool,
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
pub struct EditFeedModel {
	pub enabled: Option<bool>,
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
pub struct CategoryModel {
	pub id: QueryId,

	pub position: i32,

	pub name: String,
	pub name_lowercase: String,

	pub date_added: i64,
}


#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[table_name = "categories"]
pub struct NewCategoryModel {
	pub position: i32,

	pub name: String,
	pub name_lowercase: String,

	pub date_added: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, AsChangeset)]
#[table_name = "categories"]
pub struct EditCategoryModel {
	pub position: Option<i32>,

	pub name: Option<String>,
	pub name_lowercase: Option<String>,

	pub date_added: Option<i64>,
}


// Feeds Categories

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Identifiable)]
#[table_name = "feed_categories"]
pub struct FeedCategoryModel {
	pub id: QueryId,

	pub feed_id: QueryId,
	pub category_id: QueryId
}


#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[table_name = "feed_categories"]
pub struct NewFeedCategoryModel {
	pub feed_id: QueryId,
	pub category_id: QueryId
}


//  Filters

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Identifiable)]
#[table_name = "filters"]
pub struct FilterModel {
	pub id: QueryId,

	pub title: String,

	pub filter: String
}


#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[table_name = "filters"]
pub struct NewFilterModel {
	pub title: String,
	pub filter: String
}

#[derive(Serialize, Deserialize, Debug, Clone, AsChangeset)]
#[table_name = "filters"]
pub struct EditFilterModel {
	pub title: Option<String>,
	pub filter: Option<String>
}


// Feed Filter

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Identifiable)]
#[table_name = "feed_filters"]
pub struct FeedFilterModel {
	pub id: QueryId,

	pub feed_id: QueryId,
	pub filter_id: QueryId
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[table_name = "feed_filters"]
pub struct NewFeedFilterModel {
	pub feed_id: QueryId,
	pub filter_id: QueryId
}


// Custom Item

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Identifiable)]
#[table_name = "custom_item"]
pub struct CustomItemModel {
	pub id: QueryId,

	pub title: String,
	pub match_url: String,
	pub description: String,

	pub search_opts: String
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[table_name = "custom_item"]
pub struct NewCustomItemModel {
	pub title: String,
	pub match_url: String,
	pub description: String,

	pub search_opts: String
}

#[derive(Serialize, Deserialize, Debug, Clone, AsChangeset)]
#[table_name = "custom_item"]
pub struct EditCustomItemModel {
	pub title: Option<String>,
	pub match_url: Option<String>,
	pub description: Option<String>,

	pub search_opts: Option<String>
}



// =================
// ==== WATCHER ====
// =================

#[derive(Serialize, Deserialize, Debug, Default, Clone, Queryable, Identifiable)]
#[table_name = "watching"]
pub struct WatchingModel {
	pub id: QueryId,

	pub enabled: bool,

	pub parser_id: Option<QueryId>,

	pub url: String,
	pub title: String,
	pub description: String,

	pub sec_interval: i32,
	pub remove_after: i32,

	pub date_added: i64,
	pub last_called: i64,
}


#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[table_name = "watching"]
pub struct NewWatchingModel {
	pub parser_id: Option<QueryId>,

	pub enabled: bool,

	pub url: String,
	pub title: String,
	pub description: String,

	pub sec_interval: i32,
	pub remove_after: i32,

	pub date_added: i64,
	pub last_called: i64,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, AsChangeset)]
#[table_name = "watching"]
pub struct EditWatchingModel {
	pub parser_id: Option<QueryId>,

	pub enabled: Option<bool>,

	pub url: Option<String>,
	pub title: Option<String>,
	pub description: Option<String>,

	pub sec_interval: Option<i32>,
	pub remove_after: Option<i32>,
}




#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Identifiable)]
#[table_name = "watch_history"]
pub struct WatchHistoryModel {
	pub id: QueryId,

	pub watch_id: QueryId,
	pub items: String,

	pub date_added: i64
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[table_name = "watch_history"]
pub struct NewWatchHistoryModel {
	pub watch_id: QueryId,
	pub items: String,

	pub date_added: i64
}




#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Identifiable)]
#[table_name = "watch_parser"]
pub struct WatchParserItemModel {
	pub id: QueryId,

	pub title: String,
	pub match_url: String,
	pub description: String,

	pub match_opts: String
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[table_name = "watch_parser"]
pub struct NewWatchParserItemModel {
	pub title: String,
	pub match_url: String,
	pub description: String,

	pub match_opts: String
}

#[derive(Serialize, Deserialize, Debug, Clone, AsChangeset)]
#[table_name = "watch_parser"]
pub struct EditWatchParserItemModel {
	pub title: Option<String>,
	pub match_url: Option<String>,
	pub description: Option<String>,

	pub match_opts: Option<String>
}



// Request History


#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Identifiable)]
#[table_name = "request_history_group"]
pub struct RequestHistoryGroupModel {
	pub id: QueryId,

	pub is_manual: bool,
	pub concurrency: i32,

	pub start_time: i64,
	pub duration: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[table_name = "request_history_group"]
pub struct NewRequestHistoryGroupModel {
	pub is_manual: bool,
	pub concurrency: i32,

	pub start_time: i64,
	pub duration: i32,
}



#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Identifiable)]
#[table_name = "request_history_item"]
pub struct RequestHistoryItemModel {
	pub id: QueryId,

	pub group_id: QueryId,

	pub feed_id: Option<QueryId>,
	pub watch_id: Option<QueryId>,

	pub new_items: Option<i32>,

	pub start_time: Option<i64>,
	pub duration: Option<i32>,

	pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[table_name = "request_history_item"]
pub struct NewRequestHistoryItemModel {
	pub group_id: QueryId,

	pub feed_id: Option<QueryId>,
	pub watch_id: Option<QueryId>,

	pub new_items: Option<i32>,
	pub start_time: Option<i64>,
	pub duration: Option<i32>,

	pub error: Option<String>,
}