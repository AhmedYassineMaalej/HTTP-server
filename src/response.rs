use std::io::Write;

use crate::{headers::Headers, server::HandlerError};

pub enum StatusCode {
    Ok = 200,
    BadRequest = 400,
    InternalServerError = 500,
}

fn write_status_line(writer: &mut impl Write, status_code: StatusCode) {
    let status_code_message = match status_code {
        StatusCode::Ok => "200 OK",
        StatusCode::BadRequest => "400 Bad Request",
        StatusCode::InternalServerError => "500 Internal Server Error",
    };

    let status_line = format!("HTTP/1.1 {status_code_message}\r\n");

    writer.write(status_line.as_bytes()).unwrap();
}

fn write_headers(writer: &mut impl Write, headers: Headers) {
    for (key, val) in headers.iter() {
        write!(writer, "{key}: {val}\r\n").unwrap();
    }
    write!(writer, "\r\n").unwrap();
}

pub fn write_ok_response(mut writer: &mut impl Write, body: Vec<u8>) {
    println!("request handled successfully");

    write_status_line(&mut writer, StatusCode::Ok);
    write_headers(&mut writer, Headers::get_default(body.len()));
    writer.write(&body).unwrap();
}

pub fn write_error_response(mut writer: &mut impl Write, error: HandlerError) {
    println!("failed to handle request: {error:?}");

    write_status_line(&mut writer, error.code());
    let error_string = error.as_str();
    write_headers(&mut writer, Headers::get_default(error_string.len()));
    writer.write(error_string.as_bytes()).unwrap();
}
