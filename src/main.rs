mod chunk_reader;
mod headers;
mod request;
mod request_line;
mod response;
mod server;

use crate::server::{HandlerError, Server};

fn main() {
    let server = Server::new(8080);

    server.serve(|writer, request| {
        dbg!(request);
        match writer.write("hello world".as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(HandlerError::BadRequest),
        }
    });
}
