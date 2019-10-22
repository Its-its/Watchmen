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
		// Feed Items

		self.0.execute(
			"CREATE TABLE IF NOT EXISTS items (
				id          INTEGER PRIMARY KEY,

				guid        TEXT,
				title       TEXT,
				author      TEXT,
				content     TEXT,
				link        TEXT,
				date        LONG NOT NULL,
				hash        TEXT,

				date_added  LONG NOT NULL,
				is_read     BOOL NOT NULL DEFAULT false,
				is_starred  BOOL NOT NULL DEFAULT false,
				is_removed  BOOL NOT NULL DEFAULT false,
				tags        TEXT NOT NULL,
				feed_id     INTEGER NOT NULL
			)"
		)?;

		self.0.execute(
			"CREATE UNIQUE INDEX IF NOT EXISTS items_hash on items ( hash )"
		)?;


		// Feeds

		self.0.execute(
			"CREATE TABLE IF NOT EXISTS feeds (
				id                 INTEGER PRIMARY KEY,

				url                TEXT NOT NULL,
				title              TEXT NOT NULL,
				description        TEXT NOT NULL,
				generator          TEXT NOT NULL,

				type               INTEGER NOT NULL,

				sec_interval       INTEGER NOT NULL,
				remove_after       INTEGER NOT NULL DEFAULT 0,

				ignore_if_not_new  BOOL NOT NULL DEFAULT true,

				global_show        BOOL NOT NULL DEFAULT true,

				date_added         LONG NOT NULL,
				last_called        LONG NOT NULL
			)"
		)?;

		self.0.execute(
			"CREATE UNIQUE INDEX IF NOT EXISTS feeds_url on feeds ( url )"
		)?;


		// Categories
		// To be able to store feeds in a category.

		self.0.execute(
			"CREATE TABLE IF NOT EXISTS categories (
				id                 INTEGER PRIMARY KEY,

				position           INTEGER NOT NULL,

				name               TEXT NOT NULL,
				name_lowercase     TEXT NOT NULL,

				date_added         LONG NOT NULL
			)"
		)?;

		self.0.execute(
			"CREATE UNIQUE INDEX IF NOT EXISTS categories_name on categories ( name_lowercase )"
		)?;

		// Category store for Feeds
		// Registers what categories a feed is stored since no arrays exist in sqlite.

		self.0.execute(
			"CREATE TABLE IF NOT EXISTS feed_categories (
				id               INTEGER PRIMARY KEY,

				feed_id          INTEGER NOT NULL,
				category_id      INTEGER NOT NULL
			)"
		)?;

		Ok(())
	}

	pub fn connection(&self) -> &SqliteConnection {
		&self.0
	}
}