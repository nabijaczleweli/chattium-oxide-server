extern crate chattium_oxide_lib;
extern crate hyper;

mod client_handler;

use hyper::server::Server;


fn main() {
	match Server::http("127.0.0.1:50030") {
		Ok(server) =>
			match server.handle(client_handler::handle_client) {
				Ok(listener) => println!("Listening on {}", listener.socket),
				Err(error) => println!("Couldn't handle client: {}", error),
			},
		Err(error) => println!("Couldn't start server: {}", error),
	}
}
