mod chunk_reader;
mod headers;
mod request;
mod request_line;
mod response;
mod server;

use crate::request::Request;
use crate::response::ResponseWriter;
use crate::server::{HandlerError, Server};
use crate::{
    headers::Headers,
    response::{StatusCode, StatusLine},
};

fn main() {
    let Ok(server) = Server::new(8080) else {
        println!("couldnt not create server at port");
        return;
    };

    server.serve(handler);
}

fn handler(writer: &mut ResponseWriter, request: Request) -> Result<(), HandlerError> {
    writer.write_status_line(&StatusLine::from(StatusCode::Ok));
    writer.write_headers(&Headers::get_default(0));
    writer.write_chunked_body(b"Hello world");
    Ok(())
}
