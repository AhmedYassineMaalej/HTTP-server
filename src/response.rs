use std::io::Write;

use crate::headers::Headers;

pub enum StatusCode {
    Ok = 200,
    BadRequest = 400,
    InternalServerError = 500,
}

pub fn write_status_line(writer: &mut impl Write, status_code: StatusCode) {
    let status_code_message = match status_code {
        StatusCode::Ok => "200 OK",
        StatusCode::BadRequest => "400 Bad Request",
        StatusCode::InternalServerError => "500 Internal Server Error",
    };

    let status_line = format!("HTTP/1.1 {status_code_message}\r\n");

    writer.write(status_line.as_bytes()).unwrap();
}

pub fn get_default_headers(content_length: usize) -> Headers {
    let mut headers = Headers::new();

    headers.insert(String::from("Content-Length"), content_length.to_string());
    headers.insert(String::from("Connection"), String::from("close"));
    headers.insert(String::from("Content-Type"), String::from("text/plain"));

    headers
}

pub fn write_headers(writer: &mut impl Write, headers: Headers) {
    for (key, val) in headers.iter() {
        write!(writer, "{key}: {val}\r\n").unwrap();
    }
    write!(writer, "\r\n").unwrap();
}
