use serde::{Serialize, Deserialize};

use crate::Filter;
use crate::request::feeds::custom::{UpdateableCustomItem, CustomItem};

use super::models::{
	QueryId,
	Item as FeedItem,
	Feed,
	NewFeed,
	EditFeed,
	Category,
	FeedCategory,
	NewCategory,
	NewFeedCategory,
	EditCategory,
	NewWatching, EditWatching, Watching,
	WatchHistory
};

use super::objects::{
	NewFilterModel,
	FilterGrouping
};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Empty {}


// Front End -> Core
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "method", content = "params")]
pub enum Front2CoreNotification {
	/// Add something else to listen to.
	AddListener {
		url: String,
		custom_item_id: Option<i32>
	},

	RemoveListener {
		id: QueryId,
		#[serde(default)]
		rem_stored: bool
	},

	EditListener {
		id: QueryId,
		editing: EditFeed
	},

	//
	CategoryList(Empty),


	AddCategory {
		name: String,
		position: i32
	},

	RemoveCategory {
		id: QueryId
	},

	EditCategory {
		id: QueryId,
		editing: EditCategory
	},


	AddFeedCategory {
		feed_id: QueryId,
		category_id: QueryId
	},

	RemoveFeedCategory {
		id: QueryId
	},


	ItemList {
		category_id: Option<QueryId>,

		#[serde(default = "default_items")]
		item_count: i64,
		#[serde(default)]
		skip_count: i64
	},

	FeedList(Empty),

	/// Returns updates `since` time.
	FeedUpdates {
		since: i64
	},


	// Scraper Editor

	GetWebpage {
		url: String
	},


	// Custom Items

	CustomItemList(Empty),

	NewCustomItem {
		item: CustomItem
	},

	UpdateCustomItem {
		id: QueryId,
		item: UpdateableCustomItem
	},

	// Feed Filter

	NewFeedFilter {
		feed_id: QueryId,
		filter_id: QueryId
	},

	RemoveFeedFilter {
		feed_id: QueryId,
		filter_id: QueryId
	},

	// Filter

	FilterList(Empty),

	UpdateFilter {
		id: QueryId,
		title: String,
		filter: Filter
	},

	NewFilter {
		title: String,
		filter: Filter
	},

	RemoveFilter {
		id: QueryId
	},


	// Watching

	WatcherList(Empty),

	AddWatcher {
		url: String,
		custom_item_id: Option<i32>
	},

	RemoveWatcher {
		id: QueryId,
		#[serde(default = "default_true")]
		rem_stored: bool
	},

	EditWatcher {
		id: QueryId,
		editing: EditWatching
	},
}


// Core -> Front End
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "method", content = "params")]
pub enum Core2FrontNotification {
	NewListener {
		listener: NewFeed,
		affected: usize
	},

	RemoveListener {
		affected: usize
	},

	EditListener {
		listener: EditFeed,
		affected: usize
	},


	ItemList {
		items: Vec<FeedItem>,
		notification_ids: Vec<QueryId>,

		item_count: i64,
		skip_count: i64,

		total_items: i64
	},

	FeedList {
		items: Vec<Feed>
	},

	//

	FeedUpdates {
		since: i64,

		new_feeds: i64,
		new_watches: i64
	},


	CategoryList {
		categories: Vec<Category>,
		category_feeds: Vec<FeedCategory>
	},

	NewCategory {
		category: NewCategory,
		affected: usize
	},

	RemoveCategory {
		affected: usize
	},

	EditCategory {
		category: EditCategory,
		affected: usize
	},


	NewFeedCategory {
		category: NewFeedCategory,
		affected: usize
	},

	RemoveFeedCategory {
		affected: usize
	},


	FeedFilterList {
		items: Vec<FilterGrouping>
	},

	EditFilter {
		affected: usize
	},

	NewFilter {
		filter: NewFilterModel,
		affected: usize
	},

	RemoveFilter {
		affected_filters: usize,
		affected_feeds: usize
	},


	LinkFeedAndFilter {
		affected: usize
	},

	RemoveFeedAndFilter {
		affected: usize
	},


	// Scaper Editor

	WebpageSource {
		html: String
	},

	// Custom Items

	CustomItemList {
		items: Vec<CustomItem>
	},

	NewCustomItem {
		item: CustomItem,
		affected: usize
	},


	WatcherList {
		items: Vec<(Watching, WatchHistory)>
	},

	NewWatcher {
		listener: NewWatching,
		affected: usize
	},

	RemoveWatcher {
		affected: usize
	},

	EditWatcher {
		listener: EditWatching,
		affected: usize
	},
}


// Defaults

fn default_items() -> i64 {
	50
}


fn default_true() -> bool {
	true
}