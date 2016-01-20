use hyper::header::{Headers, Encoding, AcceptEncoding};
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

	fn encode_algo(headers: &Headers) -> Encoding {
		match headers.get::<AcceptEncoding>() {
			Some(&AcceptEncoding(ref encodings)) => {
				let mut encodings = encodings.clone();
				encodings.sort_by(|ref qil, ref qir| qil.quality.cmp(&qir.quality));

				let mut ret = Encoding::Identity;
				for enc in encodings {
					match enc.item {
						Encoding::Gzip => {
							ret = Encoding::Gzip;
							break;
						},
						Encoding::Deflate => {
							ret = Encoding::Deflate;
							break;
						},
						_ => (),
					}
				}
				ret
			},
			None => Encoding::Identity,
		}
	}
}
