mod body;
mod chunk_reader;
mod headers;
mod request;
mod request_line;
mod server;

use std::net::TcpListener;

use crate::request::Request;

fn main() {
    let listener = TcpListener::bind("[::1]:8080").unwrap();
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

        let request = Request::from_reader(&mut stream).unwrap();

        println!("Request line:");
        println!("- Method: {}", request.request_line.method);
        println!("- Target: {}", request.request_line.request_target);
        println!("- Version: {}", request.request_line.http_version);
        println!("Headers: ");

        for (key, val) in request.headers.iter() {
            println!("- {}: {}", key, val);
        }

        println!("Body: {}", String::from_utf8(request.body).unwrap());
    }
}
