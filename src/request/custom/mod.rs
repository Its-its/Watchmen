use std::io::Read;

use serde::{Serialize, Deserialize};

use regex::RegexBuilder;
use xpath::{Node, Document, Value};
use chrono::{DateTime, FixedOffset};

use crate::Result;
use super::NewFeed;


pub type CustomResult = Result<Vec<FoundItem>>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateableCustomItem {
	pub title: Option<String>,
	pub description: Option<String>,
	pub match_url: Option<String>,

	pub search_opts: Option<SearchParser>
}


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomItem {
	pub title: String,
	pub description: String,
	pub match_url: String,

	pub search_opts: SearchParser
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchParser {
	pub items: String,

	pub title: ParseOpts,
	pub link: ParseOpts,
	pub guid: ParseOpts,
	pub date: ParseOpts,

	pub author: Option<ParseOpts>,
	pub content: Option<ParseOpts>
}

#[derive(Debug, Clone, Default)]
pub struct FoundItem {
	pub title: String,
	pub link: String,
	pub guid: String,
	pub date: String,

	pub author: Option<String>,
	pub content: Option<String>
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Parse {
	None,
	// Expression
	Regex(String),
	// Format, Region (PST, EST)
	TimeFormat(String, Option<String>)
}

impl Default for Parse {
	fn default() -> Self {
		Parse::None
	}
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParseOpts {
	pub xpath: String,
	pub parse_type: Parse
}

impl ParseOpts {
	pub fn evaluate(&self, doc: &Document, node: Node) -> Option<Value> {
		doc.evaluate_from(&self.xpath, node)
	}

	pub fn parse(&self, value: String) -> Result<String> {
		Ok(
			match &self.parse_type {
				Parse::Regex(expr) => {
					RegexBuilder::new(&expr)
						.case_insensitive(true)
						.build()?
						.captures(&value)
						.expect("capture")
						.get(1)
						.expect("get")
						.as_str()
						.to_string()
				}

				Parse::TimeFormat(parse, offset) => {
					let parse = if parse.contains("%Z") || offset.is_none() {
						parse.to_string()
					} else {
						format!("{} %Z", parse)
					};

					let value = if let Some(offset) = offset {
						format!("{} {}", value, offset)
					} else {
						value
					};

					let date: DateTime<FixedOffset> = DateTime::parse_from_str(&value, &parse)?;

					date.to_rfc2822()
				}

				Parse::None |_ => value
			}
		)
	}
}

impl From<&str> for ParseOpts {
	fn from(value: &str) -> Self {
		Self {
			xpath: value.to_string(),
			parse_type: Parse::None
		}
	}
}

impl From<String> for ParseOpts {
	fn from(value: String) -> Self {
		Self {
			xpath: value,
			parse_type: Parse::None
		}
	}
}

impl std::ops::Deref for ParseOpts {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.xpath.as_str()
	}
}



pub fn new_from_feed(url: String, feed: CustomItem) -> NewFeed {
	NewFeed {
		url: url,

		title: feed.title,
		description: feed.description,
		generator: atom_syndication::Generator::default().value().to_string(),

		feed_type: 2,

		sec_interval: 60 * 5,
		remove_after: 0,

		global_show: true,
		ignore_if_not_new: true,

		date_added: chrono::Utc::now().naive_utc().timestamp(),
		last_called: chrono::Utc::now().naive_utc().timestamp(),
	}
}

pub fn get_from_url(url: &str, parser: &SearchParser) -> CustomResult {
	// TODO: url DB call for parser.
	let mut resp = reqwest::get(url)?;

	let doc = xpath::parse_doc(&mut resp);

	Ok(
		doc.evaluate(&parser.items).expect("items")
		.into_iterset()
		.map(|node| {
			let title = parser.title.evaluate(&doc, node.clone())
				.and_then(|v| v.vec_string().first().map(|i| i.clone()))
				.map(|v| parser.title.parse(v).expect("1"));

			let author = parser.author.as_ref()
				.and_then(|i| i.evaluate(&doc, node.clone()))
				.and_then(|v| v.vec_string().first().map(|i| i.clone()))
				.and_then(|v| parser.author.as_ref().map(|i| i.parse(v).expect("2")));

			let content = parser.content.as_ref()
				.and_then(|i| i.evaluate(&doc, node.clone()))
				.and_then(|v| v.into_iterset().next())
				.map(|v| v.as_simple_html())
				.and_then(|v| parser.content.as_ref().map(|i| i.parse(v).expect("3")));

			let date = parser.date.evaluate(&doc, node.clone())
				.and_then(|v| v.vec_string().first().map(|i| i.clone()))
				.map(|v| parser.date.parse(v).expect("4"));

			let guid = parser.guid.evaluate(&doc, node.clone())
				.and_then(|v| v.vec_string().first().map(|i| i.clone()))
				.map(|v| parser.guid.parse(v).expect("5"));

			let link = parser.link.evaluate(&doc, node.clone())
				.and_then(|v| v.vec_string().first().map(|i| i.clone()))
				.map(|v| parser.link.parse(v).expect("6"));

			FoundItem {
				title: title.expect("title"),
				link: link.expect("link"),
				guid: guid.expect("guid"),
				date: date.unwrap_or_default(),

				author,
				content
			}
		})
		.collect()
	)
}
