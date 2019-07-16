use crate::diesel::Connection as _Connection;

use diesel::prelude::*;

pub mod models;
pub mod schema;


pub struct Connection(pub SqliteConnection);


impl Connection {
	pub fn new() -> Self {
		let database_url = "app/feeder.db";

		Self(
			SqliteConnection::establish(&database_url)
			.unwrap_or_else(|_| panic!("DB Establishing Connection: {}", database_url))
		)
	}

	pub fn init_sql(&self) -> QueryResult<()> {
		self.0.execute(
			"CREATE TABLE IF NOT EXISTS items (
				id          INTEGER PRIMARY KEY,

				guid        TEXT,
				title       TEXT,
				authors     TEXT,
				content     TEXT,
				link        TEXT,
				date        TIMESTAMP NOT NULL DEFAULT current_timestamp,
				hash        TEXT,

				date_added  TIMESTAMP NOT NULL DEFAULT current_timestamp,
				is_read     BOOL NOT NULL DEFAULT false,
				is_starred  BOOL NOT NULL DEFAULT false,
				tags        TEXT NOT NULL,
				feed_id     INTEGER NOT NULL
			)"
		)?;

		Ok(())
	}
}