use std::{
    io::{BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    time::Duration,
};

use crate::request::Request;

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn new(port: u16) -> Self {
        Self {
            listener: TcpListener::bind(("127.0.0.1", port)).unwrap(),
        }
    }

    pub fn serve(&self) {
        for stream in self.listener.incoming() {
            let mut stream = stream.unwrap();
            println!("connection established");

            let mut bufreader = BufReader::new(&stream);

            let _request = Request::from_reader(&mut bufreader).unwrap();
            println!("received request");

            let response = "HTTP/1.1 200 OK\r\n\
                Content-Type: text/plain\r\n\
                Content-Length: 13\r\n\
                Connection: close\r\n\r\n\
                Hello World!\n";

            match stream.write_all(response.as_bytes()) {
                Ok(_) => println!("sent response!"),
                Err(e) => println!("Failed to send response: {e}"),
            };
        }
    }
}
