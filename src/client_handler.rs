use chattium_oxide_lib::json::{FromJsonnable, ToJsonnable};
use chattium_oxide_lib::ChatMessage;
use hyper::server::{Handler, Request, Response};
use hyper::header::{ContentLength, ContentType};
use hyper::status::StatusCode;
use hyper::method::Method;
use time::strftime;
use std::sync::RwLock;
use std::io::{Read, Write, stderr};


pub struct ClientHandler {
	messages  : RwLock<Vec<ChatMessage>>,
	message_id: RwLock<u64>,
}

impl ClientHandler {
	pub fn new() -> ClientHandler {
		ClientHandler{
			messages  : RwLock::new(Vec::new()),
			message_id: RwLock::new(1),
		}
	}
}

impl Handler for ClientHandler {
	fn handle(&self, mut req: Request, mut res: Response) {
		let mut body = "".to_string();

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
						body = format!("{}, use <a href=\"https://github.com/nabijaczleweli/chattium-oxide-client/releases/latest\">chattium-oxide-client</a>
							              to connect to chat.", req.remote_addr);
						res.headers_mut().set(ContentType::html());
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
										body = msgs;
										StatusCode::Ok
									},
									Err(error) => {
										let _ = stderr().write_fmt(format_args!("Couldn't create a JSON response for {}: {}\n", req.remote_addr, error));
										body = "[]".to_string();  // Empty array
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

		res.headers_mut().set(ContentLength(body.len() as u64));
		if let Err(error) = res.start().unwrap().write_all(body.as_bytes()) {
			let _ = stderr().write_fmt(format_args!("Failed to respond to {}: {}", req.remote_addr, error));
		}
	}
}
