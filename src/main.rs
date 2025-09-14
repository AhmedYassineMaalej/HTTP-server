mod chunk_reader;
mod headers;
mod request;
mod request_line;
mod server;

use std::net::TcpListener;

use crate::{request::Request, server::Server};

fn main() {
    let server = Server::new(8080);

    server.serve();
}
