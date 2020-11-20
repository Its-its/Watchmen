use std::io::Read;

use crate::Result;

use super::NewFeedModel;


pub type FeedResult = Result<rss::Channel>;


pub fn new_from_feed(url: String, feed: rss::Channel) -> NewFeedModel {
	NewFeedModel {
		url,

		enabled: true,

		title: feed.title().to_string(),
		description: feed.description().to_string(),
		generator: feed.generator().unwrap_or_default().to_string(),

		feed_type: 0,

		sec_interval: 60 * 5,
		remove_after: 0,

		global_show: true,
		ignore_if_not_new: true,

		date_added: chrono::Utc::now().naive_utc().timestamp(),
		last_called: chrono::Utc::now().naive_utc().timestamp(),
	}
}


pub fn get_from_url(url: &str) -> Result<rss::Channel> {
	let mut content = Vec::new();

	let mut resp = reqwest::get(url)?;
	resp.read_to_end(&mut content)?;

	Ok(rss::Channel::read_from(&content[..])?)
}
