use reqwest::Client;
use serde::{Serialize, Deserialize};
use url::Url;

use regex::RegexBuilder;
use xpather::value::Node;
use xpather::Document;
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
	pub fn evaluate<'a>(&self, doc: &'a Document, node: &'a Node) -> xpather::Result<xpather::factory::ProduceIter<'a>> {
		doc.evaluate_from(&self.xpath, node)
	}

	pub fn parse(&self, value: &str) -> Result<String> {
		Ok(
			match &self.parse_type {
				Parse::Regex(expr) => {
					RegexBuilder::new(expr)
						.case_insensitive(true)
						.build()?
						.captures(value)
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
						value.to_string()
					};

					let date: DateTime<FixedOffset> = DateTime::parse_from_str(&value, &parse)?;

					date.to_rfc2822()
				}

				_ => value.to_string()
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

		enabled: true,

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

		enabled: true,

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


pub async fn get_from_url(url: &str, req_client: &Client, conn: &diesel::SqliteConnection) -> CustomResult {
	let found = get_custom_item_from_url(Url::parse(url).unwrap(), conn)?;

	// turn found into SearchParser

	get_from_url_parser(url, &found.search_opts, req_client).await
}

pub async fn get_from_url_parser(url: &str, parser: &SearchParser, req_client: &Client) -> CustomResult {
	let resp = req_client.get(url).send().await?.text().await?;

	let mut reader = std::io::Cursor::new(resp);

	let doc = xpather::parse_document(&mut reader)?;

	Ok(
		doc.evaluate(&parser.items)?
		.collect_nodes()?
		.into_iter()
		.map::<Result<FoundItem>, _>(|node| {
			let title = parser.title.evaluate(&doc, &node)?
				.next()
				.transpose()?
				.map(|v| Result::Ok(parser.title.parse(&v.convert_to_string()?)?))
				.transpose()?;

			let author = parser.author.as_ref()
				.map(|i| i.evaluate(&doc, &node))
				.transpose()?
				.and_then(|mut v| v.next())
				.transpose()?
				.map(|v| Result::Ok(parser.author.as_ref().unwrap().parse(&v.convert_to_string()?)?))
				.transpose()?;

			let content = parser.content.as_ref()
				.map(|i| i.evaluate(&doc, &node))
				.transpose()?
				.and_then(|mut v| v.next())
				.transpose()?
				.map(|v| Result::Ok(v.as_node()?.as_simple_html()))
				.transpose()?.flatten()
				.map(|v| Result::Ok(parser.content.as_ref().unwrap().parse(&v)?))
				.transpose()?;

			let date = parser.date.evaluate(&doc, &node)?
				.next()
				.transpose()?
				.map(|v| Result::Ok(parser.date.parse(&v.convert_to_string()?)?))
				.transpose()?;

			let guid = parser.guid.evaluate(&doc, &node)?
				.next()
				.transpose()?
				.map(|v| Result::Ok(parser.guid.parse(&v.convert_to_string()?)?))
				.transpose()?;

			let link = parser.link.evaluate(&doc, &node)?
				.next()
				.transpose()?
				.map(|v| Result::Ok(parser.link.parse(&v.convert_to_string()?)?))
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
