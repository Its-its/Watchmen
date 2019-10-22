// TODO: Multiple notifications. Add ability to host specific notifications on process. Example: Telegram, Discord.
// Web, Email, Telegram, Discord, etc..

pub mod browser;

pub enum Notification {
	Browser,
	Telegram,
}

impl Notification {
	//
}