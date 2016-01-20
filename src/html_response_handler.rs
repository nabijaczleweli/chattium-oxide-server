use hyper::header::{Headers, Encoding, AcceptEncoding, TransferEncoding, ContentLength};
use flate2::write::{GzEncoder, DeflateEncoder};
use flate2::Compression;
use regex::Regex;
use std::sync::RwLock;
use std::io::Write;


struct HtmlResponseHandler {
	raw_content    : String,
	gzip_content   : RwLock<Option<Vec<u8>>>,
	deflate_content: RwLock<Option<Vec<u8>>>,
}


impl HtmlResponseHandler {
	pub fn new(content: &str) -> HtmlResponseHandler {
		HtmlResponseHandler{
			raw_content    : Self::compact(content),
			gzip_content   : RwLock::new(None),
			deflate_content: RwLock::new(None),
		}
	}

	pub fn respond<W: Write>(&self, out_headers: &mut Headers, in_headers: &Headers, mut out_stream: &mut W) {
		let algo = Self::encode_algo(&in_headers);
		out_headers.set(ContentLength(match algo {
			Encoding::Gzip => Self::encode(&self.gzip_content, &self.raw_content, || GzEncoder::new(Vec::new(), Compression::Best), |enc| enc.finish().unwrap(),
			                               &mut out_stream, "GZip"),
			Encoding::Deflate => Self::encode(&self.deflate_content, &self.raw_content, || DeflateEncoder::new(Vec::new(), Compression::Best),
			                                  |enc| enc.finish().unwrap(), &mut out_stream, "deflate"),
			_ => {
				out_stream.write_all(self.raw_content.as_bytes()).unwrap();
				self.raw_content.len() as u64
			},
		}));
		out_headers.set(TransferEncoding(vec![algo]));
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

	fn encode<W, Ew, EwW, EwF>(out_content: &RwLock<Option<Vec<u8>>>, raw_content: &String, encoder: Ew, finisher: EwF, ostream: &mut W, method: &str) -> u64
		where W  : Write,
		      EwW: Write,
		      Ew : FnOnce() -> EwW,
		      EwF: FnOnce(EwW) -> Vec<u8>,
	{
		let read_lock = out_content.read().unwrap();
		if read_lock.is_some() {
			let content = read_lock.as_ref().unwrap();
			ostream.write_all(content).unwrap();
			content.len() as u64
		} else {
			let mut compressor = encoder();
			compressor.write_all(raw_content.as_bytes()).unwrap();
			let compressed = finisher(compressor);
			println!("Compacted HTML message using {}.", method);

			let length = compressed.len();
			ostream.write_all(&compressed).unwrap();
			*out_content.write().unwrap() = Some(compressed);
			length as u64
		}
	}
}
