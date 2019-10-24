use serde::{Serialize, Deserialize};

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
	EditCategory
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
		url: String
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
		items: i64,
		#[serde(default)]
		skip: i64
	},

	FeedList(Empty),

	Updates {
		since: i64
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

		item_count: i64,
		skip_count: i64,

		total_items: i64
	},

	FeedList {
		items: Vec<Feed>
	},

	//

	Updates {
		since: i64,

		new_items: i64,
		notifications: i64
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

	//
}


// Defaults

fn default_items() -> i64 {
	50
}