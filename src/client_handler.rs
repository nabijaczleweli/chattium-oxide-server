use chattium_oxide_lib::json::{FromJsonnable, ToJsonnable};
use chattium_oxide_lib::ChatMessage;
use hyper::server::{Handler, Request, Response};
use hyper::header::{ContentLength, ContentType, CacheControl, CacheDirective, ContentLanguage, Server, qitem};
use hyper::status::StatusCode;
use hyper::method::Method;
use hyper::LanguageTag;
use regex::Regex;
use time::strftime;
use std::collections::btree_map::BTreeMap;
use std::sync::RwLock;
use std::io::{self, Read, Write, stderr};


pub struct ClientHandler {
	messages    : RwLock<Vec<ChatMessage>>,
	message_id  : RwLock<u64>,
	html_message: String,
}

impl ClientHandler {
	pub fn new() -> ClientHandler {
		ClientHandler{
			messages    : RwLock::new(Vec::new()),
			message_id  : RwLock::new(1),
			html_message: Self::compact(r##"<!DOCTYPE html>
			                                <html>
			                                <head>
			                                	<title>chattium-oxide — please connect with chattium-oxide client</title>
			                                	<meta charset="utf-8" />
			                                	<meta name="application-name" content="chattium-oxide" />
			                                	<meta name="author"           content="chattium-oxide server" />
			                                	<meta name="description"      content="Please reconnect with chattium-oxide client" />
			                                	<meta name="keywords"         content="chat,open source" />
			                                	<meta name="robots"           content="index,follow" />
			                                	<style type="text/css">
			                                		.q {
			                                			font-size: 1.2em;
			                                		}
			                                	</style>
			                                	<script type="text/javascript">
			                                		window.addEventListener("load", function() {
			                                			document.getElementById("own_url").textContent = document.URL.replace(/\/$/, "");
			                                		});
			                                	</script>
			                                </head>
			                                <body>
			                                	<p>
			                                		<b class="q">What is Ч<small>@</small>O<sub>2</sub>?</b><br />
			                                		Ч<small>@</small>O<sub>2</sub> (read: <i>chattium oxide</i>, as in: chemical compound) is a lightweight chat
			                                		platform written in Rust with an aim for simplicity. Both the
			                                		<a href="https://github.com/nabijaczleweli/chattium-oxide-client">client</a> and the
			                                		<a href="https://github.com/nabijaczleweli/chattium-oxide-server">server</a> reside on
			                                		<a href="https://github.com">GitHub</a>.
			                                	</p>
			                                	<p>
			                                		<b class="q">How do I connect to a Ч<small>@</small>O<sub>2</sub> server?</b><br />
			                                		Just type in the exact URL of this page (<code id="own_url"></code>) into the client, as this document is sent by the
			                                		Ч<small>@</small>O<sub>2</sub> server itself.<br />
			                                		Latest Windows and Ubuntu client binaries can be downloaded from the
			                                		<a href="https://github.com/nabijaczleweli/chattium-oxide-client/releases/latest">latest
			                                		Ч<small>@</small>O<sub>2</sub> client release page</a>.
			                                	</p>
			                                </body>
			                                </html>"##),
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

impl Handler for ClientHandler {
	fn handle(&self, mut req: Request, mut res: Response) {
		let mut body_ref: Option<&String> = None;
		let mut body    : Option<String>  = None;

		let mut reqbody = String::new();
		*res.status_mut() = match req.read_to_string(&mut reqbody) {
			Ok(_) =>
				match req.method {
					Method::Post =>
						match ChatMessage::from_json_string(&reqbody) {
							Ok(mut message) => {
								message.sender.fill_ip(req.remote_addr);
								message.fill_id(self.message_id.write().unwrap());
								println!("{}@{}: {} @ {} # {}", message.sender.name, req.remote_addr, message.value, strftime("%T", &message.time_posted).unwrap(),
								                                message.id);
								self.messages.write().unwrap().push(message);
								StatusCode::Ok
							},
							Err(error) => {
								let _ = stderr().write_fmt(format_args!("Couldn't process POSTed message from {}: {}\n", req.remote_addr, error));
								StatusCode::UnprocessableEntity
							},
						},
					Method::Get => {  // Web browser, probably
						println!("Serving {} HTML message to connect via client.", req.remote_addr);
						body_ref = Some(&self.html_message);
						res.headers_mut().set(ContentType::html());
						res.headers_mut().set(Server(concat!("chattium-oxide-server/", env!("CARGO_PKG_VERSION")).to_string()));
						res.headers_mut().set(ContentLanguage(vec![qitem(LanguageTag{
							language  : Some("en-GB".to_string()),
							extlangs  : vec![],
							script    : None,
							region    : None,
							variants  : vec![],
							extensions: BTreeMap::new(),
							privateuse: vec![],
						})]));
						res.headers_mut().set(CacheControl(vec![
							CacheDirective::Public,
							CacheDirective::MaxAge(60 * 60 * 24 * 7),
						]));
						StatusCode::Ok
					},
					Method::Trace => {
						match u64::from_json_string(&reqbody) {
							Ok(id) => {
								let messages = self.messages.read().unwrap();
								let msgs     = messages.iter().rev().take_while(|&m| m.id != id).collect::<Vec<&_>>()
								                       .iter().rev().map(|&m| m.clone()).collect::<Vec<_>>();
								match msgs.to_json_string() {
									Ok(msgs) => {
										body = Some(msgs);
										StatusCode::Ok
									},
									Err(error) => {
										let _ = stderr().write_fmt(format_args!("Couldn't create a JSON response for {}: {}\n", req.remote_addr, error));
										body = Some("[]".to_string());  // Empty array
										StatusCode::Accepted
									},
								}
							},
							Err(error) => {
								let _ = stderr().write_fmt(format_args!("Couldn't process a TRACE message id ({:?}) from {}: {}\n", reqbody, req.remote_addr, error));
								StatusCode::UnprocessableEntity
							},
						}
					},
				  _ => StatusCode::ImATeapot,
				},
			Err(error) => {
				let _ = stderr().write_fmt(format_args!("Failed reading request from {}: {}\n", req.remote_addr, error));
				StatusCode::UnsupportedMediaType  // non-UTF-8
			},
		};


		let handle_error = |error: Result<(), io::Error>| {
			if let Err(error) = error {
				let _ = stderr().write_fmt(format_args!("Failed to respond to {}: {}", req.remote_addr, error));
			}
		};

		let reply = |body: &String| {
			res.headers_mut().set(ContentLength(body.len() as u64));
			handle_error(res.start().unwrap().write_all(body.as_bytes()));
		};

		match (body, body_ref) {
			(Some(body), _)     => reply(&body),
			(_, Some(ref body)) => reply(&body),
			(None, None)        => reply(&"".to_string()),
		}
	}
}
