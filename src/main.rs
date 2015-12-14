extern crate chattium_oxide_lib;
extern crate yaml_file_handler;
extern crate hyper;
extern crate clap;

mod client_handler;
mod options;

use options::Options;
use hyper::Error as HyperError;
use hyper::net::{Openssl, NetworkListener};
use hyper::server::Server;


fn handle_server<L: 'static + NetworkListener + Send>(server: Server<L>) {
	match server.handle(client_handler::handle_client) {
		Ok(listener) => println!("Listening on {}", listener.socket),
		Err(error) => println!("Couldn't handle client: {}", error),
	}
}

fn main() {
	let options = Options::parse();
	println!("{:?}", options);

	let addr = &format!("127.0.0.1:{}", options.port)[..];
	match options.ssl {
		Some(pair) => {
			match Openssl::with_cert_and_key(pair.0, pair.1) {
				Ok(ssl) => {handle_server_error(Server::https(&addr, ssl)).map(handle_server);},
				Err(error) => println!("Couldn't set up OpenSSL: {}", error),
			}
		},
		None => {handle_server_error(Server::http(&addr)).map(handle_server);},
	};
}


fn handle_server_error<S>(res: Result<S, HyperError>) -> Option<S> {
	match res {
		Ok(server) => Some(server),
		Err(error) => {
			println!("Couldn't start server: {}", error);
			None
		},
	}
}
