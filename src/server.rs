use std::{
    io::{self, BufReader, BufWriter},
    net::{TcpListener, TcpStream},
};

use crate::{
    request::Request,
    response::{ResponseWriter, StatusCode},
};

pub type RequestHandler = fn(&mut ResponseWriter, Request) -> Result<(), HandlerError>;

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

    pub fn serve(&self, handler: RequestHandler) {
        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            println!("connection established");

            Self::handle(stream, handler);
        }
    }

    fn handle(mut stream: TcpStream, handler: RequestHandler) {
        let mut stream_reader = BufReader::new(&stream);
        let request = Request::from_reader(&mut stream_reader).unwrap();
        println!("received request");

        let stream_writer = BufWriter::new(&mut stream);
        let mut response_writer = ResponseWriter::new(stream_writer);

        handler(&mut response_writer, request).unwrap();

        println!("response sent");
    }
}
