use chrono::{DateTime, Utc, NaiveDateTime};

use diesel::{Queryable, Insertable};

use super::schema::items;


pub type QueryId = i32;

// Item

#[derive(Queryable)]
pub struct Item {
	pub id: QueryId,

	// Pre-defined
	pub guid: String, // pre-defined guild OR pre-defined link OR self.hash
	pub title: String,
	pub authors: String,
	pub content: String,
	pub link: String,
	pub date: NaiveDateTime,
	pub hash: String, // md5(link + title + authors + content + tags) | Iffy on tags. If tags change then hash needs to change.

	// User defined
	pub date_added: NaiveDateTime,
	pub is_read: bool,
	pub is_starred: bool,
	pub tags: String,
	pub feed_id: QueryId
}

#[derive(Insertable)]
#[table_name = "items"]
pub struct NewItem<'a> {
	pub guid: &'a str, // pre-defined guild OR pre-defined link OR self.hash
	pub title: &'a str,
	pub authors: &'a str,
	pub content: &'a str,
	pub link: &'a str,
	pub date: &'a NaiveDateTime,
	pub hash: &'a str, // md5(link + title + authors + content + tags) | Iffy on tags. If tags change then hash needs to change.

	// User defined
	pub date_added: &'a NaiveDateTime,
	pub is_read: bool,
	pub is_starred: bool,
	pub tags: &'a str,
	pub feed_id: QueryId
}