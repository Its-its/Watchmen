use serde_json::Value;

use crate::types::MessageId;
use crate::Result;


pub mod database;

#[cfg(feature = "website")]
pub mod frontend;

pub mod cmdline;

pub mod library;

pub mod rpc;


pub use database::*;
#[cfg(feature = "website")]
pub use frontend::*;

pub use cmdline::*;

pub use library::*;

pub use rpc::{Front2CoreNotification, Core2FrontNotification};



pub trait ResponseWrapper {
	fn respond(&mut self, message_id: Option<MessageId>, response: Result<Value>);

	fn respond_with(&mut self, message_id: Option<MessageId>, response: Core2FrontNotification) {
		match message_id {
			Some(mid) => self.respond_request(mid, response),
			None => self.respond_notification(response)
		}
	}

	fn respond_request(&mut self, message_id: MessageId, response: Core2FrontNotification) {
		self.respond_request_value(message_id, serde_json::to_value(response).map_err(|e| e.into()))
	}

	fn respond_notification(&mut self, response: Core2FrontNotification) {
		self.respond_notification_value(serde_json::to_value(response).map_err(|e| e.into()))
	}


	fn respond_request_value(&mut self, message_id: MessageId, response: Result<Value>) {
		self.respond(Some(message_id), response);
	}

	fn respond_notification_value(&mut self, response: Result<Value>) {
		self.respond(None, response);
	}
}