use crate::diesel::Connection as _Connection;

use diesel::prelude::*;

pub mod models;
pub mod schema;
pub mod objects;
pub mod analytics;

pub use models::QueryId;


pub struct Connection(pub SqliteConnection);


impl Connection {
	pub fn new() -> Self {
		let database_url = "../app/feeder.db"; // TODO: Add to config.

		Self(
			SqliteConnection::establish(database_url)
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

				enabled            BOOL NOT NULL DEFAULT true,

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


		// SearchParser store for Custom Items
		// Stores the search options for the custom url.
		// Called by match_url

		self.0.execute(
			"CREATE TABLE IF NOT EXISTS custom_item (
				id               INTEGER PRIMARY KEY,

				title            TEXT NOT NULL,
				match_url        TEXT NOT NULL,
				description      TEXT NOT NULL,

				search_opts      TEXT NOT NULL
			)"
		)?;


		// Feed Filters

		self.0.execute(
			"CREATE TABLE IF NOT EXISTS feed_filters (
				id          INTEGER PRIMARY KEY,

				feed_id     INTEGER NOT NULL,
				filter_id   INTEGER NOT NULL
			)"
		)?;


		// General Filters

		self.0.execute(
			"CREATE TABLE IF NOT EXISTS filters (
				id          INTEGER PRIMARY KEY,

				title       TEXT NOT NULL,

				filter      TEXT
			)"
		)?;


		// =================
		// ==== WATCHER ====
		// =================

		// Watchers
		self.0.execute(
			"CREATE TABLE IF NOT EXISTS watching (
				id                 INTEGER PRIMARY KEY,

				enabled            BOOL NOT NULL DEFAULT true,

				parser_id          INTEGER,

				url                TEXT NOT NULL,
				title              TEXT NOT NULL,
				description        TEXT NOT NULL,

				sec_interval       INTEGER NOT NULL,
				remove_after       INTEGER NOT NULL DEFAULT 0,

				date_added         LONG NOT NULL,
				last_called        LONG NOT NULL
			)"
		)?;

		// History for the Matcher
		// Keeps track of changes that have happened.
		self.0.execute(
			"CREATE TABLE IF NOT EXISTS watch_history (
				id               INTEGER PRIMARY KEY,

				watch_id         INTEGER NOT NULL,
				items            TEXT NOT NULL,

				date_added       LONG NOT NULL
			)"
		)?;

		// MatchParser store for Watching Items
		// Stores the search options for the custom url.
		// Called by match_url or id.

		self.0.execute(
			"CREATE TABLE IF NOT EXISTS watch_parser (
				id               INTEGER PRIMARY KEY,

				title            TEXT NOT NULL,
				match_url        TEXT NOT NULL,
				description      TEXT NOT NULL,

				match_opts       TEXT NOT NULL
			)"
		)?;



		// Request History
		self.0.execute(
			"CREATE TABLE IF NOT EXISTS request_history_group (
				id               INTEGER PRIMARY KEY,

				is_manual        BOOL NOT NULL,
				concurrency      INTEGER NOT NULL,
				start_time       LONG NOT NULL,

				duration         INTEGER NOT NULL
			)"
		)?;

		self.0.execute(
			"CREATE TABLE IF NOT EXISTS request_history_item (
				id               INTEGER PRIMARY KEY,

				group_id         INTEGER NOT NULL,

				feed_id         INTEGER,
				watch_id        INTEGER,

				new_items        INTEGER,
				start_time       LONG,

				duration         INTEGER,

				error            TEXT
			)"
		)?;

		Ok(())
	}

	pub fn connection(&self) -> &SqliteConnection {
		&self.0
	}
}