use reqwest::Client;

use crate::Result;

use super::NewFeedModel;


pub type FeedResult = Result<atom_syndication::Feed>;


pub fn new_from_feed(url: String, feed: atom_syndication::Feed) -> NewFeedModel {
	NewFeedModel {
		url,

		enabled: true,

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


pub async fn get_from_url(url: &str, req_client: &Client) -> Result<atom_syndication::Feed> {
	let resp = req_client.get(url).send().await?.bytes().await?;
	Ok(atom_syndication::Feed::read_from(&resp[..])?)
}