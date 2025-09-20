use std::{
    io::{self, BufReader, BufWriter, Write},
    net::{TcpListener, TcpStream},
};

use crate::{
    request::Request,
    response::{StatusCode, write_error_response, write_ok_response},
};

#[derive(Debug)]
pub enum HandlerError {
    BadRequest,
    IntervalServerError,
}
impl HandlerError {
    pub fn as_str(&self) -> &str {
        match self {
            HandlerError::BadRequest => "Bad Request",
            HandlerError::IntervalServerError => "Internal Server Error",
        }
    }

    pub fn code(&self) -> StatusCode {
        match self {
            HandlerError::BadRequest => StatusCode::BadRequest,
            HandlerError::IntervalServerError => StatusCode::InternalServerError,
        }
    }
}

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn new(port: u16) -> Result<Self, io::Error> {
        Ok(Self {
            listener: TcpListener::bind(("127.0.0.1", port))?,
        })
    }

    pub fn serve(&self, handler: fn(&mut dyn Write, Request) -> Result<(), HandlerError>) {
        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            println!("connection established");

            Self::handle(stream, handler);
        }
    }

    fn handle(
        mut stream: TcpStream,
        handler: fn(&mut dyn Write, Request) -> Result<(), HandlerError>,
    ) {
        let mut stream_reader = BufReader::new(&stream);
        let request = Request::from_reader(&mut stream_reader).unwrap();
        println!("received request");

        let mut body_buf = Vec::<u8>::with_capacity(1024);
        let handle_result = {
            let mut body_writer = BufWriter::new(&mut body_buf);
            handler(&mut body_writer, request)
        };

        let mut stream_writer = BufWriter::new(&mut stream);
        match handle_result {
            Ok(()) => write_ok_response(&mut stream_writer, &body_buf),
            Err(e) => write_error_response(&mut stream_writer, &e),
        }

        println!("response sent");
    }
}
