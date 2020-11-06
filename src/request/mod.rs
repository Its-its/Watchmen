

pub mod feeds;
pub mod watcher;


#[derive(Debug)]
pub enum RequestResults {
	Feed(feeds::FeedRequestResults),
	Watcher(watcher::WatcherRequestResults)
}