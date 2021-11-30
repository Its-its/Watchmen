use std::time::{Duration, SystemTime};

use reqwest::header::HeaderMap;

use crate::feature::models::{
	FeedModel, NewFeedItemModel,
	WatchingModel, NewWatchHistoryModel
};

use crate::Result;

pub mod feeds;
pub mod watcher;


#[derive(Debug)]
pub enum RequestResults {
	Feed(InnerRequestResults<FeedModel, NewFeedItemModel>),
	Watcher(InnerRequestResults<WatchingModel, NewWatchHistoryModel>)
}


#[derive(Debug)]
pub struct InnerRequestResults<I, N> {
	pub general_error: Option<String>,
	pub was_manual: bool,
	pub start_time: SystemTime,
	pub duration: Duration,
	pub concurrency: i32,
	pub items: Vec<ItemResults<I, N>>
}

#[derive(Debug)]
pub struct RequestItemResults<I> {
	pub start_time: SystemTime,
	pub duration: Duration,
	pub new_item_count: usize,
	pub item_count: i32,
	pub to_insert: Vec<I>
}


#[derive(Debug)]
pub struct ItemResults<I, N> {
	pub item: I,
	pub results: Result<RequestItemResults<N>>
}


pub fn default_headers() -> HeaderMap {
	let mut headers = HeaderMap::default();
	headers.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; rv:91.0) Gecko/20100101 Firefox/91.0".parse().unwrap());
	headers.insert("Accept-Language", "en-US,en;q=0.5".parse().unwrap());
	headers.insert("Connection", "keep-alive".parse().unwrap());
	headers
}