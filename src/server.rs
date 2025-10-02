use std::{
    io::{self, BufReader, BufWriter, Write},
    net::{TcpListener, TcpStream},
};

use crate::{
    request::Request,
    response::{Response, ResponseBuilder, StatusCode},
};

pub type RequestHandler = fn(&mut dyn Write, Request) -> Result<Response, HandlerError>;

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

        let mut body_buf = Vec::<u8>::with_capacity(1024);
        let handle_result = {
            let mut body_writer = BufWriter::new(&mut body_buf);
            handler(&mut body_writer, request)
        };

        let mut stream_writer = BufWriter::new(&mut stream);

        match handle_result {
            Ok(response) => response.write_to(&mut stream_writer),
            Err(handle_err) => {
                let response = ResponseBuilder::from_code(handle_err.code()).build();
                response.write_to(&mut stream_writer);
            }
        }

        println!("response sent");
    }
}
