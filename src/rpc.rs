use serde::{Serialize, Deserialize};

use crate::types::MessageId;
use crate::frontend::rpc::Front2CoreNotification;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "method", content = "params")]
pub enum Object2CoreNotification {
	Plugin {
		// Keeping the possibility of plugins.
	},

	Frontend {
		message_id: Option<MessageId>,
		command: Front2CoreNotification
	}
}