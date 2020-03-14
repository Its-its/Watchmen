use std::io::Read;

use crate::Result;

use super::NewFeed;


pub type FeedResult = Result<atom_syndication::Feed>;


pub fn new_from_feed(url: String, feed: atom_syndication::Feed) -> NewFeed {
	NewFeed {
		url: url,

		title: feed.title().to_string(),
		description: feed.subtitle().unwrap_or_default().to_string(),
		generator: feed.generator().unwrap_or(&atom_syndication::Generator::default()).value().to_string(),

		feed_type: 1,

		sec_interval: 60 * 5,
		remove_after: 0,

		global_show: true,
		ignore_if_not_new: true,

		date_added: chrono::Utc::now().naive_utc().timestamp(),
		last_called: chrono::Utc::now().naive_utc().timestamp(),
	}
}


pub fn get_from_url(url: &str) -> Result<atom_syndication::Feed> {
	let mut content = Vec::new();

	let mut resp = reqwest::get(url)?;
	resp.read_to_end(&mut content)?;

	Ok(atom_syndication::Feed::read_from(&content[..])?)
}