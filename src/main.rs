mod chunk_reader;
mod headers;
mod request;
mod request_line;
mod response;
mod server;

use crate::server::Server;
use crate::{
    headers::Headers,
    response::{StatusCode, StatusLine},
};

fn main() {
    let Ok(server) = Server::new(8080) else {
        println!("couldnt not create server at port");
        return;
    };

    server.serve(|writer, _request| {
        writer.write_status_line(&StatusLine::from(StatusCode::Ok));
        writer.write_headers(&Headers::get_default(23));
        writer.write_body("Welcome to burger king\n".as_bytes());

        Ok(())
    });
}
