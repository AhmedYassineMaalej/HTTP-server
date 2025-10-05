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
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            HandlerError::BadRequest => "Bad Request",
            HandlerError::IntervalServerError => "Internal Server Error",
        }
    }

    #[must_use]
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
    /// Create a new `Server` that listens on `127.0.0.1:{port}`
    ///
    /// # Errors
    ///
    /// will return an error if the underlying `TcpListener::bind` fails
    /// see <https://doc.rust-lang.org/std/net/struct.TcpListener.html#method.bind>
    pub fn new(port: u16) -> Result<Self, io::Error> {
        Ok(Self {
            listener: TcpListener::bind(("127.0.0.1", port))?,
        })
    }

    /// Serve incoming streams according to the passed `RequestHandler`
    ///
    /// This function will never return `Ok(())` as it runs indefinitely unless it encounters an
    /// error
    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying `TcpListener::incoming` fails
    pub fn serve(&self, handler: RequestHandler) -> Result<(), io::Error> {
        for stream in self.listener.incoming() {
            let stream = stream?;
            println!("connection established");

            Self::handle(stream, handler);
        }

        Ok(())
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
