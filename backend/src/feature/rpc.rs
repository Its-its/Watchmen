use serde::{Serialize, Deserialize};

use crate::FilterType;
use crate::request::feeds::custom::{
	UpdateableCustomItem,
	CustomItem
};

use crate::request::watcher::{FoundItem, MatchParser, UpdateableWatchParser, WatchParserItem};

use super::models::{
	CategoryModel,
	EditCategoryModel,
	EditFeedModel,
	EditWatchingModel,
	FeedCategoryModel,
	FeedItemModel,
	FeedModel,
	NewCategoryModel,
	NewFeedCategoryModel,
	NewFeedModel,
	NewWatchingModel,
	QueryId,
	RequestHistoryGroupModel,
	RequestHistoryItemModel,
	WatchingModel
};

use super::objects::{
	NewFilter,
	FilterGrouping,
	WatchHistoryBase
};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Empty {}


// Front End -> Core
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "method", content = "params")]
pub enum Front2CoreNotification {
	// Dashboard

	/// Most recent first
	RequestHistoryList {
		#[serde(default = "default_items")]
		item_count: i64,
		#[serde(default)]
		skip_count: i64
	},

	RequestHistoryGroupItems {
		id: QueryId
	},


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
		editing: EditFeedModel
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
		editing: EditCategoryModel
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
		filter: FilterType
	},

	NewFilter {
		title: String,
		filter: FilterType
	},

	RemoveFilter {
		id: QueryId
	},

	// ================
	// === Watching ===
	// ================

	// Watcher

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
		editing: EditWatchingModel
	},


	// Parser
	WatchParserList(Empty),

	NewWatchParser {
		item: WatchParserItem
	},

	UpdateWatchParser {
		id: QueryId,
		item: UpdateableWatchParser
	},

	RemoveWatchParser {
		id: QueryId
	},


	// History
	WatchHistoryList {
		watch_id: Option<QueryId>,
		#[serde(default = "default_items")]
		item_count: i64,
		#[serde(default)]
		skip_count: i64
	},

	// Tests

	TestWatcher {
		url: String,

		parser: Option<MatchParser>
	}
}


// Core -> Front End
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "method", content = "params")]
pub enum Core2FrontNotification {
	// Dashboard
	RequestHistoryList {
		groups: Vec<RequestHistoryGroupModel>,
		items: Vec<RequestHistoryItemModel>,

		item_count: i64,
		skip_count: i64,

		total_items: i64
	},

	RequestHistoryGroupItemsList {
		group_id: QueryId,

		items: Vec<RequestHistoryItemModel>
	},


	NewListener {
		listener: NewFeedModel,
		affected: usize
	},

	RemoveListener {
		affected: usize
	},

	EditListener {
		listener: EditFeedModel,
		affected: usize
	},


	ItemList {
		items: Vec<FeedItemModel>,
		notification_ids: Vec<QueryId>,

		item_count: i64,
		skip_count: i64,

		total_items: i64
	},

	FeedList {
		items: Vec<FeedModel>
	},

	//

	FeedUpdates {
		since: i64,

		new_feeds: i64,
		new_watches: i64
	},


	CategoryList {
		categories: Vec<CategoryModel>,
		category_feeds: Vec<FeedCategoryModel>
	},

	NewCategory {
		category: NewCategoryModel,
		affected: usize
	},

	RemoveCategory {
		affected: usize
	},

	EditCategory {
		category: EditCategoryModel,
		affected: usize
	},


	NewFeedCategory {
		category: NewFeedCategoryModel,
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
		filter: NewFilter,
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
		items: Vec<(WatchingModel, Option<WatchHistoryBase>)>
	},

	NewWatcher {
		listener: NewWatchingModel,
		affected: usize
	},

	RemoveWatcher {
		affected: usize
	},

	EditWatcher {
		listener: EditWatchingModel,
		affected: usize
	},

	// Watch Parser
	WatchParserList {
		items: Vec<WatchParserItem>
	},

	NewWatchParser {
		item: WatchParserItem,
		affected: usize
	},

	UpdateWatchParser {
		item: UpdateableWatchParser,
		affected: usize
	},

	RemoveWatchParser {
		affected: usize
	},

	// Watch History
	WatchHistoryList {
		items: Vec<WatchHistoryBase>
	},

	// Test
	TestWatcher {
		success: bool,
		items: Vec<FoundItem>
	}
}


// Defaults

fn default_items() -> i64 {
	50
}


fn default_true() -> bool {
	true
}