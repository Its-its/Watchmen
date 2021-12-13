use xpather::Document;
use xpather::value::Node;

use crate::Result;
use crate::request::feeds::custom::ParseOpts;



pub fn get_optional_string(opt: Option<&ParseOpts>, doc: &Document, node: &Node) -> Result<Option<String>> {
	let opt = match opt {
		Some(v) => v,
		None => return Ok(None)
	};

	let value = match opt.evaluate(doc, node)?.next().transpose()? {
		Some(v) => v,
		None => return Ok(None)
	};

	Ok(Some(opt.parse(&value.convert_to_string()?)?))
}