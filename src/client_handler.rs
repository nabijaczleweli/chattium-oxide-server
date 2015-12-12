use chattium_oxide_lib::json::FromJsonnable;
use chattium_oxide_lib::ChatMessage;
use hyper::server::{Request, Response};
use hyper::status::StatusCode;
use hyper::method::Method;
use std::io::Read;


pub fn handle_client(req: Request, mut res: Response) {
	let mut req = req;
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
							println!("Couldn't process POSTed message from {}: {}", req.remote_addr, error);
							StatusCode::UnprocessableEntity
						},
					},
				Err(error) => {
					println!("Failed reading request from {}: {}", req.remote_addr, error);
					StatusCode::UnsupportedMediaType  // non-UTF-8
				},
			}
		},
	  _ => StatusCode::ImATeapot,
	}
}
