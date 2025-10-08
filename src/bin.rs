use http_server::{
    headers::Headers,
    request::Request,
    response::{ResponseWriter, StatusCode, StatusLine},
    server::{HandlerError, Server},
};

fn main() {
    let Ok(server) = Server::new(8080) else {
        println!("couldnt not create server at port");
        return;
    };

    server.serve(handler).unwrap();
}

fn handler(writer: &mut ResponseWriter, request: Request) -> Result<(), HandlerError> {
    writer
        .write_status_line(&StatusLine::from(StatusCode::PermanentRedirect))
        .map_err(|_| HandlerError::IntervalServerError)?;

    let mut headers = Headers::new();
    headers.insert("Content-Type", String::from("text/html"));

    writer
        .write_headers(&headers)
        .map_err(|_| HandlerError::IntervalServerError)?;

    if let Err(_e) = writer.write_body(b"<h1>Hello world</h1>") {
        return Err(HandlerError::IntervalServerError);
    }

    Ok(())
}
