use std::{fmt::Display, io::Write};

use crate::{headers::Headers, server::HandlerError};

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

pub struct Response {
    status_line: StatusLine,
    headers: Headers,
    body: Vec<u8>,
}

impl Response {
    pub fn write_to(&self, writer: &mut impl Write) {
        // write status line
        let status_line = format!(
            "HTTP/1.1 {} {}\r\n",
            self.status_line.code, self.status_line.message
        );
        writer.write_all(status_line.as_bytes()).unwrap();

        // write headers
        for (key, val) in self.headers.iter() {
            write!(writer, "{key}: {val}\r\n").unwrap();
        }
        write!(writer, "\r\n").unwrap();

        // write body
        writer.write_all(&self.body).unwrap();
    }
}

pub struct ResponseBuilder {
    status_line: StatusLine,
    headers: Headers,
    body: Vec<u8>,
}

impl ResponseBuilder {
    pub fn from_code(status_code: StatusCode) -> Self {
        Self {
            status_line: StatusLine::from(status_code),
            headers: Headers::new(),
            body: Vec::new(),
        }
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        let content_length = body.len();
        self.body = body;
        self.headers
            .insert("Content-Length", content_length.to_string());
        self
    }

    pub fn header(mut self, header_key: &str, header_val: String) -> Self {
        self.headers.insert(&header_key, header_val);
        self
    }

    pub fn build(self) -> Response {
        Response {
            status_line: self.status_line,
            headers: self.headers,
            body: self.body,
        }
    }
}
// fn write_status_line(writer: &mut impl Write, status_line: &StatusLine) {
//     let status_line = format!("HTTP/1.1 {} {}\r\n", status_line.code, status_line.message);
//
//     writer.write_all(status_line.as_bytes()).unwrap();
// }
//
// fn write_headers(writer: &mut impl Write, headers: &Headers) {
//     for (key, val) in headers.iter() {
//         write!(writer, "{key}: {val}\r\n").unwrap();
//     }
//     write!(writer, "\r\n").unwrap();
// }
//
// pub fn write_ok_response(mut writer: &mut impl Write, body: &[u8]) {
//     println!("request handled successfully");
//
//     write_status_line(&mut writer, StatusCode::Ok);
//     write_headers(&mut writer, &Headers::get_default(body.len()));
//     writer.write_all(body).unwrap();
// }
//
// pub fn write_error_response(mut writer: &mut impl Write, error: &HandlerError) {
//     println!("failed to handle request: {error:?}");
//
//     write_status_line(&mut writer, error.code());
//     let error_string = error.as_str();
//     write_headers(&mut writer, &Headers::get_default(error_string.len()));
//     writer.write_all(error_string.as_bytes()).unwrap();
// }
//
// pub fn write_response(mut writer: &mut impl Write, response: &Response) {
//     write_status_line(writer, &response.status_line);
//     write_headers(writer, &response.headers);
// }
