use feeder::Result;
use feeder::request::custom::{ CustomItem, SearchParser, ParseOpts, Parse, get_from_url };

fn main() -> Result<()> {
	let custom_item = CustomItem {
		title: "testing".into(),
		description: "testing".into(),
		match_url: "ebay.com".into(),

		search_opts: SearchParser {
			items: r#"//ul[@class="srp-results srp-list clearfix"]/li"#.into(),
			title: ".//h3/text()".into(),
			link: ParseOpts {
				xpath: r#".//a[@class="s-item__link"]/@href"#.into(),
				parse_type: Parse::Regex(r#"^([a-z0-9:/.-]+)"#.into())
			},
			guid: ParseOpts {
				xpath: r#".//a[@class="s-item__link"]/@href"#.into(),
				parse_type: Parse::Regex(r#"^[a-z0-9:/.-]+/([0-9]+)\?"#.into())
			},
			date: ParseOpts {
				xpath: r#".//span[@class="s-item__dynamic s-item__listingDate"]/span/text()"#.into(),
				parse_type: Parse::TimeFormat("%b-%e %R".into(), Some("PST".into()))
			},
			author: Some(r#".//span[@class="s-item__seller-info-text"]/text()"#.into()),
			content: Some(ParseOpts { xpath: "./node()".into(), parse_type: Parse::None }),
		}
	};

	println!("{}", serde_json::to_string(&custom_item)?);

	// println!("{:#?}", xpath.search_opts);

	// println!("{:#?}", get_from_url("https://www.ebay.com/sch/i.html?_from=R40&_nkw=x-h1&_sacat=0&_sop=15", &xpath));

	Ok(())
}