extern crate chattium_oxide_lib;
extern crate yaml_file_handler;
extern crate hyper;
extern crate clap;
extern crate time;

mod client_handler;
mod options;

use options::Options;
use client_handler::ClientHandler;
use hyper::Error as HyperError;
use hyper::net::{Openssl, NetworkListener};
use hyper::server::{Server, Request, Response};
use chattium_oxide_lib::ChatMessage;


fn handle_server<L: 'static + NetworkListener + Send>(server: Server<L>, https: bool) {
	match server.handle(ClientHandler::new()) {
		Ok(listener) => println!("Listening on port {} with{} SSL", listener.socket.port(), if https {""} else {"out"}),
		Err(error) => println!("Couldn't handle client: {}", error),
	}
}

fn main() {
	let options = Options::parse();
	println!("{:?}", options);

	let addr = &format!("0.0.0.0:{}", options.port)[..];
	match options.ssl {
		Some(pair) => {
			match Openssl::with_cert_and_key(pair.0, pair.1) {
				Ok(ssl) => {handle_server_error(Server::https(&addr, ssl)).map(|s| handle_server(s, true));},
				Err(error) => println!("Couldn't set up OpenSSL: {}", error),
			}
		},
		None => {handle_server_error(Server::http(&addr)).map(|s| handle_server(s, false));},
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
