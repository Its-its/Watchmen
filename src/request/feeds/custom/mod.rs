use serde::{Serialize, Deserialize};
use url::Url;

use regex::RegexBuilder;
use xpath::{Node, Document, Value};
use chrono::{DateTime, FixedOffset};

use crate::feature::models::QueryId;
use crate::{Result, Error};
use super::NewFeedModel;

use crate::feature::objects::{get_custom_item_from_url, get_custom_item_by_id};


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
	pub id: Option<i32>,

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

				_ => value
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


// TODO: Impl. Title / Description scrape from webpage. Currently getting from db.

pub fn new_from_url(url: String, custom_item_id: Option<QueryId>, conn: &diesel::SqliteConnection) -> Result<NewFeedModel> {
	let item = if let Some(id) = custom_item_id {
		get_custom_item_by_id(id, conn)?
	} else {
		get_custom_item_from_url(Url::parse(&url).unwrap(), conn)?
	};

	Ok(NewFeedModel {
		url,

		title: item.title,
		description: item.description,
		generator: String::new(),

		feed_type: 2,

		sec_interval: 60 * 5,
		remove_after: 0,

		global_show: true,
		ignore_if_not_new: true,

		date_added: chrono::Utc::now().naive_utc().timestamp(),
		last_called: chrono::Utc::now().naive_utc().timestamp(),
	})
}

pub fn new_from_feed(url: String, feed: CustomItem) -> NewFeedModel {
	NewFeedModel {
		url,

		title: feed.title,
		description: feed.description,
		generator: String::new(),

		feed_type: 2,

		sec_interval: 60 * 5,
		remove_after: 0,

		global_show: true,
		ignore_if_not_new: true,

		date_added: chrono::Utc::now().naive_utc().timestamp(),
		last_called: chrono::Utc::now().naive_utc().timestamp(),
	}
}


pub fn get_from_url(url: &str, conn: &diesel::SqliteConnection) -> CustomResult {
	let found = get_custom_item_from_url(Url::parse(url).unwrap(), conn)?;

	// turn found into SearchParser

	get_from_url_parser(url, &found.search_opts)
}

pub fn get_from_url_parser(url: &str, parser: &SearchParser) -> CustomResult {
	let mut resp = reqwest::get(url)?;

	let doc = xpath::parse_doc(&mut resp);


	Ok(
		doc.evaluate(&parser.items)
		.ok_or_else(|| Error::Other("Xpath Evaluation Error!".into()))?
		.into_iterset()?
		.map::<Result<FoundItem>, _>(|node| {
			let title = parser.title.evaluate(&doc, node.clone())
				.map(|v| v.vec_string())
				.transpose()?
				.and_then(|v| v.first().cloned())
				.map(|v| parser.title.parse(v))
				.transpose()?;

			let author = parser.author.as_ref()
				.and_then(|i| i.evaluate(&doc, node.clone()))
				.map(|v| v.vec_string())
				.transpose()?
				.and_then(|v| v.first().cloned())
				.and_then(|v| parser.author.as_ref().map(|i| i.parse(v)))
				.transpose()?;

			let content = parser.content.as_ref()
				.and_then(|i| i.evaluate(&doc, node.clone()))
				.map(|v| v.into_iterset())
				.transpose()?
				.and_then(|mut v| v.next())
				.map(|v| v.as_simple_html())
				.and_then(|v| parser.content.as_ref().map(|i| i.parse(v)))
				.transpose()?;

			let date = parser.date.evaluate(&doc, node.clone())
				.map(|v| v.vec_string())
				.transpose()?
				.and_then(|v| v.first().cloned())
				.map(|v| parser.date.parse(v))
				.transpose()?;

			let guid = parser.guid.evaluate(&doc, node.clone())
				.map(|v| v.vec_string())
				.transpose()?
				.and_then(|v| v.first().cloned())
				.map(|v| parser.guid.parse(v))
				.transpose()?;

			let link = parser.link.evaluate(&doc, node)
				.map(|v| v.vec_string())
				.transpose()?
				.and_then(|v| v.first().cloned())
				.map(|v| parser.link.parse(v))
				.transpose()?;

			Ok(FoundItem {
				title: title.ok_or_else(|| Error::Other("Missing Required Title.".into()))?,
				link: link.ok_or_else(|| Error::Other("Missing Required Link.".into()))?,
				guid: guid.ok_or_else(|| Error::Other("Missing Required GUID.".into()))?,
				date: date.unwrap_or_default(),

				author,
				content
			})
		})
		.filter_map(|i| {
			if i.is_err() {
				println!("EVALUATION ERROR: {:?}", i);
			}

			i.ok()
		})
		.collect()
	)
}
