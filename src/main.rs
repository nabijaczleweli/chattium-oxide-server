use std::net::TcpListener;
use std::io::Read;


fn main() {
	match TcpListener::bind("127.0.0.1:0") {
		Ok(listener) => {
			println!("{:?}", listener);

			loop {
				match listener.accept() {
					Ok((mut stream, addr)) => {
						println!("{:?}@{}", stream, addr);
						let mut contents: Vec<u8> = Vec::new();
						match stream.read_to_end(&mut contents) {
							Ok(read) => println!("{} = \"{:?}\"", read, contents),
							Err(error) => println!("Couldn't read from the stream: {}", error),
						}
					}
					Err(error) => println!("Couldn't accept a connection: {}", error)
				};
			}
		}
		Err(error) => println!("Couldn't open the listener: {}", error),
	}
}
