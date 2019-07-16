table! {
	items (id) {
		id -> Integer,

		guid -> Text,
		title -> Text,
		authors -> Text,
		content -> Text,
		link -> Text,
		date -> Timestamp,
		hash -> Text,

		date_added -> Timestamp,
		is_read -> Bool,
		is_starred -> Bool,
		tags -> Text,
		feed_id -> Integer,
	}
}