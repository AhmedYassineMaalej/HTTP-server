mod chunk_reader;
mod headers;
mod request;
mod request_line;
mod response;
mod server;

use crate::server::{HandlerError, Server};

fn main() {
    let Ok(server) = Server::new(8080) else {
        println!("couldnt not create server at port");
        return;
    };

    // TODO: change writer from &mut dyn Write to ReponseWriter
    server.serve(|writer, request| {
        let target = request.request_line.request_target;
        if target == r"/yourproblem" {
            return Err(HandlerError::IntervalServerError);
        }

        if target == r"/myproblem" {
            return Err(HandlerError::IntervalServerError);
        }

        match writer.write_all("hello world".as_bytes()) {
            Ok(()) => todo!(),
            Err(_e) => Err(HandlerError::BadRequest),
        }
    });
}
