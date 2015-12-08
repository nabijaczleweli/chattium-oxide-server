use hyper::server::{Request, Response};
use hyper::status::StatusCode;


pub fn handle_client(req: Request, mut res: Response) {
	match req.method {
	  _ => *res.status_mut() = StatusCode::ImATeapot,
	}
}
