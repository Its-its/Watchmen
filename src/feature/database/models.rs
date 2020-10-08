use serde::{Serialize, Deserialize};


use diesel::{Queryable, Insertable};

use super::schema::*;

pub type QueryId = i32;


// Item

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Identifiable)]
#[table_name = "items"]
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


// Feeds

#[derive(Serialize, Deserialize, Debug, Default, Clone, Queryable, Identifiable)]
#[table_name = "feeds"]
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
#[table_name = "filters"]
pub struct FilterDB {
	pub id: QueryId,

	pub title: String,

	pub filter: String
}


#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[table_name = "filters"]
pub struct NewFilterDB {
	pub title: String,
	pub filter: String
}

#[derive(Serialize, Deserialize, Debug, Clone, AsChangeset)]
#[table_name = "filters"]
pub struct EditFilterDB {
	pub title: Option<String>,
	pub filter: Option<String>
}


// Feed Filter

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Identifiable)]
#[table_name = "feed_filters"]
pub struct FeedFilterDB {
	pub id: QueryId,

	pub feed_id: QueryId,
	pub filter_id: QueryId
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[table_name = "feed_filters"]
pub struct NewFeedFilter {
	pub feed_id: QueryId,
	pub filter_id: QueryId
}


// Custom Item

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Identifiable)]
#[table_name = "custom_item"]
pub struct CustomItem {
	pub id: QueryId,

	pub title: String,
	pub match_url: String,
	pub description: String,

	pub search_opts: String
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[table_name = "custom_item"]
pub struct NewCustomItem {
	pub title: String,
	pub match_url: String,
	pub description: String,

	pub search_opts: String
}

#[derive(Serialize, Deserialize, Debug, Clone, AsChangeset)]
#[table_name = "custom_item"]
pub struct EditCustomItem {
	pub title: Option<String>,
	pub match_url: Option<String>,
	pub description: Option<String>,

	pub search_opts: Option<String>
}