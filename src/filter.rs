use serde::{Serialize, Deserialize};
use regex::RegexBuilder;

use crate::feature::models::Item;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Filter {
	//    regex, opts
	Regex(String, RegexOpts),

	//       items, sensitive
	Contains(String, bool),

	//         items, sensitive
	StartsWith(String, bool),

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
	pub fn filter(&self, item: &Item) -> bool {
		match self {
			Self::Regex(regex, opts) => {
				let mut builder = RegexBuilder::new(&regex);

				builder.case_insensitive(opts.case_insensitive);
				builder.multi_line(opts.multi_line);
				builder.dot_matches_new_line(opts.dot_matches_new_line);
				builder.swap_greed(opts.swap_greed);
				builder.ignore_whitespace(opts.ignore_whitespace);
				builder.unicode(opts.unicode);
				builder.octal(opts.octal);

				builder.build().unwrap().is_match(&item.title)
			}

			Self::Contains(value, case_sensitive) => {
				if *case_sensitive {
					item.title.contains(value.as_str())
				} else {
					item.title.to_lowercase().contains(value.to_lowercase().as_str())
				}
			}

			Self::StartsWith(value, case_sensitive) => {
				if *case_sensitive {
					item.title.starts_with(value.as_str())
				} else {
					item.title.to_lowercase().starts_with(value.to_lowercase().as_str())
				}
			}

			Self::And(filters) => filters.iter().filter(|f| f.filter(item)).count() == filters.len(),
			Self::Or(filters) => filters.iter().filter(|f| f.filter(item)).count() != 0,
		}
	}

	// Display showing why said item is being filtered in or out. (new enum FilterDisplay ??)
	pub fn display(&self, _item: &Item) {
		//
	}
}