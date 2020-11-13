table! {
	items(id) {
		id -> Integer,

		guid -> Text,
		title -> Text,
		author -> Text,
		content -> Text,
		link -> Text,
		date -> BigInt,
		hash -> Text,

		date_added -> BigInt,
		is_read -> Bool,
		is_starred -> Bool,
		is_removed -> Bool,
		tags -> Text,
		feed_id -> Integer,
	}
}

// TODO: a remove duplicates option
table! {
	feeds(id) {
		id -> Integer,

		// Save favicon.ico ?

		url -> Text,
		title -> Text,
		description -> Text,
		generator -> Text,

		#[sql_name = "type"]
		feed_type -> Integer,

		sec_interval -> Integer,
		remove_after -> Integer,

		global_show -> Bool,

		ignore_if_not_new -> Bool,

		date_added -> BigInt,
		last_called -> BigInt,
	}
}

table! {
	categories(id) {
		id -> Integer,

		position -> Integer,

		name -> Text,
		name_lowercase -> Text,

		date_added -> BigInt,
	}
}

table! {
	feed_categories(id) {
		id -> Integer,

		feed_id -> Integer,
		category_id -> Integer,
	}
}

table! {
	filters(id) {
		id -> Integer,

		title -> Text,
		filter -> Text,
	}
}

table! {
	feed_filters(id) {
		id -> Integer,

		feed_id -> Integer,
		filter_id -> Integer,
	}
}

table! {
	custom_item(id) {
		id -> Integer,

		title -> Text,
		match_url -> Text,
		description -> Text,

		search_opts -> Text,
	}
}


// =================
// ==== WATCHER ====
// =================

table! {
	watching(id) {
		id -> Integer,

		parser_id -> Nullable<Integer>,

		// Save favicon.ico ?

		url -> Text,
		title -> Text,
		description -> Text,


		sec_interval -> Integer,
		remove_after -> Integer,

		date_added -> BigInt,
		last_called -> BigInt,
	}
}

table! {
	watch_history(id) {
		id -> Integer,

		watch_id -> Integer,

		items -> Text,

		date_added -> BigInt,
	}
}


table! {
	watch_parser(id) {
		id -> Integer,

		title -> Text,
		match_url -> Text,
		description -> Text,

		match_opts -> Text,
	}
}