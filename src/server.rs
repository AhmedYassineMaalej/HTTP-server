use std::{
    io::{BufReader, BufWriter, Write},
    net::{TcpListener, TcpStream},
};

use crate::{
    request::Request,
    response::{StatusCode, get_default_headers, write_headers, write_status_line},
};

#[derive(Debug)]
pub enum HandlerError {
    BadRequest,
}
impl HandlerError {
    fn as_str(&self) -> &str {
        "Internal server error"
    }
}

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn new(port: u16) -> Self {
        Self {
            listener: TcpListener::bind(("127.0.0.1", port)).unwrap(),
        }
    }

    pub fn serve(&self, handler: fn(&mut dyn Write, Request) -> Result<(), HandlerError>) {
        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            println!("connection established");

            self.handle(stream, handler);
        }
    }

    fn handle(
        &self,
        mut stream: TcpStream,
        handler: fn(&mut dyn Write, Request) -> Result<(), HandlerError>,
    ) {
        let mut stream_reader = BufReader::new(&stream);
        let request = Request::from_reader(&mut stream_reader).unwrap();
        println!("received request");

        let mut body_buf = Vec::<u8>::with_capacity(1024);

        let mut response_writer = BufWriter::new(&mut stream);

        let handle_result = {
            let mut body_writer = BufWriter::new(&mut body_buf);
            handler(&mut body_writer, request)
        };

        match handle_result {
            Ok(_) => {
                println!("request handled successfully");

                write_status_line(&mut response_writer, StatusCode::Ok);
                write_headers(&mut response_writer, get_default_headers(body_buf.len()));
                response_writer.write(&body_buf).unwrap();
            }

            Err(e) => {
                write_status_line(&mut response_writer, StatusCode::BadRequest);
                let error_string = e.as_str();
                write_headers(
                    &mut response_writer,
                    get_default_headers(error_string.len()),
                );
                println!("failed to handle request: {e:?}");
                response_writer.write(error_string.as_bytes()).unwrap();
            }
        };

        println!("response sent");
    }
}
