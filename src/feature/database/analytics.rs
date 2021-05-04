#![allow(dead_code)]

use serde::{Serialize, Deserialize};

use super::QueryId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Analytics {
	Request(Request),
	Updates(Updates)
}

pub struct AnalyticsModel {
	name: String,

	json: String,

	created_at: i64
}

impl From<Analytics> for AnalyticsModel {
	fn from(ana: Analytics) -> Self {
		let name = match ana {
			Analytics::Request(_) => "request",
			Analytics::Updates(_) => "updates"
		};

		AnalyticsModel {
			name: String::from(name),
			json: serde_json::to_string(&ana).unwrap(),
			created_at: chrono::Utc::now().timestamp()
		}
	}
}



/// Updates Analytics
/// Daily (configurable?) global update analytics.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Updates {
	/// How many times watchers have updated
	watch_updates: i32,

	/// Total added
	feeds_additions: i32,
	/// Total filtered through
	filter_filter_amount: i32,

	/// Total requests sent.
	request_amount: i32
}



// Request Analytics
// Created whenever a request is sent.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RequestReason {
	Auto,
	Manual
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RequestBy {
	Feed(QueryId),
	Watcher(QueryId)
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request {
	dispach_reason: RequestReason,
	dispached_for: RequestBy,

	response_code: i32,
	error_str: Option<String>,

	time_took: i32
}