table! {
	items (id) {
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

table! {
	feeds (id) {
		id -> Integer,

		url -> Text,

		sec_interval -> Integer,
		remove_after -> Integer,

		ignore_if_not_new -> Bool,

		date_added -> BigInt,
		last_called -> BigInt,
	}
}