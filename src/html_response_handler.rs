use regex::Regex;


struct HtmlResponseHandler {
	raw_content: String,
}


impl HtmlResponseHandler {
	pub fn new(content: &str) -> HtmlResponseHandler {
		HtmlResponseHandler{
			raw_content: Self::compact(content),
		}
	}


	fn compact(what: &str) -> String {
		let regices = [
			(Regex::new(r#"\s+"#).unwrap(), " "),
			(Regex::new(r#">\s<"#).unwrap(), "><"),
			(Regex::new(r#"\s/>"#).unwrap(), "/>"),
			(Regex::new(r#"\s?\{\s?"#).unwrap(), "{"),
			(Regex::new(r#"\s?\}\s?"#).unwrap(), "}"),
		];

		regices.iter().fold(what.to_string(), |curr, ref tpl| tpl.0.replace_all(&curr[..], &tpl.1[..]))
	}
}
