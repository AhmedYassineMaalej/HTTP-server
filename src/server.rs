use std::{
    io::{BufReader, BufWriter, Write},
    net::{TcpListener, TcpStream},
};

use crate::{
    request::Request,
    response::{StatusCode, get_default_headers, write_headers, write_status_line},
};

#[derive(Debug)]
pub enum HandlerError {}

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn new(port: u16) -> Self {
        Self {
            listener: TcpListener::bind(("127.0.0.1", port)).unwrap(),
        }
    }

    pub fn serve(&mut self, handler: fn(&mut dyn Write, Request) -> Result<(), HandlerError>) {
        for stream in self.listener.incoming() {
            let mut stream = stream.unwrap();
            println!("connection established");

            self.handle(stream, handler);
        }
    }

    fn handle(
        &self,
        mut stream: TcpStream,
        handler: fn(&mut dyn Write, Request) -> Result<(), HandlerError>,
    ) {
        let mut bufreader = BufReader::new(&stream);

        let request = Request::from_reader(&mut bufreader).unwrap();
        println!("received request");

        let mut response_buf = Vec::<u8>::with_capacity(1024);

        {
            let mut bufwriter = BufWriter::new(&mut response_buf);
            write_status_line(&mut bufwriter, StatusCode::Ok);
            write_headers(&mut bufwriter, get_default_headers(12));

            match handler(&mut bufwriter, request) {
                Ok(_) => println!("request handled successfully"),
                Err(e) => {
                    println!("failed to handle request: {e:?}");
                    return;
                }
            };
        }

        stream.write(&response_buf).unwrap();

        println!("response sent");
    }
}
