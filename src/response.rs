use std::{
    fmt::Display,
    io::{self, BufWriter, Write},
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

#[derive(Debug, PartialEq)]
enum ResponseWriterState {
    Init,
    StatusLine,
    Headers,
    Body,
}

pub struct ResponseWriter<'a> {
    inner: BufWriter<&'a mut TcpStream>,
    state: ResponseWriterState,
    body_write_done: bool,
}

impl<'a> ResponseWriter<'a> {
    #[must_use]
    pub fn new(writer: BufWriter<&'a mut TcpStream>) -> Self {
        Self {
            inner: writer,
            state: ResponseWriterState::Init,
            body_write_done: false,
        }
    }

    /// Write the passed `StatusLine` to underlying buffer
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - it fails to write to the buffer
    /// - the internal state of the writer does not allow writing a body (Status Line can only be
    ///   written once and before any other part of the response)
    pub fn write_status_line(&mut self, status_line: &StatusLine) -> Result<(), io::Error> {
        if self.state != ResponseWriterState::Init {
            todo!()
        }

        let status_line = format!("HTTP/1.1 {} {}\r\n", status_line.code, status_line.message);
        self.inner.write_all(status_line.as_bytes())?;
        self.state = ResponseWriterState::StatusLine;
        Ok(())
    }

    /// Write response headers into underlying buffer
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - it fails to write to the buffer
    /// - the internal state of the writer does not allow writing headers (can only be written
    ///   once: after status body and before body)
    pub fn write_headers(&mut self, headers: &Headers) -> Result<(), io::Error> {
        if self.state != ResponseWriterState::StatusLine {
            todo!()
        }

        for (key, val) in headers {
            write!(self.inner, "{key}: {val}\r\n")?;
        }

        write!(self.inner, "\r\n")?;
        self.state = ResponseWriterState::Headers;
        Ok(())
    }

    /// Write passed response `body` into underlying buffer
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - it fails to write to the buffer
    /// - the internal state of the writer does not allow writing a body (response line and headers
    ///   must be written before)
    pub fn write_body(&mut self, body: &[u8]) -> Result<(), io::Error> {
        if self.state != ResponseWriterState::Headers {
            todo!()
        }
        self.inner.write_all(body)?;
        self.state = ResponseWriterState::Body;
        Ok(())
    }

    /// Write passed response `body` into underlying buffer according to chunked encoding
    /// the chunk size is fixed to `1024`
    /// # Errors
    ///
    /// This function will return an error if:
    /// - it fails to write to the buffer
    /// - the internal state of the writer does not allow writing a body (response line and headers
    ///   must be written before)
    pub fn write_chunked_body(&mut self, body: &[u8]) -> Result<(), io::Error> {
        if self.state != ResponseWriterState::Headers {
            todo!()
        }
        let mut start = 0;
        let chunk_size = 1024;
        while start < body.len() {
            let chunk_range = start..(start + chunk_size).min(body.len());
            start = chunk_range.end;
            let chunk = &body[chunk_range];
            self.inner
                .write_all(format!("{:X}", chunk.len()).as_bytes())?;
            self.inner.write_all(b"\r\n")?;
            self.inner.write_all(chunk)?;
            self.inner.write_all(b"\r\n")?;
        }
        self.inner.write_all(b"0\r\n\r\n")?;
        self.body_write_done = true;
        Ok(())
    }

    #[must_use]
    pub fn write_chunked_body_done(&self) -> bool {
        self.body_write_done
    }
}
