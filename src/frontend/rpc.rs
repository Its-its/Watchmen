use serde::{Serialize, Deserialize};


use crate::types::ListenerId;


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

	RemoveListener { id: ListenerId },

	EditListener {
		id: ListenerId,
		//
	},

	Update {}
}

// Core -> Front End
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "method", content = "params")]
pub enum Core2FrontNotification {
	Init {  }
}