use chattium_oxide_lib::json::FromJsonnable;
use chattium_oxide_lib::ChatMessage;
use hyper::server::{Request, Response};
use hyper::header::ContentLength;
use hyper::status::StatusCode;
use hyper::method::Method;
use std::io::{Read, Write, stderr};


pub fn handle_client(req: Request, mut res: Response) {
	let mut req = req;
	let mut body = format!("{}, use https://github.com/nabijaczleweli/chattium-oxide-client to connect to chat", req.remote_addr);

	*res.status_mut() = match req.method {
		Method::Post => {
			let mut reqbody = String::new();
			match req.read_to_string(&mut reqbody) {
				Ok(_) =>
					match ChatMessage::from_json_string(&reqbody) {
						Ok(mut message) => {
							message.sender.fill_ip(req.remote_addr);
							println!("{:?}", message);
							StatusCode::Ok
						},
						Err(error) => {
							let _ = stderr().write_fmt(format_args!("Couldn't process POSTed message from {}: {}", req.remote_addr, error));
							StatusCode::UnprocessableEntity
						},
					},
				Err(error) => {
					let _ = stderr().write_fmt(format_args!("Failed reading request from {}: {}", req.remote_addr, error));
					StatusCode::UnsupportedMediaType  // non-UTF-8
				},
			}
		},
	  _ => StatusCode::ImATeapot,
	};

	res.headers_mut().set(ContentLength(body.len() as u64));
	if let Err(error) = res.start().unwrap().write_all(body.as_bytes()) {
		let _ = stderr().write_fmt(format_args!("Failed to respond to {}: {}", req.remote_addr, error));
	}
}
