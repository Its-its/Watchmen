use serde::{Serialize, Deserialize};
use regex::RegexBuilder;
use diesel::SqliteConnection;

use crate::Result;
use crate::feature::models::Item;
use crate::feature::objects::{get_filters, get_feed_filters};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Filter {
	//    regex, opts
	Regex(String, RegexOpts),

	//       items, sensitive
	Contains(String, bool),

	//         items, sensitive
	StartsWith(String, bool),

	//         items, sensitive
	EndsWith(String, bool),

	And(Vec<Filter>),
	Or(Vec<Filter>)
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


impl Filter {
	pub fn is_and(&self) -> bool {
		matches!(self, Filter::And(_))
	}

	pub fn is_or(&self) -> bool {
		matches!(self, Filter::Or(_))
	}


	pub fn filter(&self, item: &Item) -> bool {
		match self {
			Filter::Regex(regex, opts) => {
				let mut builder = RegexBuilder::new(&regex);

				builder.case_insensitive(opts.case_insensitive);
				builder.multi_line(opts.multi_line);
				builder.dot_matches_new_line(opts.dot_matches_new_line);
				builder.swap_greed(opts.swap_greed);
				builder.ignore_whitespace(opts.ignore_whitespace);
				builder.unicode(opts.unicode);
				builder.octal(opts.octal);

				let build = builder.build().unwrap();

				build.is_match(&item.title)
			}

			Filter::Contains(value, case_sensitive) => {
				if *case_sensitive {
					item.title.contains(value.as_str())
				} else {
					item.title.to_lowercase().contains(value.to_lowercase().as_str())
				}
			}

			Filter::StartsWith(value, case_sensitive) => {
				if *case_sensitive {
					item.title.starts_with(value.as_str())
				} else {
					item.title.to_lowercase().starts_with(value.to_lowercase().as_str())
				}
			}

			Filter::EndsWith(value, case_sensitive) => {
				if *case_sensitive {
					item.title.ends_with(value.as_str())
				} else {
					item.title.to_lowercase().ends_with(value.to_lowercase().as_str())
				}
			}

			Filter::And(filters) => filters.iter().all(|f| f.filter(item)),
			Filter::Or(filters) => filters.iter().any(|f| f.filter(item)),
		}
	}

	// Display showing why said item is being filtered in or out. (new enum FilterDisplay ??)
	pub fn display(&self, _item: &Item) {
		//
	}


	pub fn add(&mut self, filter: Filter) {
		match self {
			Filter::And(vec) |
			Filter::Or(vec) => vec.push(filter),
			_ => ()
		}
	}

	pub fn remove(&mut self, index: usize) {
		match self {
			Filter::And(vec) |
			Filter::Or(vec) => { vec.remove(index); }
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

		Filter::And(vec![self, other])
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

		Filter::Or(vec![self, other])
	}
}


pub fn filter_items<'a>(items: &'a [Item], conn: &SqliteConnection) -> Result<Vec<&'a Item>> {
	let feed_filters = get_feed_filters(conn)?;
	let filter_models = get_filters(conn)?;

	if filter_models.is_empty() {
		Ok(items.iter().collect())
	} else {
		Ok(
			items.iter()
			.filter(|item| {
				for ff in &feed_filters {
					if ff.feed_id == item.feed_id && filter_models.iter().any(|fm| ff.filter_id == fm.id && fm.filter.filter(*item)) {
						return true;
					}
				}

				false
			})
			.collect()
		)
	}
}