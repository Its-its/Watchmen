use std::ops::Range;

use serde::{Serialize, Deserialize};
use regex::RegexBuilder;
use diesel::SqliteConnection;

use crate::Result;
use crate::feature::models::{NewFeedItemModel, FeedItemModel, FeedFilterModel};
use crate::feature::objects::{get_filters, get_feed_filters, Filter};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterType {
	//    regex, opts
	Regex(String, RegexOpts),

	//       items, sensitive
	Contains(String, bool),

	//         items, sensitive
	StartsWith(String, bool),

	//         items, sensitive
	EndsWith(String, bool),

	And(Vec<FilterType>),
	Or(Vec<FilterType>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexOpts {
	case_insensitive: bool,
	multi_line: bool,
	dot_matches_new_line: bool,
	swap_greed: bool,
	ignore_whitespace: bool,
	unicode: bool,
	octal: bool
}

impl Default for RegexOpts {
	fn default() -> Self {
		RegexOpts {
			dot_matches_new_line: false,
			ignore_whitespace: false,
			case_insensitive: true,
			multi_line: false,
			swap_greed: false,
			unicode: true,
			octal: false
		}
	}
}


impl FilterType {
	pub fn is_and(&self) -> bool {
		matches!(self, FilterType::And(_))
	}

	pub fn is_or(&self) -> bool {
		matches!(self, FilterType::Or(_))
	}


	pub fn matches(&self, item: &dyn FilterableItem) -> bool {
		match self {
			FilterType::Regex(regex, opts) => {
				let mut builder = RegexBuilder::new(regex);

				builder.case_insensitive(opts.case_insensitive);
				builder.multi_line(opts.multi_line);
				builder.dot_matches_new_line(opts.dot_matches_new_line);
				builder.swap_greed(opts.swap_greed);
				builder.ignore_whitespace(opts.ignore_whitespace);
				builder.unicode(opts.unicode);
				builder.octal(opts.octal);

				let build = builder.build().unwrap();

				build.is_match(item.get_value())
			}

			FilterType::Contains(value, case_sensitive) => {
				if *case_sensitive {
					item.get_value().contains(value.as_str())
				} else {
					item.get_value().to_lowercase().contains(value.to_lowercase().as_str())
				}
			}

			FilterType::StartsWith(value, case_sensitive) => {
				if *case_sensitive {
					item.get_value().starts_with(value.as_str())
				} else {
					item.get_value().to_lowercase().starts_with(value.to_lowercase().as_str())
				}
			}

			FilterType::EndsWith(value, case_sensitive) => {
				if *case_sensitive {
					item.get_value().ends_with(value.as_str())
				} else {
					item.get_value().to_lowercase().ends_with(value.to_lowercase().as_str())
				}
			}

			FilterType::And(filters) => filters.iter().all(|f| f.matches(item)),
			FilterType::Or(filters) => filters.iter().any(|f| f.matches(item)),
		}
	}

	pub fn find_ranges(&self, item: &impl FilterableItem) -> Option<FilterRange> {
		match self {
			FilterType::Regex(regex, opts) => {
				let mut builder = RegexBuilder::new(regex);

				builder.case_insensitive(opts.case_insensitive);
				builder.multi_line(opts.multi_line);
				builder.dot_matches_new_line(opts.dot_matches_new_line);
				builder.swap_greed(opts.swap_greed);
				builder.ignore_whitespace(opts.ignore_whitespace);
				builder.unicode(opts.unicode);
				builder.octal(opts.octal);

				let build = builder.build().unwrap();

				build.find(item.get_value()).map(|m| FilterRange::Single(m.range()))
			}

			FilterType::Contains(value, case_sensitive) => {
				if *case_sensitive {
					item.get_value()
						.find(value.as_str())
						.map(|s| FilterRange::Single(s..value.len()))
				} else {
					item.get_value().to_lowercase()
						.find(value.to_lowercase().as_str())
						.map(|s| FilterRange::Single(s..value.len()))
				}
			}

			FilterType::StartsWith(value, case_sensitive) => {
				let contains = if *case_sensitive {
					item.get_value().starts_with(value.as_str())
				} else {
					item.get_value().to_lowercase().starts_with(value.to_lowercase().as_str())
				};

				Some(FilterRange::Single(0..value.len())).filter(|_| contains)
			}

			FilterType::EndsWith(value, case_sensitive) => {
				let contains = if *case_sensitive {
					item.get_value().ends_with(value.as_str())
				} else {
					item.get_value().to_lowercase().ends_with(value.to_lowercase().as_str())
				};

				Some(FilterRange::Single(item.get_value().len() - value.len()..item.get_value().len())).filter(|_| contains)
			}

			FilterType::And(filters) => {
				let mut items = FilterRange::Multiple(Vec::new());

				for filter in filters {
					items = items.append(filter.find_ranges(item)?);
				}

				Some(items)
			}

			FilterType::Or(filters) => filters.iter().find_map(|f| f.find_ranges(item)),
		};

		None
	}


	pub fn add(&mut self, filter: FilterType) {
		match self {
			FilterType::And(vec) |
			FilterType::Or(vec) => vec.push(filter),
			_ => ()
		}
	}

	pub fn remove(&mut self, index: usize) {
		match self {
			FilterType::And(vec) |
			FilterType::Or(vec) => { vec.remove(index); }
			_ => ()
		}
	}

	/// If one of the filters in an AND add to it. Otherwise make one and add both to it.
	pub fn and(mut self, mut other: Self) -> Self {
		if self.is_and() {
			self.add(other);
			return self;
		}

		if other.is_and() {
			other.add(self);
			return other;
		}

		FilterType::And(vec![self, other])
	}

	/// If one of the filters in an OR add to it. Otherwise make one and add both to it.
	pub fn or(mut self, mut other: Self) -> Self {
		if self.is_or() {
			self.add(other);
			return self;
		}

		if other.is_or() {
			other.add(self);
			return other;
		}

		FilterType::Or(vec![self, other])
	}
}


pub enum FilterRange {
	Single(Range<usize>),
	Multiple(Vec<Range<usize>>)
}

impl FilterRange {
	pub fn append(self, other: Self) -> Self {
		match (self, other) {
			(Self::Single(v1), Self::Single(v2)) => Self::Multiple(vec![v1, v2]),

			(Self::Single(single), Self::Multiple(mut multi)) |
			(Self::Multiple(mut multi), Self::Single(single)) => {
				multi.push(single);
				Self::Multiple(multi)
			}

			(Self::Multiple(mut v1), Self::Multiple(v2)) => {
				v1.extend(v2);
				Self::Multiple(v1)
			}
		}
	}
}


pub trait FilterableItem {
	fn get_value(&self) -> &str;
	fn get_feed_id(&self) -> i32;
}


impl FilterableItem for NewFeedItemModel {
	fn get_value(&self) -> &str {
		&self.title
	}

	fn get_feed_id(&self) -> i32 {
		self.feed_id
	}
}

impl FilterableItem for FeedItemModel {
	fn get_value(&self) -> &str {
		&self.title
	}

	fn get_feed_id(&self) -> i32 {
		self.feed_id
	}
}


pub fn filter_items<'a, F: FilterableItem>(items: &'a [F], conn: &SqliteConnection) -> Result<Vec<&'a F>> {
	let feed_filters = get_feed_filters(conn)?;
	let filter_models = get_filters(conn)?;

	if filter_models.is_empty() {
		Ok(items.iter().collect())
	} else {
		Ok(
			items.iter()
			.filter(|item| filter_item(*item, &filter_models, &feed_filters, conn))
			.collect()
		)
	}
}

pub fn filter_item<F: FilterableItem>(item: &F, filter_models: &[Filter], feed_filters: &[FeedFilterModel], conn: &SqliteConnection) -> bool {
	for feed_filter_model in feed_filters {
		if feed_filter_model.feed_id == item.get_feed_id() &&
			filter_models.iter().any(|filter_cont| feed_filter_model.filter_id == filter_cont.id && filter_cont.filter.matches(item)) {
			return true;
		}
	}

	false
}