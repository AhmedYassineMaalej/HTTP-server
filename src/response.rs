use std::{
    fmt::Display,
    io::{BufWriter, Write},
    net::TcpStream,
};

use crate::headers::Headers;

#[derive(Debug, Clone, Copy)]
pub enum StatusCode {
    Ok = 200,
    BadRequest = 400,
    InternalServerError = 500,
}

impl Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as isize)
    }
}

pub struct StatusLine {
    code: StatusCode,
    message: String,
}

impl From<StatusCode> for StatusLine {
    fn from(code: StatusCode) -> Self {
        let message = match code {
            StatusCode::Ok => "200 OK",
            StatusCode::BadRequest => "400 Bad Request",
            StatusCode::InternalServerError => "500 Internal Server Error",
        };

        Self {
            code,
            message: message.to_string(),
        }
    }
}

pub struct ResponseWriter<'a> {
    inner: BufWriter<&'a mut TcpStream>,
}

impl<'a> ResponseWriter<'a> {
    pub fn write_status_line(&mut self, status_line: &StatusLine) {
        let status_line = format!("HTTP/1.1 {} {}\r\n", status_line.code, status_line.message);
        self.inner.write_all(status_line.as_bytes()).unwrap();
    }

    pub fn write_headers(&mut self, headers: &Headers) {
        for (key, val) in headers.iter() {
            write!(self.inner, "{key}: {val}\r\n").unwrap();
        }
        write!(self.inner, "\r\n").unwrap();
    }

    pub fn write_body(&mut self, body: &[u8]) {
        self.inner.write_all(body).unwrap();
    }

    pub fn new(writer: BufWriter<&'a mut TcpStream>) -> Self {
        Self { inner: writer }
    }
}
