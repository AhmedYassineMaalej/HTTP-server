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
        let target = request.request_line.request_target;

        if target == r"/yourproblem" {
            return Err(HandlerError::IntervalServerError);
        }

        if target == r"/myproblem" {
            return Err(HandlerError::IntervalServerError);
        }

        match writer.write("hello world".as_bytes()) {
            Ok(_) => Ok(()),
            Err(_e) => Err(HandlerError::BadRequest),
        }
    });
}
